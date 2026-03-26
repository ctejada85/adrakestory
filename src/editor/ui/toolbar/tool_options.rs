//! Tool-specific option rendering (dropdowns, selection info).

use crate::editor::state::{EditorState, EditorTool, ToolMemory};
use crate::systems::game::components::VoxelType;
use crate::systems::game::map::format::{EntityType, SubVoxelPattern};
use bevy::prelude::*;
use bevy_egui::egui;

/// Render context-sensitive tool options (type, pattern dropdowns)
pub fn render_tool_options(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    tool_memory: &mut ToolMemory,
) {
    match &mut editor_state.active_tool {
        EditorTool::VoxelPlace {
            voxel_type,
            pattern,
        } => {
            render_voxel_place_options(ui, voxel_type, pattern, tool_memory);
        }

        EditorTool::EntityPlace { entity_type } => {
            render_entity_place_options(ui, entity_type, tool_memory);
        }

        EditorTool::Select => {
            render_select_options(ui, editor_state);
        }

        EditorTool::VoxelRemove => {
            ui.label("Click voxels to remove");
        }

        EditorTool::Camera => {
            ui.label("RMB: Orbit | MMB: Pan | Scroll: Zoom");
        }
    }
}

/// Render voxel place tool options
fn render_voxel_place_options(
    ui: &mut egui::Ui,
    voxel_type: &mut VoxelType,
    pattern: &mut SubVoxelPattern,
    tool_memory: &mut ToolMemory,
) {
    // Voxel Type dropdown
    ui.label("Type:");
    let type_changed = egui::ComboBox::from_id_salt("toolbar_voxel_type")
        .selected_text(format!("{:?}", voxel_type))
        .width(80.0)
        .show_ui(ui, |ui| {
            let mut changed = false;
            changed |= ui
                .selectable_value(voxel_type, VoxelType::Grass, "🟩 Grass")
                .changed();
            changed |= ui
                .selectable_value(voxel_type, VoxelType::Dirt, "🟫 Dirt")
                .changed();
            changed |= ui
                .selectable_value(voxel_type, VoxelType::Stone, "⬜ Stone")
                .changed();
            changed
        })
        .inner
        .unwrap_or(false);

    // Pattern dropdown
    ui.label("Pattern:");
    let pattern_changed = egui::ComboBox::from_id_salt("toolbar_pattern")
        .selected_text(pattern_short_name(pattern))
        .width(100.0)
        .show_ui(ui, |ui| {
            let mut changed = false;
            changed |= ui
                .selectable_value(pattern, SubVoxelPattern::Full, "■ Full")
                .changed();
            changed |= ui
                .selectable_value(pattern, SubVoxelPattern::PlatformXZ, "▬ Platform")
                .changed();
            changed |= ui
                .selectable_value(pattern, SubVoxelPattern::Staircase, "⌐ Stairs")
                .changed();
            changed |= ui
                .selectable_value(pattern, SubVoxelPattern::Pillar, "│ Pillar")
                .changed();
            changed |= ui
                .selectable_value(pattern, SubVoxelPattern::PlatformXY, "▐ Wall Z")
                .changed();
            changed |= ui
                .selectable_value(pattern, SubVoxelPattern::PlatformYZ, "▌ Wall X")
                .changed();
            changed |= ui
                .selectable_value(pattern, SubVoxelPattern::Fence, "┼ Fence")
                .changed();
            changed
        })
        .inner
        .unwrap_or(false);

    // Update tool memory when parameters change
    if type_changed {
        tool_memory.voxel_type = *voxel_type;
    }
    if pattern_changed {
        tool_memory.voxel_pattern = *pattern;
    }
}

/// Render entity place tool options
fn render_entity_place_options(
    ui: &mut egui::Ui,
    entity_type: &mut EntityType,
    tool_memory: &mut ToolMemory,
) {
    ui.label("Entity:");
    let entity_changed = egui::ComboBox::from_id_salt("toolbar_entity_type")
        .selected_text(entity_type_display(entity_type))
        .width(120.0)
        .show_ui(ui, |ui| {
            let mut changed = false;
            changed |= ui
                .selectable_value(entity_type, EntityType::PlayerSpawn, "🟢 Player Spawn")
                .changed();
            changed |= ui
                .selectable_value(entity_type, EntityType::Npc, "🔵 NPC")
                .changed();
            changed |= ui
                .selectable_value(entity_type, EntityType::Enemy, "🔴 Enemy")
                .changed();
            changed |= ui
                .selectable_value(entity_type, EntityType::Item, "🟡 Item")
                .changed();
            changed |= ui
                .selectable_value(entity_type, EntityType::Trigger, "🟣 Trigger")
                .changed();
            changed |= ui
                .selectable_value(entity_type, EntityType::LightSource, "💡 Light Source")
                .changed();
            changed
        })
        .inner
        .unwrap_or(false);

    // Update tool memory when entity type changes
    if entity_changed {
        tool_memory.entity_type = *entity_type;
    }
}

/// Render select tool options
fn render_select_options(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    let voxel_count = editor_state.selected_voxels.len();
    let entity_count = editor_state.selected_entities.len();

    if voxel_count > 0 || entity_count > 0 {
        ui.label(format!(
            "Selected: {} voxel{}, {} entit{}",
            voxel_count,
            if voxel_count == 1 { "" } else { "s" },
            entity_count,
            if entity_count == 1 { "y" } else { "ies" }
        ));

        if ui
            .button("🗑️ Delete")
            .on_hover_text("Delete selected (Del)")
            .clicked()
        {
            // Deletion is handled by keyboard input system
            info!("Delete button clicked");
        }

        if ui
            .button("Clear")
            .on_hover_text("Clear selection (Esc)")
            .clicked()
        {
            editor_state.clear_selections();
        }
    } else {
        ui.label("Click to select");
    }
}

/// Get a short display name for a pattern
pub fn pattern_short_name(pattern: &SubVoxelPattern) -> &'static str {
    match pattern {
        SubVoxelPattern::Full => "Full",
        SubVoxelPattern::PlatformXZ => "Platform",
        SubVoxelPattern::PlatformXY => "Wall Z",
        SubVoxelPattern::PlatformYZ => "Wall X",
        SubVoxelPattern::Staircase => "Stairs",
        SubVoxelPattern::StaircaseNegX => "Stairs -X",
        SubVoxelPattern::StaircaseZ => "Stairs +Z",
        SubVoxelPattern::StaircaseNegZ => "Stairs -Z",
        SubVoxelPattern::Pillar => "Pillar",
        SubVoxelPattern::Fence => "Fence",
    }
}

/// Get a display string for an entity type
pub fn entity_type_display(entity_type: &EntityType) -> &'static str {
    match entity_type {
        EntityType::PlayerSpawn => "🟢 Player Spawn",
        EntityType::Npc => "🔵 NPC",
        EntityType::Enemy => "🔴 Enemy",
        EntityType::Item => "🟡 Item",
        EntityType::Trigger => "🟣 Trigger",
        EntityType::LightSource => "💡 Light Source",
    }
}
