//! Dialog window rendering functions.

use crate::editor::file_io::SaveMapEvent;
use crate::editor::recent_files::OpenRecentFileEvent;
use crate::editor::state::{EditorState, EditorUIState, PendingAction};
use crate::systems::game::map::format::MapData;
use bevy::prelude::*;
use bevy_egui::egui;

use super::events::{AppExitEvent, MapDataChangedEvent};

/// Render all dialog windows
pub fn render_dialogs(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    ui_state: &mut EditorUIState,
    save_events: &mut EventWriter<SaveMapEvent>,
    map_changed_events: &mut EventWriter<MapDataChangedEvent>,
    exit_events: &mut EventWriter<AppExitEvent>,
    open_recent_events: &mut EventWriter<OpenRecentFileEvent>,
) {
    // Unsaved changes dialog
    if ui_state.unsaved_changes_dialog_open {
        render_unsaved_changes_dialog(
            ctx,
            editor_state,
            ui_state,
            save_events,
            exit_events,
            open_recent_events,
        );
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
    exit_events: &mut EventWriter<AppExitEvent>,
    open_recent_events: &mut EventWriter<OpenRecentFileEvent>,
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
                    handle_pending_action(editor_state, ui_state, exit_events, open_recent_events);
                }

                if ui.button("Don't Save").clicked() {
                    editor_state.clear_modified();
                    ui_state.unsaved_changes_dialog_open = false;
                    handle_pending_action(editor_state, ui_state, exit_events, open_recent_events);
                }

                if ui.button("Cancel").clicked() {
                    ui_state.unsaved_changes_dialog_open = false;
                    ui_state.pending_action = None;
                }
            });
        });
}

/// Handle the pending action after unsaved changes dialog
fn handle_pending_action(
    _editor_state: &mut EditorState,
    ui_state: &mut EditorUIState,
    exit_events: &mut EventWriter<AppExitEvent>,
    open_recent_events: &mut EventWriter<OpenRecentFileEvent>,
) {
    if let Some(action) = ui_state.pending_action.take() {
        match action {
            PendingAction::NewMap => {
                ui_state.new_map_dialog_open = true;
            }
            PendingAction::OpenMap => {
                ui_state.file_dialog_open = true;
            }
            PendingAction::OpenRecentFile(path) => {
                open_recent_events.send(OpenRecentFileEvent { path });
            }
            PendingAction::Quit => {
                info!("Quitting editor");
                exit_events.send(AppExitEvent);
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
