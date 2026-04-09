//! Selection tool property panels for voxel and entity selection.

use super::entity_props::render_single_entity_properties;
use super::TransformEvents;
use crate::editor::history::EditorHistory;
use crate::editor::state::EditorState;
use crate::editor::tools::{
    ActiveTransform, CancelTransform, ConfirmTransform, DeleteSelectedVoxels, StartMoveOperation,
    StartRotateOperation, TransformMode,
};
use bevy_egui::egui;
use std::collections::HashSet;

/// Bounding box of a selection as (min, max) voxel coordinates, each component optional
/// when the selection is empty.
pub type SelectionBounds = (Option<(i32, i32, i32)>, Option<(i32, i32, i32)>);

/// Select tool content
pub fn render_select_content(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    active_transform: &ActiveTransform,
    events: &mut TransformEvents,
    history: &mut EditorHistory,
) {
    // Check if in transform mode
    if active_transform.mode != TransformMode::None {
        render_transform_mode_content(ui, active_transform, events);
        return;
    }

    // Check if entities are selected
    if !editor_state.selected_entities.is_empty() {
        render_entity_selection_content(ui, editor_state, history, events);
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
            events.confirm.write(ConfirmTransform);
        }
        if ui.button("✗ Cancel").clicked() {
            events.cancel.write(CancelTransform);
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
            events.move_start.write(StartMoveOperation);
        }
        if ui.button("↻ Rotate").on_hover_text("R").clicked() {
            events.rotate_start.write(StartRotateOperation);
        }
    });

    ui.horizontal(|ui| {
        if ui.button("🗑 Delete").on_hover_text("Delete").clicked() {
            events.delete.write(DeleteSelectedVoxels);
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
fn render_entity_selection_content(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    history: &mut EditorHistory,
    events: &mut TransformEvents,
) {
    let count = editor_state.selected_entities.len();

    if count == 1 {
        // Single entity - show full properties
        render_single_entity_properties(ui, editor_state, history, events);
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

/// Calculate selection bounds for display
pub fn calculate_selection_bounds(selected: &HashSet<(i32, i32, i32)>) -> SelectionBounds {
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
