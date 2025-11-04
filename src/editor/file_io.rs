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
fn calculate_map_bounds(map: &MapData) -> (i32, i32, i32, i32, i32, i32) {
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
fn normalize_map_coordinates(map: &mut MapData) -> bool {
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
fn save_map_to_file(map: &MapData, path: &PathBuf) -> Result<(), String> {
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
