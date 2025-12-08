//! Viewport controls and status display.

use crate::editor::cursor::CursorState;
use crate::editor::state::{EditorState, EditorTool, KeyboardEditMode};
use crate::editor::tools::{ActiveTransform, TransformMode};
use bevy_egui::egui;

/// Viewport bounds for positioning overlays
/// These values should match the panel widths in outliner.rs and properties.rs
const LEFT_PANEL_WIDTH: f32 = 200.0;
const RIGHT_PANEL_WIDTH: f32 = 280.0;
const TOP_BAR_HEIGHT: f32 = 70.0; // Menu bar + toolbar
const STATUS_BAR_HEIGHT: f32 = 30.0;

/// Render all viewport overlays
/// Overlays are positioned relative to the viewport area (between side panels)
pub fn render_viewport_overlays(
    ctx: &egui::Context,
    editor_state: &EditorState,
    cursor_state: &CursorState,
    keyboard_mode: &KeyboardEditMode,
    active_transform: &ActiveTransform,
) {
    // Calculate viewport bounds (area between side panels)
    let screen_rect = ctx.screen_rect();
    let viewport_left = LEFT_PANEL_WIDTH;
    let viewport_right = screen_rect.width() - RIGHT_PANEL_WIDTH;
    let viewport_top = TOP_BAR_HEIGHT;
    let viewport_bottom = screen_rect.height() - STATUS_BAR_HEIGHT;

    let viewport_rect = egui::Rect::from_min_max(
        egui::pos2(viewport_left, viewport_top),
        egui::pos2(viewport_right, viewport_bottom),
    );

    // Keyboard mode indicator (top-right of viewport)
    if keyboard_mode.enabled {
        render_keyboard_mode_indicator(ctx, editor_state, &viewport_rect);
    }

    // Selection info tooltip (bottom-right of viewport) - only in Select mode with selection
    if matches!(editor_state.active_tool, EditorTool::Select) {
        let has_selection =
            !editor_state.selected_voxels.is_empty() || !editor_state.selected_entities.is_empty();

        if has_selection && active_transform.mode == TransformMode::None {
            render_selection_tooltip(ctx, editor_state, &viewport_rect);
        }
    }

    // Transform operation overlay (center-bottom of viewport)
    if active_transform.mode != TransformMode::None {
        render_transform_overlay(ctx, active_transform, &viewport_rect);
    }

    // Tool hint overlay (bottom-left of viewport) - context-sensitive help
    render_tool_hint(
        ctx,
        editor_state,
        cursor_state,
        keyboard_mode,
        &viewport_rect,
    );
}

/// Render keyboard mode indicator in top-right of viewport
fn render_keyboard_mode_indicator(
    ctx: &egui::Context,
    editor_state: &EditorState,
    viewport: &egui::Rect,
) {
    let pos = egui::pos2(viewport.right() - 10.0, viewport.top() + 10.0);

    egui::Area::new(egui::Id::new("keyboard_mode_indicator"))
        .fixed_pos(pos)
        .pivot(egui::Align2::RIGHT_TOP)
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_unmultiplied(40, 80, 40, 220))
                .inner_margin(egui::Margin::same(8.0))
                .rounding(4.0)
                .show(ui, |ui| {
                    ui.visuals_mut().override_text_color =
                        Some(egui::Color32::from_rgb(150, 255, 150));

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
        });
}

/// Render selection info tooltip in bottom-right of viewport
fn render_selection_tooltip(
    ctx: &egui::Context,
    editor_state: &EditorState,
    viewport: &egui::Rect,
) {
    let voxel_count = editor_state.selected_voxels.len();
    let entity_count = editor_state.selected_entities.len();

    let pos = egui::pos2(viewport.right() - 10.0, viewport.bottom() - 10.0);

    egui::Area::new(egui::Id::new("selection_tooltip"))
        .fixed_pos(pos)
        .pivot(egui::Align2::RIGHT_BOTTOM)
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_unmultiplied(50, 50, 80, 200))
                .inner_margin(egui::Margin::same(8.0))
                .rounding(4.0)
                .show(ui, |ui| {
                    ui.visuals_mut().override_text_color =
                        Some(egui::Color32::from_rgb(200, 200, 255));

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
        });
}

/// Render transform operation overlay at center-bottom of viewport
fn render_transform_overlay(
    ctx: &egui::Context,
    active_transform: &ActiveTransform,
    viewport: &egui::Rect,
) {
    let pos = egui::pos2(viewport.center().x, viewport.bottom() - 20.0);

    egui::Area::new(egui::Id::new("transform_overlay"))
        .fixed_pos(pos)
        .pivot(egui::Align2::CENTER_BOTTOM)
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_unmultiplied(80, 80, 40, 230))
                .inner_margin(egui::Margin::same(12.0))
                .rounding(6.0)
                .show(ui, |ui| {
                    ui.visuals_mut().override_text_color =
                        Some(egui::Color32::from_rgb(255, 255, 200));

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
        });
}

/// Render tool-specific hint in bottom-left of viewport
fn render_tool_hint(
    ctx: &egui::Context,
    editor_state: &EditorState,
    cursor_state: &CursorState,
    keyboard_mode: &KeyboardEditMode,
    viewport: &egui::Rect,
) {
    let pos = egui::pos2(viewport.left() + 10.0, viewport.bottom() - 10.0);

    egui::Area::new(egui::Id::new("tool_hint"))
        .fixed_pos(pos)
        .pivot(egui::Align2::LEFT_BOTTOM)
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_unmultiplied(40, 40, 40, 180))
                .inner_margin(egui::Margin::same(6.0))
                .rounding(4.0)
                .show(ui, |ui| {
                    ui.visuals_mut().override_text_color =
                        Some(egui::Color32::from_rgb(200, 200, 200));

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
        });
}

/// Legacy function - kept for compatibility but now calls the new overlay system
pub fn render_viewport_controls(ctx: &egui::Context) {
    // This function is deprecated in favor of render_viewport_overlays
    // Keeping empty implementation to avoid breaking changes
    let _ = ctx;
}
