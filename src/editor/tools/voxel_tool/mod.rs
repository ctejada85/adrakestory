//! Voxel placement and removal tools.

mod drag_state;
mod placement;
mod removal;

pub use drag_state::{VoxelDragState, VoxelRemoveDragState};
pub use placement::{handle_voxel_drag_placement, handle_voxel_placement};
pub use removal::{handle_voxel_drag_removal, handle_voxel_removal};

use crate::editor::history::EditorHistory;
use crate::editor::state::EditorState;
use crate::systems::game::map::format::VoxelData;
use bevy::prelude::*;

/// Bundle of input resources for voxel tools
#[derive(bevy::ecs::system::SystemParam)]
pub struct VoxelToolInput<'w> {
    pub mouse_button: Res<'w, ButtonInput<MouseButton>>,
    pub keyboard: Res<'w, ButtonInput<KeyCode>>,
}

/// Minimum mouse movement in pixels required to trigger drag placement/removal
/// This prevents accidental double actions when the cursor grid position changes
/// due to underlying geometry changes (e.g., after placing/removing a voxel)
pub const DRAG_MOVEMENT_THRESHOLD: f32 = 5.0;

/// Helper function to remove a voxel at a position
pub(crate) fn try_remove_voxel(
    editor_state: &mut ResMut<EditorState>,
    history: &mut ResMut<EditorHistory>,
    grid_pos: (i32, i32, i32),
) {
    use crate::editor::history::EditorAction;

    // Find and remove voxel at this position
    if let Some(index) = editor_state
        .current_map
        .world
        .voxels
        .iter()
        .position(|v| v.pos == grid_pos)
    {
        let voxel_data = editor_state.current_map.world.voxels.remove(index);
        editor_state.mark_modified();

        // Record action in history
        history.push(EditorAction::RemoveVoxel {
            pos: grid_pos,
            data: voxel_data.clone(),
        });

        info!("Removed voxel at {:?}", grid_pos);
    }
}

/// Helper function to place a voxel and update drag state
pub(crate) fn try_place_voxel(
    editor_state: &mut ResMut<EditorState>,
    history: &mut ResMut<EditorHistory>,
    grid_pos: (i32, i32, i32),
    voxel_type: crate::systems::game::components::VoxelType,
    pattern: crate::systems::game::map::format::SubVoxelPattern,
    drag_state: &mut ResMut<VoxelDragState>,
) {
    use crate::editor::history::EditorAction;

    // Check if voxel already exists at this position
    let voxel_exists = editor_state
        .current_map
        .world
        .voxels
        .iter()
        .any(|v| v.pos == grid_pos);

    if voxel_exists {
        return;
    }

    // Update last placed position
    drag_state.last_placed_pos = Some(grid_pos);

    // Create new voxel data
    let voxel_data = VoxelData {
        pos: grid_pos,
        voxel_type,
        pattern: Some(pattern),
        rotation_state: None,
    };

    // Add to map
    editor_state
        .current_map
        .world
        .voxels
        .push(voxel_data.clone());
    editor_state.mark_modified();

    // Record action in history
    history.push(EditorAction::PlaceVoxel {
        pos: grid_pos,
        data: voxel_data,
    });

    info!("Drag-placed {:?} voxel at {:?}", voxel_type, grid_pos);
}
