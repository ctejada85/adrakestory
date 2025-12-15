//! Core SubVoxelGeometry struct and basic operations.

use serde::{Deserialize, Serialize};

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
    pub(crate) layers: [u64; 8],
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
}

impl Default for SubVoxelGeometry {
    fn default() -> Self {
        Self::new()
    }
}
