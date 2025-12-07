use bevy::prelude::*;
use std::collections::HashMap;

pub const GRID_CELL_SIZE: f32 = 1.0;

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

    /// Get entities in a bounding box (e.g., player's collision area).
    ///
    /// Pre-allocates vector capacity based on expected entity count to reduce
    /// heap allocations during collision checks.
    pub fn get_entities_in_aabb(&self, min_world: Vec3, max_world: Vec3) -> Vec<Entity> {
        let min_grid = Self::world_to_grid_coords(min_world);
        let max_grid = Self::world_to_grid_coords(max_world);

        // Pre-allocate capacity based on expected entity count
        // Estimate: number of cells × average entities per cell (~8 sub-voxels per voxel)
        let num_cells = ((max_grid.x - min_grid.x + 1)
            * (max_grid.y - min_grid.y + 1)
            * (max_grid.z - min_grid.z + 1)) as usize;
        let mut entities = Vec::with_capacity(num_cells * 8);

        for x in min_grid.x..=max_grid.x {
            for y in min_grid.y..=max_grid.y {
                for z in min_grid.z..=max_grid.z {
                    if let Some(cell_entities) = self.get_entities_in_cell(IVec3::new(x, y, z)) {
                        entities.extend_from_slice(cell_entities);
                    }
                }
            }
        }
        entities
    }
}

#[derive(Resource, Default)]
pub struct GameInitialized(pub bool);
