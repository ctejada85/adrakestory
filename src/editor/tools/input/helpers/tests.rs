use super::*;

// -------------------------------------------------------------------------
// rotate_position Tests - Y Axis
// -------------------------------------------------------------------------

#[test]
fn test_rotate_position_y_axis_0_degrees() {
    let pos = (2, 0, 0);
    let pivot = Vec3::new(0.0, 0.0, 0.0);
    let result = rotate_position(pos, pivot, RotationAxis::Y, 0);
    assert_eq!(result, (2, 0, 0));
}

#[test]
fn test_rotate_position_y_axis_90_degrees() {
    let pos = (2, 0, 0);
    let pivot = Vec3::new(0.0, 0.0, 0.0);
    let result = rotate_position(pos, pivot, RotationAxis::Y, 1);
    // 90° CW around Y: (x, y, z) -> (z, y, -x)
    assert_eq!(result, (0, 0, -2));
}

#[test]
fn test_rotate_position_y_axis_180_degrees() {
    let pos = (2, 0, 0);
    let pivot = Vec3::new(0.0, 0.0, 0.0);
    let result = rotate_position(pos, pivot, RotationAxis::Y, 2);
    // 180° around Y: (x, y, z) -> (-x, y, -z)
    assert_eq!(result, (-2, 0, 0));
}

#[test]
fn test_rotate_position_y_axis_270_degrees() {
    let pos = (2, 0, 0);
    let pivot = Vec3::new(0.0, 0.0, 0.0);
    let result = rotate_position(pos, pivot, RotationAxis::Y, 3);
    // 270° CW around Y: (x, y, z) -> (-z, y, x)
    assert_eq!(result, (0, 0, 2));
}

// -------------------------------------------------------------------------
// rotate_position Tests - X Axis
// -------------------------------------------------------------------------

#[test]
fn test_rotate_position_x_axis_90_degrees() {
    let pos = (0, 2, 0);
    let pivot = Vec3::new(0.0, 0.0, 0.0);
    let result = rotate_position(pos, pivot, RotationAxis::X, 1);
    // 90° CW around X: (x, y, z) -> (x, -z, y)
    assert_eq!(result, (0, 0, 2));
}

#[test]
fn test_rotate_position_x_axis_180_degrees() {
    let pos = (0, 2, 0);
    let pivot = Vec3::new(0.0, 0.0, 0.0);
    let result = rotate_position(pos, pivot, RotationAxis::X, 2);
    // 180° around X: (x, y, z) -> (x, -y, -z)
    assert_eq!(result, (0, -2, 0));
}

// -------------------------------------------------------------------------
// rotate_position Tests - Z Axis
// -------------------------------------------------------------------------

#[test]
fn test_rotate_position_z_axis_90_degrees() {
    let pos = (2, 0, 0);
    let pivot = Vec3::new(0.0, 0.0, 0.0);
    let result = rotate_position(pos, pivot, RotationAxis::Z, 1);
    // 90° CW around Z: (x, y, z) -> (-y, x, z)
    assert_eq!(result, (0, 2, 0));
}

#[test]
fn test_rotate_position_z_axis_180_degrees() {
    let pos = (2, 0, 0);
    let pivot = Vec3::new(0.0, 0.0, 0.0);
    let result = rotate_position(pos, pivot, RotationAxis::Z, 2);
    // 180° around Z: (x, y, z) -> (-x, -y, z)
    assert_eq!(result, (-2, 0, 0));
}

// -------------------------------------------------------------------------
// rotate_position Tests - With Pivot
// -------------------------------------------------------------------------

#[test]
fn test_rotate_position_with_pivot() {
    // Rotate (3, 0, 0) around pivot (1, 0, 0) by 90° on Y axis
    // Relative pos is (2, 0, 0), after rotation (0, 0, -2), plus pivot = (1, 0, -2)
    let pos = (3, 0, 0);
    let pivot = Vec3::new(1.0, 0.0, 0.0);
    let result = rotate_position(pos, pivot, RotationAxis::Y, 1);
    assert_eq!(result, (1, 0, -2));
}

#[test]
fn test_rotate_position_at_pivot() {
    // Position at pivot should not move
    let pos = (5, 5, 5);
    let pivot = Vec3::new(5.0, 5.0, 5.0);
    let result = rotate_position(pos, pivot, RotationAxis::Y, 2);
    assert_eq!(result, (5, 5, 5));
}

#[test]
fn test_rotate_position_invalid_angle() {
    // Invalid angle (> 3) should return unrotated position
    let pos = (2, 0, 0);
    let pivot = Vec3::new(0.0, 0.0, 0.0);
    let result = rotate_position(pos, pivot, RotationAxis::Y, 4);
    assert_eq!(result, (2, 0, 0));
}

#[test]
fn test_rotate_position_negative_coords() {
    let pos = (-2, 1, 3);
    let pivot = Vec3::new(0.0, 0.0, 0.0);
    let result = rotate_position(pos, pivot, RotationAxis::Y, 1);
    // 90° CW around Y: (x, y, z) -> (z, y, -x)
    assert_eq!(result, (3, 1, 2));
}

#[test]
fn test_rotate_position_full_360() {
    // Four 90° rotations should return to original
    let pos = (2, 3, 4);
    let pivot = Vec3::new(0.0, 0.0, 0.0);

    let r1 = rotate_position(pos, pivot, RotationAxis::Y, 1);
    let r2 = rotate_position(r1, pivot, RotationAxis::Y, 1);
    let r3 = rotate_position(r2, pivot, RotationAxis::Y, 1);
    let r4 = rotate_position(r3, pivot, RotationAxis::Y, 1);

    assert_eq!(r4, pos);
}
