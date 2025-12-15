//! Transform operation initialization and confirmation.

use crate::editor::history::{EditorAction, EditorHistory};
use crate::editor::renderer::RenderMapEvent;
use crate::editor::state::EditorState;
use crate::editor::tools::selection_tool::TransformPreview;
use crate::editor::tools::{ActiveTransform, TransformMode, UpdateSelectionHighlights};
use crate::systems::game::map::format::{RotationState, VoxelData};
use crate::systems::game::map::geometry::RotationAxis;
use bevy::prelude::*;

/// Start a move operation with the currently selected voxels
pub fn start_move_operation_internal(
    active_transform: &mut ActiveTransform,
    editor_state: &EditorState,
) {
    // Check if there are selected voxels
    if editor_state.selected_voxels.is_empty() {
        warn!("No voxels selected for move operation");
        return;
    }

    // Collect selected voxel data
    let mut selected_voxels = Vec::new();
    let mut sum_pos = Vec3::ZERO;

    for &pos in &editor_state.selected_voxels {
        if let Some(voxel) = editor_state
            .current_map
            .world
            .voxels
            .iter()
            .find(|v| v.pos == pos)
        {
            selected_voxels.push(voxel.clone());
            sum_pos += Vec3::new(pos.0 as f32, pos.1 as f32, pos.2 as f32);
        }
    }

    // Calculate pivot (center of selection)
    let pivot = sum_pos / selected_voxels.len() as f32;

    // Initialize transform state
    active_transform.mode = TransformMode::Move;
    active_transform.selected_voxels = selected_voxels;
    active_transform.pivot = pivot;
    active_transform.current_offset = IVec3::ZERO;

    info!(
        "Started move operation with {} voxels",
        active_transform.selected_voxels.len()
    );
}

/// Start a rotate operation with the currently selected voxels
pub fn start_rotate_operation_internal(
    active_transform: &mut ActiveTransform,
    editor_state: &EditorState,
) {
    // Check if there are selected voxels
    if editor_state.selected_voxels.is_empty() {
        warn!("No voxels selected for rotate operation");
        return;
    }

    // Collect selected voxel data
    let mut selected_voxels = Vec::new();
    let mut sum_pos = Vec3::ZERO;

    for &pos in &editor_state.selected_voxels {
        if let Some(voxel) = editor_state
            .current_map
            .world
            .voxels
            .iter()
            .find(|v| v.pos == pos)
        {
            selected_voxels.push(voxel.clone());
            sum_pos += Vec3::new(pos.0 as f32, pos.1 as f32, pos.2 as f32);
        }
    }

    // Calculate pivot (center of selection)
    let pivot = sum_pos / selected_voxels.len() as f32;

    // Initialize transform state
    active_transform.mode = TransformMode::Rotate;
    active_transform.selected_voxels = selected_voxels;
    active_transform.pivot = pivot;
    active_transform.rotation_axis = RotationAxis::Y; // Default to Y axis
    active_transform.rotation_angle = 0;

    info!(
        "Started rotate operation with {} voxels around {:?} axis",
        active_transform.selected_voxels.len(),
        active_transform.rotation_axis
    );
}

/// Confirm and apply a move operation
pub fn confirm_move_internal(
    active_transform: &mut ActiveTransform,
    editor_state: &mut EditorState,
    history: &mut EditorHistory,
    preview_query: &Query<&TransformPreview>,
    render_events: &mut EventWriter<RenderMapEvent>,
    update_events: &mut EventWriter<UpdateSelectionHighlights>,
) {
    // Check if all previews are valid (no collisions)
    let has_collision = preview_query.iter().any(|p| !p.is_valid);
    if has_collision {
        warn!("Cannot confirm move: collision detected");
        return;
    }

    // Apply the transformation
    let offset = active_transform.current_offset;
    let mut moved_voxels = Vec::new();

    for voxel in &active_transform.selected_voxels {
        let old_pos = voxel.pos;
        let new_pos = (
            old_pos.0 + offset.x,
            old_pos.1 + offset.y,
            old_pos.2 + offset.z,
        );

        // Find and update voxel in map
        if let Some(map_voxel) = editor_state
            .current_map
            .world
            .voxels
            .iter_mut()
            .find(|v| v.pos == old_pos)
        {
            map_voxel.pos = new_pos;
            moved_voxels.push((old_pos, new_pos));
        }
    }

    // Create history action
    if !moved_voxels.is_empty() {
        history.push(EditorAction::Batch {
            description: format!(
                "Move {} voxel{}",
                moved_voxels.len(),
                if moved_voxels.len() == 1 { "" } else { "s" }
            ),
            actions: moved_voxels
                .iter()
                .map(|(old_pos, new_pos)| {
                    let voxel_data = active_transform
                        .selected_voxels
                        .iter()
                        .find(|v| v.pos == *old_pos)
                        .unwrap()
                        .clone();

                    // Create a remove and place action pair
                    EditorAction::Batch {
                        description: format!("Move voxel from {:?} to {:?}", old_pos, new_pos),
                        actions: vec![
                            EditorAction::RemoveVoxel {
                                pos: *old_pos,
                                data: voxel_data.clone(),
                            },
                            EditorAction::PlaceVoxel {
                                pos: *new_pos,
                                data: VoxelData {
                                    pos: *new_pos,
                                    ..voxel_data
                                },
                            },
                        ],
                    }
                })
                .collect(),
        });

        editor_state.mark_modified();
        info!("Moved {} voxels by offset {:?}", moved_voxels.len(), offset);
    }

    // Update selection to new positions
    editor_state.selected_voxels.clear();
    for (_, new_pos) in moved_voxels {
        editor_state.selected_voxels.insert(new_pos);
    }

    // Reset transform state
    *active_transform = ActiveTransform::default();

    // Trigger updates
    render_events.send(RenderMapEvent);
    update_events.send(UpdateSelectionHighlights);
}

/// Confirm and apply a rotate operation
pub fn confirm_rotate_internal(
    active_transform: &mut ActiveTransform,
    editor_state: &mut EditorState,
    history: &mut EditorHistory,
    preview_query: &Query<&TransformPreview>,
    render_events: &mut EventWriter<RenderMapEvent>,
    update_events: &mut EventWriter<UpdateSelectionHighlights>,
) {
    // Check if all previews are valid (no collisions)
    let has_collision = preview_query.iter().any(|p| !p.is_valid);
    if has_collision {
        warn!("Cannot confirm rotation: collision detected");
        return;
    }

    // Apply the rotation
    let mut rotated_voxels = Vec::new();

    for voxel in &active_transform.selected_voxels {
        let old_pos = voxel.pos;
        let new_pos = super::helpers::rotate_position(
            old_pos,
            active_transform.pivot,
            active_transform.rotation_axis,
            active_transform.rotation_angle,
        );

        // Find and update voxel in map
        if let Some(map_voxel) = editor_state
            .current_map
            .world
            .voxels
            .iter_mut()
            .find(|v| v.pos == old_pos)
        {
            map_voxel.pos = new_pos;

            // Update or create rotation state for the voxel
            let new_rotation_state = if let Some(existing_rotation) = map_voxel.rotation_state {
                existing_rotation.compose(
                    active_transform.rotation_axis,
                    active_transform.rotation_angle,
                )
            } else {
                RotationState::new(
                    active_transform.rotation_axis,
                    active_transform.rotation_angle,
                )
            };

            map_voxel.rotation_state = Some(new_rotation_state);

            rotated_voxels.push((old_pos, new_pos));
        }
    }

    // Create history action
    if !rotated_voxels.is_empty() {
        history.push(EditorAction::Batch {
            description: format!(
                "Rotate {} voxel{} {}° around {:?} axis",
                rotated_voxels.len(),
                if rotated_voxels.len() == 1 { "" } else { "s" },
                active_transform.rotation_angle * 90,
                active_transform.rotation_axis
            ),
            actions: rotated_voxels
                .iter()
                .map(|(old_pos, new_pos)| {
                    let voxel_data = active_transform
                        .selected_voxels
                        .iter()
                        .find(|v| v.pos == *old_pos)
                        .unwrap()
                        .clone();

                    // Update rotation state for the new voxel
                    let new_rotation_state =
                        Some(if let Some(existing_rotation) = voxel_data.rotation_state {
                            existing_rotation.compose(
                                active_transform.rotation_axis,
                                active_transform.rotation_angle,
                            )
                        } else {
                            RotationState::new(
                                active_transform.rotation_axis,
                                active_transform.rotation_angle,
                            )
                        });

                    // Create a remove and place action pair
                    EditorAction::Batch {
                        description: format!("Rotate voxel from {:?} to {:?}", old_pos, new_pos),
                        actions: vec![
                            EditorAction::RemoveVoxel {
                                pos: *old_pos,
                                data: voxel_data.clone(),
                            },
                            EditorAction::PlaceVoxel {
                                pos: *new_pos,
                                data: VoxelData {
                                    pos: *new_pos,
                                    rotation_state: new_rotation_state,
                                    ..voxel_data
                                },
                            },
                        ],
                    }
                })
                .collect(),
        });

        editor_state.mark_modified();
        info!(
            "Rotated {} voxels {}° around {:?} axis",
            rotated_voxels.len(),
            active_transform.rotation_angle * 90,
            active_transform.rotation_axis
        );
    }

    // Update selection to new positions
    editor_state.selected_voxels.clear();
    for (_, new_pos) in rotated_voxels {
        editor_state.selected_voxels.insert(new_pos);
    }

    // Reset transform state
    *active_transform = ActiveTransform::default();

    // Trigger updates
    render_events.send(RenderMapEvent);
    update_events.send(UpdateSelectionHighlights);
}
