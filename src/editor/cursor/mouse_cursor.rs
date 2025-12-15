//! Mouse-based cursor position updates.

use super::raycasting::{find_closest_voxel_intersection_with_face, intersect_ground_plane};
use super::CursorState;
use crate::editor::camera::EditorCamera;
use crate::editor::state::{EditorState, KeyboardEditMode};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContexts;

/// System to update cursor position and grid position from mouse input
/// Only updates when keyboard edit mode is disabled
pub fn update_cursor_position(
    mut cursor_state: ResMut<CursorState>,
    editor_state: Res<EditorState>,
    camera_query: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    keyboard_mode: Res<KeyboardEditMode>,
    mut contexts: EguiContexts,
) {
    // Don't update cursor from mouse when in keyboard edit mode
    if keyboard_mode.enabled {
        return;
    }

    // Don't update if mouse is over UI
    if contexts.ctx_mut().is_pointer_over_area() {
        return;
    }

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let Ok(window) = window_query.get_single() else {
        return;
    };

    // Get cursor position in window
    let Some(cursor_position) = window.cursor_position() else {
        cursor_state.position = None;
        cursor_state.grid_pos = None;
        return;
    };

    // Convert cursor position to world ray
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        cursor_state.position = None;
        cursor_state.grid_pos = None;
        return;
    };

    // Find the closest voxel that the ray intersects with face information
    let closest_voxel_hit = find_closest_voxel_intersection_with_face(&editor_state, &ray);

    if let Some((voxel_pos, hit_info)) = closest_voxel_hit {
        // Set cursor to the intersected voxel
        cursor_state.grid_pos = Some(voxel_pos);
        cursor_state.position = Some(Vec3::new(
            voxel_pos.0 as f32,
            voxel_pos.1 as f32,
            voxel_pos.2 as f32,
        ));
        cursor_state.hit_face_normal = Some(hit_info.face_normal);

        // Calculate adjacent placement position
        let offset = hit_info.face_normal;
        let placement_grid = (
            voxel_pos.0 + offset.x as i32,
            voxel_pos.1 + offset.y as i32,
            voxel_pos.2 + offset.z as i32,
        );
        cursor_state.placement_grid_pos = Some(placement_grid);
        cursor_state.placement_pos = Some(Vec3::new(
            placement_grid.0 as f32,
            placement_grid.1 as f32,
            placement_grid.2 as f32,
        ));
    } else {
        // No voxel intersection, fall back to ground plane intersection
        if let Some(ground_pos) = intersect_ground_plane(&ray) {
            // Keep cursor position as exact intersection for free movement
            cursor_state.position = Some(ground_pos);

            // Snap grid position to nearest integer coordinates
            let grid_x = ground_pos.x.round() as i32;
            let grid_y = 0;
            let grid_z = ground_pos.z.round() as i32;
            cursor_state.grid_pos = Some((grid_x, grid_y, grid_z));
            cursor_state.hit_face_normal = Some(Vec3::Y); // Upward

            // Placement position snaps to grid (integer coordinates)
            cursor_state.placement_grid_pos = Some((grid_x, grid_y, grid_z));
            cursor_state.placement_pos =
                Some(Vec3::new(grid_x as f32, grid_y as f32, grid_z as f32));
        } else {
            cursor_state.position = None;
            cursor_state.grid_pos = None;
            cursor_state.hit_face_normal = None;
            cursor_state.placement_pos = None;
            cursor_state.placement_grid_pos = None;
        }
    }
}
