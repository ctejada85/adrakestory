use super::*;

#[test]
fn test_create_infinite_grid_mesh() {
    let config = InfiniteGridConfig::default();
    let camera_pos = Vec3::ZERO;
    let mesh = create_infinite_grid_mesh(&config, camera_pos, None);

    // Verify mesh was created
    assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
    assert!(mesh.indices().is_some());
}
