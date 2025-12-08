//! Viewport controls and status display.

use crate::editor::cursor::CursorState;
use crate::editor::state::{EditorState, EditorTool, KeyboardEditMode};
use crate::editor::tools::{ActiveTransform, TransformMode};
use bevy_egui::egui;

/// Render all viewport overlays
pub fn render_viewport_overlays(
    ctx: &egui::Context,
    editor_state: &EditorState,
    cursor_state: &CursorState,
    keyboard_mode: &KeyboardEditMode,
    active_transform: &ActiveTransform,
) {
    // Keyboard mode indicator (top-right)
    if keyboard_mode.enabled {
        render_keyboard_mode_indicator(ctx, editor_state);
    }

    // Selection info tooltip (bottom-right) - only in Select mode with selection
    if matches!(editor_state.active_tool, EditorTool::Select) {
        let has_selection =
            !editor_state.selected_voxels.is_empty() || !editor_state.selected_entities.is_empty();

        if has_selection && active_transform.mode == TransformMode::None {
            render_selection_tooltip(ctx, editor_state);
        }
    }

    // Transform operation overlay (center-bottom)
    if active_transform.mode != TransformMode::None {
        render_transform_overlay(ctx, active_transform);
    }

    // Tool hint overlay (bottom-left) - context-sensitive help
    render_tool_hint(ctx, editor_state, cursor_state, keyboard_mode);
}

/// Render keyboard mode indicator in top-right
fn render_keyboard_mode_indicator(ctx: &egui::Context, editor_state: &EditorState) {
    egui::Window::new("keyboard_mode_indicator")
        .anchor(egui::Align2::RIGHT_TOP, [-10.0, 50.0])
        .title_bar(false)
        .resizable(false)
        .frame(
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_unmultiplied(40, 80, 40, 220))
                .inner_margin(egui::Margin::same(8.0))
                .rounding(4.0),
        )
        .show(ctx, |ui| {
            ui.visuals_mut().override_text_color = Some(egui::Color32::from_rgb(150, 255, 150));

            ui.horizontal(|ui| {
                ui.label("⌨");
                ui.strong("KEYBOARD MODE");
            });

            ui.separator();

            // Show shortcuts based on current tool
            match &editor_state.active_tool {
                EditorTool::VoxelPlace { .. } => {
                    ui.label("HJKL / Arrows: Move cursor");
                    ui.label("U/O: Up/Down");
                    ui.label("Space: Place voxel");
                    ui.label("X: Remove voxel");
                }
                EditorTool::VoxelRemove => {
                    ui.label("HJKL / Arrows: Move cursor");
                    ui.label("U/O: Up/Down");
                    ui.label("Space/X: Remove voxel");
                }
                EditorTool::Select => {
                    ui.label("HJKL / Arrows: Move cursor");
                    ui.label("U/O: Up/Down");
                    ui.label("Space: Toggle select");
                    ui.label("G: Move selection");
                    ui.label("R: Rotate selection");
                }
                EditorTool::EntityPlace { .. } => {
                    ui.label("HJKL / Arrows: Move cursor");
                    ui.label("U/O: Up/Down");
                    ui.label("Space: Place entity");
                }
                EditorTool::Camera => {
                    ui.label("Camera tool active");
                    ui.label("Use mouse to control");
                }
            }

            ui.separator();
            ui.small("Press ESC to exit");
        });
}

/// Render selection info tooltip in bottom-right
fn render_selection_tooltip(ctx: &egui::Context, editor_state: &EditorState) {
    let voxel_count = editor_state.selected_voxels.len();
    let entity_count = editor_state.selected_entities.len();

    egui::Window::new("selection_tooltip")
        .anchor(egui::Align2::RIGHT_BOTTOM, [-10.0, -50.0])
        .title_bar(false)
        .resizable(false)
        .frame(
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_unmultiplied(50, 50, 80, 200))
                .inner_margin(egui::Margin::same(8.0))
                .rounding(4.0),
        )
        .show(ctx, |ui| {
            ui.visuals_mut().override_text_color = Some(egui::Color32::from_rgb(200, 200, 255));

            // Selection count
            if voxel_count > 0 && entity_count > 0 {
                ui.label(format!(
                    "🔲 {} voxel{}, {} entit{}",
                    voxel_count,
                    if voxel_count == 1 { "" } else { "s" },
                    entity_count,
                    if entity_count == 1 { "y" } else { "ies" }
                ));
            } else if voxel_count > 0 {
                ui.label(format!(
                    "🔲 {} voxel{} selected",
                    voxel_count,
                    if voxel_count == 1 { "" } else { "s" }
                ));
            } else if entity_count > 0 {
                ui.label(format!(
                    "📍 {} entit{} selected",
                    entity_count,
                    if entity_count == 1 { "y" } else { "ies" }
                ));
            }

            ui.separator();

            // Quick actions
            ui.small("G: Move  R: Rotate");
            ui.small("Del: Delete  Esc: Deselect");
        });
}

/// Render transform operation overlay
fn render_transform_overlay(ctx: &egui::Context, active_transform: &ActiveTransform) {
    egui::Window::new("transform_overlay")
        .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -80.0])
        .title_bar(false)
        .resizable(false)
        .frame(
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_unmultiplied(80, 80, 40, 230))
                .inner_margin(egui::Margin::same(12.0))
                .rounding(6.0),
        )
        .show(ctx, |ui| {
            ui.visuals_mut().override_text_color = Some(egui::Color32::from_rgb(255, 255, 200));

            match active_transform.mode {
                TransformMode::Move => {
                    let offset = active_transform.current_offset;
                    ui.horizontal(|ui| {
                        ui.strong("🔄 MOVE");
                        ui.label(format!(
                            "│ {} voxel{}",
                            active_transform.selected_voxels.len(),
                            if active_transform.selected_voxels.len() == 1 {
                                ""
                            } else {
                                "s"
                            }
                        ));
                    });
                    ui.label(format!(
                        "Offset: ({}, {}, {})",
                        offset.x, offset.y, offset.z
                    ));
                    ui.separator();
                    ui.small("Arrow keys: Move │ Shift: ×5 │ PageUp/Down: Y-axis");
                }
                TransformMode::Rotate => {
                    ui.horizontal(|ui| {
                        ui.strong("↻ ROTATE");
                        ui.label(format!(
                            "│ {} voxel{}",
                            active_transform.selected_voxels.len(),
                            if active_transform.selected_voxels.len() == 1 {
                                ""
                            } else {
                                "s"
                            }
                        ));
                    });
                    ui.label(format!(
                        "Axis: {:?} │ Angle: {}°",
                        active_transform.rotation_axis,
                        active_transform.rotation_angle * 90
                    ));
                    ui.separator();
                    ui.small("X/Y/Z: Change axis │ ←/→: Rotate 90°");
                }
                TransformMode::None => {}
            }

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::from_rgb(150, 255, 150), "ENTER: Confirm");
                ui.label("│");
                ui.colored_label(egui::Color32::from_rgb(255, 150, 150), "ESC: Cancel");
            });
        });
}

/// Render tool-specific hint in bottom-left
fn render_tool_hint(
    ctx: &egui::Context,
    editor_state: &EditorState,
    cursor_state: &CursorState,
    keyboard_mode: &KeyboardEditMode,
) {
    egui::Window::new("tool_hint")
        .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -50.0])
        .title_bar(false)
        .resizable(false)
        .frame(
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_unmultiplied(40, 40, 40, 180))
                .inner_margin(egui::Margin::same(6.0))
                .rounding(4.0),
        )
        .show(ctx, |ui| {
            ui.visuals_mut().override_text_color = Some(egui::Color32::from_rgb(200, 200, 200));

            // Cursor position
            if let Some(grid_pos) = cursor_state.grid_pos {
                ui.label(format!(
                    "📍 ({}, {}, {})",
                    grid_pos.0, grid_pos.1, grid_pos.2
                ));
            }

            // Tool-specific hints (only when not in keyboard mode - keyboard mode has its own overlay)
            if !keyboard_mode.enabled {
                ui.separator();
                match &editor_state.active_tool {
                    EditorTool::VoxelPlace {
                        voxel_type,
                        pattern,
                    } => {
                        ui.small(format!("{:?} │ {:?}", voxel_type, pattern));
                        ui.small("Click: Place │ I: Keyboard mode");
                    }
                    EditorTool::VoxelRemove => {
                        ui.small("Click: Remove voxel");
                        ui.small("I: Keyboard mode");
                    }
                    EditorTool::Select => {
                        ui.small("Click: Select │ I: Keyboard mode");
                    }
                    EditorTool::EntityPlace { entity_type } => {
                        ui.small(format!("{:?}", entity_type));
                        ui.small("Click: Place │ I: Keyboard mode");
                    }
                    EditorTool::Camera => {
                        ui.small("RMB: Orbit │ MMB: Pan");
                        ui.small("Scroll: Zoom │ Home: Reset");
                    }
                }
            }
        });
}

/// Legacy function - kept for compatibility but now calls the new overlay system
pub fn render_viewport_controls(ctx: &egui::Context) {
    // This function is deprecated in favor of render_viewport_overlays
    // Keeping empty implementation to avoid breaking changes
    let _ = ctx;
}
