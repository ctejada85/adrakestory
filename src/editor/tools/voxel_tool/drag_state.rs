//! Drag state resources for voxel tools.

use bevy::prelude::*;

/// Resource tracking drag-to-place state for voxel tool
#[derive(Resource, Default)]
pub struct VoxelDragState {
    /// Whether we're currently in a drag-place operation
    pub is_dragging: bool,
    /// Last grid position we placed a voxel at
    pub last_placed_pos: Option<(i32, i32, i32)>,
    /// Last cursor grid position (to detect movement)
    pub last_cursor_grid_pos: Option<(i32, i32, i32)>,
    /// Screen position where the drag started (to detect actual mouse movement)
    pub drag_start_screen_pos: Option<Vec2>,
}

/// Resource tracking drag-to-remove state for voxel remove tool
#[derive(Resource, Default)]
pub struct VoxelRemoveDragState {
    /// Whether we're currently in a drag-remove operation
    pub is_dragging: bool,
    /// Last grid position we removed a voxel at (to avoid duplicates)
    pub last_grid_pos: Option<(i32, i32, i32)>,
    /// Screen position where the drag started (to detect actual mouse movement)
    pub drag_start_screen_pos: Option<Vec2>,
}
