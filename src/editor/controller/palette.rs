//! Item palette UI for controller mode.
//!
//! Renders a full-screen inventory-style UI for selecting items to place.

use crate::editor::controller::camera::ControllerCameraMode;
use crate::editor::controller::hotbar::PaletteCategory;
use crate::editor::controller::input::ControllerEditMode;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

/// System to render the controller palette UI.
pub fn render_controller_palette(
    _mode: Res<ControllerCameraMode>,
    edit_mode: Res<ControllerEditMode>,
    mut contexts: EguiContexts,
) {
    // Mode check removed - palette works for all input methods

    // Only render if palette is open
    if !edit_mode.palette_open {
        return;
    }

    let ctx = contexts.ctx_mut();

    // Semi-transparent background
    egui::Area::new(egui::Id::new("palette_background"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .show(ctx, |ui| {
            let screen_rect = ui.ctx().screen_rect();
            ui.painter().rect_filled(
                screen_rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180),
            );
        });

    // Main palette window
    egui::Window::new("Item Palette")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .default_width(400.0)
        .show(ctx, |ui| {
            // Category tabs
            ui.horizontal(|ui| {
                for category in PaletteCategory::all() {
                    let selected = edit_mode.palette_category == *category;
                    let button = egui::Button::new(category.name())
                        .fill(if selected {
                            egui::Color32::from_rgb(60, 100, 180)
                        } else {
                            egui::Color32::from_gray(60)
                        });

                    if ui.add(button).clicked() {
                        // Category switching handled by input system
                    }
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Items grid
            let items = edit_mode.palette_items();
            let columns = 6;

            egui::Grid::new("palette_grid")
                .num_columns(columns)
                .spacing([8.0, 8.0])
                .show(ui, |ui| {
                    for (i, item) in items.iter().enumerate() {
                        let is_selected = i == edit_mode.palette_selection;

                        let frame = egui::Frame::default()
                            .fill(if is_selected {
                                egui::Color32::from_rgb(80, 140, 220)
                            } else {
                                egui::Color32::from_gray(50)
                            })
                            .inner_margin(8.0)
                            .rounding(4.0);

                        frame.show(ui, |ui| {
                            ui.set_min_size(egui::vec2(50.0, 50.0));

                            ui.vertical_centered(|ui| {
                                ui.label(
                                    egui::RichText::new(item.icon())
                                        .size(24.0)
                                        .strong(),
                                );
                                ui.small(truncate_name(&item.name(), 8));
                            });
                        });

                        if (i + 1) % columns == 0 {
                            ui.end_row();
                        }
                    }
                });

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);

            // Current hotbar display
            ui.label("Hotbar:");
            ui.horizontal(|ui| {
                for (i, slot_item) in edit_mode.hotbar.iter().enumerate() {
                    let is_current = i == edit_mode.hotbar_slot;

                    let frame = egui::Frame::default()
                        .fill(if is_current {
                            egui::Color32::from_rgb(100, 180, 100)
                        } else {
                            egui::Color32::from_gray(40)
                        })
                        .stroke(if is_current {
                            egui::Stroke::new(2.0, egui::Color32::WHITE)
                        } else {
                            egui::Stroke::NONE
                        })
                        .inner_margin(4.0)
                        .rounding(2.0);

                    frame.show(ui, |ui| {
                        ui.set_min_size(egui::vec2(32.0, 32.0));
                        ui.centered_and_justified(|ui| {
                            ui.label(slot_item.icon());
                        });
                    });
                }
            });

            ui.add_space(8.0);

            // Controls hint
            ui.separator();
            ui.horizontal(|ui| {
                ui.small("[A] Select  [B] Close  [LB/RB] Category  [D-Pad] Navigate");
            });
        });
}

/// System to render the controller HUD (hotbar, crosshair, etc.).
pub fn render_controller_hud(
    _mode: Res<ControllerCameraMode>,
    edit_mode: Res<ControllerEditMode>,
    cursor: Res<super::cursor::ControllerCursor>,
    mut contexts: EguiContexts,
) {
    // Mode check removed - HUD works for all input methods

    // Don't render HUD while palette is open
    if edit_mode.palette_open {
        return;
    }

    let ctx = contexts.ctx_mut();

    // Crosshair in center
    let screen_rect = ctx.screen_rect();
    let center = screen_rect.center();

    egui::Area::new(egui::Id::new("crosshair"))
        .fixed_pos(center - egui::vec2(10.0, 10.0))
        .show(ctx, |ui| {
            let color = if cursor.in_reach {
                egui::Color32::WHITE
            } else {
                egui::Color32::from_gray(128)
            };

            // Simple + crosshair
            let painter = ui.painter();
            let c = center;
            painter.line_segment(
                [egui::pos2(c.x - 8.0, c.y), egui::pos2(c.x + 8.0, c.y)],
                egui::Stroke::new(2.0, color),
            );
            painter.line_segment(
                [egui::pos2(c.x, c.y - 8.0), egui::pos2(c.x, c.y + 8.0)],
                egui::Stroke::new(2.0, color),
            );
        });

    // Hotbar at bottom of screen
    egui::Area::new(egui::Id::new("hotbar_hud"))
        .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -20.0))
        .show(ctx, |ui| {
            let frame = egui::Frame::default()
                .fill(egui::Color32::from_rgba_unmultiplied(30, 30, 30, 200))
                .inner_margin(8.0)
                .rounding(4.0);

            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (i, slot_item) in edit_mode.hotbar.iter().enumerate() {
                        let is_current = i == edit_mode.hotbar_slot;

                        let slot_frame = egui::Frame::default()
                            .fill(if is_current {
                                egui::Color32::from_rgb(80, 140, 80)
                            } else {
                                egui::Color32::from_gray(50)
                            })
                            .stroke(if is_current {
                                egui::Stroke::new(2.0, egui::Color32::WHITE)
                            } else {
                                egui::Stroke::new(1.0, egui::Color32::from_gray(80))
                            })
                            .inner_margin(6.0)
                            .rounding(2.0);

                        slot_frame.show(ui, |ui| {
                            ui.set_min_size(egui::vec2(36.0, 36.0));
                            ui.centered_and_justified(|ui| {
                                ui.label(
                                    egui::RichText::new(slot_item.icon())
                                        .size(20.0),
                                );
                            });
                        });
                    }
                });
            });
        });

    // Current item name and controls hint
    egui::Area::new(egui::Id::new("item_name"))
        .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -80.0))
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new(edit_mode.current_item().name())
                    .size(16.0)
                    .color(egui::Color32::WHITE),
            );
        });

    // Controls hint at bottom
    egui::Area::new(egui::Id::new("controls_hint"))
        .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -100.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.small("[LT] Remove  [RT] Place  [LB/RB] Hotbar  [Y] Palette");
            });
        });

    // Coordinates display in corner
    egui::Area::new(egui::Id::new("coords_display"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(10.0, 10.0))
        .show(ctx, |ui| {
            let frame = egui::Frame::default()
                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150))
                .inner_margin(8.0)
                .rounding(4.0);

            frame.show(ui, |ui| {
                ui.label(
                    egui::RichText::new("Controller Mode")
                        .size(14.0)
                        .color(egui::Color32::from_rgb(100, 200, 100)),
                );

                if let Some(target) = cursor.target_voxel {
                    ui.label(format!("Target: ({}, {}, {})", target.x, target.y, target.z));
                }
                if let Some(place) = cursor.placement_position {
                    ui.label(format!("Place: ({}, {}, {})", place.x, place.y, place.z));
                }
                ui.label(format!("Distance: {:.1}", cursor.distance));
            });
        });
}

/// Truncate a string to a maximum length.
fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else {
        format!("{}…", &name[..max_len - 1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_name_short() {
        assert_eq!(truncate_name("Grass", 8), "Grass");
    }

    #[test]
    fn test_truncate_name_long() {
        assert_eq!(truncate_name("VeryLongName", 8), "VeryLon…");
    }

    #[test]
    fn test_truncate_name_exact() {
        assert_eq!(truncate_name("Exactly8", 8), "Exactly8");
    }
}
