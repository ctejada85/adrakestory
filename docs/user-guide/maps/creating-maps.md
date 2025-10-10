# Creating Custom Maps

Learn how to create your own custom maps for A Drake's Story using the RON (Rusty Object Notation) format.

## Overview

Maps in A Drake's Story are defined in human-readable RON files stored in the `assets/maps/` directory. The map system supports:
- Custom voxel layouts with sub-voxel patterns
- Entity placement (player spawns, enemies, items)
- Lighting configuration
- Camera positioning
- Custom properties for game logic

## Quick Start

### 1. Create a New Map File

Create a new file in `assets/maps/` with a `.ron` extension:

```bash
touch assets/maps/my_map.ron
```

### 2. Basic Map Template

Start with this minimal template:

```ron
(
    metadata: (
        name: "My First Map",
        author: "Your Name",
        description: "A custom test map",
        version: "1.0.0",
        created: "2025-01-10",
    ),
    world: (
        width: 5,
        height: 3,
        depth: 5,
        voxels: [
            // Add voxels here
            (pos: (0, 0, 0), voxel_type: Grass, pattern: Some(Full)),
            (pos: (1, 0, 0), voxel_type: Grass, pattern: Some(Full)),
            (pos: (2, 0, 0), voxel_type: Grass, pattern: Some(Full)),
        ],
    ),
    entities: [
        (
            entity_type: PlayerSpawn,
            position: (2.5, 1.0, 2.5),
            properties: {},
        ),
    ],
    lighting: (
        ambient_intensity: 0.3,
        directional_light: Some((
            direction: (-0.5, -1.0, -0.5),
            illuminance: 10000.0,
            color: (1.0, 1.0, 1.0),
        )),
    ),
    camera: (
        position: (2.5, 8.0, 5.5),
        look_at: (2.5, 0.0, 2.5),
        rotation_offset: -1.5707963,
    ),
    custom_properties: {},
)
```

### 3. Load Your Map

Update the map path in `src/main.rs`:

```rust
let map = match MapLoader::load_from_file("assets/maps/my_map.ron", &mut progress) {
    // ...
}
```

Or keep it as `default.ron` and rename your file.

## Map Structure

### Metadata Section

```ron
metadata: (
    name: "Map Display Name",
    author: "Creator Name",
    description: "Brief description of the map",
    version: "1.0.0",           // Must start with "1."
    created: "2025-01-10",      // ISO date format
)
```

**Requirements:**
- `name`: Display name (any string)
- `author`: Creator name (any string)
- `description`: Brief description (any string)
- `version`: Must start with "1." (e.g., "1.0.0", "1.2.3")
- `created`: Date in YYYY-MM-DD format

### World Section

```ron
world: (
    width: 10,    // X dimension (must be positive)
    height: 5,    // Y dimension (must be positive)
    depth: 10,    // Z dimension (must be positive)
    voxels: [
        // Voxel definitions
    ],
)
```

**Coordinate System:**
- X: Width (left to right)
- Y: Height (bottom to top)
- Z: Depth (front to back)
- Origin (0,0,0) is bottom-front-left corner

### Voxel Definitions

Each voxel has a position, type, and optional pattern:

```ron
(pos: (x, y, z), voxel_type: Type, pattern: Some(Pattern))
```

**Voxel Types:**
- `Grass` - Grass blocks (green)
- `Dirt` - Dirt blocks (brown)
- `Stone` - Stone blocks (gray)
- `Air` - Empty space (usually omitted)

**Sub-Voxel Patterns:**
- `Full` - Solid 8×8×8 cube (default)
- `Platform` - Thin 8×8×1 horizontal platform
- `Staircase` - Progressive 8-step staircase
- `Pillar` - Small 2×2×2 centered column

**Examples:**
```ron
// Solid grass block
(pos: (0, 0, 0), voxel_type: Grass, pattern: Some(Full))

// Thin platform
(pos: (1, 1, 0), voxel_type: Stone, pattern: Some(Platform))

// Staircase
(pos: (2, 0, 0), voxel_type: Stone, pattern: Some(Staircase))

// Small pillar
(pos: (3, 0, 0), voxel_type: Stone, pattern: Some(Pillar))
```

### Entity Placement

```ron
entities: [
    (
        entity_type: PlayerSpawn,
        position: (x, y, z),      // Float coordinates
        properties: {
            "key": "value",       // Optional custom properties
        },
    ),
]
```

**Entity Types:**
- `PlayerSpawn` - Player starting position (required, at least one)
- `Enemy` - Enemy spawn point (planned)
- `Item` - Item pickup location (planned)
- `Trigger` - Event trigger zone (planned)

**Position Notes:**
- Use float coordinates (e.g., 2.5, 1.0, 2.5)
- Position is in world space, not voxel grid
- Y coordinate should be above ground level
- Center of map is often a good spawn point

### Lighting Configuration

```ron
lighting: (
    ambient_intensity: 0.3,     // 0.0 to 1.0
    directional_light: Some((
        direction: (-0.5, -1.0, -0.5),  // Normalized vector
        illuminance: 10000.0,            // Light intensity in lux
        color: (1.0, 1.0, 1.0),         // RGB (0.0 to 1.0)
    )),
)
```

**Ambient Intensity:**
- Range: 0.0 (dark) to 1.0 (bright)
- Recommended: 0.2 to 0.4
- Affects overall scene brightness

**Directional Light:**
- `direction`: Light direction vector (will be normalized)
- `illuminance`: Brightness in lux (typical: 5000-15000)
- `color`: RGB values (1.0, 1.0, 1.0 = white)
- Use `None` to disable directional light

### Camera Setup

```ron
camera: (
    position: (x, y, z),        // Camera position
    look_at: (x, y, z),         // Point to look at
    rotation_offset: -1.5707963, // Additional rotation (radians)
)
```

**Camera Tips:**
- Position camera above and away from the map center
- `look_at` should point to map center or player spawn
- `rotation_offset`: -π/2 (-1.5707963) for typical isometric view
- Experiment with different angles

### Custom Properties

```ron
custom_properties: {
    "difficulty": "hard",
    "time_limit": "300",
    "background_music": "theme_2.ogg",
    "weather": "rain",
}
```

Add any custom key-value pairs for your game logic. These are optional and can be used for:
- Difficulty settings
- Time limits
- Music selection
- Weather effects
- Special rules
- Metadata

## Building Your Map

### Step 1: Plan Your Layout

Sketch your map on paper or use a grid:
```
Top view (Y=0):
[G][G][G][G][G]
[G][ ][ ][ ][G]
[G][ ][P][ ][G]
[G][ ][ ][ ][G]
[G][G][G][G][G]

G = Grass block
P = Player spawn
```

### Step 2: Create the Floor

Start with a solid floor layer:

```ron
voxels: [
    // Row 0
    (pos: (0, 0, 0), voxel_type: Grass, pattern: Some(Full)),
    (pos: (1, 0, 0), voxel_type: Grass, pattern: Some(Full)),
    (pos: (2, 0, 0), voxel_type: Grass, pattern: Some(Full)),
    // ... continue for all floor tiles
]
```

### Step 3: Add Vertical Elements

Add walls, pillars, or elevated platforms:

```ron
// Wall at x=0
(pos: (0, 1, 0), voxel_type: Stone, pattern: Some(Full)),
(pos: (0, 2, 0), voxel_type: Stone, pattern: Some(Full)),

// Platform at height 2
(pos: (2, 2, 2), voxel_type: Stone, pattern: Some(Platform)),
```

### Step 4: Add Navigation Elements

Include staircases and platforms:

```ron
// Staircase going up
(pos: (1, 0, 1), voxel_type: Stone, pattern: Some(Staircase)),

// Jumping platforms
(pos: (3, 1, 1), voxel_type: Stone, pattern: Some(Platform)),
(pos: (4, 1, 2), voxel_type: Stone, pattern: Some(Platform)),
```

### Step 5: Place Entities

Add player spawn and other entities:

```ron
entities: [
    (
        entity_type: PlayerSpawn,
        position: (2.5, 1.0, 2.5),  // Center of map, above floor
        properties: {},
    ),
]
```

### Step 6: Configure Lighting

Set up ambient and directional lighting:

```ron
lighting: (
    ambient_intensity: 0.3,
    directional_light: Some((
        direction: (-0.5, -1.0, -0.5),
        illuminance: 10000.0,
        color: (1.0, 1.0, 0.9),  // Slightly warm white
    )),
)
```

### Step 7: Position Camera

Set camera for best view of your map:

```ron
camera: (
    position: (width/2, height*2, depth*1.5),
    look_at: (width/2, 0.0, depth/2),
    rotation_offset: -1.5707963,
)
```

## Testing Your Map

### 1. Validate Syntax

Check for RON syntax errors:
```bash
cargo run --release
```

The loader will report any parsing errors.

### 2. Check Validation

The system validates:
- World dimensions are positive
- Voxel positions are within bounds
- At least one player spawn exists
- Lighting values are in valid ranges
- Version format is correct

### 3. Test In-Game

1. Load the map
2. Check player spawn location
3. Test navigation
4. Verify lighting
5. Check camera angle
6. Enable collision boxes (C key) to debug

### 4. Iterate

Make adjustments based on testing:
- Move player spawn if needed
- Adjust lighting for better visibility
- Reposition camera for better view
- Add or remove voxels
- Change patterns for variety

## Common Patterns

### Creating a Room

```ron
// Floor (5x5)
(pos: (0, 0, 0), voxel_type: Grass, pattern: Some(Full)),
// ... more floor tiles

// Walls (height 3)
(pos: (0, 1, 0), voxel_type: Stone, pattern: Some(Full)),
(pos: (0, 2, 0), voxel_type: Stone, pattern: Some(Full)),
// ... more wall tiles
```

### Creating a Platform Course

```ron
// Starting platform
(pos: (0, 0, 0), voxel_type: Stone, pattern: Some(Platform)),

// Jump platforms
(pos: (2, 1, 0), voxel_type: Stone, pattern: Some(Platform)),
(pos: (4, 2, 0), voxel_type: Stone, pattern: Some(Platform)),
(pos: (6, 3, 0), voxel_type: Stone, pattern: Some(Platform)),
```

### Creating a Stairway

```ron
// Ground level
(pos: (0, 0, 0), voxel_type: Stone, pattern: Some(Full)),

// Staircase
(pos: (1, 0, 0), voxel_type: Stone, pattern: Some(Staircase)),

// Upper level
(pos: (2, 1, 0), voxel_type: Stone, pattern: Some(Full)),
```

## Tips & Best Practices

### Design Tips
1. **Start Small**: Begin with simple layouts
2. **Test Often**: Load and test frequently
3. **Use Patterns**: Mix different sub-voxel patterns
4. **Plan Vertically**: Think in 3D, not just 2D
5. **Leave Space**: Don't make maps too cramped

### Performance Tips
1. **Limit Size**: Keep dimensions reasonable (< 50×50×50)
2. **Use Patterns**: Platform/Pillar patterns are lighter than Full
3. **Avoid Redundancy**: Don't place voxels that won't be seen
4. **Test Performance**: Check frame rate with your map

### Aesthetic Tips
1. **Vary Materials**: Mix Grass, Dirt, and Stone
2. **Use Lighting**: Good lighting makes maps look better
3. **Create Depth**: Use different heights
4. **Add Details**: Small pillars and platforms add interest

## Troubleshooting

### Map Won't Load

**Check:**
- File is in `assets/maps/` directory
- File has `.ron` extension
- RON syntax is valid (matching parentheses, commas)
- Path in code matches filename

### Validation Errors

**Common Issues:**
- Voxel position outside world bounds
- No player spawn entity
- Invalid lighting values (must be 0.0-1.0)
- Invalid version format (must start with "1.")

### Visual Issues

**Solutions:**
- Adjust ambient_intensity (try 0.2-0.4)
- Change directional light direction
- Reposition camera
- Check voxel positions aren't overlapping incorrectly

### Performance Issues

**Optimizations:**
- Reduce world dimensions
- Use simpler patterns (Platform instead of Full)
- Remove unnecessary voxels
- Test in release mode

## Next Steps

- **[Map Format Reference](map-format.md)** - Complete format specification
- **[Example Maps](examples.md)** - Study included examples
- **[Troubleshooting](../troubleshooting.md)** - Solve common issues
- **[Developer Guide](../../developer-guide/systems/map-loader.md)** - System internals

---

**Happy mapping!** Share your creations with the community!