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
