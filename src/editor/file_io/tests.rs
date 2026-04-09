use super::*;
use crate::systems::game::components::VoxelType;
use crate::systems::game::map::format::{
    CameraData, EntityData, EntityType, LightingData, MapMetadata, SubVoxelPattern, VoxelData,
    WorldData,
};
use std::collections::HashMap;

fn create_test_voxel(x: i32, y: i32, z: i32) -> VoxelData {
    VoxelData {
        pos: (x, y, z),
        voxel_type: VoxelType::Grass,
        pattern: Some(SubVoxelPattern::Full),
        rotation: None,
        rotation_state: None,
    }
}

fn create_test_map_with_voxels(voxels: Vec<VoxelData>) -> MapData {
    MapData {
        metadata: MapMetadata {
            name: "Test Map".to_string(),
            author: "Test".to_string(),
            description: "".to_string(),
            version: "1.0.0".to_string(),
            created: "".to_string(),
        },
        world: WorldData {
            width: 10,
            height: 10,
            depth: 10,
            voxels,
        },
        entities: vec![],
        lighting: LightingData::default(),
        camera: CameraData::default(),
        custom_properties: HashMap::new(),
        orientations: vec![],
    }
}

// calculate_map_bounds tests
#[test]
fn test_calculate_bounds_empty_map() {
    let map = create_test_map_with_voxels(vec![]);
    let bounds = calculate_map_bounds(&map);
    assert_eq!(bounds, (0, 0, 0, 0, 0, 0));
}

#[test]
fn test_calculate_bounds_single_voxel() {
    let map = create_test_map_with_voxels(vec![create_test_voxel(5, 3, 7)]);
    let bounds = calculate_map_bounds(&map);
    assert_eq!(bounds, (5, 5, 3, 3, 7, 7));
}

#[test]
fn test_calculate_bounds_multiple_voxels() {
    let map = create_test_map_with_voxels(vec![
        create_test_voxel(0, 0, 0),
        create_test_voxel(10, 5, 8),
        create_test_voxel(3, 2, 1),
    ]);
    let bounds = calculate_map_bounds(&map);
    assert_eq!(bounds, (0, 10, 0, 5, 0, 8));
}

#[test]
fn test_calculate_bounds_negative_coordinates() {
    let map = create_test_map_with_voxels(vec![
        create_test_voxel(-5, -3, -2),
        create_test_voxel(5, 3, 2),
    ]);
    let bounds = calculate_map_bounds(&map);
    assert_eq!(bounds, (-5, 5, -3, 3, -2, 2));
}

// normalize_map_coordinates tests
#[test]
fn test_normalize_empty_map() {
    let mut map = create_test_map_with_voxels(vec![]);
    map.world.width = 0;
    map.world.height = 0;
    map.world.depth = 0;

    let normalized = normalize_map_coordinates(&mut map);

    assert!(!normalized);
    assert!(map.world.width >= 1);
    assert!(map.world.height >= 1);
    assert!(map.world.depth >= 1);
}

#[test]
fn test_normalize_positive_coordinates_no_change() {
    let mut map =
        create_test_map_with_voxels(vec![create_test_voxel(0, 0, 0), create_test_voxel(5, 5, 5)]);

    let normalized = normalize_map_coordinates(&mut map);

    assert!(!normalized);
    // Voxel positions should remain unchanged
    assert_eq!(map.world.voxels[0].pos, (0, 0, 0));
    assert_eq!(map.world.voxels[1].pos, (5, 5, 5));
}

#[test]
fn test_normalize_negative_x_coordinate() {
    let mut map = create_test_map_with_voxels(vec![
        create_test_voxel(-3, 0, 0),
        create_test_voxel(2, 0, 0),
    ]);

    let normalized = normalize_map_coordinates(&mut map);

    assert!(normalized);
    // Voxels should be shifted by +3 in X
    assert_eq!(map.world.voxels[0].pos, (0, 0, 0));
    assert_eq!(map.world.voxels[1].pos, (5, 0, 0));
}

#[test]
fn test_normalize_negative_all_coordinates() {
    let mut map = create_test_map_with_voxels(vec![
        create_test_voxel(-2, -3, -4),
        create_test_voxel(1, 2, 3),
    ]);

    let normalized = normalize_map_coordinates(&mut map);

    assert!(normalized);
    // Voxels should be shifted by (+2, +3, +4)
    assert_eq!(map.world.voxels[0].pos, (0, 0, 0));
    assert_eq!(map.world.voxels[1].pos, (3, 5, 7));
}

#[test]
fn test_normalize_updates_dimensions() {
    let mut map =
        create_test_map_with_voxels(vec![create_test_voxel(0, 0, 0), create_test_voxel(9, 4, 7)]);
    map.world.width = 100; // Wrong dimensions
    map.world.height = 100;
    map.world.depth = 100;

    normalize_map_coordinates(&mut map);

    // Dimensions should be adjusted to fit voxels
    assert_eq!(map.world.width, 10); // 0 to 9 = 10 wide
    assert_eq!(map.world.height, 5); // 0 to 4 = 5 high
    assert_eq!(map.world.depth, 8); // 0 to 7 = 8 deep
}

#[test]
fn test_normalize_shifts_entities() {
    let mut map = create_test_map_with_voxels(vec![create_test_voxel(-5, -3, -2)]);
    map.entities.push(EntityData {
        entity_type: EntityType::PlayerSpawn,
        position: (0.0, 0.0, 0.0),
        properties: HashMap::new(),
    });

    normalize_map_coordinates(&mut map);

    // Entity should be shifted by (+5, +3, +2)
    assert_eq!(map.entities[0].position, (5.0, 3.0, 2.0));
}

#[test]
fn test_normalize_shifts_camera() {
    let mut map = create_test_map_with_voxels(vec![create_test_voxel(-5, -3, -2)]);
    map.camera.position = (10.0, 10.0, 10.0);
    map.camera.look_at = (0.0, 0.0, 0.0);

    normalize_map_coordinates(&mut map);

    // Camera position and look_at should be shifted by (+5, +3, +2)
    assert_eq!(map.camera.position, (15.0, 13.0, 12.0));
    assert_eq!(map.camera.look_at, (5.0, 3.0, 2.0));
}

#[test]
fn test_normalize_single_voxel_at_negative() {
    let mut map = create_test_map_with_voxels(vec![create_test_voxel(-10, -20, -30)]);

    normalize_map_coordinates(&mut map);

    assert_eq!(map.world.voxels[0].pos, (0, 0, 0));
    assert_eq!(map.world.width, 1);
    assert_eq!(map.world.height, 1);
    assert_eq!(map.world.depth, 1);
}

// Integration test - save and reload
#[test]
fn test_save_creates_valid_ron() {
    use std::io::Read;
    use tempfile::NamedTempFile;

    let map =
        create_test_map_with_voxels(vec![create_test_voxel(0, 0, 0), create_test_voxel(1, 1, 1)]);

    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let path = temp_file.path().to_path_buf();

    let result = save_map_to_file(&map, &path);
    assert!(result.is_ok(), "Save failed: {:?}", result.err());

    // Read and verify the file contains valid RON
    let mut file = std::fs::File::open(&path).expect("Failed to open file");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Failed to read file");

    // Parse to verify it's valid
    let parsed: Result<MapData, _> = ron::from_str(&content);
    assert!(parsed.is_ok(), "Invalid RON: {:?}", parsed.err());
}

#[test]
fn test_save_preserves_voxel_count() {
    use tempfile::NamedTempFile;

    let map = create_test_map_with_voxels(vec![
        create_test_voxel(0, 0, 0),
        create_test_voxel(1, 2, 3),
        create_test_voxel(5, 5, 5),
    ]);

    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let path = temp_file.path().to_path_buf();

    save_map_to_file(&map, &path).expect("Save failed");

    let content = std::fs::read_to_string(&path).expect("Failed to read file");
    let loaded: MapData = ron::from_str(&content).expect("Failed to parse");

    assert_eq!(loaded.world.voxels.len(), 3);
}

#[test]
fn test_save_normalizes_negative_coords() {
    use tempfile::NamedTempFile;

    let map = create_test_map_with_voxels(vec![
        create_test_voxel(-5, -5, -5),
        create_test_voxel(0, 0, 0),
    ]);

    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let path = temp_file.path().to_path_buf();

    save_map_to_file(&map, &path).expect("Save failed");

    let content = std::fs::read_to_string(&path).expect("Failed to read file");
    let loaded: MapData = ron::from_str(&content).expect("Failed to parse");

    // Verify all coordinates are non-negative
    for voxel in &loaded.world.voxels {
        assert!(voxel.pos.0 >= 0, "X is negative: {}", voxel.pos.0);
        assert!(voxel.pos.1 >= 0, "Y is negative: {}", voxel.pos.1);
        assert!(voxel.pos.2 >= 0, "Z is negative: {}", voxel.pos.2);
    }
}
