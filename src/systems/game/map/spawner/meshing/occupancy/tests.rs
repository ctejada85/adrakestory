use super::*;

#[test]
fn test_new_creates_empty_grid() {
    let grid = OccupancyGrid::new();
    // Check that a random position is not occupied
    assert!(!grid.has_neighbor(0, 0, 0, 0, 0, 0, Face::PosX));
}

#[test]
fn test_insert_and_has_neighbor() {
    let mut grid = OccupancyGrid::new();
    // Insert a sub-voxel at (0,0,0) voxel, (1,0,0) sub-voxel
    grid.insert(0, 0, 0, 1, 0, 0);

    // Check that (0,0,0,0,0,0) has a neighbor in +X direction
    assert!(grid.has_neighbor(0, 0, 0, 0, 0, 0, Face::PosX));
}

#[test]
fn test_has_neighbor_neg_x() {
    let mut grid = OccupancyGrid::new();
    grid.insert(0, 0, 0, 0, 0, 0);

    // Check that (0,0,0,1,0,0) has a neighbor in -X direction
    assert!(grid.has_neighbor(0, 0, 0, 1, 0, 0, Face::NegX));
}

#[test]
fn test_has_neighbor_crosses_voxel_boundary() {
    let mut grid = OccupancyGrid::new();
    // Insert at voxel (1,0,0), sub-voxel (0,0,0)
    // This is global sub-voxel (8,0,0)
    grid.insert(1, 0, 0, 0, 0, 0);

    // Check that voxel (0,0,0), sub-voxel (7,0,0) has a neighbor in +X
    // Global sub-voxel (7,0,0) + (1,0,0) = (8,0,0)
    assert!(grid.has_neighbor(0, 0, 0, 7, 0, 0, Face::PosX));
}

#[test]
fn test_has_neighbor_pos_y() {
    let mut grid = OccupancyGrid::new();
    grid.insert(0, 0, 0, 0, 1, 0);

    assert!(grid.has_neighbor(0, 0, 0, 0, 0, 0, Face::PosY));
}

#[test]
fn test_has_neighbor_neg_y() {
    let mut grid = OccupancyGrid::new();
    grid.insert(0, 0, 0, 0, 0, 0);

    assert!(grid.has_neighbor(0, 0, 0, 0, 1, 0, Face::NegY));
}

#[test]
fn test_has_neighbor_pos_z() {
    let mut grid = OccupancyGrid::new();
    grid.insert(0, 0, 0, 0, 0, 1);

    assert!(grid.has_neighbor(0, 0, 0, 0, 0, 0, Face::PosZ));
}

#[test]
fn test_has_neighbor_neg_z() {
    let mut grid = OccupancyGrid::new();
    grid.insert(0, 0, 0, 0, 0, 0);

    assert!(grid.has_neighbor(0, 0, 0, 0, 0, 1, Face::NegZ));
}

#[test]
fn test_no_false_positives() {
    let mut grid = OccupancyGrid::new();
    grid.insert(0, 0, 0, 0, 0, 0);

    // Check that positions far away don't falsely report neighbors
    assert!(!grid.has_neighbor(5, 5, 5, 0, 0, 0, Face::PosX));
    assert!(!grid.has_neighbor(5, 5, 5, 0, 0, 0, Face::NegX));
}

#[test]
fn test_global_coordinate_calculation() {
    let mut grid = OccupancyGrid::new();
    // Voxel (2, 3, 4), sub-voxel (5, 6, 7)
    // Global = (2*8+5, 3*8+6, 4*8+7) = (21, 30, 39)
    grid.insert(2, 3, 4, 5, 6, 7);

    // The neighbor at (2, 3, 4, 4, 6, 7) in +X should exist
    assert!(grid.has_neighbor(2, 3, 4, 4, 6, 7, Face::PosX));
}
