# Map Loader System

Technical documentation for the map loading system implementation.

## Overview

The map loader system provides a complete solution for loading game maps from RON files with progress tracking, validation, and world spawning. This document covers the internal implementation details.

## Architecture

### Module Structure

```
src/systems/game/map/
├── mod.rs          # Public API and re-exports
├── format.rs       # Data structures and types
├── loader.rs       # File I/O and parsing
├── spawner.rs      # World instantiation
├── validation.rs   # Map validation logic
└── error.rs        # Error types
```

### Data Flow

```
RON File → MapLoader → Validation → MapData → MapSpawner → Game World
            ↓                                      ↓
      Progress Events                      Entity Spawning
```

## Core Components

### format.rs - Data Structures

**MapData**: Root structure containing all map information

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MapData {
    pub metadata: MapMetadata,
    pub world: WorldData,
    pub entities: Vec<EntityData>,
    pub lighting: LightingData,
    pub camera: CameraData,
    pub custom_properties: HashMap<String, String>,
}
```

**Key Design Decisions:**
- Uses `serde` for serialization/deserialization
- All fields are public for easy access
- Implements `Clone` for resource storage
- `Debug` for development/debugging

**SubVoxelPattern**: Defines voxel shapes

```rust
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum SubVoxelPattern {
    Full,        // 8×8×8 solid cube
    Platform,    // 8×8×1 thin platform
    Staircase,   // Progressive 8-step staircase
    Pillar,      // 2×2×2 centered column
}
```

**Implementation Details:**
- Each pattern generates different sub-voxel configurations
- Patterns are applied during spawning, not stored per sub-voxel
- Enables high detail without excessive memory usage

### loader.rs - File Loading

**MapLoader**: Handles file I/O and parsing

```rust
pub struct MapLoader;

impl MapLoader {
    pub fn load_from_file(
        path: &str,
        progress: &mut MapLoadProgress,
    ) -> Result<MapData, MapLoadError> {
        // 1. Update progress: LoadingFile
        progress.update(LoadProgress::LoadingFile(0.0));
        
        // 2. Read file
        let contents = std::fs::read_to_string(path)?;
        progress.update(LoadProgress::LoadingFile(1.0));
        
        // 3. Parse RON
        progress.update(LoadProgress::ParsingData(0.0));
        let map: MapData = ron::from_str(&contents)?;
        progress.update(LoadProgress::ParsingData(1.0));
        
        // 4. Validate
        progress.update(LoadProgress::ValidatingMap(0.0));
        validate_map(&map)?;
        progress.update(LoadProgress::ValidatingMap(1.0));
        
        Ok(map)
    }
}
```

**Progress Tracking:**
- Emits progress events at each stage
- Allows UI to display real-time feedback
- Percentage-based for consistent UI

**Error Handling:**
- Uses `Result` type for error propagation
- Converts I/O and parse errors to `MapLoadError`
- Provides context for debugging

### validation.rs - Map Validation

**Validation Rules:**

```rust
pub fn validate_map(map: &MapData) -> Result<(), MapLoadError> {
    // 1. Check world dimensions
    if map.world.width <= 0 || map.world.height <= 0 || map.world.depth <= 0 {
        return Err(MapLoadError::ValidationError(
            "World dimensions must be positive".to_string()
        ));
    }
    
    // 2. Check voxel positions
    for voxel in &map.world.voxels {
        if !is_position_valid(voxel.pos, &map.world) {
            return Err(MapLoadError::InvalidVoxelPosition(
                voxel.pos.0, voxel.pos.1, voxel.pos.2
            ));
        }
    }
    
    // 3. Check for player spawn
    let has_player_spawn = map.entities.iter()
        .any(|e| e.entity_type == EntityType::PlayerSpawn);
    if !has_player_spawn {
        return Err(MapLoadError::ValidationError(
            "Map must have at least one PlayerSpawn entity".to_string()
        ));
    }
    
    // 4. Validate lighting
    validate_lighting(&map.lighting)?;
    
    // 5. Check version
    if !map.metadata.version.starts_with("1.") {
        return Err(MapLoadError::ValidationError(
            "Version must start with '1.'".to_string()
        ));
    }
    
    Ok(())
}
```

**Validation Strategy:**
- Fail fast: Return on first error
- Descriptive errors: Help users fix issues
- Comprehensive: Check all requirements
- Extensible: Easy to add new rules

### spawner.rs - World Instantiation

**MapSpawner System:**

```rust
pub fn spawn_map_system(
    mut commands: Commands,
    map_data: Res<LoadedMapData>,
    mut progress: ResMut<MapLoadProgress>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spatial_grid: ResMut<SpatialGrid>,
) {
    let map = &map_data.map;
    
    // 1. Spawn voxels
    progress.update(LoadProgress::SpawningVoxels(0.0));
    spawn_voxels(&mut commands, &map.world, &mut meshes, &mut materials, &mut spatial_grid, &mut progress);
    
    // 2. Spawn entities
    progress.update(LoadProgress::SpawningEntities(0.0));
    spawn_entities(&mut commands, &map.entities, &mut progress);
    
    // 3. Setup lighting
    progress.update(LoadProgress::Finalizing(0.5));
    setup_lighting(&mut commands, &map.lighting);
    
    // 4. Setup camera
    setup_camera(&mut commands, &map.camera);
    
    // 5. Complete
    progress.update(LoadProgress::Complete);
}
```

**Voxel Spawning:**

```rust
fn spawn_voxels(
    commands: &mut Commands,
    world: &WorldData,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    spatial_grid: &mut SpatialGrid,
    progress: &mut MapLoadProgress,
) {
    let total = world.voxels.len();
    
    for (i, voxel_data) in world.voxels.iter().enumerate() {
        // Spawn parent voxel entity
        let voxel_entity = commands.spawn((
            Voxel,
            Transform::from_translation(voxel_pos_to_world(voxel_data.pos)),
        )).id();
        
        // Spawn sub-voxels based on pattern
        spawn_sub_voxels(
            commands,
            voxel_entity,
            voxel_data,
            meshes,
            materials,
            spatial_grid,
        );
        
        // Update progress
        let progress_pct = (i + 1) as f32 / total as f32;
        progress.update(LoadProgress::SpawningVoxels(progress_pct));
    }
}
```

**Sub-Voxel Pattern Implementation:**

```rust
fn spawn_sub_voxels(
    commands: &mut Commands,
    parent: Entity,
    voxel_data: &VoxelData,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    spatial_grid: &mut SpatialGrid,
) {
    let pattern = voxel_data.pattern.unwrap_or(SubVoxelPattern::Full);
    
    match pattern {
        SubVoxelPattern::Full => {
            // Spawn all 8×8×8 = 512 sub-voxels
            for x in 0..8 {
                for y in 0..8 {
                    for z in 0..8 {
                        spawn_sub_voxel(commands, parent, IVec3::new(x, y, z), meshes, materials, spatial_grid);
                    }
                }
            }
        }
        SubVoxelPattern::Platform => {
            // Spawn only bottom layer (8×1×8 = 64 sub-voxels)
            for x in 0..8 {
                for z in 0..8 {
                    spawn_sub_voxel(commands, parent, IVec3::new(x, 0, z), meshes, materials, spatial_grid);
                }
            }
        }
        SubVoxelPattern::Staircase => {
            // Spawn progressive steps
            for x in 0..8 {
                let height = x + 1; // Height increases with x
                for y in 0..height {
                    for z in 0..8 {
                        spawn_sub_voxel(commands, parent, IVec3::new(x, y, z), meshes, materials, spatial_grid);
                    }
                }
            }
        }
        SubVoxelPattern::Pillar => {
            // Spawn centered 2×2×2 column
            for x in 3..5 {
                for y in 0..8 {
                    for z in 3..5 {
                        spawn_sub_voxel(commands, parent, IVec3::new(x, y, z), meshes, materials, spatial_grid);
                    }
                }
            }
        }
    }
}
```

### error.rs - Error Handling

**MapLoadError Type:**

```rust
#[derive(Debug, thiserror::Error)]
pub enum MapLoadError {
    #[error("Failed to read map file: {0}")]
    FileReadError(#[from] std::io::Error),
    
    #[error("Failed to parse map data: {0}")]
    ParseError(#[from] ron::error::SpannedError),
    
    #[error("Map validation failed: {0}")]
    ValidationError(String),
    
    #[error("Invalid voxel position: ({0}, {1}, {2})")]
    InvalidVoxelPosition(i32, i32, i32),
}
```

**Error Handling Strategy:**
- Use `thiserror` for ergonomic error types
- Provide context in error messages
- Convert external errors automatically
- Enable easy error propagation with `?`

## Progress Tracking

### LoadProgress Enum

```rust
#[derive(Clone, Debug)]
pub enum LoadProgress {
    Started,
    LoadingFile(f32),      // 0-20%
    ParsingData(f32),      // 20-40%
    ValidatingMap(f32),    // 40-60%
    SpawningVoxels(f32),   // 60-90%
    SpawningEntities(f32), // 90-95%
    Finalizing(f32),       // 95-100%
    Complete,
    Error(String),
}

impl LoadProgress {
    pub fn percentage(&self) -> f32 {
        match self {
            Self::Started => 0.0,
            Self::LoadingFile(p) => p * 0.2,
            Self::ParsingData(p) => 0.2 + (p * 0.2),
            Self::ValidatingMap(p) => 0.4 + (p * 0.2),
            Self::SpawningVoxels(p) => 0.6 + (p * 0.3),
            Self::SpawningEntities(p) => 0.9 + (p * 0.05),
            Self::Finalizing(p) => 0.95 + (p * 0.05),
            Self::Complete => 1.0,
            Self::Error(_) => 0.0,
        }
    }
}
```

**Design Rationale:**
- Weighted stages based on typical duration
- Voxel spawning gets largest share (30%)
- Granular progress for better UX
- Easy to display in UI

## Performance Considerations

### Spatial Grid Optimization

The spawner updates a spatial grid for efficient collision detection:

```rust
// O(1) insertion into spatial grid
spatial_grid.insert(entity, position);

// O(1) query for nearby entities
let nearby = spatial_grid.query_cell(position);
```

**Benefits:**
- Reduces collision checks from O(n²) to O(n)
- Essential for large maps
- Minimal memory overhead

### Incremental Spawning

For very large maps, spawning can be split across frames:

```rust
#[derive(Resource)]
pub struct MapSpawnState {
    pub voxels_spawned: usize,
    pub total_voxels: usize,
}

// Spawn N voxels per frame
const VOXELS_PER_FRAME: usize = 100;
```

**Not Currently Implemented** but architecture supports it.

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_map_validation() {
        let map = create_test_map();
        assert!(validate_map(&map).is_ok());
    }
    
    #[test]
    fn test_invalid_dimensions() {
        let mut map = create_test_map();
        map.world.width = -1;
        assert!(validate_map(&map).is_err());
    }
}
```

### Integration Tests

```rust
#[test]
fn test_full_load_cycle() {
    let mut progress = MapLoadProgress::default();
    let result = MapLoader::load_from_file("assets/maps/simple_test.ron", &mut progress);
    assert!(result.is_ok());
    assert_eq!(progress.current, Some(LoadProgress::Complete));
}
```

## Future Enhancements

### Planned Features

1. **Async Loading**
   ```rust
   pub async fn load_async(path: &str) -> Result<MapData, MapLoadError>
   ```

2. **Streaming**
   ```rust
   pub struct ChunkedMapLoader {
       chunk_size: usize,
       loaded_chunks: HashSet<IVec3>,
   }
   ```

3. **Additional Formats**
   ```rust
   pub trait MapFormat {
       fn load(&self, path: &str) -> Result<MapData, MapLoadError>;
       fn save(&self, map: &MapData, path: &str) -> Result<(), MapLoadError>;
   }
   ```

4. **Network Loading**
   ```rust
   pub async fn load_from_url(url: &str) -> Result<MapData, MapLoadError>
   ```

## Related Documentation

- **[Map Format Reference](../../user-guide/maps/map-format.md)** - Format specification
- **[Creating Maps](../../user-guide/maps/creating-maps.md)** - User guide
- **[Architecture Overview](../architecture.md)** - System architecture

---

**Implementation Version:** 1.0.0  
**Last Updated:** 2025-01-10