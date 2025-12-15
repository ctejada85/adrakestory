//! Voxel tool property panels for place and remove operations.

use crate::systems::game::components::VoxelType;
use crate::systems::game::map::format::SubVoxelPattern;
use bevy_egui::egui;

/// Voxel Place tool content
pub fn render_voxel_place_content(
    ui: &mut egui::Ui,
    voxel_type: &mut VoxelType,
    pattern: &mut SubVoxelPattern,
) {
    // Type selection with preview color
    ui.group(|ui| {
        ui.label("Voxel Type");
        ui.horizontal(|ui| {
            let color = get_voxel_color(voxel_type);
            let (rect, _) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 2.0, color);

            egui::ComboBox::from_id_salt("voxel_type_prop")
                .selected_text(format!("{:?}", voxel_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(voxel_type, VoxelType::Grass, "🟩 Grass");
                    ui.selectable_value(voxel_type, VoxelType::Dirt, "🟫 Dirt");
                    ui.selectable_value(voxel_type, VoxelType::Stone, "⬜ Stone");
                });
        });
    });

    ui.add_space(8.0);

    // Pattern selection with visual preview
    ui.group(|ui| {
        ui.label("Pattern");
        egui::ComboBox::from_id_salt("voxel_pattern_prop")
            .selected_text(get_pattern_name(pattern))
            .show_ui(ui, |ui| {
                ui.selectable_value(pattern, SubVoxelPattern::Full, "█ Full");
                ui.selectable_value(
                    pattern,
                    SubVoxelPattern::PlatformXZ,
                    "▬ Platform (Horizontal)",
                );
                ui.selectable_value(pattern, SubVoxelPattern::PlatformXY, "▐ Wall (Z-axis)");
                ui.selectable_value(pattern, SubVoxelPattern::PlatformYZ, "▌ Wall (X-axis)");
                ui.selectable_value(pattern, SubVoxelPattern::StaircaseX, "⟋ Staircase (+X)");
                ui.selectable_value(pattern, SubVoxelPattern::StaircaseNegX, "⟍ Staircase (-X)");
                ui.selectable_value(pattern, SubVoxelPattern::StaircaseZ, "⟋ Staircase (+Z)");
                ui.selectable_value(pattern, SubVoxelPattern::StaircaseNegZ, "⟍ Staircase (-Z)");
                ui.selectable_value(pattern, SubVoxelPattern::Pillar, "│ Pillar");
                ui.selectable_value(pattern, SubVoxelPattern::Fence, "┼ Fence");
            });

        ui.add_space(4.0);

        // Visual pattern preview
        render_pattern_preview(ui, pattern);
    });

    ui.add_space(8.0);

    // Shortcuts
    ui.group(|ui| {
        ui.label("Shortcuts");
        ui.small("• Click to place voxel");
        ui.small("• [ / ] to change type");
        ui.small("• Shift+[ / ] to change pattern");
    });
}

/// Render a visual preview of the voxel pattern
fn render_pattern_preview(ui: &mut egui::Ui, pattern: &SubVoxelPattern) {
    let size = 60.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let painter = ui.painter_at(rect);

    // Background
    painter.rect_filled(rect, 2.0, egui::Color32::from_gray(40));

    let cell = size / 4.0;
    let color = egui::Color32::from_rgb(100, 180, 100);
    let dark = egui::Color32::from_rgb(60, 120, 60);

    // Draw pattern visualization (simplified 2D side view)
    match pattern {
        SubVoxelPattern::Full => {
            // Full cube - draw all cells
            for y in 0..4 {
                for x in 0..4 {
                    let c = if (x + y) % 2 == 0 { color } else { dark };
                    let cell_rect = egui::Rect::from_min_size(
                        rect.min + egui::vec2(x as f32 * cell, (3 - y) as f32 * cell),
                        egui::vec2(cell - 1.0, cell - 1.0),
                    );
                    painter.rect_filled(cell_rect, 1.0, c);
                }
            }
        }
        SubVoxelPattern::PlatformXZ => {
            // Horizontal platform - bottom row only
            for x in 0..4 {
                let c = if x % 2 == 0 { color } else { dark };
                let cell_rect = egui::Rect::from_min_size(
                    rect.min + egui::vec2(x as f32 * cell, 3.0 * cell),
                    egui::vec2(cell - 1.0, cell - 1.0),
                );
                painter.rect_filled(cell_rect, 1.0, c);
            }
        }
        SubVoxelPattern::PlatformXY | SubVoxelPattern::PlatformYZ => {
            // Vertical wall
            for y in 0..4 {
                let c = if y % 2 == 0 { color } else { dark };
                let cell_rect = egui::Rect::from_min_size(
                    rect.min + egui::vec2(1.5 * cell, (3 - y) as f32 * cell),
                    egui::vec2(cell - 1.0, cell - 1.0),
                );
                painter.rect_filled(cell_rect, 1.0, c);
            }
        }
        SubVoxelPattern::StaircaseX | SubVoxelPattern::StaircaseZ => {
            // Ascending stairs
            for i in 0..4 {
                let c = if i % 2 == 0 { color } else { dark };
                let cell_rect = egui::Rect::from_min_size(
                    rect.min + egui::vec2(i as f32 * cell, (3 - i) as f32 * cell),
                    egui::vec2(cell - 1.0, cell - 1.0),
                );
                painter.rect_filled(cell_rect, 1.0, c);
            }
        }
        SubVoxelPattern::StaircaseNegX | SubVoxelPattern::StaircaseNegZ => {
            // Descending stairs
            for i in 0..4 {
                let c = if i % 2 == 0 { color } else { dark };
                let cell_rect = egui::Rect::from_min_size(
                    rect.min + egui::vec2((3 - i) as f32 * cell, (3 - i) as f32 * cell),
                    egui::vec2(cell - 1.0, cell - 1.0),
                );
                painter.rect_filled(cell_rect, 1.0, c);
            }
        }
        SubVoxelPattern::Pillar => {
            // Vertical pillar in center
            for y in 0..4 {
                let c = if y % 2 == 0 { color } else { dark };
                let cell_rect = egui::Rect::from_min_size(
                    rect.min + egui::vec2(1.5 * cell, (3 - y) as f32 * cell),
                    egui::vec2(cell - 1.0, cell - 1.0),
                );
                painter.rect_filled(cell_rect, 1.0, c);
            }
        }
        SubVoxelPattern::Fence => {
            // Fence pattern: center post with rails extending in all 4 directions (cross shape)
            // Center post
            for y in 0..4 {
                let c = if y % 2 == 0 { color } else { dark };
                let cell_rect = egui::Rect::from_min_size(
                    rect.min + egui::vec2(1.5 * cell, (3 - y) as f32 * cell),
                    egui::vec2(cell - 1.0, cell - 1.0),
                );
                painter.rect_filled(cell_rect, 1.0, c);
            }
            // Horizontal rails (top)
            let cell_rect = egui::Rect::from_min_size(
                rect.min + egui::vec2(0.0, 0.5 * cell),
                egui::vec2(cell - 1.0, cell - 1.0),
            );
            painter.rect_filled(cell_rect, 1.0, dark);
            let cell_rect = egui::Rect::from_min_size(
                rect.min + egui::vec2(3.0 * cell, 0.5 * cell),
                egui::vec2(cell - 1.0, cell - 1.0),
            );
            painter.rect_filled(cell_rect, 1.0, dark);
            // Horizontal rails (bottom)
            let cell_rect = egui::Rect::from_min_size(
                rect.min + egui::vec2(0.0, 2.5 * cell),
                egui::vec2(cell - 1.0, cell - 1.0),
            );
            painter.rect_filled(cell_rect, 1.0, color);
            let cell_rect = egui::Rect::from_min_size(
                rect.min + egui::vec2(3.0 * cell, 2.5 * cell),
                egui::vec2(cell - 1.0, cell - 1.0),
            );
            painter.rect_filled(cell_rect, 1.0, color);
        }
    }
}

/// Voxel Remove tool content
pub fn render_voxel_remove_content(ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.label("🗑️ Remove voxels by clicking");
        ui.add_space(8.0);
        ui.small("Click on a voxel to remove it.");
        ui.small("Use Select tool for bulk removal.");
    });

    ui.add_space(8.0);

    ui.group(|ui| {
        ui.label("Shortcuts");
        ui.small("• Click to remove voxel");
        ui.small("• Delete/Backspace when selected");
    });
}

/// Get color for a voxel type
pub fn get_voxel_color(voxel_type: &VoxelType) -> egui::Color32 {
    match voxel_type {
        VoxelType::Air => egui::Color32::TRANSPARENT,
        VoxelType::Grass => egui::Color32::from_rgb(76, 153, 0),
        VoxelType::Dirt => egui::Color32::from_rgb(139, 90, 43),
        VoxelType::Stone => egui::Color32::from_rgb(128, 128, 128),
    }
}

/// Get display name for a pattern
pub fn get_pattern_name(pattern: &SubVoxelPattern) -> &'static str {
    match pattern {
        SubVoxelPattern::Full => "█ Full",
        SubVoxelPattern::PlatformXZ => "▬ Platform",
        SubVoxelPattern::PlatformXY => "▐ Wall (Z)",
        SubVoxelPattern::PlatformYZ => "▌ Wall (X)",
        SubVoxelPattern::StaircaseX => "⟋ Stairs (+X)",
        SubVoxelPattern::StaircaseNegX => "⟍ Stairs (-X)",
        SubVoxelPattern::StaircaseZ => "⟋ Stairs (+Z)",
        SubVoxelPattern::StaircaseNegZ => "⟍ Stairs (-Z)",
        SubVoxelPattern::Pillar => "│ Pillar",
        SubVoxelPattern::Fence => "┼ Fence",
    }
}
