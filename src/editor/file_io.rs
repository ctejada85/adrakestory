//! File I/O operations for the map editor.

use crate::editor::state::{EditorState, EditorUIState};
use crate::systems::game::map::format::MapData;
use bevy::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::sync::{
    mpsc::{channel, Receiver},
    Arc, Mutex,
};

/// Event sent when the user wants to save the current map
#[derive(Event)]
pub struct SaveMapEvent;

/// Event sent when the user wants to save the current map with a new name/location
#[derive(Event)]
pub struct SaveMapAsEvent;

/// Event sent when a file has been successfully saved
#[derive(Event)]
pub struct FileSavedEvent {
    pub path: PathBuf,
}

/// Resource to track the save file dialog receiver
#[derive(Resource, Default)]
pub struct SaveFileDialogReceiver {
    pub receiver: Option<Arc<Mutex<Receiver<Option<PathBuf>>>>>,
}

/// System to handle SaveMapEvent - saves to existing path or triggers Save As
pub fn handle_save_map(
    mut save_events: EventReader<SaveMapEvent>,
    editor_state: Res<EditorState>,
    mut save_as_events: EventWriter<SaveMapAsEvent>,
    mut file_saved_events: EventWriter<FileSavedEvent>,
    mut ui_state: ResMut<EditorUIState>,
) {
    for _event in save_events.read() {
        if let Some(path) = &editor_state.file_path {
            // We have a file path, save directly
            match save_map_to_file(&editor_state.current_map, path) {
                Ok(()) => {
                    info!("Map saved successfully to: {:?}", path);
                    file_saved_events.send(FileSavedEvent { path: path.clone() });
                }
                Err(e) => {
                    error!("Failed to save map: {}", e);
                    ui_state.error_message = format!("Failed to save map:\n{}", e);
                    ui_state.error_dialog_open = true;
                }
            }
        } else {
            // No file path, trigger Save As dialog
            info!("No file path set, triggering Save As dialog");
            save_as_events.send(SaveMapAsEvent);
        }
    }
}

/// System to handle SaveMapAsEvent - triggers the save file dialog
pub fn handle_save_map_as(
    mut save_as_events: EventReader<SaveMapAsEvent>,
    mut dialog_receiver: ResMut<SaveFileDialogReceiver>,
) {
    for _event in save_as_events.read() {
        // Create a channel for communication
        let (sender, receiver) = channel();
        dialog_receiver.receiver = Some(Arc::new(Mutex::new(receiver)));

        // Spawn file dialog in a separate thread to avoid blocking
        std::thread::spawn(move || {
            let result = rfd::FileDialog::new()
                .add_filter("RON Map Files", &["ron"])
                .set_title("Save Map File")
                .save_file();

            // Send result back through channel
            let _ = sender.send(result);
        });

        info!("Save file dialog opened in background thread");
    }
}

/// System to check for save file dialog results
pub fn check_save_dialog_result(
    mut dialog_receiver: ResMut<SaveFileDialogReceiver>,
    editor_state: Res<EditorState>,
    mut file_saved_events: EventWriter<FileSavedEvent>,
    mut ui_state: ResMut<EditorUIState>,
) {
    // Check if we have a receiver
    let should_clear = if let Some(receiver_arc) = &dialog_receiver.receiver {
        // Try to lock and receive without blocking
        if let Ok(receiver) = receiver_arc.lock() {
            if let Ok(result) = receiver.try_recv() {
                // Process the result
                if let Some(path) = result {
                    info!("Save file selected: {:?}", path);

                    // Save the map to the selected file
                    match save_map_to_file(&editor_state.current_map, &path) {
                        Ok(()) => {
                            info!("Map saved successfully to: {:?}", path);
                            file_saved_events.send(FileSavedEvent { path });
                        }
                        Err(e) => {
                            error!("Failed to save map: {}", e);
                            ui_state.error_message = format!("Failed to save map:\n{}", e);
                            ui_state.error_dialog_open = true;
                        }
                    }
                } else {
                    info!("Save file dialog cancelled");
                }
                true // Signal that we should clear the receiver
            } else {
                false // No result yet
            }
        } else {
            false // Failed to lock
        }
    } else {
        false // No receiver
    };

    // Clear the receiver if we got a result
    if should_clear {
        dialog_receiver.receiver = None;
    }
}

/// System to handle file saved events - updates editor state
pub fn handle_file_saved(
    mut events: EventReader<FileSavedEvent>,
    mut editor_state: ResMut<EditorState>,
) {
    for event in events.read() {
        // Update the file path and clear modified flag
        editor_state.file_path = Some(event.path.clone());
        editor_state.clear_modified();
        info!("Editor state updated after save: {:?}", event.path);
    }
}

/// Calculate the bounding box of all voxels in the map
#[cfg_attr(test, allow(dead_code))]
pub(crate) fn calculate_map_bounds(map: &MapData) -> (i32, i32, i32, i32, i32, i32) {
    if map.world.voxels.is_empty() {
        // Return default bounds for empty map
        return (0, 0, 0, 0, 0, 0);
    }

    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;
    let mut min_z = i32::MAX;
    let mut max_z = i32::MIN;

    for voxel in &map.world.voxels {
        let (x, y, z) = voxel.pos;
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
        min_z = min_z.min(z);
        max_z = max_z.max(z);
    }

    (min_x, max_x, min_y, max_y, min_z, max_z)
}

/// Normalize map coordinates to ensure all voxels start at (0, 0, 0).
///
/// This function handles maps with negative coordinates by:
/// 1. Calculating the bounding box of all voxels
/// 2. Determining the offset needed to shift minimum coordinates to (0, 0, 0)
/// 3. Applying the offset to all voxels, entities, and camera positions
/// 4. Setting map dimensions to match the actual span of voxels
///
/// This ensures saved maps are always valid and can be loaded without errors.
///
/// Returns true if normalization was performed (coordinates were shifted).
pub(crate) fn normalize_map_coordinates(map: &mut MapData) -> bool {
    // Handle empty maps
    if map.world.voxels.is_empty() {
        map.world.width = map.world.width.max(1);
        map.world.height = map.world.height.max(1);
        map.world.depth = map.world.depth.max(1);
        return false;
    }

    let (min_x, max_x, min_y, max_y, min_z, max_z) = calculate_map_bounds(map);

    // Calculate offsets needed to shift minimum coordinates to (0, 0, 0)
    // Only offset if we have negative coordinates
    let offset_x = -min_x.min(0);
    let offset_y = -min_y.min(0);
    let offset_z = -min_z.min(0);

    let needs_normalization = offset_x != 0 || offset_y != 0 || offset_z != 0;

    if needs_normalization {
        info!(
            "Normalizing map coordinates: bounds=({}, {}, {}, {}, {}, {}), offset=({}, {}, {})",
            min_x, max_x, min_y, max_y, min_z, max_z, offset_x, offset_y, offset_z
        );

        // Shift all voxel positions to normalize coordinates
        for voxel in &mut map.world.voxels {
            voxel.pos.0 += offset_x;
            voxel.pos.1 += offset_y;
            voxel.pos.2 += offset_z;
        }

        // Shift all entity positions to match voxel coordinate system
        for entity in &mut map.entities {
            entity.position.0 += offset_x as f32;
            entity.position.1 += offset_y as f32;
            entity.position.2 += offset_z as f32;
        }

        // Shift camera position and look_at point to match coordinate system
        map.camera.position.0 += offset_x as f32;
        map.camera.position.1 += offset_y as f32;
        map.camera.position.2 += offset_z as f32;

        map.camera.look_at.0 += offset_x as f32;
        map.camera.look_at.1 += offset_y as f32;
        map.camera.look_at.2 += offset_z as f32;
    }

    // Calculate dimensions based on actual span of voxels
    // After normalization, min values are guaranteed to be 0
    let required_width = (max_x - min_x + 1).max(1);
    let required_height = (max_y - min_y + 1).max(1);
    let required_depth = (max_z - min_z + 1).max(1);

    let old_width = map.world.width;
    let old_height = map.world.height;
    let old_depth = map.world.depth;

    // Set dimensions to match actual voxel span
    map.world.width = required_width;
    map.world.height = required_height;
    map.world.depth = required_depth;

    if needs_normalization {
        info!(
            "Map normalized: dimensions=({}, {}, {}) -> ({}, {}, {})",
            old_width, old_height, old_depth, map.world.width, map.world.height, map.world.depth
        );
    } else if required_width != old_width
        || required_height != old_height
        || required_depth != old_depth
    {
        info!(
            "Map dimensions adjusted: ({}, {}, {}) -> ({}, {}, {})",
            old_width, old_height, old_depth, map.world.width, map.world.height, map.world.depth
        );
    }

    needs_normalization
}

/// Save a map to a file.
///
/// This function:
/// 1. Clones the map to avoid modifying the editor state
/// 2. Normalizes coordinates to ensure all voxels start at (0, 0, 0)
/// 3. Adjusts dimensions to match the actual voxel span
/// 4. Serializes to RON format with pretty printing
/// 5. Writes to the specified file path
pub fn save_map_to_file(map: &MapData, path: &PathBuf) -> Result<(), String> {
    // Clone the map so we can modify coordinates without affecting the editor state
    let mut map_to_save = map.clone();

    // Normalize coordinates and adjust dimensions to fit all voxels
    normalize_map_coordinates(&mut map_to_save);

    // Serialize to RON format with pretty printing
    let ron_string = ron::ser::to_string_pretty(&map_to_save, ron::ser::PrettyConfig::default())
        .map_err(|e| format!("Failed to serialize map: {}", e))?;

    // Write to file
    fs::write(path, ron_string).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
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
        let mut map = create_test_map_with_voxels(vec![
            create_test_voxel(0, 0, 0),
            create_test_voxel(5, 5, 5),
        ]);

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
        let mut map = create_test_map_with_voxels(vec![
            create_test_voxel(0, 0, 0),
            create_test_voxel(9, 4, 7),
        ]);
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

        let map = create_test_map_with_voxels(vec![
            create_test_voxel(0, 0, 0),
            create_test_voxel(1, 1, 1),
        ]);

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
}
