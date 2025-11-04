# Map Format API Specification

Technical specification for the A Drake's Story map format (Version 1.0.0).

## Document Information

- **Version**: 1.0.0
- **Format**: RON (Rusty Object Notation)
- **File Extension**: `.ron`
- **MIME Type**: `text/plain`
- **Character Encoding**: UTF-8

## Format Overview

Maps are defined in RON format with a root tuple containing six required fields:

```ron
(
    metadata: MapMetadata,
    world: WorldData,
    entities: Vec<EntityData>,
    lighting: LightingData,
    camera: CameraData,
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
    pattern: Option<SubVoxelPattern>,
    rotation_state: Option<RotationState>,
}
```

**Fields:**

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `pos` | (i32, i32, i32) | Yes | Within world bounds | Grid position |
| `voxel_type` | VoxelType | Yes | Valid enum variant | Material type |
| `pattern` | Option<SubVoxelPattern> | No | Valid enum variant or None | Shape pattern |
| `rotation_state` | Option<RotationState> | No | Valid rotation state | Rotation applied to pattern |

**Position Constraints:**
- `0 <= pos.0 < width`
- `0 <= pos.1 < height`
- `0 <= pos.2 < depth`

### VoxelType

**Type**: Enum  
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
| `StaircaseX` | Variable (288) | Stairs ascending in +X direction |
| `StaircaseNegX` | Variable (288) | Stairs ascending in -X direction |
| `StaircaseZ` | Variable (288) | Stairs ascending in +Z direction |
| `StaircaseNegZ` | Variable (288) | Stairs ascending in -Z direction |
| `Pillar` | 2×2×2 (8) | Centered cube (symmetric) |

**RON Syntax:**
```ron
pattern: Some(Full)
pattern: Some(PlatformXZ)
pattern: Some(PlatformXY)
pattern: Some(PlatformYZ)
pattern: Some(StaircaseX)
pattern: Some(StaircaseNegX)
pattern: Some(StaircaseZ)
pattern: Some(StaircaseNegZ)
pattern: Some(Pillar)
pattern: None  // Defaults to Full

// Backward compatibility (deprecated but supported)
pattern: Some(Platform)    // Maps to PlatformXZ
pattern: Some(Staircase)   // Maps to StaircaseX
```

**Pattern Details:**

**Full**: All 512 sub-voxels present (8×8×8)

**PlatformXZ**: Horizontal slab (8×1×8)
- Bottom layer only on XZ plane

### RotationState

**Type**: Struct  
**Required**: No

```rust
struct RotationState {
    axis: RotationAxis,
    angle: i32,
}
```

**Fields:**

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `axis` | RotationAxis | Yes | Valid enum variant | Axis of rotation |
| `angle` | i32 | Yes | 0-3 | Rotation angle in 90° increments |

**Angle Values:**
- `0` = 0° (no rotation)
- `1` = 90° clockwise
- `2` = 180°
- `3` = 270° clockwise (or 90° counter-clockwise)

**RON Syntax:**
```ron
rotation_state: Some((axis: Y, angle: 1))  // 90° around Y axis
rotation_state: Some((axis: X, angle: 2))  // 180° around X axis
rotation_state: None  // No rotation
```

### RotationAxis

**Type**: Enum  
**Variants:**

| Variant | Description |
|---------|-------------|
| `X` | Rotation around the X axis |
| `Y` | Rotation around the Y axis (default) |
| `Z` | Rotation around the Z axis |

**RON Syntax:**
```ron
axis: X
axis: Y
axis: Z
```

**Usage Example:**
```ron
(
    pos: (1, 0, 1),
    voxel_type: Stone,
    pattern: Some(StaircaseX),
    rotation_state: Some((axis: Y, angle: 1)),  // Rotate stairs 90° to face Z
)
```

**PlatformXY**: Vertical wall (8×8×1)
- Wall on XY plane, facing Z direction

**PlatformYZ**: Vertical wall (1×8×8)
- Wall on YZ plane, facing X direction

**StaircaseX**: Progressive height in +X (288 sub-voxels)
- Each step in X has progressively more height in Y

**StaircaseNegX**: Progressive height in -X (288 sub-voxels)
- Reverse of StaircaseX

**StaircaseZ**: Progressive height in +Z (288 sub-voxels)
- Each step in Z has progressively more height in Y

**StaircaseNegZ**: Progressive height in -Z (288 sub-voxels)
- Reverse of StaircaseZ

**Pillar**: Centered 2×2×2 cube (8 sub-voxels)
- Small centered cube, not a full-height column

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
| `Enemy` | No | Planned | Enemy spawn point |
| `Item` | No | Planned | Item pickup location |
| `Trigger` | No | Planned | Event trigger zone |

**RON Syntax:**
```ron
entity_type: PlayerSpawn
entity_type: Enemy
entity_type: Item
entity_type: Trigger
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
}
```

**Fields:**

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `position` | (f32, f32, f32) | Yes | Any | Camera position |
| `look_at` | (f32, f32, f32) | Yes | Any | Target point |
| `rotation_offset` | f32 | Yes | Radians | Additional rotation |

**Common Values:**
- `rotation_offset: -1.5707963` (-π/2) for isometric view
- `rotation_offset: 0.0` for no additional rotation

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

3. **Player Spawn**
   - At least one `EntityType::PlayerSpawn` required

4. **Lighting Values**
   - `0.0 <= ambient_intensity <= 1.0`
   - `0.0 <= color.r, color.g, color.b <= 1.0`

5. **Version Format**
   - Must match regex: `^1\.`
   - Examples: "1.0.0", "1.2.3", "1.0.0-beta"

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
- Sub-voxel patterns (Full, Platform, Staircase, Pillar)
- PlayerSpawn entities
- Ambient and directional lighting
- Camera configuration
- Custom properties

**Not Supported:**
- Enemy, Item, Trigger entities (defined but not implemented)
- Additional voxel types
- Animated voxels
- Scripting

### Future Versions

**Planned for 1.1.0:**
- Additional entity types
- More voxel types
- Animation support

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
**Last Updated**: 2025-01-10  
**Status**: Stable