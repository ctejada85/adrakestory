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
