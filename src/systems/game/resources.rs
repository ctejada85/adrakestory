use bevy::prelude::*;
use super::components::VoxelType;

#[derive(Resource)]
pub struct VoxelWorld {
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    pub voxels: Vec<Vec<Vec<VoxelType>>>,
}

impl Default for VoxelWorld {
    fn default() -> Self {
        let width = 4;
        let height = 3;
        let depth = 4;

        // Initialize with air
        let mut voxels = vec![vec![vec![VoxelType::Air; depth as usize]; height as usize]; width as usize];

        // Create a floor layer (y=0) with grass
        for x in 0..width as usize {
            for z in 0..depth as usize {
                voxels[x][0][z] = VoxelType::Grass;
            }
        }

        // Add voxels on top of the four corners
        // Corners: (0,0), (0,3), (3,0), (3,3)
        voxels[0][1][0] = VoxelType::Stone;
        voxels[0][1][3] = VoxelType::Stone;
        voxels[3][1][0] = VoxelType::Stone;
        voxels[3][1][3] = VoxelType::Stone;

        Self {
            width,
            height,
            depth,
            voxels,
        }
    }
}

impl VoxelWorld {
    pub fn get_voxel(&self, x: i32, y: i32, z: i32) -> Option<VoxelType> {
        if x >= 0 && x < self.width && y >= 0 && y < self.height && z >= 0 && z < self.depth {
            Some(self.voxels[x as usize][y as usize][z as usize])
        } else {
            None
        }
    }

    pub fn set_voxel(&mut self, x: i32, y: i32, z: i32, voxel_type: VoxelType) {
        if x >= 0 && x < self.width && y >= 0 && y < self.height && z >= 0 && z < self.depth {
            self.voxels[x as usize][y as usize][z as usize] = voxel_type;
        }
    }
}