use super::*;

#[test]
fn test_calculate_grid_bounds() {
    let camera_pos = Vec3::new(5.0, 0.0, 5.0);
    let bounds = calculate_grid_bounds(camera_pos, 10.0, 1.0);

    assert!(bounds.min_x <= -5.0);
    assert!(bounds.max_x >= 15.0);
    assert!(bounds.min_z <= -5.0);
    assert!(bounds.max_z >= 15.0);
}
