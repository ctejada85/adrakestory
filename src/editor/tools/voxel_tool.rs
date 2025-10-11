//! Voxel placement and removal tools.

use crate::editor::history::{EditorAction, EditorHistory};
use crate::editor::state::{EditorState, EditorTool};
use crate::systems::game::map::format::VoxelData;
use bevy::prelude::*;

/// Handle voxel placement when the tool is active
pub fn handle_voxel_placement(
    mut editor_state: ResMut<EditorState>,
    mut history: ResMut<EditorHistory>,
    mouse_button: Res<ButtonInput<MouseButton>>,
) {
    // Check if voxel place tool is active
    let (voxel_type, pattern) = match &editor_state.active_tool {
        EditorTool::VoxelPlace {
            voxel_type,
            pattern,
        } => (*voxel_type, *pattern),
        _ => return,
    };

    // Check if left mouse button was just pressed
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    // Get cursor grid position
    let Some(grid_pos) = editor_state.cursor_grid_pos else {
        return;
    };

    // Check if voxel already exists at this position
    let voxel_exists = editor_state
        .current_map
        .world
        .voxels
        .iter()
        .any(|v| v.pos == grid_pos);

    if voxel_exists {
        info!("Voxel already exists at {:?}", grid_pos);
        return;
    }

    // Create new voxel data
    let voxel_data = VoxelData {
        pos: grid_pos,
        voxel_type,
        pattern: Some(pattern),
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

    info!("Placed {:?} voxel at {:?}", voxel_type, grid_pos);
}

/// Handle voxel removal when the tool is active
pub fn handle_voxel_removal(
    mut editor_state: ResMut<EditorState>,
    mut history: ResMut<EditorHistory>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Check if voxel remove tool is active
    if !matches!(editor_state.active_tool, EditorTool::VoxelRemove) {
        return;
    }

    // Check if left mouse button or delete key was just pressed
    let should_remove = mouse_button.just_pressed(MouseButton::Left)
        || keyboard.just_pressed(KeyCode::Delete)
        || keyboard.just_pressed(KeyCode::Backspace);

    if !should_remove {
        return;
    }

    // Get cursor grid position
    let Some(grid_pos) = editor_state.cursor_grid_pos else {
        return;
    };

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
    } else {
        info!("No voxel at {:?} to remove", grid_pos);
    }
}
