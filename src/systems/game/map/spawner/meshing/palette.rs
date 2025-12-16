//! Material palette for efficient voxel rendering.

use bevy::prelude::*;

/// Pre-generated material palette for efficient voxel rendering.
///
/// Instead of creating a unique material for each sub-voxel (which would be millions
/// of materials in a large world), we pre-create a fixed palette and hash positions
/// to palette indices. This enables GPU batching and reduces memory usage by 99.99%.
#[derive(Resource, Clone)]
pub struct VoxelMaterialPalette {
    pub materials: Vec<Handle<StandardMaterial>>,
}

impl VoxelMaterialPalette {
    /// Number of unique materials in the palette.
    /// 64 provides good visual variety while keeping material count low.
    pub const PALETTE_SIZE: usize = 64;

    /// Create a new material palette with pre-generated colors.
    pub fn new(materials_asset: &mut Assets<StandardMaterial>) -> Self {
        let materials: Vec<_> = (0..Self::PALETTE_SIZE)
            .map(|i| {
                let t = i as f32 / Self::PALETTE_SIZE as f32;
                // Generate visually distinct colors across the palette
                let color = Color::srgb(
                    0.2 + t * 0.6,
                    0.3 + ((t * std::f32::consts::PI * 2.0).sin() * 0.5 + 0.5) * 0.4,
                    0.4 + ((t * std::f32::consts::PI * 3.0).cos() * 0.5 + 0.5) * 0.4,
                );
                materials_asset.add(color)
            })
            .collect();

        Self { materials }
    }

    /// Get material index for a sub-voxel position using spatial hashing.
    /// Uses prime number multiplication for good hash distribution.
    #[inline]
    pub fn get_material_index(x: i32, y: i32, z: i32, sub_x: i32, sub_y: i32, sub_z: i32) -> usize {
        // Spatial hash using prime numbers for good distribution
        let hash = (x.wrapping_mul(73856093))
            ^ (y.wrapping_mul(19349663))
            ^ (z.wrapping_mul(83492791))
            ^ (sub_x.wrapping_mul(15485863))
            ^ (sub_y.wrapping_mul(32452843))
            ^ (sub_z.wrapping_mul(49979687));
        (hash.unsigned_abs() as usize) % Self::PALETTE_SIZE
    }

    /// Get a material handle for the given sub-voxel position.
    #[inline]
    pub fn get_material(
        &self,
        x: i32,
        y: i32,
        z: i32,
        sub_x: i32,
        sub_y: i32,
        sub_z: i32,
    ) -> Handle<StandardMaterial> {
        let index = Self::get_material_index(x, y, z, sub_x, sub_y, sub_z);
        self.materials[index].clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_size_is_64() {
        assert_eq!(VoxelMaterialPalette::PALETTE_SIZE, 64);
    }

    #[test]
    fn test_material_index_in_range() {
        // Test various positions to ensure index is always in valid range
        let test_positions = [
            (0, 0, 0, 0, 0, 0),
            (1, 2, 3, 4, 5, 6),
            (-1, -2, -3, 0, 0, 0),
            (100, 200, 300, 7, 7, 7),
            (i32::MAX, i32::MAX, i32::MAX, 7, 7, 7),
            (i32::MIN, i32::MIN, i32::MIN, 0, 0, 0),
        ];

        for (x, y, z, sx, sy, sz) in test_positions {
            let index = VoxelMaterialPalette::get_material_index(x, y, z, sx, sy, sz);
            assert!(
                index < VoxelMaterialPalette::PALETTE_SIZE,
                "Index {} out of range for position ({}, {}, {}, {}, {}, {})",
                index,
                x,
                y,
                z,
                sx,
                sy,
                sz
            );
        }
    }

    #[test]
    fn test_same_position_same_index() {
        let index1 = VoxelMaterialPalette::get_material_index(5, 10, 15, 3, 4, 5);
        let index2 = VoxelMaterialPalette::get_material_index(5, 10, 15, 3, 4, 5);
        assert_eq!(index1, index2);
    }

    #[test]
    fn test_different_positions_may_differ() {
        let index1 = VoxelMaterialPalette::get_material_index(0, 0, 0, 0, 0, 0);
        let index2 = VoxelMaterialPalette::get_material_index(1, 0, 0, 0, 0, 0);
        let index3 = VoxelMaterialPalette::get_material_index(0, 1, 0, 0, 0, 0);
        let index4 = VoxelMaterialPalette::get_material_index(0, 0, 1, 0, 0, 0);

        // While not guaranteed to be different, they should not all be the same
        // due to the prime-based hash
        let all_same = index1 == index2 && index2 == index3 && index3 == index4;
        assert!(!all_same, "Hash collision for adjacent positions");
    }

    #[test]
    fn test_hash_distribution() {
        // Test that the hash function produces reasonable distribution
        // by checking that we get multiple different indices over a range
        let mut indices = std::collections::HashSet::new();

        for x in 0..8 {
            for y in 0..8 {
                for z in 0..8 {
                    let index = VoxelMaterialPalette::get_material_index(x, y, z, 0, 0, 0);
                    indices.insert(index);
                }
            }
        }

        // We should get a reasonable spread of indices (at least 10 unique values)
        assert!(
            indices.len() >= 10,
            "Poor hash distribution: only {} unique indices",
            indices.len()
        );
    }

    #[test]
    fn test_sub_voxel_coords_affect_index() {
        let index1 = VoxelMaterialPalette::get_material_index(0, 0, 0, 0, 0, 0);
        let index2 = VoxelMaterialPalette::get_material_index(0, 0, 0, 1, 0, 0);
        let index3 = VoxelMaterialPalette::get_material_index(0, 0, 0, 0, 1, 0);
        let index4 = VoxelMaterialPalette::get_material_index(0, 0, 0, 0, 0, 1);

        // Sub-voxel coordinates should affect the hash
        let all_same = index1 == index2 && index2 == index3 && index3 == index4;
        assert!(!all_same, "Sub-voxel coordinates don't affect hash");
    }
}
