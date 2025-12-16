//! Mouse-based cursor position updates.

use super::raycasting::{find_closest_voxel_intersection_with_face, intersect_ground_plane};
use super::CursorState;
use crate::editor::camera::{EditorCamera, GamepadCameraState};
use crate::editor::state::{EditorState, KeyboardEditMode};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// System to update cursor position from mouse or center-screen raycast
/// When gamepad is active: cursor follows center of screen (raycast from camera forward)
/// When mouse is active: cursor follows mouse pointer (raycast from mouse position)
pub fn update_cursor_position(
    mut cursor_state: ResMut<CursorState>,
    gamepad_state: Res<GamepadCameraState>,
    editor_state: Res<EditorState>,
    camera_query: Query<(&Camera, &GlobalTransform, &EditorCamera)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    keyboard_mode: Res<KeyboardEditMode>,
) {
    // Don't update cursor from raycasting when in keyboard edit mode
    if keyboard_mode.enabled {
        return;
    }

    let Ok((camera, camera_transform, editor_cam)) = camera_query.get_single() else {
        return;
    };

    let Ok(window) = window_query.get_single() else {
        return;
    };

    // If gamepad is active, use center-screen raycast from GamepadCameraState
    if gamepad_state.active {
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
            cursor_state.grid_pos = gamepad_state.action_grid_pos;
            cursor_state.position = Some(action_pos);
        }

        // Calculate face normal from camera direction for gamepad
        let forward = editor_cam.forward();
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
    } else {
        // Mouse mode: raycast from mouse cursor position
        let Some(cursor_position) = window.cursor_position() else {
            return;
        };

        let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
            return;
        };

        // Try to find a voxel intersection
        if let Some((voxel_pos, hit_info)) =
            find_closest_voxel_intersection_with_face(&editor_state, &ray)
        {
            // Hit a voxel - set grid position to hit voxel
            cursor_state.grid_pos = Some(voxel_pos);
            cursor_state.position = Some(Vec3::new(
                voxel_pos.0 as f32,
                voxel_pos.1 as f32,
                voxel_pos.2 as f32,
            ));
            cursor_state.hit_face_normal = Some(hit_info.face_normal);

            // Calculate placement position (adjacent to hit face)
            let placement_pos = (
                voxel_pos.0 + hit_info.face_normal.x as i32,
                voxel_pos.1 + hit_info.face_normal.y as i32,
                voxel_pos.2 + hit_info.face_normal.z as i32,
            );
            cursor_state.placement_grid_pos = Some(placement_pos);
            cursor_state.placement_pos = Some(Vec3::new(
                placement_pos.0 as f32,
                placement_pos.1 as f32,
                placement_pos.2 as f32,
            ));
        } else if let Some(ground_pos) = intersect_ground_plane(&ray) {
            // No voxel hit, use ground plane intersection
            let grid_x = ground_pos.x.floor() as i32;
            let grid_z = ground_pos.z.floor() as i32;
            let grid_pos = (grid_x, 0, grid_z);

            cursor_state.grid_pos = Some(grid_pos);
            cursor_state.position = Some(Vec3::new(grid_x as f32, 0.0, grid_z as f32));
            cursor_state.hit_face_normal = Some(Vec3::Y);

            // For ground plane, placement is at the same position
            cursor_state.placement_grid_pos = Some(grid_pos);
            cursor_state.placement_pos = Some(Vec3::new(grid_x as f32, 0.0, grid_z as f32));
        } else {
            // No hit at all - clear cursor state
            cursor_state.grid_pos = None;
            cursor_state.position = None;
            cursor_state.hit_face_normal = None;
            cursor_state.placement_grid_pos = None;
            cursor_state.placement_pos = None;
        }
    }
}
