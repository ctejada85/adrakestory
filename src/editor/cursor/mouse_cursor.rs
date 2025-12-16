//! Mouse-based cursor position updates.

use super::CursorState;
use crate::editor::camera::{EditorCamera, GamepadCameraState};
use crate::editor::state::KeyboardEditMode;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// System to update cursor position from center-screen raycast (fly camera mode)
/// The cursor is always positioned at where the camera is looking
pub fn update_cursor_position(
    mut cursor_state: ResMut<CursorState>,
    gamepad_state: Res<GamepadCameraState>,
    camera_query: Query<&EditorCamera, With<EditorCamera>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    keyboard_mode: Res<KeyboardEditMode>,
) {
    // Don't update cursor from raycasting when in keyboard edit mode
    if keyboard_mode.enabled {
        return;
    }

    let Ok(editor_cam) = camera_query.get_single() else {
        return;
    };

    let Ok(_window) = window_query.get_single() else {
        return;
    };

    // Use center-screen raycast (from GamepadCameraState which is always updated)
    // This works for both gamepad and keyboard/mouse since we're always in fly mode
    if let Some(grid_pos) = gamepad_state.action_grid_pos {
        cursor_state.placement_grid_pos = Some(grid_pos);
        cursor_state.placement_pos = Some(Vec3::new(
            grid_pos.0 as f32,
            grid_pos.1 as f32,
            grid_pos.2 as f32,
        ));
    }

    if let Some(target_pos) = gamepad_state.target_voxel_pos {
        cursor_state.grid_pos = Some(target_pos);
        cursor_state.position = Some(Vec3::new(
            target_pos.0 as f32,
            target_pos.1 as f32,
            target_pos.2 as f32,
        ));
    } else if let Some(action_pos) = gamepad_state.action_position {
        // No voxel being looked at, use the placement position
        cursor_state.grid_pos = gamepad_state.action_grid_pos;
        cursor_state.position = Some(action_pos);
    }

    // Calculate face normal from camera direction
    let forward = editor_cam.forward();
    
    // Determine dominant axis for face normal (opposite of camera direction)
    let abs_x = forward.x.abs();
    let abs_y = forward.y.abs();
    let abs_z = forward.z.abs();
    
    if abs_y > abs_x && abs_y > abs_z {
        cursor_state.hit_face_normal = Some(if forward.y > 0.0 { Vec3::NEG_Y } else { Vec3::Y });
    } else if abs_x > abs_z {
        cursor_state.hit_face_normal = Some(if forward.x > 0.0 { Vec3::NEG_X } else { Vec3::X });
    } else {
        cursor_state.hit_face_normal = Some(if forward.z > 0.0 { Vec3::NEG_Z } else { Vec3::Z });
    }
}
