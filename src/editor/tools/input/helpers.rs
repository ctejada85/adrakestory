//! Helper functions for input handling operations.

use crate::editor::history::{EditorAction, EditorHistory};
use crate::editor::renderer::RenderMapEvent;
use crate::editor::state::EditorState;
use crate::editor::tools::UpdateSelectionHighlights;
use crate::systems::game::map::geometry::RotationAxis;
use bevy::prelude::*;

/// Delete all selected voxels and entities
pub fn delete_selected_items(
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
            actions.push(EditorAction::RemoveEntity {
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
            actions.push(EditorAction::RemoveVoxel {
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

        history.push(EditorAction::Batch {
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

/// Move selected entities by an offset
pub fn move_selected_entities(
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
            actions.push(EditorAction::ModifyEntity {
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
        history.push(EditorAction::Batch {
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

/// Calculate rotated position around pivot
pub fn rotate_position(
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
