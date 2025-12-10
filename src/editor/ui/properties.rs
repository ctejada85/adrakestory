//! Properties panel UI for editing object properties.
//!
//! Phase 5: Simplified tool-specific properties panel with:
//! - Tool-specific views with relevant controls only
//! - Visual pattern preview for voxel placement
//! - Improved entity property editing
//! - Quick action buttons
//! - Removed redundant info (now in outliner/status bar)

use crate::editor::cursor::CursorState;
use crate::editor::state::{EditorState, EditorTool};
use crate::editor::tools::{
    ActiveTransform, CancelTransform, ConfirmTransform, DeleteSelectedVoxels, StartMoveOperation,
    StartRotateOperation, TransformMode,
};
use crate::systems::game::components::VoxelType;
use crate::systems::game::map::format::{EntityType, SubVoxelPattern};
use bevy::prelude::*;
use bevy_egui::egui;

/// Bundle of event writers for transform operations
#[derive(bevy::ecs::system::SystemParam)]
pub struct TransformEvents<'w> {
    pub delete: EventWriter<'w, DeleteSelectedVoxels>,
    pub move_start: EventWriter<'w, StartMoveOperation>,
    pub rotate_start: EventWriter<'w, StartRotateOperation>,
    pub confirm: EventWriter<'w, ConfirmTransform>,
    pub cancel: EventWriter<'w, CancelTransform>,
}

/// Render the right-side properties panel
/// Simplified and tool-specific with clear sections
pub fn render_properties_panel(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    _cursor_state: &CursorState,
    active_transform: &ActiveTransform,
    events: &mut TransformEvents,
) {
    let response = egui::SidePanel::right("properties")
        .default_width(280.0)
        .min_width(200.0)
        .max_width(400.0)
        .show(ctx, |ui| {
            // Tool header with icon
            render_tool_header(ui, &editor_state.active_tool);

            ui.separator();

            // Tool-specific content
            render_tool_content(ui, editor_state, active_transform, events);
        });

    // Store panel width in egui memory for viewport overlays to use
    let panel_width = response.response.rect.width();
    ctx.memory_mut(|mem| {
        mem.data.insert_temp(
            egui::Id::new("properties").with("__panel_width"),
            panel_width,
        );
    });
}

/// Render the tool header with icon and name
fn render_tool_header(ui: &mut egui::Ui, tool: &EditorTool) {
    let (icon, name) = match tool {
        EditorTool::VoxelPlace { .. } => ("✏️", "Voxel Place"),
        EditorTool::VoxelRemove => ("🗑️", "Voxel Remove"),
        EditorTool::EntityPlace { .. } => ("📍", "Entity Place"),
        EditorTool::Select => ("🔲", "Select"),
        EditorTool::Camera => ("📷", "Camera"),
    };

    ui.horizontal(|ui| {
        ui.heading(format!("{} {}", icon, name));
    });
}

/// Render tool-specific content
fn render_tool_content(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    active_transform: &ActiveTransform,
    events: &mut TransformEvents,
) {
    match &mut editor_state.active_tool {
        EditorTool::VoxelPlace {
            voxel_type,
            pattern,
        } => {
            render_voxel_place_content(ui, voxel_type, pattern);
        }
        EditorTool::VoxelRemove => {
            render_voxel_remove_content(ui);
        }
        EditorTool::EntityPlace { entity_type } => {
            render_entity_place_content(ui, entity_type);
        }
        EditorTool::Select => {
            render_select_content(ui, editor_state, active_transform, events);
        }
        EditorTool::Camera => {
            render_camera_content(ui);
        }
    }
}

/// Voxel Place tool content
fn render_voxel_place_content(
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
fn render_voxel_remove_content(ui: &mut egui::Ui) {
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

/// Entity Place tool content
fn render_entity_place_content(ui: &mut egui::Ui, entity_type: &mut EntityType) {
    ui.group(|ui| {
        ui.label("Entity Type");
        ui.horizontal(|ui| {
            let icon = get_entity_icon(entity_type);
            ui.label(icon);

            egui::ComboBox::from_id_salt("entity_type_prop")
                .selected_text(format!("{:?}", entity_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(entity_type, EntityType::PlayerSpawn, "🟢 Player Spawn");
                    ui.selectable_value(entity_type, EntityType::Npc, "🔵 NPC");
                    ui.selectable_value(entity_type, EntityType::Enemy, "🔴 Enemy");
                    ui.selectable_value(entity_type, EntityType::Item, "🟡 Item");
                    ui.selectable_value(entity_type, EntityType::Trigger, "🟣 Trigger");
                });
        });
    });

    ui.add_space(8.0);

    // Entity type description
    ui.group(|ui| {
        let description = match entity_type {
            EntityType::PlayerSpawn => "Starting position for the player character.",
            EntityType::Npc => "Non-player character that can have dialog and interactions.",
            EntityType::Enemy => "Hostile entity that the player can fight.",
            EntityType::Item => "Collectible item or interactive object.",
            EntityType::Trigger => "Invisible trigger zone for events.",
        };
        ui.small(description);
    });

    ui.add_space(8.0);

    ui.group(|ui| {
        ui.label("Shortcuts");
        ui.small("• Click to place entity");
        ui.small("• Edit properties in Outliner");
    });
}

/// Select tool content
fn render_select_content(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    active_transform: &ActiveTransform,
    events: &mut TransformEvents,
) {
    // Check if in transform mode
    if active_transform.mode != TransformMode::None {
        render_transform_mode_content(ui, active_transform, events);
        return;
    }

    // Check if entities are selected
    if !editor_state.selected_entities.is_empty() {
        render_entity_selection_content(ui, editor_state);
        return;
    }

    // Check if voxels are selected
    if !editor_state.selected_voxels.is_empty() {
        render_voxel_selection_content(ui, editor_state, events);
        return;
    }

    // No selection
    ui.group(|ui| {
        ui.label("No Selection");
        ui.add_space(4.0);
        ui.small("Click on voxels or entities to select them.");
        ui.small("Hold Shift to add to selection.");
    });

    ui.add_space(8.0);

    ui.group(|ui| {
        ui.label("Shortcuts");
        ui.small("• Click to select");
        ui.small("• Shift+Click to multi-select");
        ui.small("• Escape to deselect all");
    });
}

/// Render content when in transform mode
fn render_transform_mode_content(
    ui: &mut egui::Ui,
    active_transform: &ActiveTransform,
    events: &mut TransformEvents,
) {
    ui.group(|ui| {
        ui.colored_label(egui::Color32::YELLOW, "🔄 Transform Active");
        ui.add_space(4.0);

        match active_transform.mode {
            TransformMode::Move => {
                ui.label("Mode: Move");
                ui.label(format!(
                    "Offset: ({}, {}, {})",
                    active_transform.current_offset.x,
                    active_transform.current_offset.y,
                    active_transform.current_offset.z
                ));
            }
            TransformMode::Rotate => {
                ui.label("Mode: Rotate");
                ui.label(format!("Axis: {:?}", active_transform.rotation_axis));
                ui.label(format!("Angle: {}°", active_transform.rotation_angle * 90));
            }
            TransformMode::None => {}
        }

        ui.label(format!(
            "Voxels: {}",
            active_transform.selected_voxels.len()
        ));
    });

    ui.add_space(8.0);

    // Action buttons
    ui.horizontal(|ui| {
        if ui.button("✓ Confirm").clicked() {
            events.confirm.send(ConfirmTransform);
        }
        if ui.button("✗ Cancel").clicked() {
            events.cancel.send(CancelTransform);
        }
    });

    ui.add_space(8.0);

    ui.group(|ui| {
        ui.label("Controls");
        ui.small("• Arrow keys: Move X/Z");
        ui.small("• PageUp/Down: Move Y");
        ui.small("• Shift: Move 5 units");
        ui.small("• Enter: Confirm");
        ui.small("• Escape: Cancel");
    });
}

/// Render content when voxels are selected
fn render_voxel_selection_content(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    events: &mut TransformEvents,
) {
    let count = editor_state.selected_voxels.len();

    ui.group(|ui| {
        ui.label(format!(
            "🔲 {} voxel{} selected",
            count,
            if count == 1 { "" } else { "s" }
        ));

        // Calculate bounds
        if let (Some(min), Some(max)) = calculate_selection_bounds(&editor_state.selected_voxels) {
            ui.add_space(4.0);
            ui.small(format!("Min: ({}, {}, {})", min.0, min.1, min.2));
            ui.small(format!("Max: ({}, {}, {})", max.0, max.1, max.2));
        }
    });

    ui.add_space(8.0);

    // Action buttons
    ui.label("Actions");
    ui.horizontal(|ui| {
        if ui.button("🔄 Move").on_hover_text("G").clicked() {
            events.move_start.send(StartMoveOperation);
        }
        if ui.button("↻ Rotate").on_hover_text("R").clicked() {
            events.rotate_start.send(StartRotateOperation);
        }
    });

    ui.horizontal(|ui| {
        if ui.button("🗑 Delete").on_hover_text("Delete").clicked() {
            events.delete.send(DeleteSelectedVoxels);
        }
        if ui.button("Clear").on_hover_text("Escape").clicked() {
            editor_state.clear_selections();
        }
    });

    ui.add_space(8.0);

    ui.group(|ui| {
        ui.label("Shortcuts");
        ui.small("• G: Start move");
        ui.small("• R: Start rotate");
        ui.small("• Delete: Remove voxels");
    });
}

/// Render content when entities are selected
fn render_entity_selection_content(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    let count = editor_state.selected_entities.len();

    if count == 1 {
        // Single entity - show full properties
        render_single_entity_properties(ui, editor_state);
    } else {
        // Multiple entities
        ui.group(|ui| {
            ui.label(format!("📍 {} entities selected", count));
        });

        ui.add_space(8.0);

        if ui.button("Clear Selection").clicked() {
            editor_state.selected_entities.clear();
        }
    }
}

/// Render properties for a single selected entity
fn render_single_entity_properties(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    let index = match editor_state.selected_entities.iter().next() {
        Some(&idx) => idx,
        None => return,
    };

    if index >= editor_state.current_map.entities.len() {
        ui.label("Invalid entity");
        return;
    }

    let entity_type = editor_state.current_map.entities[index].entity_type;
    let icon = get_entity_icon(&entity_type);

    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label(icon);
            ui.strong(format!("{:?}", entity_type));
        });
    });

    ui.add_space(8.0);

    // Position
    ui.group(|ui| {
        ui.label("Position");

        let mut position_changed = false;
        let (mut x, mut y, mut z) = editor_state.current_map.entities[index].position;

        ui.horizontal(|ui| {
            ui.label("X:");
            if ui
                .add(egui::DragValue::new(&mut x).speed(0.1).fixed_decimals(1))
                .changed()
            {
                position_changed = true;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Y:");
            if ui
                .add(egui::DragValue::new(&mut y).speed(0.1).fixed_decimals(1))
                .changed()
            {
                position_changed = true;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Z:");
            if ui
                .add(egui::DragValue::new(&mut z).speed(0.1).fixed_decimals(1))
                .changed()
            {
                position_changed = true;
            }
        });

        if position_changed {
            editor_state.current_map.entities[index].position = (x, y, z);
            editor_state.mark_modified();
        }
    });

    // Entity-specific properties
    if entity_type == EntityType::Npc {
        ui.add_space(8.0);
        render_npc_properties(ui, editor_state, index);
    }

    ui.add_space(8.0);

    // Actions
    ui.horizontal(|ui| {
        if ui.button("🗑 Delete").clicked() {
            editor_state.current_map.entities.remove(index);
            editor_state.selected_entities.clear();
            editor_state.mark_modified();
        }
        if ui.button("Clear").clicked() {
            editor_state.selected_entities.clear();
        }
    });
}

/// Render NPC-specific properties
fn render_npc_properties(ui: &mut egui::Ui, editor_state: &mut EditorState, index: usize) {
    ui.group(|ui| {
        ui.label("NPC Properties");

        // Name
        let current_name = editor_state.current_map.entities[index]
            .properties
            .get("name")
            .cloned()
            .unwrap_or_else(|| "NPC".to_string());
        let mut name = current_name;

        ui.horizontal(|ui| {
            ui.label("Name:");
            if ui.text_edit_singleline(&mut name).changed() {
                editor_state.current_map.entities[index]
                    .properties
                    .insert("name".to_string(), name);
                editor_state.mark_modified();
            }
        });

        // Radius
        let current_radius: f32 = editor_state.current_map.entities[index]
            .properties
            .get("radius")
            .and_then(|r| r.parse().ok())
            .unwrap_or(0.3);
        let mut radius = current_radius;

        ui.horizontal(|ui| {
            ui.label("Radius:");
            if ui
                .add(egui::Slider::new(&mut radius, 0.1..=1.0).step_by(0.05))
                .changed()
            {
                editor_state.current_map.entities[index]
                    .properties
                    .insert("radius".to_string(), format!("{:.2}", radius));
                editor_state.mark_modified();
            }
        });
    });
}

/// Camera tool content
fn render_camera_content(ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.label("Camera Controls");
        ui.add_space(4.0);
        ui.small("• Right-drag: Orbit");
        ui.small("• Middle-drag: Pan");
        ui.small("• Scroll: Zoom");
    });

    ui.add_space(8.0);

    ui.group(|ui| {
        ui.label("Shortcuts");
        ui.small("• Home: Reset camera");
        ui.small("• Numpad 7: Top view");
        ui.small("• Numpad 1: Front view");
    });
}

// Helper functions

fn get_voxel_color(voxel_type: &VoxelType) -> egui::Color32 {
    match voxel_type {
        VoxelType::Air => egui::Color32::TRANSPARENT,
        VoxelType::Grass => egui::Color32::from_rgb(76, 153, 0),
        VoxelType::Dirt => egui::Color32::from_rgb(139, 90, 43),
        VoxelType::Stone => egui::Color32::from_rgb(128, 128, 128),
    }
}

fn get_pattern_name(pattern: &SubVoxelPattern) -> &'static str {
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

fn get_entity_icon(entity_type: &EntityType) -> &'static str {
    match entity_type {
        EntityType::PlayerSpawn => "🟢",
        EntityType::Npc => "🔵",
        EntityType::Enemy => "🔴",
        EntityType::Item => "🟡",
        EntityType::Trigger => "🟣",
    }
}

fn calculate_selection_bounds(
    selected: &std::collections::HashSet<(i32, i32, i32)>,
) -> (Option<(i32, i32, i32)>, Option<(i32, i32, i32)>) {
    if selected.is_empty() {
        return (None, None);
    }

    let mut min = (i32::MAX, i32::MAX, i32::MAX);
    let mut max = (i32::MIN, i32::MIN, i32::MIN);

    for &(x, y, z) in selected {
        min.0 = min.0.min(x);
        min.1 = min.1.min(y);
        min.2 = min.2.min(z);
        max.0 = max.0.max(x);
        max.1 = max.1.max(y);
        max.2 = max.2.max(z);
    }

    (Some(min), Some(max))
}
