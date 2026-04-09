use super::*;

#[test]
fn test_create_cursor_mesh() {
    let mesh = create_cursor_mesh();

    // Verify mesh was created with correct topology
    assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
    assert!(mesh.indices().is_some());
}
