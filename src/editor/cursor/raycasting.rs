//! Ray casting utilities for cursor position detection.

use crate::editor::state::EditorState;
use bevy::prelude::*;

/// Information about a ray-box intersection
#[derive(Debug, Clone, Copy)]
pub struct RayHitInfo {
    pub distance: f32,
    pub face_normal: Vec3,
}

/// Find the closest voxel that the ray intersects with face information
pub fn find_closest_voxel_intersection_with_face(
    editor_state: &EditorState,
    ray: &Ray3d,
) -> Option<((i32, i32, i32), RayHitInfo)> {
    let mut closest_distance = f32::MAX;
    let mut closest_result = None;

    // Check each voxel in the map
    for voxel_data in &editor_state.current_map.world.voxels {
        let voxel_pos = voxel_data.pos;

        // Check if ray intersects this voxel's bounding box
        if let Some(hit_info) = ray_box_intersection_with_face(
            ray,
            Vec3::new(voxel_pos.0 as f32, voxel_pos.1 as f32, voxel_pos.2 as f32),
            Vec3::splat(1.0), // Voxel size is 1x1x1
        ) {
            if hit_info.distance < closest_distance {
                closest_distance = hit_info.distance;
                closest_result = Some((voxel_pos, hit_info));
            }
        }
    }

    closest_result
}

/// Ray-box intersection test (AABB) with face detection
/// Returns hit information including which face was hit
pub fn ray_box_intersection_with_face(
    ray: &Ray3d,
    box_center: Vec3,
    box_size: Vec3,
) -> Option<RayHitInfo> {
    let box_min = box_center - box_size * 0.5;
    let box_max = box_center + box_size * 0.5;

    let ray_origin = ray.origin;
    let ray_dir = ray.direction.normalize();

    // Calculate intersection distances for each axis
    let mut tmin = f32::NEG_INFINITY;
    let mut tmax = f32::INFINITY;
    let mut hit_axis = 0; // 0=X, 1=Y, 2=Z
    let mut hit_min_face = true; // true if hit min face, false if hit max face

    // X axis
    if ray_dir.x.abs() > 0.0001 {
        let tx1 = (box_min.x - ray_origin.x) / ray_dir.x;
        let tx2 = (box_max.x - ray_origin.x) / ray_dir.x;
        let tx_min = tx1.min(tx2);
        let tx_max = tx1.max(tx2);

        if tx_min > tmin {
            tmin = tx_min;
            hit_axis = 0;
            hit_min_face = tx1 < tx2;
        }
        tmax = tmax.min(tx_max);
    } else if ray_origin.x < box_min.x || ray_origin.x > box_max.x {
        return None;
    }

    // Y axis
    if ray_dir.y.abs() > 0.0001 {
        let ty1 = (box_min.y - ray_origin.y) / ray_dir.y;
        let ty2 = (box_max.y - ray_origin.y) / ray_dir.y;
        let ty_min = ty1.min(ty2);
        let ty_max = ty1.max(ty2);

        if ty_min > tmin {
            tmin = ty_min;
            hit_axis = 1;
            hit_min_face = ty1 < ty2;
        }
        tmax = tmax.min(ty_max);
    } else if ray_origin.y < box_min.y || ray_origin.y > box_max.y {
        return None;
    }

    // Z axis
    if ray_dir.z.abs() > 0.0001 {
        let tz1 = (box_min.z - ray_origin.z) / ray_dir.z;
        let tz2 = (box_max.z - ray_origin.z) / ray_dir.z;
        let tz_min = tz1.min(tz2);
        let tz_max = tz1.max(tz2);

        if tz_min > tmin {
            tmin = tz_min;
            hit_axis = 2;
            hit_min_face = tz1 < tz2;
        }
        tmax = tmax.min(tz_max);
    } else if ray_origin.z < box_min.z || ray_origin.z > box_max.z {
        return None;
    }

    // Check if there's a valid intersection
    if tmax >= tmin && tmax >= 0.0 {
        let distance = if tmin >= 0.0 { tmin } else { tmax };

        // Calculate face normal based on hit axis and face
        let face_normal = match (hit_axis, hit_min_face) {
            (0, true) => Vec3::NEG_X,
            (0, false) => Vec3::X,
            (1, true) => Vec3::NEG_Y,
            (1, false) => Vec3::Y,
            (2, true) => Vec3::NEG_Z,
            (2, false) => Vec3::Z,
            _ => Vec3::Y, // fallback
        };

        Some(RayHitInfo {
            distance,
            face_normal,
        })
    } else {
        None
    }
}

/// Intersect ray with ground plane (y=0) as fallback
pub fn intersect_ground_plane(ray: &Ray3d) -> Option<Vec3> {
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
