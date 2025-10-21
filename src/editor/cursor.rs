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

    // Find the closest voxel that the ray intersects
    let closest_voxel = find_closest_voxel_intersection(&editor_state, &ray);

    if let Some(voxel_pos) = closest_voxel {
        // Set cursor to the intersected voxel
        editor_state.cursor_grid_pos = Some(voxel_pos);
        editor_state.cursor_position = Some(Vec3::new(
            voxel_pos.0 as f32,
            voxel_pos.1 as f32,
            voxel_pos.2 as f32,
        ));
    } else {
        // No voxel intersection, fall back to ground plane intersection
        if let Some(ground_pos) = intersect_ground_plane(&ray) {
            editor_state.cursor_position = Some(ground_pos);
            let grid_x = ground_pos.x.round() as i32;
            let grid_y = 0;
            let grid_z = ground_pos.z.round() as i32;
            editor_state.cursor_grid_pos = Some((grid_x, grid_y, grid_z));
        } else {
            editor_state.cursor_position = None;
            editor_state.cursor_grid_pos = None;
        }
    }
}

/// Find the closest voxel that the ray intersects
fn find_closest_voxel_intersection(
    editor_state: &EditorState,
    ray: &Ray3d,
) -> Option<(i32, i32, i32)> {
    let mut closest_distance = f32::MAX;
    let mut closest_voxel = None;

    // Check each voxel in the map
    for voxel_data in &editor_state.current_map.world.voxels {
        let voxel_pos = voxel_data.pos;
        
        // Check if ray intersects this voxel's bounding box
        if let Some(distance) = ray_box_intersection(
            ray,
            Vec3::new(voxel_pos.0 as f32, voxel_pos.1 as f32, voxel_pos.2 as f32),
            Vec3::splat(1.0), // Voxel size is 1x1x1
        ) {
            if distance < closest_distance {
                closest_distance = distance;
                closest_voxel = Some(voxel_pos);
            }
        }
    }

    closest_voxel
}

/// Ray-box intersection test (AABB)
/// Returns the distance along the ray if there's an intersection, None otherwise
fn ray_box_intersection(ray: &Ray3d, box_center: Vec3, box_size: Vec3) -> Option<f32> {
    let box_min = box_center - box_size * 0.5;
    let box_max = box_center + box_size * 0.5;

    let ray_origin = ray.origin;
    let ray_dir = ray.direction.normalize();

    // Calculate intersection distances for each axis
    let mut tmin = f32::NEG_INFINITY;
    let mut tmax = f32::INFINITY;

    // X axis
    if ray_dir.x.abs() > 0.0001 {
        let tx1 = (box_min.x - ray_origin.x) / ray_dir.x;
        let tx2 = (box_max.x - ray_origin.x) / ray_dir.x;
        tmin = tmin.max(tx1.min(tx2));
        tmax = tmax.min(tx1.max(tx2));
    } else if ray_origin.x < box_min.x || ray_origin.x > box_max.x {
        return None;
    }

    // Y axis
    if ray_dir.y.abs() > 0.0001 {
        let ty1 = (box_min.y - ray_origin.y) / ray_dir.y;
        let ty2 = (box_max.y - ray_origin.y) / ray_dir.y;
        tmin = tmin.max(ty1.min(ty2));
        tmax = tmax.min(ty1.max(ty2));
    } else if ray_origin.y < box_min.y || ray_origin.y > box_max.y {
        return None;
    }

    // Z axis
    if ray_dir.z.abs() > 0.0001 {
        let tz1 = (box_min.z - ray_origin.z) / ray_dir.z;
        let tz2 = (box_max.z - ray_origin.z) / ray_dir.z;
        tmin = tmin.max(tz1.min(tz2));
        tmax = tmax.min(tz1.max(tz2));
    } else if ray_origin.z < box_min.z || ray_origin.z > box_max.z {
        return None;
    }

    // Check if there's a valid intersection
    if tmax >= tmin && tmax >= 0.0 {
        // Return the closest intersection point (tmin if positive, otherwise tmax)
        Some(if tmin >= 0.0 { tmin } else { tmax })
    } else {
        None
    }
}

/// Intersect ray with ground plane (y=0) as fallback
fn intersect_ground_plane(ray: &Ray3d) -> Option<Vec3> {
    let ray_origin = ray.origin;
    let ray_direction = ray.direction.normalize();

    // Check if ray is parallel to ground
    if ray_direction.y.abs() < 0.001 {
        return None;
    }

    // Calculate t where ray intersects y=0
    let t = -ray_origin.y / ray_direction.y;

    if t < 0.0 {
        return None;
    }

    // Calculate world position at intersection
    Some(ray_origin + ray_direction * t)
}