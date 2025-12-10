//! Sub-voxel geometry representation and transformation.
//!
//! This module provides a generic geometry system for representing and transforming
//! sub-voxel patterns within voxels. Instead of hardcoding rotation logic for each
//! pattern type, we store the actual 3D geometry and apply mathematical transformations.

use serde::{Deserialize, Serialize};

/// Rotation axis for 3D transformations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RotationAxis {
    /// Rotation around the X axis
    X,
    /// Rotation around the Y axis (default)
    #[default]
    Y,
    /// Rotation around the Z axis
    Z,
}

/// Represents the 3D geometry of sub-voxels within a voxel.
///
/// A voxel is divided into 8×8×8 sub-voxels. This structure efficiently stores
/// which sub-voxels are occupied using a bit array.
///
/// # Memory Layout
/// - 8 layers (Y-axis), each layer is 8×8 bits (64 bits = u64)
/// - Bit layout: `layer[y]` where bit `(z * 8 + x)` represents position `(x, y, z)`
/// - Total size: 64 bytes
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubVoxelGeometry {
    /// 8 layers (Y-axis), each layer is 8×8 bits
    layers: [u64; 8],
}

impl SubVoxelGeometry {
    /// Create an empty geometry with no occupied sub-voxels.
    pub fn new() -> Self {
        Self { layers: [0; 8] }
    }

    /// Check if a sub-voxel at the given position is occupied.
    ///
    /// # Arguments
    /// * `x` - X coordinate (0-7)
    /// * `y` - Y coordinate (0-7)
    /// * `z` - Z coordinate (0-7)
    ///
    /// # Returns
    /// `true` if the sub-voxel is occupied, `false` otherwise.
    pub fn is_occupied(&self, x: i32, y: i32, z: i32) -> bool {
        if !(0..8).contains(&x) || !(0..8).contains(&y) || !(0..8).contains(&z) {
            return false;
        }
        let bit_index = (z * 8 + x) as u64;
        (self.layers[y as usize] & (1u64 << bit_index)) != 0
    }

    /// Set a sub-voxel at the given position as occupied.
    ///
    /// # Arguments
    /// * `x` - X coordinate (0-7)
    /// * `y` - Y coordinate (0-7)
    /// * `z` - Z coordinate (0-7)
    pub fn set_occupied(&mut self, x: i32, y: i32, z: i32) {
        if !(0..8).contains(&x) || !(0..8).contains(&y) || !(0..8).contains(&z) {
            return;
        }
        let bit_index = (z * 8 + x) as u64;
        self.layers[y as usize] |= 1u64 << bit_index;
    }

    /// Clear a sub-voxel at the given position (mark as unoccupied).
    ///
    /// # Arguments
    /// * `x` - X coordinate (0-7)
    /// * `y` - Y coordinate (0-7)
    /// * `z` - Z coordinate (0-7)
    pub fn clear(&mut self, x: i32, y: i32, z: i32) {
        if !(0..8).contains(&x) || !(0..8).contains(&y) || !(0..8).contains(&z) {
            return;
        }
        let bit_index = (z * 8 + x) as u64;
        self.layers[y as usize] &= !(1u64 << bit_index);
    }

    /// Get an iterator over all occupied sub-voxel positions.
    ///
    /// # Returns
    /// An iterator yielding `(x, y, z)` tuples for each occupied sub-voxel.
    pub fn occupied_positions(&self) -> impl Iterator<Item = (i32, i32, i32)> + '_ {
        (0..8).flat_map(move |y| {
            (0..8).flat_map(move |z| {
                (0..8).filter_map(move |x| {
                    if self.is_occupied(x, y, z) {
                        Some((x, y, z))
                    } else {
                        None
                    }
                })
            })
        })
    }

    /// Count the number of occupied sub-voxels.
    pub fn count_occupied(&self) -> usize {
        self.layers
            .iter()
            .map(|layer| layer.count_ones() as usize)
            .sum()
    }

    /// Rotate this geometry around the given axis by the specified angle.
    ///
    /// # Arguments
    /// * `axis` - The axis to rotate around (X, Y, or Z)
    /// * `angle` - The rotation angle in 90° increments (0-3 for 0°, 90°, 180°, 270°)
    ///
    /// # Returns
    /// A new `SubVoxelGeometry` with the rotated positions.
    pub fn rotate(&self, axis: RotationAxis, angle: i32) -> Self {
        let mut result = Self::new();
        let angle = angle.rem_euclid(4);

        // No rotation needed
        if angle == 0 {
            return self.clone();
        }

        // Rotate each occupied sub-voxel
        for (x, y, z) in self.occupied_positions() {
            let (nx, ny, nz) = Self::rotate_point(x, y, z, axis, angle);
            result.set_occupied(nx, ny, nz);
        }

        result
    }

    /// Rotate a single point around an axis.
    ///
    /// This uses standard 3D rotation matrices for 90° increments.
    /// Coordinates are centered around (3.5, 3.5, 3.5) for rotation.
    fn rotate_point(x: i32, y: i32, z: i32, axis: RotationAxis, angle: i32) -> (i32, i32, i32) {
        // Center coordinates around origin (subtract 3.5, but we use integer math)
        // We'll work with doubled coordinates to avoid floating point
        let cx = x * 2 - 7;
        let cy = y * 2 - 7;
        let cz = z * 2 - 7;

        let (rx, ry, rz) = match axis {
            RotationAxis::X => match angle {
                1 => (cx, -cz, cy),  // 90° CW around X
                2 => (cx, -cy, -cz), // 180° around X
                3 => (cx, cz, -cy),  // 270° CW around X
                _ => (cx, cy, cz),
            },
            RotationAxis::Y => match angle {
                1 => (cz, cy, -cx),  // 90° CW around Y
                2 => (-cx, cy, -cz), // 180° around Y
                3 => (-cz, cy, cx),  // 270° CW around Y
                _ => (cx, cy, cz),
            },
            RotationAxis::Z => match angle {
                1 => (-cy, cx, cz),  // 90° CW around Z
                2 => (-cx, -cy, cz), // 180° around Z
                3 => (cy, -cx, cz),  // 270° CW around Z
                _ => (cx, cy, cz),
            },
        };

        // Translate back (add 3.5 and convert back to integer)
        ((rx + 7) / 2, (ry + 7) / 2, (rz + 7) / 2)
    }

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
}

impl Default for SubVoxelGeometry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_geometry() {
        let geom = SubVoxelGeometry::new();
        assert_eq!(geom.count_occupied(), 0);
        assert!(!geom.is_occupied(0, 0, 0));
    }

    #[test]
    fn test_set_and_check_occupied() {
        let mut geom = SubVoxelGeometry::new();
        geom.set_occupied(3, 4, 5);
        assert!(geom.is_occupied(3, 4, 5));
        assert!(!geom.is_occupied(3, 4, 6));
        assert_eq!(geom.count_occupied(), 1);
    }

    #[test]
    fn test_clear() {
        let mut geom = SubVoxelGeometry::new();
        geom.set_occupied(2, 3, 4);
        assert!(geom.is_occupied(2, 3, 4));
        geom.clear(2, 3, 4);
        assert!(!geom.is_occupied(2, 3, 4));
    }

    #[test]
    fn test_full_geometry() {
        let geom = SubVoxelGeometry::full();
        assert_eq!(geom.count_occupied(), 512); // 8×8×8
        assert!(geom.is_occupied(0, 0, 0));
        assert!(geom.is_occupied(7, 7, 7));
    }

    #[test]
    fn test_platform_horizontal() {
        let geom = SubVoxelGeometry::platform_horizontal();
        assert_eq!(geom.count_occupied(), 64); // 8×1×8
        assert!(geom.is_occupied(0, 0, 0));
        assert!(geom.is_occupied(7, 0, 7));
        assert!(!geom.is_occupied(0, 1, 0));
    }

    #[test]
    fn test_staircase_x() {
        let geom = SubVoxelGeometry::staircase_x();
        // Step 0: 1 high, Step 1: 2 high, ..., Step 7: 8 high
        // Total: (1+2+3+4+5+6+7+8) * 8 = 36 * 8 = 288
        assert_eq!(geom.count_occupied(), 288);
        assert!(geom.is_occupied(0, 0, 0));
        assert!(geom.is_occupied(7, 7, 7));
        assert!(!geom.is_occupied(0, 1, 0)); // Step 0 is only 1 high
    }

    #[test]
    fn test_pillar() {
        let geom = SubVoxelGeometry::pillar();
        assert_eq!(geom.count_occupied(), 8); // 2×2×2
        assert!(geom.is_occupied(3, 3, 3));
        assert!(geom.is_occupied(4, 4, 4));
        assert!(!geom.is_occupied(2, 3, 3));
    }

    #[test]
    fn test_fence_x() {
        let geom = SubVoxelGeometry::fence_x();
        // Left post: 1×8×1 = 8
        // Right post: 1×8×1 = 8
        // Bottom rail: 6×1×1 = 6
        // Top rail: 6×1×1 = 6
        // Total: 8 + 8 + 6 + 6 = 28
        assert_eq!(geom.count_occupied(), 28);
        // Check left post at z=0 edge
        assert!(geom.is_occupied(0, 0, 0));
        assert!(geom.is_occupied(0, 7, 0));
        // Check right post at z=0 edge
        assert!(geom.is_occupied(7, 0, 0));
        assert!(geom.is_occupied(7, 7, 0));
        // Check rails at z=0 edge
        assert!(geom.is_occupied(3, 2, 0)); // bottom rail
        assert!(geom.is_occupied(3, 5, 0)); // top rail
        // Check gaps
        assert!(!geom.is_occupied(3, 0, 0)); // below bottom rail
        assert!(!geom.is_occupied(3, 4, 0)); // between rails
    }

    #[test]
    fn test_rotation_preserves_count() {
        let geom = SubVoxelGeometry::platform_horizontal();
        let count = geom.count_occupied();

        let rotated_x = geom.rotate(RotationAxis::X, 1);
        assert_eq!(rotated_x.count_occupied(), count);

        let rotated_y = geom.rotate(RotationAxis::Y, 1);
        assert_eq!(rotated_y.count_occupied(), count);

        let rotated_z = geom.rotate(RotationAxis::Z, 1);
        assert_eq!(rotated_z.count_occupied(), count);
    }

    #[test]
    fn test_rotation_360_returns_original() {
        let geom = SubVoxelGeometry::staircase_x();

        let rotated = geom
            .rotate(RotationAxis::Y, 1)
            .rotate(RotationAxis::Y, 1)
            .rotate(RotationAxis::Y, 1)
            .rotate(RotationAxis::Y, 1);

        assert_eq!(geom, rotated);
    }

    #[test]
    fn test_rotation_180_twice_returns_original() {
        let geom = SubVoxelGeometry::platform_horizontal();

        let rotated = geom.rotate(RotationAxis::X, 2).rotate(RotationAxis::X, 2);

        assert_eq!(geom, rotated);
    }

    #[test]
    fn test_no_rotation() {
        let geom = SubVoxelGeometry::full();
        let rotated = geom.rotate(RotationAxis::Y, 0);
        assert_eq!(geom, rotated);
    }

    #[test]
    fn test_platform_rotation_x_90() {
        let platform = SubVoxelGeometry::platform_horizontal();
        let rotated = platform.rotate(RotationAxis::X, 1);

        // After 90° rotation around X, horizontal platform (8×1×8) becomes vertical (8×8×1)
        // All sub-voxels should be at z=0
        for (_, _, z) in rotated.occupied_positions() {
            assert_eq!(z, 0, "All sub-voxels should be at z=0 after X rotation");
        }
        assert_eq!(rotated.count_occupied(), 64);
    }

    #[test]
    fn test_platform_rotation_z_90() {
        let platform = SubVoxelGeometry::platform_horizontal();
        let rotated = platform.rotate(RotationAxis::Z, 1);

        // After 90° rotation around Z, horizontal platform (8×1×8) becomes vertical (1×8×8)
        // All sub-voxels should be at x=7 (rotated to the far side)
        for (x, _, _) in rotated.occupied_positions() {
            assert_eq!(x, 7, "All sub-voxels should be at x=7 after Z rotation");
        }
        assert_eq!(rotated.count_occupied(), 64);
    }

    #[test]
    fn test_occupied_positions_iterator() {
        let mut geom = SubVoxelGeometry::new();
        geom.set_occupied(1, 2, 3);
        geom.set_occupied(4, 5, 6);

        let positions: Vec<_> = geom.occupied_positions().collect();
        assert_eq!(positions.len(), 2);
        assert!(positions.contains(&(1, 2, 3)));
        assert!(positions.contains(&(4, 5, 6)));
    }

    #[test]
    fn test_bounds_checking() {
        let mut geom = SubVoxelGeometry::new();

        // Out of bounds should not panic
        geom.set_occupied(-1, 0, 0);
        geom.set_occupied(8, 0, 0);
        geom.set_occupied(0, -1, 0);
        geom.set_occupied(0, 8, 0);

        assert_eq!(geom.count_occupied(), 0);
        assert!(!geom.is_occupied(-1, 0, 0));
        assert!(!geom.is_occupied(8, 0, 0));
    }
}
