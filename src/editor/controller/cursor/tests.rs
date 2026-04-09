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
