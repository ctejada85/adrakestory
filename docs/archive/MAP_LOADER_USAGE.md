# Map Loader System - Usage Guide

## Overview

The map loader system allows you to create, load, and manage game maps using RON (Rusty Object Notation) files. Maps can include voxel terrain, entities, lighting, camera settings, and custom properties.

## Quick Start

### Loading a Map

Maps are automatically loaded when transitioning from the Title Screen to the game. The system:
1. Enters `LoadingMap` state
2. Displays a loading screen with progress bar
3. Loads the map from `assets/maps/default.ron`
4. Validates the map data
5. Spawns all voxels and entities
6. Transitions to `InGame` state

### Creating a New Map

Create a new `.ron` file in the `assets/maps/` directory:

```ron
(
    metadata: (
        name: "My Custom Map",
        author: "Your Name",
        description: "A custom map for testing",
        version: "1.0.0",
        created: "2025-01-10",
    ),
    world: (
        width: 5,
        height: 3,
        depth: 5,
        voxels: [
            // Add your voxels here
            (pos: (0, 0, 0), voxel_type: Grass, pattern: Some(Full)),
        ],
    ),
    entities: [
        (
            entity_type: PlayerSpawn,
            position: (2.5, 0.8, 2.5),
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

## Map Format Reference

### Metadata

```ron
metadata: (
    name: "Map Name",           // Display name
    author: "Author Name",      // Creator
    description: "Description", // Brief description
    version: "1.0.0",          // Version (must start with "1.")
    created: "2025-01-10",     // Creation date
)
```

### World Data

```ron
world: (
    width: 10,   // Width in voxels (must be positive)
    height: 5,   // Height in voxels (must be positive)
    depth: 10,   // Depth in voxels (must be positive)
    voxels: [
        // List of voxels
    ],
)
```

### Voxel Types

Available voxel types:
- `Air` - Empty space (usually not included in map files)
- `Grass` - Grass blocks
- `Dirt` - Dirt blocks
- `Stone` - Stone blocks

### Sub-Voxel Patterns

Each voxel can have a sub-voxel pattern that determines its shape:

- `Full` - Complete 8x8x8 cube (default)
- `Platform` - Thin 8x1x8 platform
- `Staircase` - Progressive height increase (8 steps)
- `Pillar` - Small 2x2x2 centered column

Example:
```ron
(pos: (2, 1, 1), voxel_type: Stone, pattern: Some(Staircase))
```

### Entities

Currently supported entity types:
- `PlayerSpawn` - Player starting position (required, at least one)
- `Enemy` - Enemy spawn point (not yet implemented)
- `Item` - Item spawn point (not yet implemented)
- `Trigger` - Trigger volume (not yet implemented)

Example:
```ron
entities: [
    (
        entity_type: PlayerSpawn,
        position: (5.0, 1.0, 5.0),
        properties: {
            "facing": "north",
        },
    ),
]
```

### Lighting

```ron
lighting: (
    ambient_intensity: 0.3,  // 0.0 to 1.0
    directional_light: Some((
        direction: (-0.5, -1.0, -0.5),  // Direction vector (normalized)
        illuminance: 10000.0,            // Light intensity in lux
        color: (1.0, 1.0, 1.0),         // RGB color (0.0 to 1.0)
    )),
)
```

To disable directional light:
```ron
directional_light: None,
```

### Camera

```ron
camera: (
    position: (5.0, 10.0, 8.0),      // Camera position
    look_at: (5.0, 0.0, 5.0),        // Point to look at
    rotation_offset: -1.5707963,     // Additional rotation in radians
)
```

### Custom Properties

Add any custom key-value pairs for your game logic:

```ron
custom_properties: {
    "difficulty": "hard",
    "time_limit": "300",
    "background_music": "theme_2.ogg",
    "weather": "rain",
}
```

## Validation Rules

The map loader validates maps before loading:

1. **World Dimensions**: Must be positive integers
2. **Voxel Positions**: Must be within world bounds (0 to width/height/depth - 1)
3. **Player Spawn**: At least one PlayerSpawn entity is required
4. **Entity Positions**: Should be within reasonable bounds
5. **Lighting Values**: Ambient intensity must be 0.0-1.0, colors must be 0.0-1.0
6. **Version**: Must start with "1." (e.g., "1.0.0", "1.2.3")

## Example Maps

### Minimal Map

See `assets/maps/simple_test.ron` for a minimal valid map.

### Full-Featured Map

See `assets/maps/default.ron` for a complete example with all features.

## Programmatic Usage

### Loading a Map in Code

```rust
use crate::systems::game::map::{MapLoader, MapLoadProgress};

fn load_custom_map(mut commands: Commands, mut progress: ResMut<MapLoadProgress>) {
    match MapLoader::load_from_file("assets/maps/custom.ron", &mut progress) {
        Ok(map) => {
            info!("Loaded map: {}", map.metadata.name);
            commands.insert_resource(LoadedMapData { map });
        }
        Err(e) => {
            error!("Failed to load map: {}", e);
        }
    }
}
```

### Saving a Map

```rust
use crate::systems::game::map::{MapLoader, MapData};

fn save_map(map: &MapData) {
    match MapLoader::save_to_file(map, "assets/maps/saved_map.ron") {
        Ok(_) => info!("Map saved successfully"),
        Err(e) => error!("Failed to save map: {}", e),
    }
}
```

### Creating a Map Programmatically

```rust
use crate::systems::game::map::format::*;
use std::collections::HashMap;

fn create_custom_map() -> MapData {
    MapData {
        metadata: MapMetadata {
            name: "Generated Map".to_string(),
            author: "System".to_string(),
            description: "Procedurally generated".to_string(),
            version: "1.0.0".to_string(),
            created: "2025-01-10".to_string(),
        },
        world: WorldData {
            width: 10,
            height: 5,
            depth: 10,
            voxels: vec![
                // Add voxels programmatically
            ],
        },
        entities: vec![
            EntityData {
                entity_type: EntityType::PlayerSpawn,
                position: (5.0, 1.0, 5.0),
                properties: HashMap::new(),
            },
        ],
        lighting: LightingData::default(),
        camera: CameraData::default(),
        custom_properties: HashMap::new(),
    }
}
```

## Progress Tracking

The map loader provides detailed progress information:

```rust
use crate::systems::game::map::LoadProgress;

fn check_progress(progress: &MapLoadProgress) {
    if let Some(current) = &progress.current {
        match current {
            LoadProgress::LoadingFile(p) => println!("Loading file: {:.0}%", p * 100.0),
            LoadProgress::ParsingData(p) => println!("Parsing: {:.0}%", p * 100.0),
            LoadProgress::ValidatingMap(p) => println!("Validating: {:.0}%", p * 100.0),
            LoadProgress::SpawningVoxels(p) => println!("Spawning voxels: {:.0}%", p * 100.0),
            LoadProgress::SpawningEntities(p) => println!("Spawning entities: {:.0}%", p * 100.0),
            LoadProgress::Complete => println!("Complete!"),
            LoadProgress::Error(e) => println!("Error: {}", e),
            _ => {}
        }
    }
}
```

## Troubleshooting

### Map Won't Load

1. Check the file path is correct: `assets/maps/yourmap.ron`
2. Verify RON syntax is valid (use a RON validator)
3. Check console for validation errors
4. Ensure at least one PlayerSpawn entity exists
5. Verify world dimensions are positive
6. Check all voxel positions are within bounds

### Performance Issues

For large maps:
- Consider using simpler sub-voxel patterns
- Reduce the number of Full pattern voxels
- Use Platform or Pillar patterns where possible
- Keep world dimensions reasonable (< 50x50x50)

### Visual Issues

- Check lighting values are in valid ranges
- Verify camera position and look_at are reasonable
- Ensure voxel positions don't overlap incorrectly

## Future Features

Planned enhancements:
- [ ] Additional file formats (JSON, binary)
- [ ] Streaming/chunked loading for large maps
- [ ] Map editor tool
- [ ] Network map loading
- [ ] Map compression
- [ ] Procedural generation templates
- [ ] More entity types
- [ ] Animated voxels
- [ ] Custom voxel types

## Contributing

To add new features to the map system:

1. Update data structures in `src/systems/game/map/format.rs`
2. Add validation in `src/systems/game/map/validation.rs`
3. Update spawner in `src/systems/game/map/spawner.rs`
4. Update this documentation
5. Create example maps demonstrating the new features