//! Map validation logic.

use super::error::{MapLoadError, MapResult};
use super::format::{is_valid_rotation_matrix, MapData};
use bevy::log::warn;

/// Prefix reserved for engine-owned keys in `MapData::custom_properties`
/// and `EntityData::properties`.
///
/// Authors must not use this prefix. Engine subsystems that need to store
/// persistent data in either property map must add their key to
/// `KNOWN_MAP_ENGINE_KEYS` or `KNOWN_ENTITY_ENGINE_KEYS` below before
/// writing it to any map file.
const ENGINE_KEY_PREFIX: &str = "adrakestory:";

/// Engine-owned keys permitted in `MapData::custom_properties`.
///
/// Add entries here before introducing a new engine feature that writes
/// to this map. The validator will warn on unknown `adrakestory:` keys
/// to catch typos and forward-compat mismatches early.
const KNOWN_MAP_ENGINE_KEYS: &[&str] = &[
    // (none yet — convention established for future use)
];

/// Engine-owned keys permitted in `EntityData::properties`.
///
/// Same contract as `KNOWN_MAP_ENGINE_KEYS`.
const KNOWN_ENTITY_ENGINE_KEYS: &[&str] = &[
    // (none yet)
];

/// Validates a loaded map for correctness and consistency.
pub fn validate_map(map: &MapData) -> MapResult<()> {
    // Validate world dimensions
    validate_world_dimensions(map)?;

    // Validate voxel positions
    validate_voxel_positions(map)?;

    // Validate orientation matrices and voxel rotation indices
    validate_orientations(map)?;

    // Validate metadata
    validate_metadata(map)?;

    // Validate entities
    validate_entities(map)?;

    // Validate lighting
    validate_lighting(map)?;

    // Warn on unknown adrakestory:-prefixed keys (soft check, never fails)
    validate_custom_property_namespaces(map);

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

/// Validates that all voxel positions are within world bounds and that no two
/// voxels share the same position.
fn validate_voxel_positions(map: &MapData) -> MapResult<()> {
    let world = &map.world;
    let mut seen = std::collections::HashSet::new();

    for voxel in &world.voxels {
        let (x, y, z) = voxel.pos;

        // Bounds check first — consistent with all other early-return checks here.
        if x < 0 || x >= world.width || y < 0 || y >= world.height || z < 0 || z >= world.depth {
            return Err(MapLoadError::InvalidVoxelPosition(x, y, z));
        }

        // Duplicate check — two entries at the same position cause superimposed
        // geometry in the chunk mesh with no other indication of the problem.
        if !seen.insert(voxel.pos) {
            return Err(MapLoadError::ValidationError(format!(
                "Duplicate voxel position {:?}",
                voxel.pos
            )));
        }
    }

    Ok(())
}

/// Validates orientation matrices and voxel rotation index references.
fn validate_orientations(map: &MapData) -> MapResult<()> {
    // Validate each matrix in the orientations list
    for (i, matrix) in map.orientations.iter().enumerate() {
        if !is_valid_rotation_matrix(matrix) {
            return Err(MapLoadError::ValidationError(format!(
                "orientations[{}] is not a valid 90°-grid rotation matrix: {:?}",
                i, matrix
            )));
        }
    }

    // Validate each voxel's rotation index is within bounds
    for voxel in &map.world.voxels {
        if let Some(index) = voxel.rotation {
            if index >= map.orientations.len() {
                return Err(MapLoadError::ValidationError(format!(
                    "Voxel at {:?} has rotation index {} but orientations list has only {} entries",
                    voxel.pos,
                    index,
                    map.orientations.len()
                )));
            }
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

        // Validate that known entity property strings are well-formed.
        validate_entity_properties(entity)?;
    }

    Ok(())
}

/// Validates that string-valued properties for known entity types are parseable.
///
/// Unknown keys and unknown entity types are accepted without error for
/// forward-compatibility. The spawner's fallback logic is not changed by this
/// function; this validator and the spawner are independent layers.
fn validate_entity_properties(entity: &super::format::EntityData) -> MapResult<()> {
    use super::format::EntityType;

    match entity.entity_type {
        EntityType::LightSource => {
            if let Some(v) = entity.properties.get("intensity") {
                match v.parse::<f32>() {
                    Ok(f) if f >= 0.0 => {}
                    _ => {
                        return Err(MapLoadError::ValidationError(format!(
                            "LightSource entity has invalid 'intensity': \
                             expected non-negative f32, got {:?}",
                            v
                        )))
                    }
                }
            }
            if let Some(v) = entity.properties.get("range") {
                match v.parse::<f32>() {
                    Ok(f) if f > 0.0 => {}
                    _ => {
                        return Err(MapLoadError::ValidationError(format!(
                            "LightSource entity has invalid 'range': \
                             expected positive f32, got {:?}",
                            v
                        )))
                    }
                }
            }
            if let Some(v) = entity.properties.get("shadows") {
                if !matches!(v.as_str(), "true" | "false" | "1" | "0") {
                    return Err(MapLoadError::ValidationError(format!(
                        "LightSource entity has invalid 'shadows': \
                         expected true/false/1/0, got {:?}",
                        v
                    )));
                }
            }
            if let Some(v) = entity.properties.get("color") {
                let parts: Vec<f32> = v.split(',').filter_map(|p| p.trim().parse().ok()).collect();
                if parts.len() != 3 {
                    return Err(MapLoadError::ValidationError(format!(
                        "LightSource entity has invalid 'color': \
                         expected three comma-separated f32 values, got {:?}",
                        v
                    )));
                }
            }
            if let Some(v) = entity.properties.get("flicker") {
                if !matches!(v.as_str(), "true" | "false" | "1" | "0") {
                    return Err(MapLoadError::ValidationError(format!(
                        "LightSource entity has invalid 'flicker': \
                         expected true/false/1/0, got {:?}",
                        v
                    )));
                }
            }
            if let Some(v) = entity.properties.get("flicker_amplitude") {
                match v.parse::<f32>() {
                    Ok(f) if f >= 0.0 => {}
                    _ => {
                        return Err(MapLoadError::ValidationError(format!(
                            "LightSource entity has invalid 'flicker_amplitude': \
                             expected non-negative f32, got {:?}",
                            v
                        )))
                    }
                }
            }
            if let Some(v) = entity.properties.get("flicker_speed") {
                match v.parse::<f32>() {
                    Ok(f) if f > 0.0 => {}
                    _ => {
                        return Err(MapLoadError::ValidationError(format!(
                            "LightSource entity has invalid 'flicker_speed': \
                             expected positive f32, got {:?}",
                            v
                        )))
                    }
                }
            }
        }
        EntityType::Npc => {
            if let Some(v) = entity.properties.get("radius") {
                match v.parse::<f32>() {
                    Ok(f) if f > 0.0 => {}
                    _ => {
                        return Err(MapLoadError::ValidationError(format!(
                            "Npc entity has invalid 'radius': \
                             expected positive f32, got {:?}",
                            v
                        )))
                    }
                }
            }
        }
        // Other entity types: no property validation (forward-compatible).
        _ => {}
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

/// Warns on `adrakestory:`-prefixed keys that are not in the known engine key
/// lists.
///
/// This is a soft check: the function logs a warning but never returns an
/// error, so maps with unrecognised engine keys still load. This preserves
/// forward-compatibility when an older engine reads a map written by a newer
/// engine that introduced new keys.
fn validate_custom_property_namespaces(map: &MapData) {
    let map_name = &map.metadata.name;

    // Check MapData::custom_properties
    for key in map.custom_properties.keys() {
        if key.starts_with(ENGINE_KEY_PREFIX) && !KNOWN_MAP_ENGINE_KEYS.contains(&key.as_str()) {
            warn!(
                "Map '{}': custom_properties key '{}' uses the reserved \
                 'adrakestory:' prefix but is not a known engine key. \
                 This key will be ignored.",
                map_name, key
            );
        }
    }

    // Check EntityData::properties for each entity
    for entity in &map.entities {
        for key in entity.properties.keys() {
            if key.starts_with(ENGINE_KEY_PREFIX)
                && !KNOWN_ENTITY_ENGINE_KEYS.contains(&key.as_str())
            {
                warn!(
                    "Map '{}': entity {:?} has property key '{}' using the \
                     reserved 'adrakestory:' prefix but is not a known engine \
                     key. This key will be ignored.",
                    map_name, entity.entity_type, key
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::game::map::format::*;

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
            rotation: None,
            rotation_state: None,
        });
        assert!(validate_map(&map).is_err());
    }

    #[test]
    fn test_missing_player_spawn() {
        let mut map = MapData::default_map();
        map.entities.clear();
        assert!(validate_map(&map).is_err());
    }

    #[test]
    fn test_duplicate_voxel_position() {
        let mut map = MapData::default_map();
        // Clone an existing voxel to create a duplicate at the same position.
        let duplicate = map.world.voxels[0].clone();
        map.world.voxels.push(duplicate);
        assert!(validate_map(&map).is_err());
    }

    // --- Finding 5: entity property validation ---

    fn make_light_source(props: Vec<(&str, &str)>) -> EntityData {
        EntityData {
            entity_type: EntityType::LightSource,
            position: (1.5, 0.5, 1.5),
            properties: props
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    #[test]
    fn light_source_invalid_intensity_is_rejected() {
        let mut map = MapData::default_map();
        map.entities
            .push(make_light_source(vec![("intensity", "bright")]));
        assert!(validate_map(&map).is_err());
    }

    #[test]
    fn light_source_negative_intensity_is_rejected() {
        let mut map = MapData::default_map();
        map.entities
            .push(make_light_source(vec![("intensity", "-1.0")]));
        assert!(validate_map(&map).is_err());
    }

    #[test]
    fn light_source_negative_range_is_rejected() {
        let mut map = MapData::default_map();
        map.entities
            .push(make_light_source(vec![("range", "-5.0")]));
        assert!(validate_map(&map).is_err());
    }

    #[test]
    fn light_source_zero_range_is_rejected() {
        let mut map = MapData::default_map();
        map.entities.push(make_light_source(vec![("range", "0.0")]));
        assert!(validate_map(&map).is_err());
    }

    #[test]
    fn light_source_invalid_shadows_is_rejected() {
        let mut map = MapData::default_map();
        map.entities
            .push(make_light_source(vec![("shadows", "yes")]));
        assert!(validate_map(&map).is_err());
    }

    #[test]
    fn light_source_invalid_color_is_rejected() {
        let mut map = MapData::default_map();
        map.entities.push(make_light_source(vec![("color", "red")]));
        assert!(validate_map(&map).is_err());
    }

    #[test]
    fn light_source_incomplete_color_is_rejected() {
        let mut map = MapData::default_map();
        map.entities
            .push(make_light_source(vec![("color", "1.0,0.5")]));
        assert!(validate_map(&map).is_err());
    }

    #[test]
    fn npc_invalid_radius_is_rejected() {
        let mut map = MapData::default_map();
        map.entities.push(EntityData {
            entity_type: EntityType::Npc,
            position: (1.0, 0.5, 1.0),
            properties: [("radius".to_string(), "big".to_string())].into(),
        });
        assert!(validate_map(&map).is_err());
    }

    #[test]
    fn npc_zero_radius_is_rejected() {
        let mut map = MapData::default_map();
        map.entities.push(EntityData {
            entity_type: EntityType::Npc,
            position: (1.0, 0.5, 1.0),
            properties: [("radius".to_string(), "0.0".to_string())].into(),
        });
        assert!(validate_map(&map).is_err());
    }

    #[test]
    fn light_source_valid_properties_pass() {
        let mut map = MapData::default_map();
        map.entities.push(make_light_source(vec![
            ("intensity", "5000.0"),
            ("range", "15.0"),
            ("shadows", "false"),
            ("color", "1.0, 0.9, 0.7"),
        ]));
        assert!(validate_map(&map).is_ok());
    }

    #[test]
    fn light_source_shadows_true_passes() {
        let mut map = MapData::default_map();
        map.entities
            .push(make_light_source(vec![("shadows", "true")]));
        assert!(validate_map(&map).is_ok());
    }

    #[test]
    fn light_source_shadows_numeric_passes() {
        let mut map = MapData::default_map();
        map.entities.push(make_light_source(vec![("shadows", "1")]));
        assert!(validate_map(&map).is_ok());
    }

    #[test]
    fn unknown_property_key_is_accepted() {
        let mut map = MapData::default_map();
        map.entities
            .push(make_light_source(vec![("custom_tag", "anything")]));
        assert!(validate_map(&map).is_ok());
    }

    // --- Finding 9: custom_properties namespace convention ---

    #[test]
    fn author_key_without_prefix_is_accepted() {
        let mut map = MapData::default_map();
        map.custom_properties
            .insert("my_tool:scene_id".to_string(), "42".to_string());
        // validate_custom_property_namespaces is warn-only; validate_map must succeed.
        assert!(validate_map(&map).is_ok());
    }

    #[test]
    fn unknown_engine_key_does_not_cause_error() {
        let mut map = MapData::default_map();
        map.custom_properties.insert(
            "adrakestory:unknown_future_key".to_string(),
            "value".to_string(),
        );
        // Must still return Ok — warn-only.
        assert!(validate_map(&map).is_ok());
    }

    #[test]
    fn entity_author_key_is_accepted() {
        let mut map = MapData::default_map();
        // Add to an existing entity (PlayerSpawn) to avoid disrupting required-spawn check.
        if let Some(entity) = map
            .entities
            .iter_mut()
            .find(|e| matches!(e.entity_type, EntityType::PlayerSpawn))
        {
            entity
                .properties
                .insert("my_tool:tag".to_string(), "spawn_a".to_string());
        }
        assert!(validate_map(&map).is_ok());
    }

    #[test]
    fn entity_unknown_engine_key_does_not_cause_error() {
        let mut map = MapData::default_map();
        if let Some(entity) = map
            .entities
            .iter_mut()
            .find(|e| matches!(e.entity_type, EntityType::PlayerSpawn))
        {
            entity
                .properties
                .insert("adrakestory:future_key".to_string(), "x".to_string());
        }
        assert!(validate_map(&map).is_ok());
    }

    // --- FlickerLight validation ---

    #[test]
    fn light_source_flicker_invalid_is_rejected() {
        let mut map = MapData::default_map();
        map.entities
            .push(make_light_source(vec![("flicker", "yes")]));
        assert!(validate_map(&map).is_err());
    }

    #[test]
    fn light_source_flicker_true_passes() {
        let mut map = MapData::default_map();
        map.entities
            .push(make_light_source(vec![("flicker", "true")]));
        assert!(validate_map(&map).is_ok());
    }

    #[test]
    fn light_source_flicker_one_passes() {
        let mut map = MapData::default_map();
        map.entities.push(make_light_source(vec![("flicker", "1")]));
        assert!(validate_map(&map).is_ok());
    }

    #[test]
    fn light_source_flicker_amplitude_negative_is_rejected() {
        let mut map = MapData::default_map();
        map.entities
            .push(make_light_source(vec![("flicker_amplitude", "-1.0")]));
        assert!(validate_map(&map).is_err());
    }

    #[test]
    fn light_source_flicker_amplitude_invalid_is_rejected() {
        let mut map = MapData::default_map();
        map.entities
            .push(make_light_source(vec![("flicker_amplitude", "big")]));
        assert!(validate_map(&map).is_err());
    }

    #[test]
    fn light_source_flicker_amplitude_zero_passes() {
        let mut map = MapData::default_map();
        map.entities
            .push(make_light_source(vec![("flicker_amplitude", "0.0")]));
        assert!(validate_map(&map).is_ok());
    }

    #[test]
    fn light_source_flicker_speed_zero_is_rejected() {
        let mut map = MapData::default_map();
        map.entities
            .push(make_light_source(vec![("flicker_speed", "0.0")]));
        assert!(validate_map(&map).is_err());
    }

    #[test]
    fn light_source_flicker_speed_negative_is_rejected() {
        let mut map = MapData::default_map();
        map.entities
            .push(make_light_source(vec![("flicker_speed", "-2.0")]));
        assert!(validate_map(&map).is_err());
    }

    #[test]
    fn light_source_flicker_speed_invalid_is_rejected() {
        let mut map = MapData::default_map();
        map.entities
            .push(make_light_source(vec![("flicker_speed", "fast")]));
        assert!(validate_map(&map).is_err());
    }

    #[test]
    fn light_source_flicker_speed_positive_passes() {
        let mut map = MapData::default_map();
        map.entities
            .push(make_light_source(vec![("flicker_speed", "4.0")]));
        assert!(validate_map(&map).is_ok());
    }

    #[test]
    fn light_source_all_flicker_properties_pass() {
        let mut map = MapData::default_map();
        map.entities.push(make_light_source(vec![
            ("flicker", "true"),
            ("flicker_amplitude", "3000.0"),
            ("flicker_speed", "4.0"),
        ]));
        assert!(validate_map(&map).is_ok());
    }
}
