# Map Format API Specification

Technical specification for the A Drake's Story map format (Version 1.0.0).

## Document Information

- **Version**: 1.0.0
- **Format**: RON (Rusty Object Notation)
- **File Extension**: `.ron`
- **MIME Type**: `text/plain`
- **Character Encoding**: UTF-8

## Format Overview

Maps are defined in RON format with a root tuple. Six fields are required; two additional fields are optional (omit to use defaults):

```ron
(
    metadata: MapMetadata,
    world: WorldData,
    entities: Vec<EntityData>,
    lighting: LightingData,
    camera: CameraData,
    // Optional — omit entirely to default to an empty list:
    orientations: Vec<OrientationMatrix>,
    // Optional — omit entirely to default to an empty map:
    custom_properties: HashMap<String, String>,
)
```

## Type Definitions

### MapMetadata

**Type**: Struct  
**Required**: Yes

```rust
struct MapMetadata {
    name: String,
    author: String,
    description: String,
    version: String,
    created: String,
}
```

**Fields:**

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `name` | String | Yes | Any | Display name of the map |
| `author` | String | Yes | Any | Creator's name |
| `description` | String | Yes | Any | Brief description |
| `version` | String | Yes | Must match `^1\.` | Map format version |
| `created` | String | Yes | ISO 8601 date | Creation date (YYYY-MM-DD) |

**Example:**
```ron
metadata: (
    name: "Test Map",
    author: "Developer",
    description: "A test map",
    version: "1.0.0",
    created: "2025-01-10",
)
```

### WorldData

**Type**: Struct  
**Required**: Yes

```rust
struct WorldData {
    width: i32,
    height: i32,
    depth: i32,
    voxels: Vec<VoxelData>,
}
```

**Fields:**

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `width` | i32 | Yes | > 0 | X dimension in voxels |
| `height` | i32 | Yes | > 0 | Y dimension in voxels |
| `depth` | i32 | Yes | > 0 | Z dimension in voxels |
| `voxels` | Vec<VoxelData> | Yes | - | List of voxels |

**Coordinate System:**
- Origin (0,0,0) at bottom-front-left
- X-axis: left to right (width)
- Y-axis: bottom to top (height)
- Z-axis: front to back (depth)

**Note on Coordinate Normalization (Map Editor):**
When saving maps through the map editor, coordinates are automatically normalized to ensure all voxels start at (0,0,0). If you manually create maps with negative coordinates, they will be rejected during validation. The map editor handles this automatically by:
1. Calculating the bounding box of all voxels
2. Shifting all voxels, entities, and camera positions to start at origin
3. Adjusting dimensions to match the actual span

This ensures all saved maps are valid and can be loaded without errors.

### VoxelData

**Type**: Struct  
**Required**: No (empty list valid)

```rust
struct VoxelData {
    pos: (i32, i32, i32),
    voxel_type: VoxelType,
    pattern: Option<SubVoxelPattern>,        // #[serde(default)] — None is Full
    rotation: Option<usize>,                 // #[serde(default)] — index into MapData::orientations
    rotation_state: Option<LegacyRotationState>, // #[serde(default)] — load-only backward compat
}
```

**Fields:**

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `pos` | (i32, i32, i32) | Yes | Within world bounds | Grid position |
| `voxel_type` | VoxelType | Yes | Valid enum variant | Material type |
| `pattern` | Option<SubVoxelPattern> | No | Valid enum variant or None | Shape pattern; defaults to `Full` when absent |
| `rotation` | Option<usize> | No | Valid index into `MapData::orientations`, or None | Orientation matrix index; None means no rotation |
| `rotation_state` | Option<LegacyRotationState> | No | Load-only | **Backward compatibility only** — accepted on load, converted to `rotation` internally, never written on save. See [Legacy Rotation](#legacyrotationstate-legacy) section. |

**Position Constraints:**
- `0 <= pos.0 < width`
- `0 <= pos.1 < height`
- `0 <= pos.2 < depth`

### OrientationMatrix

**Type**: `[[i32; 3]; 3]` (type alias)  
**Required**: No (the `orientations` field on `MapData` defaults to an empty list)  
**Defined in**: `src/systems/game/map/format/rotation.rs`

An `OrientationMatrix` is a 3×3 integer rotation matrix that describes a rigid orientation in the 90° grid. All entries must be −1, 0, or 1, with exactly one non-zero entry per row and per column (determinant = 1 for proper rotations).

**Identity matrix** (no rotation):
```ron
[[1,0,0],[0,1,0],[0,0,1]]
```

**Example — 90° rotation around the Y axis:**
```ron
[[0,0,1],[0,1,0],[-1,0,0]]
```

**RON Syntax — declaring orientations on a map:**
```ron
orientations: [
    [[1,0,0],[0,1,0],[0,0,1]],   // index 0: identity (no rotation)
    [[0,0,1],[0,1,0],[-1,0,0]],  // index 1: 90° around Y
    [[-1,0,0],[0,1,0],[0,0,-1]], // index 2: 180° around Y
    [[0,0,-1],[0,1,0],[1,0,0]],  // index 3: 270° around Y
]
```

A `VoxelData.rotation` value of `Some(1)` means "apply `orientations[1]`". `None` (or absent) means no rotation.

**Constraints:**
- Each entry ∈ {−1, 0, 1}
- Exactly one non-zero entry per row and per column
- Determinant must equal 1 (proper rotation, no reflection)
- Index in `VoxelData.rotation` must be a valid index into `MapData::orientations`

### VoxelType

**Type**: Enum  
**Defined in**: `src/systems/game/map/format/voxel_type.rs`  
**Re-exported via**: `src/systems/game/map/format/mod.rs` and `src/systems/game/components.rs`  
**Variants:**

| Variant | Value | Description |
|---------|-------|-------------|
| `Air` | 0 | Empty space (usually omitted) |
| `Grass` | 1 | Grass blocks |
| `Dirt` | 2 | Dirt blocks |
| `Stone` | 3 | Stone blocks |

**RON Syntax:**
```ron
voxel_type: Grass
voxel_type: Dirt
voxel_type: Stone
```

### SubVoxelPattern

**Type**: Enum
**Variants:**

| Variant | Sub-Voxels | Description |
|---------|------------|-------------|
| `Full` | 8×8×8 (512) | Solid cube (symmetric) |
| `PlatformXZ` | 8×1×8 (64) | Horizontal platform on XZ plane |
| `PlatformXY` | 8×8×1 (64) | Vertical wall on XY plane (facing Z) |
| `PlatformYZ` | 1×8×8 (64) | Vertical wall on YZ plane (facing X) |
| `Staircase` | Variable (288) | Canonical staircase — stairs in the +X direction; facing direction set via `rotation` field |
| `Pillar` | 2×8×2 (32) | Full-height centred column; no gap when stacking vertically |
| `CenterCube` | 2×2×2 (8) | Small centred cube (symmetric, no orientation) |
| `Fence` | Variable | Fence post with neighbor-aware connection rails; `rotation` fully supported |

**Load-only aliases** (backward compatibility — accepted on load, never written on save):

| Alias | Canonical form after normalisation | Notes |
|-------|------------------------------------|-------|
| `Platform` | `PlatformXZ` | Old name (v1.0); normalised via `#[serde(alias)]` |
| `StaircaseX` | `Staircase, rotation: None` | Old name before rename (v1.0); normalised at load via `#[serde(alias)]` — no pre-bake, no matrix change |
| `StaircaseNegX` | `Staircase` + Y+180° absorbed into `rotation` | Normalised by `normalise_staircase_variants()` pass |
| `StaircaseZ` | `Staircase` + Y+90° absorbed into `rotation` | Normalised by `normalise_staircase_variants()` pass |
| `StaircaseNegZ` | `Staircase` + Y+270° absorbed into `rotation` | Normalised by `normalise_staircase_variants()` pass |
| `Pillar` (old) | `Pillar` (new column geometry) | `Pillar` previously described a 2×2×2 cube; from v1.1 it is a full-height 2×8×2 column. Old map files deserialise unchanged via `#[serde(alias)]` on `Pillar`. The 2×2×2 cube geometry is now `CenterCube`. |

**RON Syntax (canonical — new maps):**
```ron
pattern: Some(Full)
pattern: Some(PlatformXZ)
pattern: Some(PlatformXY)
pattern: Some(PlatformYZ)
pattern: Some(Staircase)
pattern: Some(Pillar)
pattern: Some(CenterCube)
pattern: Some(Fence)
pattern: None  // Defaults to Full
```

**RON Syntax (backward-compat aliases — accepted on load only):**
```ron
pattern: Some(Platform)      // Maps to PlatformXZ
pattern: Some(StaircaseX)    // Maps to Staircase (old name, no geometry change)
pattern: Some(StaircaseNegX) // Normalised to Staircase + Y+180° orientation
pattern: Some(StaircaseZ)    // Normalised to Staircase + Y+90°  orientation
pattern: Some(StaircaseNegZ) // Normalised to Staircase + Y+270° orientation
```

**Pattern Details:**

**Full**: All 512 sub-voxels present (8×8×8)

**PlatformXZ**: Horizontal slab (8×1×8)
- Bottom layer only on XZ plane

### LegacyRotationState (legacy)

> **This type is load-only backward compatibility.** It is accepted on load and converted to an `OrientationMatrix` internally, but is **never written on save**. New maps must use the `rotation: Option<usize>` field pointing into `MapData::orientations`. Do not use `rotation_state` in new map files.

`LegacyRotationState` was the original rotation representation before Finding 1 replaced it with the orientation-matrix system. It stored a single axis + angle pair. The type remains in the codebase only to allow old map files to continue loading without migration.

**Legacy RON syntax (accepted on load, not written on save):**
```ron
// These forms are still accepted but converted internally on load:
rotation_state: Some((axis: Y, angle: 1))  // Was: 90° around Y axis
rotation_state: Some((axis: X, angle: 2))  // Was: 180° around X axis
rotation_state: None                        // Was: no rotation
```

**New equivalent using `rotation`:**
```ron
// Declare orientation matrices in MapData:
orientations: [
    [[0,0,1],[0,1,0],[-1,0,0]],  // index 0: Y+90°
]

// Then reference by index on each voxel:
(
    pos: (1, 0, 1),
    voxel_type: Stone,
    pattern: Some(Staircase),
    rotation: Some(0),  // Apply orientations[0] — same as old rotation_state Y/angle:1
)
```

**PlatformXY**: Vertical wall (8×8×1)
- Wall on XY plane, facing Z direction

**PlatformYZ**: Vertical wall (1×8×8)
- Wall on YZ plane, facing X direction

**StaircaseX**: Progressive height in +X (288 sub-voxels)
- **Deprecated name.** Deserialises as `Staircase` via `#[serde(alias)]`. No geometry change.

**StaircaseNegX**: Progressive height in -X (288 sub-voxels)
- **Load-only alias.** Normalised to `Staircase` + Y+180° orientation matrix on load.

**StaircaseZ**: Progressive height in +Z (288 sub-voxels)
- **Load-only alias.** Normalised to `Staircase` + Y+90° orientation matrix on load.

**StaircaseNegZ**: Progressive height in -Z (288 sub-voxels)
- **Load-only alias.** Normalised to `Staircase` + Y+270° orientation matrix on load.

**Pillar**: Full-height 2×8×2 column (32 sub-voxels)
- Centred at x∈{3,4}, z∈{3,4}, spans all 8 Y layers
- No collision gap when stacking vertically

**CenterCube**: Centred 2×2×2 cube (8 sub-voxels)
- Occupies sub-voxels (3,3,3)–(4,4,4): centred in the voxel cell
- Previously (incorrectly) named `Pillar` before v1.1; old map files deserialise unchanged via backward-compat alias

**Fence**: Neighbor-aware fence post with connection rails
- Generates a post and extends rails toward any adjacent fence voxels (checked in world-aligned ±X and ±Z directions).
- The `rotation` field is **fully supported**: the orientation matrix is applied to the generated geometry after neighbor detection.
- Neighbor detection always uses world-axis-aligned positions — rotating the fence post does not change which adjacent cells are queried for connectivity.
- A fence with `rotation: None` behaves identically to the pre-rotation-system behavior.

**Example — rotated fence:**
```ron
(
    pos: (3, 0, 5),
    voxel_type: Stone,
    pattern: Some(Fence),
    rotation: Some(0),  // orientations[0] applied after neighbor geometry is generated
)
```

### EntityData

**Type**: Struct  
**Required**: At least one PlayerSpawn

```rust
struct EntityData {
    entity_type: EntityType,
    position: (f32, f32, f32),
    properties: HashMap<String, String>,
}
```

**Fields:**

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `entity_type` | EntityType | Yes | Valid enum variant | Entity type |
| `position` | (f32, f32, f32) | Yes | Float coordinates | World position |
| `properties` | HashMap<String, String> | Yes | Can be empty | Custom properties |

**Position Notes:**
- Uses world coordinates (floats), not grid coordinates
- Y should typically be above ground level
- No strict bounds checking (warning only)

### EntityType

**Type**: Enum  
**Variants:**

| Variant | Required | Status | Description |
|---------|----------|--------|-------------|
| `PlayerSpawn` | At least one | Implemented | Player starting position |
| `Npc` | No | Implemented | Non-player character spawn point |
| `Enemy` | No | Implemented | Enemy spawn point |
| `Item` | No | Implemented | Item pickup location |
| `Trigger` | No | Implemented | Event trigger zone |
| `LightSource` | No | Implemented | Point light with configurable properties |

**RON Syntax:**
```ron
entity_type: PlayerSpawn
entity_type: Npc
entity_type: Enemy
entity_type: Item
entity_type: Trigger
entity_type: LightSource
```

### LightingData

**Type**: Struct  
**Required**: Yes

```rust
struct LightingData {
    ambient_intensity: f32,
    directional_light: Option<DirectionalLightData>,
}
```

**Fields:**

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `ambient_intensity` | f32 | Yes | 0.0 ≤ x ≤ 1.0 | Ambient light level |
| `directional_light` | Option<DirectionalLightData> | No | - | Directional light config |

### DirectionalLightData

**Type**: Struct  
**Required**: No

```rust
struct DirectionalLightData {
    direction: (f32, f32, f32),
    illuminance: f32,
    color: (f32, f32, f32),
}
```

**Fields:**

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `direction` | (f32, f32, f32) | Yes | Any (normalized) | Light direction vector |
| `illuminance` | f32 | Yes | > 0.0 | Brightness in lux |
| `color` | (f32, f32, f32) | Yes | 0.0 ≤ x ≤ 1.0 each | RGB color |

**Example:**
```ron
directional_light: Some((
    direction: (-0.5, -1.0, -0.5),
    illuminance: 10000.0,
    color: (1.0, 1.0, 1.0),
))
```

### CameraData

**Type**: Struct  
**Required**: Yes

```rust
struct CameraData {
    position: (f32, f32, f32),
    look_at: (f32, f32, f32),
    rotation_offset: f32,
    follow_speed: Option<f32>,    // optional — engine default: 15.0
    rotation_speed: Option<f32>,  // optional — engine default: 5.0
    fov_degrees: Option<f32>,     // optional — engine default: ~60°
}
```

**Fields:**

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `position` | (f32, f32, f32) | Yes | Any | Camera starting position in world space |
| `look_at` | (f32, f32, f32) | Yes | Any | World-space point the camera initially aims at |
| `rotation_offset` | f32 | Yes | Radians | Y-axis rotation applied around `look_at` after the initial transform. `-π/2` (≈ `-1.5707964`) rotates 90° left; `π/2` rotates 90° right. |
| `follow_speed` | Option\<f32\> | No | `> 0` | Exponential decay rate for camera position follow. Higher = more responsive. Default: `15.0`. |
| `rotation_speed` | Option\<f32\> | No | `> 0` | Exponential decay rate for camera rotation interpolation. Default: `5.0`. |
| `fov_degrees` | Option\<f32\> | No | `5`–`150` | Vertical field of view in degrees. Default: engine default (~60°). |

**Common Values:**
- `rotation_offset: -1.5707963` (-π/2) for a 90° left isometric view
- `rotation_offset: 0.0` for no additional rotation

**Optional field usage:**

Omit `follow_speed`, `rotation_speed`, and `fov_degrees` to use engine defaults (all existing map files are unaffected). Set them to tune camera feel per-map:

```ron
camera: (
    position: (8.5, 8.0, 5.5),
    look_at: (8.5, 0.0, 1.5),
    rotation_offset: -1.5707964,
    follow_speed: Some(5.0),      // slower, more cinematic follow
    rotation_speed: Some(3.0),    // slower rotation interpolation
    fov_degrees: Some(75.0),      // slightly wider field of view
)
```

### Custom Properties

**Type**: HashMap<String, String>  
**Required**: Yes (can be empty)

```rust
custom_properties: HashMap<String, String>
```

**Format:**
```ron
custom_properties: {
    "key1": "value1",
    "key2": "value2",
}
```

**Notes:**
- All keys and values must be strings
- No type constraints on values
- Application-specific interpretation
- Can be empty: `{}`
- **No namespace convention is currently enforced.** A reserved key-prefix scheme (e.g. `engine:*` for engine-defined properties) is tracked in [`docs/bugs/custom-properties-namespace/`](../bugs/custom-properties-namespace/ticket.md) and is planned for a future version. Until then, avoid keys that start with `engine:` to prevent collision with the forthcoming reserved namespace.

## Validation Rules

### Required Validations

1. **World Dimensions**
   - `width > 0`
   - `height > 0`
   - `depth > 0`

2. **Voxel Positions**
   - `0 <= voxel.pos.0 < width`
   - `0 <= voxel.pos.1 < height`
   - `0 <= voxel.pos.2 < depth`
   - No two voxels may share the same `pos` (duplicate positions cause mesh corruption and are rejected with a validation error)

3. **Player Spawn**
   - At least one `EntityType::PlayerSpawn` required

4. **Lighting Values**
   - `0.0 <= ambient_intensity <= 1.0`
   - `0.0 <= color.r, color.g, color.b <= 1.0`

5. **Version Format**
   - Must match regex: `^1\.`
   - Examples: "1.0.0", "1.2.3", "1.0.0-beta"

6. **Orientation Matrices** (`validate_orientations`)
   - Every matrix in `MapData::orientations` must have entries ∈ {−1, 0, 1}
   - Exactly one non-zero entry per row and per column (pure axis permutation)
   - Determinant must equal 1 (proper rotation, no reflection)
   - Every `VoxelData.rotation` index must be a valid index into the `orientations` list

7. **Entity Properties**
   - `LightSource` entities: `intensity` must parse as a positive `f32`; `range` must parse as a positive `f32`; `color` must be a valid `(r, g, b)` string with each component 0.0–1.0; `shadows` must parse as a `bool`. Invalid values produce a validation warning and fall back to engine defaults.
   - `Npc` entities: `model` property, if present, must be a non-empty string path. Invalid or missing `model` produces a warning and uses a placeholder mesh.

### Optional Validations (Warnings)

1. **Entity Positions**
   - Should be within reasonable bounds
   - Y coordinate should be above ground

2. **Lighting Intensity**
   - Recommended: 5000-15000 lux
   - Too low/high may affect visibility

3. **Camera Position**
   - Should provide good view of map
   - Recommended: above and away from center

## Error Codes

| Code | Type | Description |
|------|------|-------------|
| `E001` | FileReadError | Cannot read file |
| `E002` | ParseError | Invalid RON syntax |
| `E003` | ValidationError | Validation failed |
| `E004` | InvalidVoxelPosition | Voxel out of bounds |
| `E005` | InvalidEntityType | Unknown entity type |

## Compatibility

### Version 1.0.0

**Supported Features:**
- Basic voxel types (Air, Grass, Dirt, Stone)
- Sub-voxel patterns: Full, PlatformXZ, PlatformXY, PlatformYZ, Staircase, Pillar (full-height column), CenterCube, Fence
- All entity types: PlayerSpawn, Npc, Enemy, Item, Trigger, LightSource
- Orientation-matrix rotation system (`MapData::orientations` + `VoxelData::rotation`)
- Ambient and directional lighting
- Camera configuration (with optional `follow_speed`, `rotation_speed`, `fov_degrees`)
- Custom properties

**Load-only backward compatibility (accepted on load, never written on save):**
- `rotation_state` field on `VoxelData` (legacy single-axis rotation)
- Pattern aliases: `Platform` → `PlatformXZ`, `StaircaseX` → `Staircase`, `StaircaseNegX/Z/NegZ` (normalised), `FenceX/Z/Corner` → `Fence`

**Not Supported:**
- Animated voxels
- Scripting
- Area lights or baked lightmaps

### Future Versions

**Planned for 2.0.0:**
- Breaking changes to format
- New features requiring incompatible changes

## Examples

### Minimal Valid Map

```ron
(
    metadata: (
        name: "Minimal",
        author: "System",
        description: "Minimal valid map",
        version: "1.0.0",
        created: "2025-01-10",
    ),
    world: (
        width: 1,
        height: 1,
        depth: 1,
        voxels: [
            (pos: (0, 0, 0), voxel_type: Grass, pattern: Some(Full)),
        ],
    ),
    entities: [
        (entity_type: PlayerSpawn, position: (0.5, 0.5, 0.5), properties: {}),
    ],
    lighting: (
        ambient_intensity: 0.3,
        directional_light: None,
    ),
    camera: (
        position: (2.0, 2.0, 2.0),
        look_at: (0.0, 0.0, 0.0),
        rotation_offset: 0.0,
    ),
    custom_properties: {},
)
```

### Complete Example

See `assets/maps/default.ron` in the repository.

## Implementation Notes

### Parsing

- Use `ron` crate version 0.8+
- Enable `serde` derive features
- Handle `SpannedError` for better error messages

### Validation

- Validate immediately after parsing
- Fail fast on first error
- Provide descriptive error messages

### Spawning

- Spawn voxels before entities
- Update spatial grid during spawning
- Emit progress events for UI

## Related Documentation

- **[Map Format Reference](../user-guide/maps/map-format.md)** - User-friendly format guide
- **[Creating Maps](../user-guide/maps/creating-maps.md)** - Map creation tutorial
- **[Map Loader System](../developer-guide/systems/map-loader.md)** - Implementation details

---

**Specification Version**: 1.0.0  
**Last Updated**: 2026-03-31  
**Status**: Stable