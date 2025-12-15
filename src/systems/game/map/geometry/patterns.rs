//! Pattern factory methods for SubVoxelGeometry.

use super::sub_voxel_geometry::SubVoxelGeometry;

impl SubVoxelGeometry {
    /// Create a full 8×8×8 cube of sub-voxels.
    pub fn full() -> Self {
        let mut geom = Self::new();
        for x in 0..8 {
            for y in 0..8 {
                for z in 0..8 {
                    geom.set_occupied(x, y, z);
                }
            }
        }
        geom
    }

    /// Create a horizontal platform (8×1×8).
    ///
    /// This is a thin slab on the XZ plane at Y=0.
    pub fn platform_horizontal() -> Self {
        let mut geom = Self::new();
        for x in 0..8 {
            for z in 0..8 {
                geom.set_occupied(x, 0, z);
            }
        }
        geom
    }

    /// Create stairs ascending in the +X direction.
    ///
    /// Each step in X has progressively more height in Y.
    pub fn staircase_x() -> Self {
        let mut geom = Self::new();
        for step in 0..8 {
            let height = step + 1;
            for y in 0..height {
                for z in 0..8 {
                    geom.set_occupied(step, y, z);
                }
            }
        }
        geom
    }

    /// Create a small 2×2×2 centered pillar.
    ///
    /// The pillar is centered at (3.5, 3.5, 3.5) and occupies
    /// sub-voxels from (3,3,3) to (4,4,4).
    pub fn pillar() -> Self {
        let mut geom = Self::new();
        for x in 3..5 {
            for y in 3..5 {
                for z in 3..5 {
                    geom.set_occupied(x, y, z);
                }
            }
        }
        geom
    }

    /// Create a fence pattern along the X axis.
    ///
    /// Fence has thin vertical posts at both ends and horizontal rails connecting them.
    /// Positioned at z=0 edge of the voxel so it can be placed at perimeters.
    pub fn fence_x() -> Self {
        let mut geom = Self::new();
        // Vertical posts at x=0 (left post) - thin 1x8x1 column at z=0 edge
        for y in 0..8 {
            geom.set_occupied(0, y, 0);
        }
        // Vertical posts at x=7 (right post) - thin 1x8x1 column at z=0 edge
        for y in 0..8 {
            geom.set_occupied(7, y, 0);
        }
        // Bottom horizontal rail (y=2) at z=0 edge
        for x in 1..7 {
            geom.set_occupied(x, 2, 0);
        }
        // Top horizontal rail (y=5) at z=0 edge
        for x in 1..7 {
            geom.set_occupied(x, 5, 0);
        }
        geom
    }

    /// Create a fence corner pattern (L-shaped).
    ///
    /// Corner post at (0,y,0) with rails extending along +X and +Z edges.
    pub fn fence_corner() -> Self {
        let mut geom = Self::new();
        // Corner post at (0,0) - full height
        for y in 0..8 {
            geom.set_occupied(0, y, 0);
        }
        // Rail along X axis (z=0 edge)
        for x in 1..8 {
            geom.set_occupied(x, 2, 0); // bottom rail
            geom.set_occupied(x, 5, 0); // top rail
        }
        // Rail along Z axis (x=0 edge)
        for z in 1..8 {
            geom.set_occupied(0, 2, z); // bottom rail
            geom.set_occupied(0, 5, z); // top rail
        }
        geom
    }

    /// Create a fence post (single vertical stick in center).
    ///
    /// This is the base fence shape when no neighbors are connected.
    pub fn fence_post() -> Self {
        let mut geom = Self::new();
        // Vertical post in center (x=3-4, z=3-4)
        for y in 0..8 {
            for x in 3..5 {
                for z in 3..5 {
                    geom.set_occupied(x, y, z);
                }
            }
        }
        geom
    }

    /// Create a fence with connections to neighboring fences.
    ///
    /// Generates geometry based on which directions have adjacent fences.
    /// The center post is always present, with rails extending to connected neighbors.
    ///
    /// # Arguments
    /// * `neg_x` - Connect to fence in -X direction
    /// * `pos_x` - Connect to fence in +X direction
    /// * `neg_z` - Connect to fence in -Z direction
    /// * `pos_z` - Connect to fence in +Z direction
    pub fn fence_with_connections(neg_x: bool, pos_x: bool, neg_z: bool, pos_z: bool) -> Self {
        let mut geom = Self::new();

        // Center post (always present) - 2x8x2 in center
        for y in 0..8 {
            for x in 3..5 {
                for z in 3..5 {
                    geom.set_occupied(x, y, z);
                }
            }
        }

        // Rails to -X direction
        if neg_x {
            for x in 0..3 {
                for z in 3..5 {
                    geom.set_occupied(x, 2, z); // bottom rail
                    geom.set_occupied(x, 5, z); // top rail
                }
            }
        }

        // Rails to +X direction
        if pos_x {
            for x in 5..8 {
                for z in 3..5 {
                    geom.set_occupied(x, 2, z); // bottom rail
                    geom.set_occupied(x, 5, z); // top rail
                }
            }
        }

        // Rails to -Z direction
        if neg_z {
            for z in 0..3 {
                for x in 3..5 {
                    geom.set_occupied(x, 2, z); // bottom rail
                    geom.set_occupied(x, 5, z); // top rail
                }
            }
        }

        // Rails to +Z direction
        if pos_z {
            for z in 5..8 {
                for x in 3..5 {
                    geom.set_occupied(x, 2, z); // bottom rail
                    geom.set_occupied(x, 5, z); // top rail
                }
            }
        }

        geom
    }
}
