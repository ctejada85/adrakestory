//! Toolbar UI with menu bar and quick actions.

use crate::editor::history::EditorHistory;
use crate::editor::state::{EditorState, EditorTool, EditorUIState};
use crate::systems::game::components::VoxelType;
use crate::systems::game::map::format::{EntityType, SubVoxelPattern};
use bevy::prelude::*;
use bevy_egui::egui;

/// Render the top toolbar with menu bar and quick actions
pub fn render_toolbar(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    ui_state: &mut EditorUIState,
    history: &EditorHistory,
) {
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        // Menu bar
        egui::menu::bar(ui, |ui| {
            // File menu
            ui.menu_button("File", |ui| {
                if ui.button("New").clicked() {
                    if editor_state.is_modified {
                        ui_state.unsaved_changes_dialog_open = true;
                        ui_state.pending_action = Some(crate::editor::state::PendingAction::NewMap);
                    } else {
                        ui_state.new_map_dialog_open = true;
                    }
                    ui.close_menu();
                }

                if ui.button("Open...").clicked() {
                    if editor_state.is_modified {
                        ui_state.unsaved_changes_dialog_open = true;
                        ui_state.pending_action =
                            Some(crate::editor::state::PendingAction::OpenMap);
                    } else {
                        ui_state.file_dialog_open = true;
                    }
                    ui.close_menu();
                }

                ui.separator();

                if ui.button("Save").clicked() {
                    // TODO: Implement save
                    info!("Save clicked");
                    ui.close_menu();
                }

                if ui.button("Save As...").clicked() {
                    // TODO: Implement save as
                    info!("Save As clicked");
                    ui.close_menu();
                }

                ui.separator();

                if ui.button("Quit").clicked() {
                    if editor_state.is_modified {
                        ui_state.unsaved_changes_dialog_open = true;
                        ui_state.pending_action = Some(crate::editor::state::PendingAction::Quit);
                    } else {
                        // TODO: Implement quit
                        info!("Quit clicked");
                    }
                    ui.close_menu();
                }
            });

            // Edit menu
            ui.menu_button("Edit", |ui| {
                let can_undo = history.can_undo();
                let can_redo = history.can_redo();

                ui.add_enabled_ui(can_undo, |ui| {
                    let undo_text = if let Some(desc) = history.undo_description() {
                        format!("Undo {}", desc)
                    } else {
                        "Undo".to_string()
                    };

                    if ui.button(undo_text).clicked() {
                        // TODO: Implement undo
                        info!("Undo clicked");
                        ui.close_menu();
                    }
                });

                ui.add_enabled_ui(can_redo, |ui| {
                    let redo_text = if let Some(desc) = history.redo_description() {
                        format!("Redo {}", desc)
                    } else {
                        "Redo".to_string()
                    };

                    if ui.button(redo_text).clicked() {
                        // TODO: Implement redo
                        info!("Redo clicked");
                        ui.close_menu();
                    }
                });
            });

            // View menu
            ui.menu_button("View", |ui| {
                if ui
                    .checkbox(&mut editor_state.show_grid, "Show Grid")
                    .clicked()
                {
                    info!("Grid visibility: {}", editor_state.show_grid);
                }

                if ui
                    .checkbox(&mut editor_state.snap_to_grid, "Snap to Grid")
                    .clicked()
                {
                    info!("Snap to grid: {}", editor_state.snap_to_grid);
                }

                ui.separator();

                ui.label("Grid Opacity");
                ui.add(egui::Slider::new(&mut editor_state.grid_opacity, 0.0..=1.0));
            });

            // Tools menu
            ui.menu_button("Tools", |ui| {
                if ui.button("Voxel Place").clicked() {
                    editor_state.active_tool = EditorTool::VoxelPlace {
                        voxel_type: VoxelType::Grass,
                        pattern: SubVoxelPattern::Full,
                    };
                    ui.close_menu();
                }

                if ui.button("Voxel Remove").clicked() {
                    editor_state.active_tool = EditorTool::VoxelRemove;
                    ui.close_menu();
                }

                if ui.button("Entity Place").clicked() {
                    editor_state.active_tool = EditorTool::EntityPlace {
                        entity_type: EntityType::PlayerSpawn,
                    };
                    ui.close_menu();
                }

                if ui.button("Select").clicked() {
                    editor_state.active_tool = EditorTool::Select;
                    ui.close_menu();
                }

                if ui.button("Camera").clicked() {
                    editor_state.active_tool = EditorTool::Camera;
                    ui.close_menu();
                }
            });

            // Help menu
            ui.menu_button("Help", |ui| {
                if ui.button("Keyboard Shortcuts").clicked() {
                    ui_state.shortcuts_help_open = true;
                    ui.close_menu();
                }

                if ui.button("About").clicked() {
                    ui_state.about_dialog_open = true;
                    ui.close_menu();
                }
            });
        });

        ui.separator();

        // Quick action buttons
        ui.horizontal(|ui| {
            if ui.button("📄 New").clicked() {
                ui_state.new_map_dialog_open = true;
            }

            if ui.button("📁 Open").clicked() {
                ui_state.file_dialog_open = true;
            }

            if ui.button("💾 Save").clicked() {
                info!("Quick save clicked");
            }

            ui.separator();

            ui.add_enabled_ui(history.can_undo(), |ui| {
                if ui.button("↶ Undo").clicked() {
                    info!("Quick undo clicked");
                }
            });

            ui.add_enabled_ui(history.can_redo(), |ui| {
                if ui.button("↷ Redo").clicked() {
                    info!("Quick redo clicked");
                }
            });

            ui.separator();

            // Tool selection buttons
            let is_voxel_place = matches!(editor_state.active_tool, EditorTool::VoxelPlace { .. });
            if ui.selectable_label(is_voxel_place, "🧱 Place").clicked() {
                editor_state.active_tool = EditorTool::VoxelPlace {
                    voxel_type: VoxelType::Grass,
                    pattern: SubVoxelPattern::Full,
                };
            }

            let is_voxel_remove = matches!(editor_state.active_tool, EditorTool::VoxelRemove);
            if ui.selectable_label(is_voxel_remove, "🗑 Remove").clicked() {
                editor_state.active_tool = EditorTool::VoxelRemove;
            }

            let is_select = matches!(editor_state.active_tool, EditorTool::Select);
            if ui.selectable_label(is_select, "👆 Select").clicked() {
                editor_state.active_tool = EditorTool::Select;
            }

            ui.separator();

            // Grid controls
            if ui.checkbox(&mut editor_state.show_grid, "Grid").clicked() {
                info!("Grid toggled: {}", editor_state.show_grid);
            }

            if ui
                .checkbox(&mut editor_state.snap_to_grid, "Snap")
                .clicked()
            {
                info!("Snap toggled: {}", editor_state.snap_to_grid);
            }
        });
    });
}
