//! Map validation logic.

use super::error::{MapLoadError, MapResult};
use super::format::MapData;

/// Validates a loaded map for correctness and consistency.
pub fn validate_map(map: &MapData) -> MapResult<()> {
    // Validate world dimensions
    validate_world_dimensions(map)?;

    // Validate voxel positions
    validate_voxel_positions(map)?;

    // Validate metadata
    validate_metadata(map)?;

    // Validate entities
    validate_entities(map)?;

    // Validate lighting
    validate_lighting(map)?;

    Ok(())
}

/// Validates that world dimensions are positive.
fn validate_world_dimensions(map: &MapData) -> MapResult<()> {
    let world = &map.world;

    if world.width <= 0 || world.height <= 0 || world.depth <= 0 {
        return Err(MapLoadError::InvalidWorldDimensions(
            world.width,
            world.height,
            world.depth,
        ));
    }

    Ok(())
}

/// Validates that all voxel positions are within world bounds.
fn validate_voxel_positions(map: &MapData) -> MapResult<()> {
    let world = &map.world;

    for voxel in &world.voxels {
        let (x, y, z) = voxel.pos;

        if x < 0 || x >= world.width || y < 0 || y >= world.height || z < 0 || z >= world.depth {
            return Err(MapLoadError::InvalidVoxelPosition(x, y, z));
        }
    }

    Ok(())
}

/// Validates that required metadata fields are present and non-empty.
fn validate_metadata(map: &MapData) -> MapResult<()> {
    let metadata = &map.metadata;

    if metadata.name.is_empty() {
        return Err(MapLoadError::MissingField("metadata.name".to_string()));
    }

    if metadata.version.is_empty() {
        return Err(MapLoadError::MissingField("metadata.version".to_string()));
    }

    // Check version compatibility (currently only support 1.x.x)
    if !metadata.version.starts_with("1.") {
        return Err(MapLoadError::UnsupportedVersion(metadata.version.clone()));
    }

    Ok(())
}

/// Validates entity data.
fn validate_entities(map: &MapData) -> MapResult<()> {
    // Check that at least one player spawn exists
    let has_player_spawn = map
        .entities
        .iter()
        .any(|e| matches!(e.entity_type, super::format::EntityType::PlayerSpawn));

    if !has_player_spawn {
        return Err(MapLoadError::ValidationError(
            "Map must have at least one PlayerSpawn entity".to_string(),
        ));
    }

    // Validate entity positions are reasonable (within or near world bounds)
    let world = &map.world;
    let max_x = world.width as f32;
    let max_y = world.height as f32 * 2.0; // Allow some height above world
    let max_z = world.depth as f32;

    for entity in &map.entities {
        let (x, y, z) = entity.position;

        if x < -1.0 || x > max_x + 1.0 || y < -1.0 || y > max_y || z < -1.0 || z > max_z + 1.0 {
            return Err(MapLoadError::ValidationError(format!(
                "Entity position ({}, {}, {}) is outside reasonable bounds",
                x, y, z
            )));
        }
    }

    Ok(())
}

/// Validates lighting data.
fn validate_lighting(map: &MapData) -> MapResult<()> {
    let lighting = &map.lighting;

    // Validate ambient intensity is in valid range
    if !(0.0..=1.0).contains(&lighting.ambient_intensity) {
        return Err(MapLoadError::ValidationError(format!(
            "Ambient intensity must be between 0.0 and 1.0, got {}",
            lighting.ambient_intensity
        )));
    }

    // Validate directional light if present
    if let Some(dir_light) = &lighting.directional_light {
        // Check that illuminance is positive
        if dir_light.illuminance < 0.0 {
            return Err(MapLoadError::ValidationError(format!(
                "Directional light illuminance must be positive, got {}",
                dir_light.illuminance
            )));
        }

        // Check that color components are in valid range
        let (r, g, b) = dir_light.color;
        if !(0.0..=1.0).contains(&r) || !(0.0..=1.0).contains(&g) || !(0.0..=1.0).contains(&b) {
            return Err(MapLoadError::ValidationError(format!(
                "Directional light color components must be between 0.0 and 1.0, got ({}, {}, {})",
                r, g, b
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::game::map::format::*;
    use std::collections::HashMap;

    #[test]
    fn test_validate_default_map() {
        let map = MapData::default_map();
        assert!(validate_map(&map).is_ok());
    }

    #[test]
    fn test_invalid_world_dimensions() {
        let mut map = MapData::default_map();
        map.world.width = 0;
        assert!(validate_map(&map).is_err());
    }

    #[test]
    fn test_invalid_voxel_position() {
        let mut map = MapData::default_map();
        map.world.voxels.push(VoxelData {
            pos: (100, 0, 0), // Outside bounds
            voxel_type: crate::systems::game::components::VoxelType::Stone,
            pattern: None,
        });
        assert!(validate_map(&map).is_err());
    }

    #[test]
    fn test_missing_player_spawn() {
        let mut map = MapData::default_map();
        map.entities.clear();
        assert!(validate_map(&map).is_err());
    }
}
