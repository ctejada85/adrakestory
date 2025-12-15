//! Properties panel UI for editing object properties.
//!
//! This module provides tool-specific property panels with:
//! - Visual pattern preview for voxel placement
//! - Entity property editing
//! - Transform operation controls
//! - Quick action buttons

mod entity_props;
mod entity_tools;
mod selection;
mod voxel_tools;

pub use entity_props::render_single_entity_properties;
pub use entity_tools::{get_entity_icon, render_entity_place_content};
pub use selection::{calculate_selection_bounds, render_select_content};
pub use voxel_tools::{
    get_pattern_name, get_voxel_color, render_voxel_place_content, render_voxel_remove_content,
};

use crate::editor::cursor::CursorState;
use crate::editor::state::{EditorState, EditorTool};
use crate::editor::tools::{
    ActiveTransform, CancelTransform, ConfirmTransform, DeleteSelectedVoxels, StartMoveOperation,
    StartRotateOperation,
};
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
