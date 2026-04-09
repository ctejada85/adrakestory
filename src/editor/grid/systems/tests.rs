use super::*;

#[test]
fn test_should_regenerate_grid() {
    let pos1 = Vec3::new(0.0, 0.0, 0.0);
    let pos2 = Vec3::new(1.0, 0.0, 0.0);
    let pos3 = Vec3::new(3.0, 0.0, 0.0);

    assert!(!should_regenerate_grid(pos1, pos2, 2.0));
    assert!(should_regenerate_grid(pos1, pos3, 2.0));
}
