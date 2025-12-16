//! 3D cursor and targeting for controller editing.
//!
//! Provides raycast-based targeting to show where voxels will be placed/removed.

use crate::editor::state::EditorState;
use bevy::prelude::*;

use super::camera::{ControllerCamera, ControllerCameraMode};

/// Information about a ray-box intersection (local copy to avoid private module access)
#[derive(Debug, Clone, Copy)]
pub struct RayHitInfo {
    pub distance: f32,
    pub face_normal: Vec3,
}

/// Resource tracking the controller cursor state.
#[derive(Resource, Default)]
pub struct ControllerCursor {
    /// Position of the voxel being targeted (for removal)
    pub target_voxel: Option<IVec3>,
    /// Position where a new voxel would be placed (adjacent to target)
    pub placement_position: Option<IVec3>,
    /// Normal of the face that was hit
    pub hit_face: Option<Vec3>,
    /// Distance to the target
    pub distance: f32,
    /// Whether the target is within reach
    pub in_reach: bool,
    /// Maximum reach distance in voxels
    pub max_reach: f32,
}

impl ControllerCursor {
    pub fn new() -> Self {
        Self {
            target_voxel: None,
            placement_position: None,
            hit_face: None,
            distance: 0.0,
            in_reach: false,
            max_reach: 7.0,
        }
    }

    /// Clear all targeting state.
    pub fn clear(&mut self) {
        self.target_voxel = None;
        self.placement_position = None;
        self.hit_face = None;
        self.distance = 0.0;
        self.in_reach = false;
    }

    /// Update targeting from a raycast hit.
    pub fn update_from_hit(&mut self, voxel_pos: (i32, i32, i32), hit_info: RayHitInfo) {
        self.target_voxel = Some(IVec3::new(voxel_pos.0, voxel_pos.1, voxel_pos.2));
        self.hit_face = Some(hit_info.face_normal);
        self.distance = hit_info.distance;
        self.in_reach = hit_info.distance <= self.max_reach;

        // Calculate placement position (adjacent to hit face)
        let face_offset = IVec3::new(
            hit_info.face_normal.x.round() as i32,
            hit_info.face_normal.y.round() as i32,
            hit_info.face_normal.z.round() as i32,
        );
        self.placement_position = Some(IVec3::new(voxel_pos.0, voxel_pos.1, voxel_pos.2) + face_offset);
    }

    /// Update targeting from ground plane hit.
    pub fn update_from_ground(&mut self, world_pos: Vec3, distance: f32) {
        let grid_pos = IVec3::new(
            world_pos.x.floor() as i32,
            0,
            world_pos.z.floor() as i32,
        );
        self.target_voxel = None; // No voxel to remove
        self.placement_position = Some(grid_pos);
        self.hit_face = Some(Vec3::Y);
        self.distance = distance;
        self.in_reach = distance <= self.max_reach;
    }
}

/// Ray-box intersection test (AABB) with face detection
fn ray_box_intersection_with_face(
    ray_origin: Vec3,
    ray_dir: Vec3,
    box_center: Vec3,
    box_size: Vec3,
) -> Option<RayHitInfo> {
    let box_min = box_center - box_size * 0.5;
    let box_max = box_center + box_size * 0.5;

    let ray_dir = ray_dir.normalize();

    let mut tmin = f32::NEG_INFINITY;
    let mut tmax = f32::INFINITY;
    let mut hit_axis = 0;
    let mut hit_min_face = true;

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

    if tmax >= tmin && tmax >= 0.0 {
        let distance = if tmin >= 0.0 { tmin } else { tmax };

        let face_normal = match (hit_axis, hit_min_face) {
            (0, true) => Vec3::NEG_X,
            (0, false) => Vec3::X,
            (1, true) => Vec3::NEG_Y,
            (1, false) => Vec3::Y,
            (2, true) => Vec3::NEG_Z,
            (2, false) => Vec3::Z,
            _ => Vec3::Y,
        };

        Some(RayHitInfo {
            distance,
            face_normal,
        })
    } else {
        None
    }
}

/// Find the closest voxel that the ray intersects
fn find_closest_voxel_intersection(
    editor_state: &EditorState,
    ray_origin: Vec3,
    ray_dir: Vec3,
) -> Option<((i32, i32, i32), RayHitInfo)> {
    let mut closest_distance = f32::MAX;
    let mut closest_result = None;

    for voxel_data in &editor_state.current_map.world.voxels {
        let voxel_pos = voxel_data.pos;
        let center = Vec3::new(
            voxel_pos.0 as f32 + 0.5,
            voxel_pos.1 as f32 + 0.5,
            voxel_pos.2 as f32 + 0.5,
        );

        if let Some(hit_info) = ray_box_intersection_with_face(
            ray_origin,
            ray_dir,
            center,
            Vec3::splat(1.0),
        ) {
            if hit_info.distance < closest_distance && hit_info.distance > 0.0 {
                closest_distance = hit_info.distance;
                closest_result = Some((voxel_pos, hit_info));
            }
        }
    }

    closest_result
}

/// Intersect ray with ground plane (y=0)
fn intersect_ground_plane(ray_origin: Vec3, ray_dir: Vec3) -> Option<Vec3> {
    let ray_dir = ray_dir.normalize();

    if ray_dir.y.abs() < 0.001 {
        return None;
    }

    let t = -ray_origin.y / ray_dir.y;

    if t < 0.0 {
        return None;
    }

    Some(ray_origin + ray_dir * t)
}

/// System to update the controller cursor via raycasting.
pub fn update_controller_cursor(
    mode: Res<ControllerCameraMode>,
    camera_query: Query<&ControllerCamera, With<Camera3d>>,
    editor_state: Res<EditorState>,
    mut cursor: ResMut<ControllerCursor>,
) {
    // Only update in first-person mode
    if *mode != ControllerCameraMode::FirstPerson {
        cursor.clear();
        return;
    }

    let Ok(controller_cam) = camera_query.get_single() else {
        cursor.clear();
        return;
    };

    let ray_origin = controller_cam.position;
    let ray_dir = controller_cam.forward_3d();

    // Try to hit a voxel first
    if let Some((voxel_pos, hit_info)) =
        find_closest_voxel_intersection(&editor_state, ray_origin, ray_dir)
    {
        cursor.update_from_hit(voxel_pos, hit_info);
    } else {
        // Fall back to ground plane
        if let Some(ground_hit) = intersect_ground_plane(ray_origin, ray_dir) {
            let distance = (ground_hit - ray_origin).length();
            cursor.update_from_ground(ground_hit, distance);
        } else {
            cursor.clear();
        }
    }
}

/// Marker component for the cursor visualization mesh.
#[derive(Component)]
pub struct CursorHighlight;

/// System to render the cursor highlight (wireframe cube).
pub fn render_cursor_highlight(
    mode: Res<ControllerCameraMode>,
    cursor: Res<ControllerCursor>,
    mut gizmos: Gizmos,
) {
    // Only render in first-person mode
    if *mode != ControllerCameraMode::FirstPerson {
        return;
    }

    // Render target voxel highlight (for removal - red/orange)
    if let Some(target) = cursor.target_voxel {
        let color = if cursor.in_reach {
            Color::srgba(1.0, 0.3, 0.3, 0.8)
        } else {
            Color::srgba(0.5, 0.2, 0.2, 0.5)
        };

        let center = Vec3::new(target.x as f32 + 0.5, target.y as f32 + 0.5, target.z as f32 + 0.5);
        gizmos.cuboid(
            Transform::from_translation(center).with_scale(Vec3::splat(1.02)),
            color,
        );
    }

    // Render placement position highlight (for placement - green/cyan)
    if let Some(placement) = cursor.placement_position {
        let color = if cursor.in_reach {
            Color::srgba(0.3, 1.0, 0.5, 0.6)
        } else {
            Color::srgba(0.2, 0.5, 0.3, 0.3)
        };

        let center = Vec3::new(
            placement.x as f32 + 0.5,
            placement.y as f32 + 0.5,
            placement.z as f32 + 0.5,
        );
        gizmos.cuboid(
            Transform::from_translation(center).with_scale(Vec3::splat(1.01)),
            color,
        );
    }
}

/// System to render a crosshair in the center of the screen.
pub fn render_crosshair(mode: Res<ControllerCameraMode>, _gizmos: Gizmos) {
    // Only render in first-person mode
    if *mode != ControllerCameraMode::FirstPerson {
        return;
    }

    // The crosshair is rendered via egui in the HUD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_default() {
        let cursor = ControllerCursor::new();
        assert!(cursor.target_voxel.is_none());
        assert!(cursor.placement_position.is_none());
        assert!(!cursor.in_reach);
        assert!((cursor.max_reach - 7.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cursor_clear() {
        let mut cursor = ControllerCursor::new();
        cursor.target_voxel = Some(IVec3::new(1, 2, 3));
        cursor.in_reach = true;

        cursor.clear();

        assert!(cursor.target_voxel.is_none());
        assert!(!cursor.in_reach);
    }

    #[test]
    fn test_cursor_update_from_hit() {
        let mut cursor = ControllerCursor::new();

        let hit_info = RayHitInfo {
            distance: 3.0,
            face_normal: Vec3::Y,
        };

        cursor.update_from_hit((5, 2, 8), hit_info);

        assert_eq!(cursor.target_voxel, Some(IVec3::new(5, 2, 8)));
        assert_eq!(cursor.placement_position, Some(IVec3::new(5, 3, 8))); // +Y
        assert!(cursor.in_reach);
    }

    #[test]
    fn test_cursor_out_of_reach() {
        let mut cursor = ControllerCursor::new();
        cursor.max_reach = 5.0;

        let hit_info = RayHitInfo {
            distance: 10.0,
            face_normal: Vec3::Y,
        };

        cursor.update_from_hit((5, 2, 8), hit_info);

        assert!(!cursor.in_reach);
    }
}
