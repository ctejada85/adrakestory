//! File dialog operations and handlers.

use crate::editor::state::{EditorState, EditorUIState};
use crate::systems::game::map::format::MapData;
use bevy::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::sync::{
    mpsc::channel,
    Arc, Mutex,
};

use super::events::{FileDialogReceiver, FileSelectedEvent, MapDataChangedEvent};

/// Handle file operations - spawns file dialog in separate thread
pub fn handle_file_operations(
    ui_state: &mut EditorUIState,
    mut dialog_receiver: ResMut<FileDialogReceiver>,
) {
    // Handle file open dialog request
    if ui_state.file_dialog_open {
        ui_state.file_dialog_open = false;

        // Create a channel for communication
        let (sender, receiver) = channel();
        dialog_receiver.receiver = Some(Arc::new(Mutex::new(receiver)));

        // Spawn file dialog in a separate thread to avoid blocking
        std::thread::spawn(move || {
            let result = rfd::FileDialog::new()
                .add_filter("RON Map Files", &["ron"])
                .set_title("Open Map File")
                .pick_file();

            // Send result back through channel
            let _ = sender.send(result);
        });

        info!("File dialog opened in background thread");
    }
}

/// System to check for file dialog results and send events
pub fn check_file_dialog_result(
    mut dialog_receiver: ResMut<FileDialogReceiver>,
    mut file_selected_events: EventWriter<FileSelectedEvent>,
) {
    // Check if we have a receiver
    let should_clear = if let Some(receiver_arc) = &dialog_receiver.receiver {
        // Try to lock and receive without blocking
        if let Ok(receiver) = receiver_arc.lock() {
            if let Ok(result) = receiver.try_recv() {
                // Process the result
                if let Some(path) = result {
                    info!("File selected: {:?}", path);
                    file_selected_events.send(FileSelectedEvent { path });
                } else {
                    info!("File dialog cancelled");
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

/// System to handle file selected events
pub fn handle_file_selected(
    mut events: EventReader<FileSelectedEvent>,
    mut editor_state: ResMut<EditorState>,
    mut ui_state: ResMut<EditorUIState>,
    mut recent_files: ResMut<crate::editor::recent_files::RecentFiles>,
    mut map_changed_events: EventWriter<MapDataChangedEvent>,
) {
    for event in events.read() {
        match load_map_from_file(&event.path) {
            Ok(map_data) => {
                info!("Successfully loaded map from: {:?}", event.path);
                editor_state.current_map = map_data;
                editor_state.file_path = Some(event.path.clone());
                editor_state.clear_modified();
                editor_state.clear_selections();
                // Update recent files
                recent_files.add(event.path.clone());
                // Send event to trigger lighting update
                map_changed_events.send(MapDataChangedEvent);
            }
            Err(e) => {
                error!("Failed to load map: {}", e);
                ui_state.error_message = format!("Failed to load map:\n{}", e);
                ui_state.error_dialog_open = true;
            }
        }
    }
}

/// Load a map from a file
fn load_map_from_file(path: &PathBuf) -> Result<MapData, String> {
    // Read file contents
    let contents = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    // Parse RON format
    let map_data: MapData =
        ron::from_str(&contents).map_err(|e| format!("Failed to parse map file: {}", e))?;

    // Validate the map
    if map_data.world.width == 0 || map_data.world.height == 0 || map_data.world.depth == 0 {
        return Err(
            "Invalid map dimensions: width, height, and depth must be greater than 0".to_string(),
        );
    }

    Ok(map_data)
}
