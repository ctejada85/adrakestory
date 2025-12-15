//! Menu bar rendering functions.

use crate::editor::file_io::{SaveMapAsEvent, SaveMapEvent};
use crate::editor::history::EditorHistory;
use crate::editor::play::{PlayMapEvent, PlayTestState, StopGameEvent};
use crate::editor::recent_files::{OpenRecentFileEvent, RecentFiles};
use crate::editor::shortcuts::{RedoEvent, UndoEvent};
use crate::editor::state::{EditorState, EditorTool, EditorUIState, PendingAction, ToolMemory};
use bevy::prelude::*;
use bevy_egui::egui;

/// Render the File menu
pub fn render_file_menu(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    ui_state: &mut EditorUIState,
    recent_files: &mut RecentFiles,
    save_events: &mut EventWriter<SaveMapEvent>,
    save_as_events: &mut EventWriter<SaveMapAsEvent>,
    open_recent_events: &mut EventWriter<OpenRecentFileEvent>,
) {
    ui.menu_button("File", |ui| {
        if ui.button("📄 New (Ctrl+N)").clicked() {
            if editor_state.is_modified {
                ui_state.unsaved_changes_dialog_open = true;
                ui_state.pending_action = Some(PendingAction::NewMap);
            } else {
                ui_state.new_map_dialog_open = true;
            }
            ui.close_menu();
        }

        if ui.button("📁 Open... (Ctrl+O)").clicked() {
            if editor_state.is_modified {
                ui_state.unsaved_changes_dialog_open = true;
                ui_state.pending_action = Some(PendingAction::OpenMap);
            } else {
                ui_state.file_dialog_open = true;
            }
            ui.close_menu();
        }

        // Recent Files submenu
        ui.menu_button("🕐 Recent Files", |ui| {
            if recent_files.is_empty() {
                ui.label("No recent files");
            } else {
                for path in recent_files.files.iter() {
                    let display_name = RecentFiles::get_display_name(path);
                    let tooltip = path.display().to_string();

                    if ui.button(&display_name).on_hover_text(&tooltip).clicked() {
                        if editor_state.is_modified {
                            ui_state.unsaved_changes_dialog_open = true;
                            ui_state.pending_action =
                                Some(PendingAction::OpenRecentFile(path.clone()));
                        } else {
                            open_recent_events.send(OpenRecentFileEvent { path: path.clone() });
                        }
                        ui.close_menu();
                    }
                }

                ui.separator();

                if ui.button("🗑 Clear Recent Files").clicked() {
                    recent_files.clear();
                    ui.close_menu();
                }
            }
        });

        ui.separator();

        if ui.button("💾 Save (Ctrl+S)").clicked() {
            save_events.send(SaveMapEvent);
            ui.close_menu();
        }

        if ui.button("💾 Save As... (Ctrl+Shift+S)").clicked() {
            save_as_events.send(SaveMapAsEvent);
            ui.close_menu();
        }

        ui.separator();

        if ui.button("🚪 Quit").clicked() {
            if editor_state.is_modified {
                ui_state.unsaved_changes_dialog_open = true;
                ui_state.pending_action = Some(PendingAction::Quit);
            } else {
                info!("Quit clicked");
            }
            ui.close_menu();
        }
    });
}

/// Render the Edit menu
pub fn render_edit_menu(
    ui: &mut egui::Ui,
    history: &EditorHistory,
    undo_events: &mut EventWriter<UndoEvent>,
    redo_events: &mut EventWriter<RedoEvent>,
) {
    ui.menu_button("Edit", |ui| {
        let can_undo = history.can_undo();
        let can_redo = history.can_redo();

        ui.add_enabled_ui(can_undo, |ui| {
            let undo_text = if let Some(desc) = history.undo_description() {
                format!("↶ Undo {} (Ctrl+Z)", desc)
            } else {
                "↶ Undo (Ctrl+Z)".to_string()
            };

            if ui.button(undo_text).clicked() {
                undo_events.send(UndoEvent);
                info!("Undo clicked");
                ui.close_menu();
            }
        });

        ui.add_enabled_ui(can_redo, |ui| {
            let redo_text = if let Some(desc) = history.redo_description() {
                format!("↷ Redo {} (Ctrl+Y)", desc)
            } else {
                "↷ Redo (Ctrl+Y)".to_string()
            };

            if ui.button(redo_text).clicked() {
                redo_events.send(RedoEvent);
                info!("Redo clicked");
                ui.close_menu();
            }
        });
    });
}

/// Render the View menu
pub fn render_view_menu(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    ui.menu_button("View", |ui| {
        if ui
            .checkbox(&mut editor_state.show_grid, "▦ Show Grid")
            .clicked()
        {
            info!("Grid visibility: {}", editor_state.show_grid);
        }

        if ui
            .checkbox(&mut editor_state.snap_to_grid, "⊞ Snap to Grid")
            .clicked()
        {
            info!("Snap to grid: {}", editor_state.snap_to_grid);
        }

        ui.separator();

        ui.label("Grid Opacity");
        ui.add(egui::Slider::new(&mut editor_state.grid_opacity, 0.0..=1.0));
    });
}

/// Render the Run menu
pub fn render_run_menu(
    ui: &mut egui::Ui,
    play_state: &PlayTestState,
    play_events: &mut EventWriter<PlayMapEvent>,
    stop_events: &mut EventWriter<StopGameEvent>,
) {
    ui.menu_button("Run", |ui| {
        if play_state.is_running {
            if ui.button("⏹ Stop Game          Shift+F5").clicked() {
                stop_events.send(StopGameEvent);
                ui.close_menu();
            }
            ui.add_enabled(false, egui::Button::new("▶ Play Map                  F5"));
        } else {
            if ui.button("▶ Play Map                  F5").clicked() {
                play_events.send(PlayMapEvent);
                ui.close_menu();
            }
            ui.add_enabled(false, egui::Button::new("⏹ Stop Game          Shift+F5"));
        }

        ui.separator();

        // Hot reload info section
        ui.label(
            egui::RichText::new("Hot Reload")
                .small()
                .color(egui::Color32::GRAY),
        );

        if play_state.is_running {
            ui.horizontal(|ui| {
                ui.label("🔄");
                ui.label(
                    egui::RichText::new("Active - saves will auto-reload")
                        .small()
                        .color(egui::Color32::from_rgb(100, 180, 100)),
                );
            });
        } else {
            ui.horizontal(|ui| {
                ui.label("⏸");
                ui.label(
                    egui::RichText::new("Inactive - start game to enable")
                        .small()
                        .color(egui::Color32::GRAY),
                );
            });
        }
    });
}

/// Render the Tools menu
pub fn render_tools_menu(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    tool_memory: &mut ToolMemory,
) {
    ui.menu_button("Tools", |ui| {
        // Helper to save current tool parameters before switching
        let save_current_params =
            |editor_state: &EditorState, tool_memory: &mut ToolMemory| match &editor_state
                .active_tool
            {
                EditorTool::VoxelPlace {
                    voxel_type,
                    pattern,
                } => {
                    tool_memory.voxel_type = *voxel_type;
                    tool_memory.voxel_pattern = *pattern;
                }
                EditorTool::EntityPlace { entity_type } => {
                    tool_memory.entity_type = *entity_type;
                }
                _ => {}
            };

        let is_select = matches!(editor_state.active_tool, EditorTool::Select);
        if ui.selectable_label(is_select, "🔲 Select (V)").clicked() {
            if !is_select {
                save_current_params(editor_state, tool_memory);
                editor_state.active_tool = EditorTool::Select;
            }
            ui.close_menu();
        }

        let is_voxel_place = matches!(editor_state.active_tool, EditorTool::VoxelPlace { .. });
        if ui
            .selectable_label(is_voxel_place, "✏️ Voxel Place (B)")
            .clicked()
        {
            if !is_voxel_place {
                save_current_params(editor_state, tool_memory);
                editor_state.active_tool = EditorTool::VoxelPlace {
                    voxel_type: tool_memory.voxel_type,
                    pattern: tool_memory.voxel_pattern,
                };
            }
            ui.close_menu();
        }

        let is_voxel_remove = matches!(editor_state.active_tool, EditorTool::VoxelRemove);
        if ui
            .selectable_label(is_voxel_remove, "🗑️ Voxel Remove (X)")
            .clicked()
        {
            if !is_voxel_remove {
                save_current_params(editor_state, tool_memory);
                editor_state.active_tool = EditorTool::VoxelRemove;
            }
            ui.close_menu();
        }

        let is_entity_place = matches!(editor_state.active_tool, EditorTool::EntityPlace { .. });
        if ui
            .selectable_label(is_entity_place, "📍 Entity Place (E)")
            .clicked()
        {
            if !is_entity_place {
                save_current_params(editor_state, tool_memory);
                editor_state.active_tool = EditorTool::EntityPlace {
                    entity_type: tool_memory.entity_type,
                };
            }
            ui.close_menu();
        }

        let is_camera = matches!(editor_state.active_tool, EditorTool::Camera);
        if ui.selectable_label(is_camera, "📷 Camera (C)").clicked() {
            if !is_camera {
                save_current_params(editor_state, tool_memory);
                editor_state.active_tool = EditorTool::Camera;
            }
            ui.close_menu();
        }
    });
}

/// Render the Help menu
pub fn render_help_menu(ui: &mut egui::Ui, ui_state: &mut EditorUIState) {
    ui.menu_button("Help", |ui| {
        if ui.button("⌨️ Keyboard Shortcuts").clicked() {
            ui_state.shortcuts_help_open = true;
            ui.close_menu();
        }

        if ui.button("ℹ️ About").clicked() {
            ui_state.about_dialog_open = true;
            ui.close_menu();
        }
    });
}
