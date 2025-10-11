//! Selection tool for selecting and manipulating objects.

use crate::editor::state::EditorState;
use bevy::prelude::*;

/// Handle selection when the tool is active
pub fn handle_selection(
    mut editor_state: ResMut<EditorState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
) {
    // Check if select tool is active
    if !matches!(
        editor_state.active_tool,
        crate::editor::state::EditorTool::Select
    ) {
        return;
    }

    // Check if left mouse button was just pressed
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    // Get cursor grid position
    let Some(grid_pos) = editor_state.cursor_grid_pos else {
        return;
    };

    // Toggle selection of voxel at this position
    if editor_state.selected_voxels.contains(&grid_pos) {
        editor_state.selected_voxels.remove(&grid_pos);
        info!("Deselected voxel at {:?}", grid_pos);
    } else {
        editor_state.selected_voxels.insert(grid_pos);
        info!("Selected voxel at {:?}", grid_pos);
    }
}
