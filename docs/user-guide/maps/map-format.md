# Map Format Reference

Complete technical reference for the RON map format used in A Drake's Story.

## File Format

Maps use **RON (Rusty Object Notation)** format with `.ron` file extension.

**Location:** `assets/maps/*.ron`

## Complete Structure

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

## Data Types

### MapMetadata

```ron
metadata: (
    name: String,           // Display name
    author: String,         // Creator name
    description: String,    // Brief description
    version: String,        // Version (must start with "1.")
    created: String,        // Date (YYYY-MM-DD)
)
```

**Validation:**
- `version` must match pattern `^1\.`
- All fields are required

### WorldData

```ron
world: (
    width: i32,             // X dimension (must be > 0)
    height: i32,            // Y dimension (must be > 0)
    depth: i32,             // Z dimension (must be > 0)
    voxels: Vec<VoxelData>,
)
```

**Coordinate System:**
- Origin: (0, 0, 0) at bottom-front-left
- X: Width (left to right)
- Y: Height (bottom to top)
- Z: Depth (front to back)

### VoxelData

```ron
(
    pos: (i32, i32, i32),           // Position in grid
    voxel_type: VoxelType,          // Material type
    pattern: Option<SubVoxelPattern>, // Shape pattern
)
```

**VoxelType Enum:**
```ron
Air     // Empty space (usually omitted)
Grass   // Grass blocks
Dirt    // Dirt blocks
Stone   // Stone blocks
```

**SubVoxelPattern Enum:**
```ron
Full        // 8×8×8 solid cube
Platform    // 8×8×1 thin platform
Staircase   // 8-step progressive staircase
Pillar      // 2×2×2 centered column
```

**Validation:**
- Position must be within world bounds: `0 <= pos < (width, height, depth)`
- Pattern is optional (defaults to Full if None)

### EntityData

```ron
(
    entity_type: EntityType,
    position: (f32, f32, f32),      // World position (floats)
    properties: HashMap<String, String>,
)
```

**EntityType Enum:**
```ron
PlayerSpawn  // Player starting position (required)
Enemy        // Enemy spawn (not yet implemented)
Item         // Item pickup (not yet implemented)
Trigger      // Event trigger (not yet implemented)
```

**Validation:**
- At least one `PlayerSpawn` entity is required
- Position uses float coordinates (world space, not grid)

### LightingData

```ron
lighting: (
    ambient_intensity: f32,                 // 0.0 to 1.0
    directional_light: Option<DirectionalLightData>,
)
```

**DirectionalLightData:**
```ron
(
    direction: (f32, f32, f32),    // Direction vector (normalized)
    illuminance: f32,               // Brightness in lux
    color: (f32, f32, f32),        // RGB (0.0 to 1.0 each)
)
```

**Validation:**
- `ambient_intensity`: 0.0 ≤ value ≤ 1.0
- `color` components: 0.0 ≤ value ≤ 1.0
- `direction` will be normalized automatically

### CameraData

```ron
camera: (
    position: (f32, f32, f32),      // Camera position
    look_at: (f32, f32, f32),       // Target point
    rotation_offset: f32,            // Additional rotation (radians)
)
```

**Common Values:**
- `rotation_offset`: -π/2 (-1.5707963) for isometric view
- Position typically above and away from map center

### Custom Properties

```ron
custom_properties: {
    "key1": "value1",
    "key2": "value2",
    // ... any key-value pairs
}
```

**Usage:**
- Optional metadata for game logic
- All keys and values are strings
- Can be empty: `{}`

## Validation Rules

### Required Elements
1. ✅ Metadata section with all fields
2. ✅ World section with positive dimensions
3. ✅ At least one PlayerSpawn entity
4. ✅ Lighting section (can have None for directional)
5. ✅ Camera section

### Constraints
1. **World Dimensions:** Must be positive integers
2. **Voxel Positions:** Must be within `[0, dimension)` for each axis
3. **Version:** Must start with "1." (e.g., "1.0.0", "1.2.3")
4. **Lighting:** Intensity and color values must be in [0.0, 1.0]
5. **Player Spawn:** At least one required

### Optional Elements
- Directional light (can be `None`)
- Custom properties (can be empty `{}`)
- Voxel pattern (defaults to `Full` if `None`)
- Entity properties (can be empty `{}`)

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
        width: 2,
        height: 2,
        depth: 2,
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

See `assets/maps/default.ron` for a full-featured example with:
- Multiple voxel types and patterns
- Proper lighting setup
- Well-positioned camera
- Custom properties

## Common Patterns

### Floor Layer

```ron
voxels: [
    (pos: (0, 0, 0), voxel_type: Grass, pattern: Some(Full)),
    (pos: (1, 0, 0), voxel_type: Grass, pattern: Some(Full)),
    (pos: (2, 0, 0), voxel_type: Grass, pattern: Some(Full)),
    // ... continue pattern
]
```

### Platform Series

```ron
voxels: [
    (pos: (0, 1, 0), voxel_type: Stone, pattern: Some(Platform)),
    (pos: (2, 2, 0), voxel_type: Stone, pattern: Some(Platform)),
    (pos: (4, 3, 0), voxel_type: Stone, pattern: Some(Platform)),
]
```

### Staircase

```ron
voxels: [
    (pos: (0, 0, 0), voxel_type: Stone, pattern: Some(Full)),
    (pos: (1, 0, 0), voxel_type: Stone, pattern: Some(Staircase)),
    (pos: (2, 1, 0), voxel_type: Stone, pattern: Some(Full)),
]
```

### Corner Pillars

```ron
voxels: [
    (pos: (0, 0, 0), voxel_type: Stone, pattern: Some(Pillar)),
    (pos: (9, 0, 0), voxel_type: Stone, pattern: Some(Pillar)),
    (pos: (0, 0, 9), voxel_type: Stone, pattern: Some(Pillar)),
    (pos: (9, 0, 9), voxel_type: Stone, pattern: Some(Pillar)),
]
```

## Error Messages

### Common Errors

**Parse Error:**
```
Failed to parse map data: unexpected character at line X
```
→ Check RON syntax (parentheses, commas, quotes)

**Validation Error:**
```
Map validation failed: No player spawn entity found
```
→ Add at least one PlayerSpawn entity

**Invalid Position:**
```
Invalid voxel position: (10, 0, 0)
```
→ Position exceeds world dimensions

**Invalid Version:**
```
Map validation failed: Version must start with "1."
```
→ Use version format like "1.0.0"

## Best Practices

### Organization
1. Group voxels by layer (Y coordinate)
2. Comment sections for clarity
3. Use consistent indentation
4. Order voxels logically

### Performance
1. Omit Air voxels (they're default)
2. Use simpler patterns when possible
3. Keep dimensions reasonable
4. Avoid redundant voxels

### Maintainability
1. Use descriptive metadata
2. Add comments for complex sections
3. Version your maps
4. Document custom properties

## Tools & Utilities

### Validation

The map loader automatically validates:
```rust
MapLoader::load_from_file("assets/maps/your_map.ron", &mut progress)
```

### Programmatic Creation

```rust
use crate::systems::game::map::format::*;

let map = MapData {
    metadata: MapMetadata { /* ... */ },
    world: WorldData { /* ... */ },
    entities: vec![/* ... */],
    lighting: LightingData::default(),
    camera: CameraData::default(),
    custom_properties: HashMap::new(),
};
```

### Saving Maps

```rust
MapLoader::save_to_file(&map, "assets/maps/saved.ron")?;
```

## Version History

### Version 1.0.0
- Initial map format
- Basic voxel types and patterns
- Entity system foundation
- Lighting and camera support
- Custom properties

### Future Versions
- Additional voxel types
- More entity types
- Advanced lighting options
- Animation support
- Scripting integration

## Related Documentation

- **[Creating Maps Guide](creating-maps.md)** - Step-by-step tutorial
- **[Example Maps](examples.md)** - Study examples
- **[Map Loader System](../../developer-guide/systems/map-loader.md)** - Implementation details
- **[API Specification](../../api/map-format-spec.md)** - Technical spec

---

**Reference Version:** 1.0.0  
**Last Updated:** 2025-01-10