//! Viewport controls and status display.

use crate::editor::cursor::CursorState;
use crate::editor::renderer::EditorEntityMarker;
use crate::editor::state::{EditorState, EditorTool, KeyboardEditMode};
use crate::editor::tools::{ActiveTransform, TransformMode};
use crate::systems::game::map::format::EntityType;
use bevy::prelude::*;
use bevy_egui::egui;
use bevy_egui::EguiContexts;

/// World units above the NPC sphere center at which the label is anchored.
const LABEL_Y_OFFSET: f32 = 0.8;

/// Label font size in logical pixels — matches the in-game NPC label font size.
const LABEL_FONT_SIZE: f32 = 24.0;

/// egui named font family used for NPC name labels.
///
/// Must be registered with [`setup::setup_egui_fonts`] at startup before this
/// name can be referenced in [`egui::FontFamily::Name`].
pub const FIRA_MONO_FAMILY: &str = "FiraMono";

/// Exact name value treated as the default placeholder — labels with this name are suppressed.
const DEFAULT_NPC_NAME: &str = "NPC";

/// Default panel widths (used as fallback if egui memory doesn't have them yet)
const DEFAULT_LEFT_PANEL_WIDTH: f32 = 200.0;
const DEFAULT_RIGHT_PANEL_WIDTH: f32 = 280.0;
const DEFAULT_STATUS_BAR_HEIGHT: f32 = 26.0;

/// Get the current width of a side panel from egui's memory
fn get_panel_width(ctx: &egui::Context, panel_id: &str, default: f32) -> f32 {
    let id = egui::Id::new(panel_id);
    ctx.memory(|mem| {
        mem.data
            .get_temp::<f32>(id.with("__panel_width"))
            .unwrap_or(default)
    })
}

/// Get the current height of a top/bottom panel from egui's memory
fn get_panel_height(ctx: &egui::Context, panel_id: &str, default: f32) -> f32 {
    let id = egui::Id::new(panel_id);
    ctx.memory(|mem| {
        mem.data
            .get_temp::<f32>(id.with("__panel_height"))
            .unwrap_or(default)
    })
}

/// Render all viewport overlays
/// Overlays are positioned relative to the viewport area (between side panels)
pub fn render_viewport_overlays(
    ctx: &egui::Context,
    editor_state: &EditorState,
    cursor_state: &CursorState,
    keyboard_mode: &KeyboardEditMode,
    active_transform: &ActiveTransform,
) {
    // Get actual panel widths from egui's memory (panels store their width there when resized)
    let left_panel_width = get_panel_width(ctx, "outliner", DEFAULT_LEFT_PANEL_WIDTH);
    let right_panel_width = get_panel_width(ctx, "properties", DEFAULT_RIGHT_PANEL_WIDTH);
    let status_bar_height = get_panel_height(ctx, "status_bar", DEFAULT_STATUS_BAR_HEIGHT);

    // Calculate viewport bounds (area between side panels)
    let screen_rect = ctx.content_rect();
    let viewport_left = left_panel_width;
    let viewport_right = screen_rect.width() - right_panel_width;

    // Get top panel height from egui's available rect (toolbar is rendered first)
    let available = ctx.available_rect();
    let viewport_top = available.top();
    // Bottom accounts for status bar height
    let viewport_bottom = screen_rect.height() - status_bar_height;

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
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(40, 80, 40, 220))
                .inner_margin(egui::Margin::same(8))
                .corner_radius(4.0)
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
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(50, 50, 80, 200))
                .inner_margin(egui::Margin::same(8))
                .corner_radius(4.0)
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
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(80, 80, 40, 230))
                .inner_margin(egui::Margin::same(12))
                .corner_radius(6.0)
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
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(40, 40, 40, 180))
                .inner_margin(egui::Margin::same(6))
                .corner_radius(4.0)
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

/// Returns `true` when `name` should be displayed as a floating viewport label.
///
/// Suppresses labels for names that are absent (empty string) or equal to the
/// [`DEFAULT_NPC_NAME`] placeholder so that un-configured NPCs do not create
/// visual noise in the viewport.
fn should_show_npc_label(name: &str) -> bool {
    !name.is_empty() && name != DEFAULT_NPC_NAME
}

/// Bevy system that draws floating egui name labels above NPC entities in the 3D viewport.
///
/// Projects each NPC's world position to screen space each frame via
/// [`Camera::world_to_viewport`], then draws a [`egui::Area`] label centered
/// above the sphere.  Labels are skipped for NPCs whose name is absent, empty,
/// or equal to the default placeholder `"NPC"`.
///
/// The system must be registered **after** `ui_system::render_ui` so that the
/// egui context is already in its drawing phase when the labels are submitted.
pub fn render_npc_name_labels(
    mut contexts: EguiContexts,
    editor_state: Res<EditorState>,
    markers: Query<(&EditorEntityMarker, &GlobalTransform)>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    let ctx = contexts.ctx_mut().expect("egui context");
    let (camera_comp, camera_transform) = camera.into_inner();

    for (marker, marker_transform) in &markers {
        let index = marker.entity_index;

        // Bounds-guard: entity list may not yet match the spawned markers.
        let Some(entity_data) = editor_state.current_map.entities.get(index) else {
            continue;
        };

        // Only label NPC entities.
        if entity_data.entity_type != EntityType::Npc {
            continue;
        }

        // Resolve name, skip defaults and empty strings.
        let name = entity_data
            .properties
            .get("name")
            .map(String::as_str)
            .unwrap_or("");
        if !should_show_npc_label(name) {
            continue;
        }

        // Offset the label anchor above the sphere centre.
        let world_pos = marker_transform.translation() + Vec3::Y * LABEL_Y_OFFSET;

        // Project to screen; skip if behind the camera or outside the viewport.
        let Ok(screen_pos) = camera_comp.world_to_viewport(camera_transform, world_pos) else {
            continue;
        };

        egui::Area::new(egui::Id::new(("npc_label", index)))
            .fixed_pos(egui::pos2(screen_pos.x, screen_pos.y))
            .pivot(egui::Align2::CENTER_BOTTOM)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new(name)
                        .size(LABEL_FONT_SIZE)
                        .color(egui::Color32::WHITE)
                        .family(egui::FontFamily::Name(FIRA_MONO_FAMILY.into())),
                );
            });
    }
}

/// Legacy function - kept for compatibility but now calls the new overlay system
pub fn render_viewport_controls(ctx: &egui::Context) {
    // This function is deprecated in favor of render_viewport_overlays
    // Keeping empty implementation to avoid breaking changes
    let _ = ctx;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_npc_is_shown() {
        assert!(should_show_npc_label("Alice"));
    }

    #[test]
    fn default_placeholder_is_suppressed() {
        assert!(!should_show_npc_label("NPC"));
    }

    #[test]
    fn empty_name_is_suppressed() {
        assert!(!should_show_npc_label(""));
    }

    #[test]
    fn lowercase_npc_is_shown() {
        // Filter is case-sensitive: "npc" != "NPC"
        assert!(should_show_npc_label("npc"));
    }

    #[test]
    fn npc_with_trailing_space_is_shown() {
        // Only the exact string "NPC" is suppressed
        assert!(should_show_npc_label("NPC "));
    }
}
