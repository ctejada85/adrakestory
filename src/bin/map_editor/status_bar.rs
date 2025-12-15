//! Status bar rendering.

use adrakestory::editor::tools::{ActiveTransform, TransformMode};
use adrakestory::editor::{state, CursorState, EditorHistory, EditorState, KeyboardEditMode};
use bevy_egui::egui;

/// Render the status bar at the bottom
pub fn render_status_bar(
    ctx: &egui::Context,
    editor_state: &EditorState,
    cursor_state: &CursorState,
    history: &EditorHistory,
    keyboard_mode: &KeyboardEditMode,
    active_transform: &ActiveTransform,
) {
    let response = egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // === Section 1: Current Tool with Icon ===
            let (tool_icon, tool_name) = get_tool_display(&editor_state.active_tool);
            ui.label(format!("{} {}", tool_icon, tool_name));

            ui.separator();

            // === Section 2: Operation Status (if transform active) ===
            if active_transform.mode != TransformMode::None {
                let mode_text = match active_transform.mode {
                    TransformMode::Move => {
                        let offset = active_transform.current_offset;
                        format!(
                            "🔄 MOVING {} voxel{} │ Offset: ({}, {}, {})",
                            active_transform.selected_voxels.len(),
                            if active_transform.selected_voxels.len() == 1 {
                                ""
                            } else {
                                "s"
                            },
                            offset.x,
                            offset.y,
                            offset.z
                        )
                    }
                    TransformMode::Rotate => {
                        format!(
                            "↻ ROTATING {} voxel{} │ Axis: {:?} │ Angle: {}°",
                            active_transform.selected_voxels.len(),
                            if active_transform.selected_voxels.len() == 1 {
                                ""
                            } else {
                                "s"
                            },
                            active_transform.rotation_axis,
                            active_transform.rotation_angle * 90
                        )
                    }
                    TransformMode::None => String::new(),
                };
                ui.colored_label(egui::Color32::YELLOW, mode_text);
                ui.label("│ ENTER: confirm, ESC: cancel");
                ui.separator();
            }

            // === Section 3: Cursor Position ===
            if let Some(grid_pos) = cursor_state.grid_pos {
                ui.label(format!(
                    "Cursor: ({}, {}, {})",
                    grid_pos.0, grid_pos.1, grid_pos.2
                ));
            } else {
                ui.label("Cursor: --");
            }

            ui.separator();

            // === Section 4: Map Statistics ===
            ui.label(format!(
                "Voxels: {}",
                editor_state.current_map.world.voxels.len()
            ));
            ui.label(format!(
                "Entities: {}",
                editor_state.current_map.entities.len()
            ));

            ui.separator();

            // === Section 5: Selection Info ===
            let voxel_sel = editor_state.selected_voxels.len();
            let entity_sel = editor_state.selected_entities.len();
            if voxel_sel > 0 || entity_sel > 0 {
                ui.label(format!("Sel: {}v {}e", voxel_sel, entity_sel));
                ui.separator();
            }

            // === Section 6: Keyboard Mode Indicator ===
            if keyboard_mode.enabled {
                ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "⌨ KEYBOARD");
                ui.separator();
            }

            // === Section 7: Modified Indicator ===
            if editor_state.is_modified {
                ui.colored_label(egui::Color32::from_rgb(255, 200, 100), "● Modified");
            } else {
                ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "✓ Saved");
            }

            // === Right-aligned: History Stats ===
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!(
                    "Undo: {} │ Redo: {}",
                    history.undo_count(),
                    history.redo_count()
                ));
            });
        });
    });

    // Store status bar height in egui memory for viewport overlays to use
    let panel_height = response.response.rect.height();
    ctx.memory_mut(|mem| {
        mem.data.insert_temp(
            egui::Id::new("status_bar").with("__panel_height"),
            panel_height,
        );
    });
}

/// Get tool icon and display name
pub fn get_tool_display(tool: &state::EditorTool) -> (&'static str, &'static str) {
    match tool {
        state::EditorTool::Select => ("🔲", "Select"),
        state::EditorTool::VoxelPlace { .. } => ("✏️", "Voxel Place"),
        state::EditorTool::VoxelRemove => ("🗑️", "Voxel Remove"),
        state::EditorTool::EntityPlace { .. } => ("📍", "Entity Place"),
        state::EditorTool::Camera => ("📷", "Camera"),
    }
}
