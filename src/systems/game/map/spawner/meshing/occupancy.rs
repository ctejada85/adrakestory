//! Occupancy grid for fast neighbor lookups during face culling.

use super::{Face, SUB_VOXEL_COUNT};
use std::collections::HashSet;

/// Occupancy grid for fast neighbor lookups during face culling.
/// Uses a HashSet of sub-voxel global coordinates.
pub struct OccupancyGrid {
    occupied: HashSet<(i32, i32, i32)>,
}

impl OccupancyGrid {
    pub fn new() -> Self {
        Self {
            occupied: HashSet::new(),
        }
    }

    /// Insert an occupied position (voxel coords + sub-voxel coords combined into global sub-voxel coords).
    #[inline]
    pub fn insert(&mut self, x: i32, y: i32, z: i32, sub_x: i32, sub_y: i32, sub_z: i32) {
        // Convert to global sub-voxel coordinates
        let global_x = x * SUB_VOXEL_COUNT + sub_x;
        let global_y = y * SUB_VOXEL_COUNT + sub_y;
        let global_z = z * SUB_VOXEL_COUNT + sub_z;
        self.occupied.insert((global_x, global_y, global_z));
    }

    /// Check if a neighbor exists in the given direction.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn has_neighbor(
        &self,
        x: i32,
        y: i32,
        z: i32,
        sub_x: i32,
        sub_y: i32,
        sub_z: i32,
        face: Face,
    ) -> bool {
        let (dx, dy, dz) = face.offset();
        let global_x = x * SUB_VOXEL_COUNT + sub_x + dx;
        let global_y = y * SUB_VOXEL_COUNT + sub_y + dy;
        let global_z = z * SUB_VOXEL_COUNT + sub_z + dz;
        self.occupied.contains(&(global_x, global_y, global_z))
    }
}

impl Default for OccupancyGrid {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
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
}
