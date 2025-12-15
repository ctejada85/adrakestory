//! Unified input handling for the map editor.
//!
//! This module provides a centralized input handling system that replaces
//! multiple scattered input handler systems with a clean, event-driven architecture.

mod helpers;
mod keyboard;
mod transforms;

pub use helpers::{delete_selected_items, move_selected_entities, rotate_position};
pub use transforms::{
    confirm_move_internal, confirm_rotate_internal, start_move_operation_internal,
    start_rotate_operation_internal,
};

use crate::editor::history::EditorHistory;
use crate::editor::renderer::RenderMapEvent;
use crate::editor::state::{EditorState, EditorTool};
use crate::editor::tools::selection_tool::TransformPreview;
use crate::editor::tools::{ActiveTransform, TransformMode, UpdateSelectionHighlights};
use crate::systems::game::map::geometry::RotationAxis;
use bevy::prelude::*;
use bevy_egui::EguiContexts;

/// Semantic events representing user input actions in the editor
#[derive(Event, Debug, Clone)]
pub enum EditorInputEvent {
    // Selection operations
    StartMove,
    StartRotate,
    DeleteSelected,
    DeselectAll,

    // Transform operations - Move mode
    UpdateMoveOffset(IVec3),

    // Entity operations
    MoveSelectedEntities(Vec3),

    // Transform operations - Rotate mode
    SetRotationAxis(RotationAxis),
    RotateDelta(i32), // +1 or -1 for 90-degree rotations

    // Generic transform operations
    ConfirmTransform,
    CancelTransform,
}

/// Single system to handle all keyboard input for the editor
///
/// Replaces these systems:
/// - handle_delete_selected
/// - handle_move_shortcut
/// - handle_rotate_shortcut
/// - handle_arrow_key_movement
/// - handle_arrow_key_rotation
/// - handle_rotation_axis_selection
/// - handle_deselect_shortcut
pub fn handle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    editor_state: Res<EditorState>,
    active_transform: Res<ActiveTransform>,
    mut input_events: EventWriter<EditorInputEvent>,
) {
    // Single UI focus check for ALL keyboard input
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    // Only handle input when Select tool is active
    if !matches!(editor_state.active_tool, EditorTool::Select) {
        return;
    }

    // Context-aware input mapping based on current mode
    match active_transform.mode {
        TransformMode::None => {
            keyboard::handle_selection_mode_input(&keyboard, &mut input_events);
        }
        TransformMode::Move => {
            keyboard::handle_move_mode_input(&keyboard, &mut input_events);
        }
        TransformMode::Rotate => {
            keyboard::handle_rotate_mode_input(&keyboard, &mut input_events);
        }
    }
}

/// System to handle transformation operations based on input events
///
/// Replaces the logic portions of these systems:
/// - start_move_operation
/// - start_rotate_operation
/// - update_transform_preview
/// - update_rotation
/// - update_rotation_axis
/// - confirm_transform
/// - confirm_rotation
/// - cancel_transform
/// - handle_delete_selected (deletion logic)
/// - handle_deselect_shortcut (deselection logic)
pub fn handle_transformation_operations(
    mut input_events: EventReader<EditorInputEvent>,
    mut active_transform: ResMut<ActiveTransform>,
    mut editor_state: ResMut<EditorState>,
    mut history: ResMut<EditorHistory>,
    mut render_events: EventWriter<RenderMapEvent>,
    mut update_events: EventWriter<UpdateSelectionHighlights>,
    preview_query: Query<&TransformPreview>,
) {
    for event in input_events.read() {
        match event {
            EditorInputEvent::StartMove => {
                start_move_operation_internal(&mut active_transform, &editor_state);
            }

            EditorInputEvent::StartRotate => {
                start_rotate_operation_internal(&mut active_transform, &editor_state);
            }

            EditorInputEvent::DeleteSelected => {
                delete_selected_items(
                    &mut editor_state,
                    &mut history,
                    &mut render_events,
                    &mut update_events,
                );
            }

            EditorInputEvent::DeselectAll => {
                if !editor_state.selected_voxels.is_empty()
                    || !editor_state.selected_entities.is_empty()
                {
                    editor_state.clear_selections();
                    info!("Cleared all selections");
                    update_events.send(UpdateSelectionHighlights);
                }
            }

            EditorInputEvent::UpdateMoveOffset(offset) => {
                active_transform.current_offset += *offset;
                info!(
                    "Transform offset updated to: {:?}",
                    active_transform.current_offset
                );
            }

            EditorInputEvent::SetRotationAxis(axis) => {
                active_transform.rotation_axis = *axis;
                active_transform.rotation_angle = 0;
                info!(
                    "Rotation axis changed to: {:?}",
                    active_transform.rotation_axis
                );
            }

            EditorInputEvent::RotateDelta(delta) => {
                active_transform.rotation_angle =
                    (active_transform.rotation_angle + delta).rem_euclid(4);
                info!(
                    "Rotation angle updated to: {} ({}°)",
                    active_transform.rotation_angle,
                    active_transform.rotation_angle * 90
                );
            }

            EditorInputEvent::ConfirmTransform => match active_transform.mode {
                TransformMode::Move => {
                    confirm_move_internal(
                        &mut active_transform,
                        &mut editor_state,
                        &mut history,
                        &preview_query,
                        &mut render_events,
                        &mut update_events,
                    );
                }
                TransformMode::Rotate => {
                    confirm_rotate_internal(
                        &mut active_transform,
                        &mut editor_state,
                        &mut history,
                        &preview_query,
                        &mut render_events,
                        &mut update_events,
                    );
                }
                TransformMode::None => {}
            },

            EditorInputEvent::CancelTransform => {
                info!("Transform operation cancelled");
                *active_transform = ActiveTransform::default();
            }

            EditorInputEvent::MoveSelectedEntities(offset) => {
                move_selected_entities(
                    &mut editor_state,
                    &mut history,
                    &mut render_events,
                    *offset,
                );
            }
        }
    }
}
