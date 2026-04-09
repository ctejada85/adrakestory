use super::*;

#[test]
fn test_world_to_grid_coords_positive() {
    // Simple positive coordinates
    assert_eq!(
        SpatialGrid::world_to_grid_coords(Vec3::new(0.5, 0.5, 0.5)),
        IVec3::new(0, 0, 0)
    );
    assert_eq!(
        SpatialGrid::world_to_grid_coords(Vec3::new(1.5, 2.5, 3.5)),
        IVec3::new(1, 2, 3)
    );
}

#[test]
fn test_world_to_grid_coords_negative() {
    // Negative coordinates should floor correctly
    assert_eq!(
        SpatialGrid::world_to_grid_coords(Vec3::new(-0.5, -0.5, -0.5)),
        IVec3::new(-1, -1, -1)
    );
    assert_eq!(
        SpatialGrid::world_to_grid_coords(Vec3::new(-1.5, -2.5, -3.5)),
        IVec3::new(-2, -3, -4)
    );
}

#[test]
fn test_world_to_grid_coords_on_boundary() {
    // Exact boundaries
    assert_eq!(
        SpatialGrid::world_to_grid_coords(Vec3::new(1.0, 2.0, 3.0)),
        IVec3::new(1, 2, 3)
    );
    assert_eq!(
        SpatialGrid::world_to_grid_coords(Vec3::new(0.0, 0.0, 0.0)),
        IVec3::new(0, 0, 0)
    );
}

#[test]
fn test_insert_and_query_single_entity() {
    let mut grid = SpatialGrid::default();
    let entity = Entity::from_raw_u32(42).unwrap();
    let cell = IVec3::new(1, 2, 3);

    grid.cells.entry(cell).or_default().push(entity);

    let result = grid.get_entities_in_cell(cell);
    assert!(result.is_some());
    assert_eq!(result.unwrap().len(), 1);
    assert_eq!(result.unwrap()[0], entity);
}

#[test]
fn test_get_entities_in_cell_empty() {
    let grid = SpatialGrid::default();
    assert!(grid.get_entities_in_cell(IVec3::new(0, 0, 0)).is_none());
}

#[test]
fn test_get_entities_in_aabb_single_cell() {
    let mut grid = SpatialGrid::default();
    let entity = Entity::from_raw_u32(1).unwrap();
    grid.cells
        .entry(IVec3::new(0, 0, 0))
        .or_default()
        .push(entity);

    let entities = grid.get_entities_in_aabb(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.5, 0.5, 0.5));
    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0], entity);
}

#[test]
fn test_get_entities_in_aabb_multiple_cells() {
    let mut grid = SpatialGrid::default();
    let e1 = Entity::from_raw_u32(1).unwrap();
    let e2 = Entity::from_raw_u32(2).unwrap();
    let e3 = Entity::from_raw_u32(3).unwrap();

    grid.cells.entry(IVec3::new(0, 0, 0)).or_default().push(e1);
    grid.cells.entry(IVec3::new(1, 0, 0)).or_default().push(e2);
    grid.cells.entry(IVec3::new(0, 1, 0)).or_default().push(e3);

    // Query spanning multiple cells
    let entities = grid.get_entities_in_aabb(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.5, 1.5, 0.5));
    assert_eq!(entities.len(), 3);
}

#[test]
fn test_get_entities_in_aabb_empty_cells() {
    let mut grid = SpatialGrid::default();
    let entity = Entity::from_raw_u32(1).unwrap();
    grid.cells
        .entry(IVec3::new(5, 5, 5))
        .or_default()
        .push(entity);

    // Query area that doesn't include the entity's cell
    let entities = grid.get_entities_in_aabb(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
    assert!(entities.is_empty());
}

#[test]
fn test_cell_boundary_handling() {
    let mut grid = SpatialGrid::default();
    let entity = Entity::from_raw_u32(1).unwrap();
    // Entity at exact boundary (1.0, 1.0, 1.0) should be in cell (1, 1, 1)
    grid.cells
        .entry(IVec3::new(1, 1, 1))
        .or_default()
        .push(entity);

    // Query that includes cell (1, 1, 1)
    let entities = grid.get_entities_in_aabb(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.5, 1.5, 1.5));
    assert_eq!(entities.len(), 1);
}
