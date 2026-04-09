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
