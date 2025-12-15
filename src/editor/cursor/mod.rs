//! Cursor ray casting system for detecting voxel positions from mouse input.

mod keyboard_cursor;
mod keyboard_mode;
mod mouse_cursor;
mod raycasting;

pub use keyboard_cursor::{handle_keyboard_cursor_movement, handle_keyboard_selection};
pub use keyboard_mode::{handle_play_shortcuts, handle_tool_switching, toggle_keyboard_edit_mode};
pub use mouse_cursor::update_cursor_position;

use bevy::prelude::*;

/// Resource to track cursor position separately from editor state.
/// This prevents cursor updates from triggering change detection on EditorState.
#[derive(Resource, Default)]
pub struct CursorState {
    /// Current cursor position in world space (voxel being pointed at)
    pub position: Option<Vec3>,

    /// Current cursor grid position (voxel being pointed at)
    pub grid_pos: Option<(i32, i32, i32)>,

    /// Face normal of the hit surface
    pub hit_face_normal: Option<Vec3>,

    /// Position where a new voxel would be placed (adjacent to hit face)
    pub placement_pos: Option<Vec3>,

    /// Grid position where a new voxel would be placed
    pub placement_grid_pos: Option<(i32, i32, i32)>,
}

impl CursorState {
    /// Create a new cursor state
    pub fn new() -> Self {
        Self::default()
    }
}
