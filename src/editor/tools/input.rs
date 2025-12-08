//! Unified input handling for the map editor.
//!
//! This module provides a centralized input handling system that replaces
//! multiple scattered input handler systems with a clean, event-driven architecture.

use crate::editor::history::EditorHistory;
use crate::editor::renderer::RenderMapEvent;
use crate::editor::state::{EditorState, EditorTool};
use crate::editor::tools::{ActiveTransform, TransformMode, UpdateSelectionHighlights};
use crate::systems::game::map::format::VoxelData;
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
            handle_selection_mode_input(&keyboard, &mut input_events);
        }
        TransformMode::Move => {
            handle_move_mode_input(&keyboard, &mut input_events);
        }
        TransformMode::Rotate => {
            handle_rotate_mode_input(&keyboard, &mut input_events);
        }
    }
}

/// Handle input when in selection mode (no active transformation)
fn handle_selection_mode_input(
    keyboard: &ButtonInput<KeyCode>,
    events: &mut EventWriter<EditorInputEvent>,
) {
    // Start operations
    if keyboard.just_pressed(KeyCode::KeyG) {
        events.send(EditorInputEvent::StartMove);
    }
    if keyboard.just_pressed(KeyCode::KeyR) {
        events.send(EditorInputEvent::StartRotate);
    }

    // Delete selected
    if keyboard.just_pressed(KeyCode::Delete) || keyboard.just_pressed(KeyCode::Backspace) {
        events.send(EditorInputEvent::DeleteSelected);
    }

    // Deselect all
    if keyboard.just_pressed(KeyCode::Escape) {
        events.send(EditorInputEvent::DeselectAll);
    }

    // Entity movement with arrow keys (grid-aligned movement for entities)
    // Entities should move in full grid units to stay aligned with voxels
    let step = if keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        5.0 // Move 5 grid units with Shift
    } else {
        1.0 // Move 1 grid unit normally
    };

    let mut offset = Vec3::ZERO;
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        offset.z -= step;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        offset.z += step;
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        offset.x -= step;
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        offset.x += step;
    }
    if keyboard.just_pressed(KeyCode::PageUp) {
        offset.y += step;
    }
    if keyboard.just_pressed(KeyCode::PageDown) {
        offset.y -= step;
    }

    if offset != Vec3::ZERO {
        events.send(EditorInputEvent::MoveSelectedEntities(offset));
    }
}

/// Handle input during move operation
fn handle_move_mode_input(
    keyboard: &ButtonInput<KeyCode>,
    events: &mut EventWriter<EditorInputEvent>,
) {
    // Calculate movement step (1 or 5 with Shift)
    let step = if keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        5
    } else {
        1
    };

    // Arrow key movement
    let mut offset = IVec3::ZERO;
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        offset.z -= step;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        offset.z += step;
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        offset.x -= step;
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        offset.x += step;
    }

    // Y-axis movement - Space for jump/up, C for crouch/down
    if keyboard.just_pressed(KeyCode::PageUp) || keyboard.just_pressed(KeyCode::Space) {
        offset.y += step;
    }
    if keyboard.just_pressed(KeyCode::PageDown) || keyboard.just_pressed(KeyCode::KeyC) {
        offset.y -= step;
    }

    // Send offset update if any movement occurred
    if offset != IVec3::ZERO {
        events.send(EditorInputEvent::UpdateMoveOffset(offset));
    }

    // Confirm or cancel
    if keyboard.just_pressed(KeyCode::Enter) {
        events.send(EditorInputEvent::ConfirmTransform);
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        events.send(EditorInputEvent::CancelTransform);
    }
}

/// Handle input during rotate operation
fn handle_rotate_mode_input(
    keyboard: &ButtonInput<KeyCode>,
    events: &mut EventWriter<EditorInputEvent>,
) {
    // Axis selection
    if keyboard.just_pressed(KeyCode::KeyX) {
        events.send(EditorInputEvent::SetRotationAxis(RotationAxis::X));
    }
    if keyboard.just_pressed(KeyCode::KeyY) {
        events.send(EditorInputEvent::SetRotationAxis(RotationAxis::Y));
    }
    if keyboard.just_pressed(KeyCode::KeyZ) {
        events.send(EditorInputEvent::SetRotationAxis(RotationAxis::Z));
    }

    // Rotation with arrow keys
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        events.send(EditorInputEvent::RotateDelta(-1));
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        events.send(EditorInputEvent::RotateDelta(1));
    }

    // Confirm or cancel
    if keyboard.just_pressed(KeyCode::Enter) {
        events.send(EditorInputEvent::ConfirmTransform);
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        events.send(EditorInputEvent::CancelTransform);
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
    preview_query: Query<&crate::editor::tools::selection_tool::TransformPreview>,
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

// Internal helper functions (extracted from selection_tool.rs)

fn start_move_operation_internal(
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

fn start_rotate_operation_internal(
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

fn delete_selected_items(
    editor_state: &mut EditorState,
    history: &mut EditorHistory,
    render_events: &mut EventWriter<RenderMapEvent>,
    update_events: &mut EventWriter<UpdateSelectionHighlights>,
) {
    // Nothing to delete
    if editor_state.selected_voxels.is_empty() && editor_state.selected_entities.is_empty() {
        info!("No items selected to delete");
        return;
    }

    // Create batch action for undo/redo
    let mut actions = Vec::new();
    let voxel_count = editor_state.selected_voxels.len();
    let entity_count = editor_state.selected_entities.len();

    // --- Delete selected entities ---
    // Sort indices in descending order to safely remove from the vector
    let mut entity_indices: Vec<usize> = editor_state.selected_entities.iter().copied().collect();
    entity_indices.sort_by(|a, b| b.cmp(a)); // Descending order

    for index in entity_indices {
        if index < editor_state.current_map.entities.len() {
            let entity_data = editor_state.current_map.entities[index].clone();
            actions.push(crate::editor::history::EditorAction::RemoveEntity {
                index,
                data: entity_data,
            });
            editor_state.current_map.entities.remove(index);
        }
    }

    // Clear entity selection
    editor_state.selected_entities.clear();

    // --- Delete selected voxels ---
    // Collect selected positions to avoid borrow checker issues
    let selected_positions: Vec<(i32, i32, i32)> =
        editor_state.selected_voxels.iter().copied().collect();

    // Find and remove each selected voxel
    for pos in selected_positions {
        if let Some(index) = editor_state
            .current_map
            .world
            .voxels
            .iter()
            .position(|v| v.pos == pos)
        {
            let voxel_data = editor_state.current_map.world.voxels[index].clone();
            actions.push(crate::editor::history::EditorAction::RemoveVoxel {
                pos,
                data: voxel_data,
            });
            editor_state.current_map.world.voxels.remove(index);
        }
    }

    // Push batch action to history
    if !actions.is_empty() {
        let description = match (voxel_count, entity_count) {
            (0, e) => format!("Delete {} entit{}", e, if e == 1 { "y" } else { "ies" }),
            (v, 0) => format!("Delete {} voxel{}", v, if v == 1 { "" } else { "s" }),
            (v, e) => format!(
                "Delete {} voxel{} and {} entit{}",
                v,
                if v == 1 { "" } else { "s" },
                e,
                if e == 1 { "y" } else { "ies" }
            ),
        };

        history.push(crate::editor::history::EditorAction::Batch {
            description,
            actions,
        });

        editor_state.mark_modified();
        info!(
            "Deleted {} voxels and {} entities",
            voxel_count, entity_count
        );
    }

    // Clear voxel selection
    editor_state.selected_voxels.clear();

    // Trigger re-render
    render_events.send(RenderMapEvent);
    update_events.send(UpdateSelectionHighlights);
}

fn move_selected_entities(
    editor_state: &mut EditorState,
    history: &mut EditorHistory,
    render_events: &mut EventWriter<RenderMapEvent>,
    offset: Vec3,
) {
    // Nothing to move
    if editor_state.selected_entities.is_empty() {
        return;
    }

    let mut actions = Vec::new();
    let entity_count = editor_state.selected_entities.len();

    // Move each selected entity
    for &index in &editor_state.selected_entities {
        if index < editor_state.current_map.entities.len() {
            let old_data = editor_state.current_map.entities[index].clone();
            let mut new_data = old_data.clone();
            new_data.position.0 += offset.x;
            new_data.position.1 += offset.y;
            new_data.position.2 += offset.z;

            // Record for undo
            actions.push(crate::editor::history::EditorAction::ModifyEntity {
                index,
                old_data,
                new_data: new_data.clone(),
            });

            // Apply the change
            editor_state.current_map.entities[index] = new_data;
        }
    }

    // Push to history
    if !actions.is_empty() {
        history.push(crate::editor::history::EditorAction::Batch {
            description: format!(
                "Move {} entit{}",
                entity_count,
                if entity_count == 1 { "y" } else { "ies" }
            ),
            actions,
        });

        editor_state.mark_modified();
        info!("Moved {} entities by {:?}", entity_count, offset);
    }

    // Trigger re-render
    render_events.send(RenderMapEvent);
}

fn confirm_move_internal(
    active_transform: &mut ActiveTransform,
    editor_state: &mut EditorState,
    history: &mut EditorHistory,
    preview_query: &Query<&crate::editor::tools::selection_tool::TransformPreview>,
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
        history.push(crate::editor::history::EditorAction::Batch {
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
                    crate::editor::history::EditorAction::Batch {
                        description: format!("Move voxel from {:?} to {:?}", old_pos, new_pos),
                        actions: vec![
                            crate::editor::history::EditorAction::RemoveVoxel {
                                pos: *old_pos,
                                data: voxel_data.clone(),
                            },
                            crate::editor::history::EditorAction::PlaceVoxel {
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

fn confirm_rotate_internal(
    active_transform: &mut ActiveTransform,
    editor_state: &mut EditorState,
    history: &mut EditorHistory,
    preview_query: &Query<&crate::editor::tools::selection_tool::TransformPreview>,
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
        let new_pos = rotate_position(
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
            use crate::systems::game::map::format::RotationState;

            // Compose with existing rotation or create new rotation state
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
        history.push(crate::editor::history::EditorAction::Batch {
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
                    use crate::systems::game::map::format::RotationState;
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
                    crate::editor::history::EditorAction::Batch {
                        description: format!("Rotate voxel from {:?} to {:?}", old_pos, new_pos),
                        actions: vec![
                            crate::editor::history::EditorAction::RemoveVoxel {
                                pos: *old_pos,
                                data: voxel_data.clone(),
                            },
                            crate::editor::history::EditorAction::PlaceVoxel {
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

/// Calculate rotated position around pivot
fn rotate_position(
    pos: (i32, i32, i32),
    pivot: Vec3,
    axis: RotationAxis,
    angle: i32,
) -> (i32, i32, i32) {
    // Convert to Vec3 relative to pivot
    let rel_pos = Vec3::new(
        pos.0 as f32 - pivot.x,
        pos.1 as f32 - pivot.y,
        pos.2 as f32 - pivot.z,
    );

    // Rotate based on axis and angle (in 90-degree increments)
    let rotated = match axis {
        RotationAxis::X => {
            // Rotate around X axis (affects Y and Z)
            match angle {
                0 => rel_pos,
                1 => Vec3::new(rel_pos.x, -rel_pos.z, rel_pos.y), // 90° CW
                2 => Vec3::new(rel_pos.x, -rel_pos.y, -rel_pos.z), // 180°
                3 => Vec3::new(rel_pos.x, rel_pos.z, -rel_pos.y), // 270° CW
                _ => rel_pos,
            }
        }
        RotationAxis::Y => {
            // Rotate around Y axis (affects X and Z)
            match angle {
                0 => rel_pos,
                1 => Vec3::new(rel_pos.z, rel_pos.y, -rel_pos.x), // 90° CW
                2 => Vec3::new(-rel_pos.x, rel_pos.y, -rel_pos.z), // 180°
                3 => Vec3::new(-rel_pos.z, rel_pos.y, rel_pos.x), // 270° CW
                _ => rel_pos,
            }
        }
        RotationAxis::Z => {
            // Rotate around Z axis (affects X and Y)
            match angle {
                0 => rel_pos,
                1 => Vec3::new(-rel_pos.y, rel_pos.x, rel_pos.z), // 90° CW
                2 => Vec3::new(-rel_pos.x, -rel_pos.y, rel_pos.z), // 180°
                3 => Vec3::new(rel_pos.y, -rel_pos.x, rel_pos.z), // 270° CW
                _ => rel_pos,
            }
        }
    };

    // Convert back to world coordinates and round to integers
    (
        (rotated.x + pivot.x).round() as i32,
        (rotated.y + pivot.y).round() as i32,
        (rotated.z + pivot.z).round() as i32,
    )
}
