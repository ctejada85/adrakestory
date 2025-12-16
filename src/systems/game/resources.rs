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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_to_grid_coords_positive() {
        // Simple positive coordinates
        assert_eq!(
            SpatialGrid::world_to_grid_coords(Vec3::new(0.5, 0.5, 0.5)),
            IVec3::new(0, 0, 0)
        );
        assert_eq!(
            SpatialGrid::world_to_grid_coords(Vec3::new(1.5, 2.5, 3.5)),
            IVec3::new(1, 2, 3)
        );
    }

    #[test]
    fn test_world_to_grid_coords_negative() {
        // Negative coordinates should floor correctly
        assert_eq!(
            SpatialGrid::world_to_grid_coords(Vec3::new(-0.5, -0.5, -0.5)),
            IVec3::new(-1, -1, -1)
        );
        assert_eq!(
            SpatialGrid::world_to_grid_coords(Vec3::new(-1.5, -2.5, -3.5)),
            IVec3::new(-2, -3, -4)
        );
    }

    #[test]
    fn test_world_to_grid_coords_on_boundary() {
        // Exact boundaries
        assert_eq!(
            SpatialGrid::world_to_grid_coords(Vec3::new(1.0, 2.0, 3.0)),
            IVec3::new(1, 2, 3)
        );
        assert_eq!(
            SpatialGrid::world_to_grid_coords(Vec3::new(0.0, 0.0, 0.0)),
            IVec3::new(0, 0, 0)
        );
    }

    #[test]
    fn test_insert_and_query_single_entity() {
        let mut grid = SpatialGrid::default();
        let entity = Entity::from_raw(42);
        let cell = IVec3::new(1, 2, 3);

        grid.cells.entry(cell).or_default().push(entity);

        let result = grid.get_entities_in_cell(cell);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
        assert_eq!(result.unwrap()[0], entity);
    }

    #[test]
    fn test_get_entities_in_cell_empty() {
        let grid = SpatialGrid::default();
        assert!(grid.get_entities_in_cell(IVec3::new(0, 0, 0)).is_none());
    }

    #[test]
    fn test_get_entities_in_aabb_single_cell() {
        let mut grid = SpatialGrid::default();
        let entity = Entity::from_raw(1);
        grid.cells.entry(IVec3::new(0, 0, 0)).or_default().push(entity);

        let entities = grid.get_entities_in_aabb(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.5, 0.5, 0.5));
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0], entity);
    }

    #[test]
    fn test_get_entities_in_aabb_multiple_cells() {
        let mut grid = SpatialGrid::default();
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let e3 = Entity::from_raw(3);

        grid.cells.entry(IVec3::new(0, 0, 0)).or_default().push(e1);
        grid.cells.entry(IVec3::new(1, 0, 0)).or_default().push(e2);
        grid.cells.entry(IVec3::new(0, 1, 0)).or_default().push(e3);

        // Query spanning multiple cells
        let entities = grid.get_entities_in_aabb(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.5, 1.5, 0.5));
        assert_eq!(entities.len(), 3);
    }

    #[test]
    fn test_get_entities_in_aabb_empty_cells() {
        let mut grid = SpatialGrid::default();
        let entity = Entity::from_raw(1);
        grid.cells.entry(IVec3::new(5, 5, 5)).or_default().push(entity);

        // Query area that doesn't include the entity's cell
        let entities = grid.get_entities_in_aabb(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
        assert!(entities.is_empty());
    }

    #[test]
    fn test_cell_boundary_handling() {
        let mut grid = SpatialGrid::default();
        let entity = Entity::from_raw(1);
        // Entity at exact boundary (1.0, 1.0, 1.0) should be in cell (1, 1, 1)
        grid.cells.entry(IVec3::new(1, 1, 1)).or_default().push(entity);

        // Query that includes cell (1, 1, 1)
        let entities = grid.get_entities_in_aabb(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.5, 1.5, 1.5));
        assert_eq!(entities.len(), 1);
    }
}
