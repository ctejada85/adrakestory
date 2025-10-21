//! Cursor ray casting system for detecting voxel positions from mouse input.

use crate::editor::camera::EditorCamera;
use crate::editor::state::EditorState;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// System to update cursor position and grid position from mouse input
pub fn update_cursor_position(
    mut editor_state: ResMut<EditorState>,
    camera_query: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let Ok(window) = window_query.get_single() else {
        return;
    };

    // Get cursor position in window
    let Some(cursor_position) = window.cursor_position() else {
        editor_state.cursor_position = None;
        editor_state.cursor_grid_pos = None;
        return;
    };

    // Convert cursor position to world ray
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        editor_state.cursor_position = None;
        editor_state.cursor_grid_pos = None;
        return;
    };

    // Cast ray to find intersection with y=0 plane (ground level)
    // This is a simple implementation - we'll intersect with the ground plane
    let ray_origin = ray.origin;
    let ray_direction = ray.direction.normalize();

    // Find intersection with y=0 plane
    if ray_direction.y.abs() < 0.001 {
        // Ray is parallel to ground, no intersection
        editor_state.cursor_position = None;
        editor_state.cursor_grid_pos = None;
        return;
    }

    // Calculate t where ray intersects y=0
    // ray_origin.y + t * ray_direction.y = 0
    // t = -ray_origin.y / ray_direction.y
    let t = -ray_origin.y / ray_direction.y;

    if t < 0.0 {
        // Intersection is behind camera
        editor_state.cursor_position = None;
        editor_state.cursor_grid_pos = None;
        return;
    }

    // Calculate world position at intersection
    let world_pos = ray_origin + ray_direction * t;
    editor_state.cursor_position = Some(world_pos);

    // Convert to grid position (round to nearest integer)
    let grid_x = world_pos.x.round() as i32;
    let grid_y = 0; // Always at ground level for now
    let grid_z = world_pos.z.round() as i32;

    editor_state.cursor_grid_pos = Some((grid_x, grid_y, grid_z));
}

/// System to check if cursor is over an existing voxel
pub fn check_voxel_at_cursor(
    editor_state: Res<EditorState>,
) -> Option<(i32, i32, i32)> {
    let grid_pos = editor_state.cursor_grid_pos?;
    
    // Check if there's a voxel at this position
    let voxel_exists = editor_state
        .current_map
        .world
        .voxels
        .iter()
        .any(|v| v.pos == grid_pos);
    
    if voxel_exists {
        Some(grid_pos)
    } else {
        None
    }
}