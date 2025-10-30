//! Dialog windows for file operations and confirmations.

use crate::editor::file_io::SaveMapEvent;
use crate::editor::state::{EditorState, EditorUIState, PendingAction};
use crate::systems::game::map::format::MapData;
use bevy::prelude::*;
use bevy_egui::egui;
use std::fs;
use std::path::PathBuf;
use std::sync::{
    mpsc::{channel, Receiver},
    Arc, Mutex,
};

/// Event sent when a file is selected from the file dialog
#[derive(Event)]
pub struct FileSelectedEvent {
    pub path: PathBuf,
}

/// Event sent when map data changes (needs to be public for map_editor.rs)
#[derive(Event)]
pub struct MapDataChangedEvent;

/// Resource to track the file dialog receiver
#[derive(Resource, Default)]
pub struct FileDialogReceiver {
    pub receiver: Option<Arc<Mutex<Receiver<Option<PathBuf>>>>>,
}

/// Render all dialog windows
pub fn render_dialogs(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    ui_state: &mut EditorUIState,
    save_events: &mut EventWriter<SaveMapEvent>,
    map_changed_events: &mut EventWriter<MapDataChangedEvent>,
) {
    // Unsaved changes dialog
    if ui_state.unsaved_changes_dialog_open {
        render_unsaved_changes_dialog(ctx, editor_state, ui_state, save_events);
    }

    // New map dialog
    if ui_state.new_map_dialog_open {
        render_new_map_dialog(ctx, editor_state, ui_state, map_changed_events);
    }

    // About dialog
    if ui_state.about_dialog_open {
        render_about_dialog(ctx, ui_state);
    }

    // Keyboard shortcuts help
    if ui_state.shortcuts_help_open {
        render_shortcuts_help(ctx, ui_state);
    }

    // Error dialog
    if ui_state.error_dialog_open {
        render_error_dialog(ctx, ui_state);
    }
}

/// Render unsaved changes confirmation dialog
fn render_unsaved_changes_dialog(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    ui_state: &mut EditorUIState,
    save_events: &mut EventWriter<SaveMapEvent>,
) {
    egui::Window::new("Unsaved Changes")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("You have unsaved changes.");
            ui.label("Do you want to save before continuing?");

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    save_events.send(SaveMapEvent);
                    ui_state.unsaved_changes_dialog_open = false;
                    handle_pending_action(editor_state, ui_state);
                }

                if ui.button("Don't Save").clicked() {
                    editor_state.clear_modified();
                    ui_state.unsaved_changes_dialog_open = false;
                    handle_pending_action(editor_state, ui_state);
                }

                if ui.button("Cancel").clicked() {
                    ui_state.unsaved_changes_dialog_open = false;
                    ui_state.pending_action = None;
                }
            });
        });
}

/// Handle the pending action after unsaved changes dialog
fn handle_pending_action(_editor_state: &mut EditorState, ui_state: &mut EditorUIState) {
    if let Some(action) = ui_state.pending_action.take() {
        match action {
            PendingAction::NewMap => {
                ui_state.new_map_dialog_open = true;
            }
            PendingAction::OpenMap => {
                ui_state.file_dialog_open = true;
            }
            PendingAction::Quit => {
                info!("Quitting editor");
                // TODO: Implement quit
            }
        }
    }
}

/// Render new map dialog
fn render_new_map_dialog(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    ui_state: &mut EditorUIState,
    map_changed_events: &mut EventWriter<MapDataChangedEvent>,
) {
    egui::Window::new("New Map")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("Create a new map?");
            ui.label("This will replace the current map.");

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Create").clicked() {
                    *editor_state = EditorState::new();
                    editor_state.current_map = MapData::default_map();
                    ui_state.new_map_dialog_open = false;
                    info!("Created new map");
                    // Send event to trigger lighting update
                    map_changed_events.send(MapDataChangedEvent);
                }

                if ui.button("Cancel").clicked() {
                    ui_state.new_map_dialog_open = false;
                }
            });
        });
}

/// Render about dialog
fn render_about_dialog(ctx: &egui::Context, ui_state: &mut EditorUIState) {
    egui::Window::new("About Map Editor")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.heading("A Drake's Story - Map Editor");
            ui.separator();
            ui.label("Version: 0.1.0");
            ui.label("A voxel-based map editor for creating");
            ui.label("custom maps in RON format.");
            ui.separator();
            ui.label("Built with Bevy and bevy_egui");
            ui.separator();

            if ui.button("Close").clicked() {
                ui_state.about_dialog_open = false;
            }
        });
}

/// Render keyboard shortcuts help
fn render_shortcuts_help(ctx: &egui::Context, ui_state: &mut EditorUIState) {
    egui::Window::new("Keyboard Shortcuts")
        .collapsible(false)
        .resizable(true)
        .default_width(400.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.heading("File Operations");
            ui.label("Ctrl+N - New Map");
            ui.label("Ctrl+O - Open Map");
            ui.label("Ctrl+S - Save");
            ui.label("Ctrl+Shift+S - Save As");

            ui.separator();
            ui.heading("Edit Operations");
            ui.label("Ctrl+Z - Undo");
            ui.label("Ctrl+Y - Redo");
            ui.label("Delete/Backspace - Remove");

            ui.separator();
            ui.heading("View Controls");
            ui.label("G - Toggle Grid");
            ui.label("Shift+G - Toggle Snap");
            ui.label("Home - Reset Camera");

            ui.separator();
            ui.heading("Tools");
            ui.label("V - Select Tool");
            ui.label("B - Voxel Place Tool");
            ui.label("E - Entity Place Tool");
            ui.label("C - Camera Tool");

            ui.separator();
            ui.heading("Camera");
            ui.label("Right-click drag - Orbit");
            ui.label("Middle-click drag - Pan");
            ui.label("Scroll - Zoom");

            ui.separator();

            if ui.button("Close").clicked() {
                ui_state.shortcuts_help_open = false;
            }
        });
}

/// Render error dialog
fn render_error_dialog(ctx: &egui::Context, ui_state: &mut EditorUIState) {
    egui::Window::new("Error")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(&ui_state.error_message);
            ui.separator();

            if ui.button("OK").clicked() {
                ui_state.error_dialog_open = false;
                ui_state.error_message.clear();
            }
        });
}

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
