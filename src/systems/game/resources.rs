use super::components::VoxelType;
use bevy::prelude::*;
use std::collections::HashMap;

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
        let mut voxels =
            vec![vec![vec![VoxelType::Air; depth as usize]; height as usize]; width as usize];

        // Create a floor layer (y=0) with grass
        for x_voxels in voxels.iter_mut() {
            for z in 0..depth as usize {
                x_voxels[0][z] = VoxelType::Grass;
            }
        }

        // Add voxels on top of the four corners
        // Corners: (0,0), (0,3), (3,0), (3,3)
        voxels[0][1][0] = VoxelType::Stone;
        voxels[0][1][3] = VoxelType::Stone;
        voxels[3][1][0] = VoxelType::Stone;
        voxels[3][1][3] = VoxelType::Stone;

        // Add some 1-sub-voxel-height platforms (step-up test)
        voxels[1][1][1] = VoxelType::Dirt;
        voxels[2][1][2] = VoxelType::Dirt;

        // Add a voxel for stairs
        voxels[2][1][1] = VoxelType::Stone;

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
}

pub const GRID_CELL_SIZE: f32 = 1.0; // Or adjust based on desired granularity

#[derive(Resource, Default)]
pub struct SpatialGrid {
    pub cells: HashMap<IVec3, Vec<Entity>>,
}

impl SpatialGrid {
    // Helper to convert world position to grid coordinates
    pub fn world_to_grid_coords(pos: Vec3) -> IVec3 {
        IVec3::new(
            (pos.x / GRID_CELL_SIZE).floor() as i32,
            (pos.y / GRID_CELL_SIZE).floor() as i32,
            (pos.z / GRID_CELL_SIZE).floor() as i32,
        )
    }

    // Helper to get entities in a specific cell
    pub fn get_entities_in_cell(&self, grid_coords: IVec3) -> Option<&Vec<Entity>> {
        self.cells.get(&grid_coords)
    }

    // Helper to get entities in a bounding box (e.g., player's collision area)
    pub fn get_entities_in_aabb(&self, min_world: Vec3, max_world: Vec3) -> Vec<Entity> {
        let min_grid = Self::world_to_grid_coords(min_world);
        let max_grid = Self::world_to_grid_coords(max_world);

        let mut entities = Vec::new();
        for x in min_grid.x..=max_grid.x {
            for y in min_grid.y..=max_grid.y {
                for z in min_grid.z..=max_grid.z {
                    if let Some(cell_entities) = self.get_entities_in_cell(IVec3::new(x, y, z)) {
                        entities.extend(cell_entities.iter().copied());
                    }
                }
            }
        }
        entities
    }
}

#[derive(Resource, Default)]
pub struct GameInitialized(pub bool);
