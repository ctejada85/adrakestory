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

/// Save a map to a file
fn save_map_to_file(map: &MapData, path: &PathBuf) -> Result<(), String> {
    // Serialize to RON format with pretty printing
    let ron_string = ron::ser::to_string_pretty(map, ron::ser::PrettyConfig::default())
        .map_err(|e| format!("Failed to serialize map: {}", e))?;

    // Write to file
    fs::write(path, ron_string).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}