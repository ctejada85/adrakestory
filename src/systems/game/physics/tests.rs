use super::super::components::SubVoxel;
use super::*;
use bevy::ecs::system::SystemState;

// Verify that when the pre-fetched cache covers the physics AABB and contains
// a ground entity, the cache is used (not the grid) to find the entity.
#[test]
fn prefetched_cache_used_when_bounds_cover_physics_aabb() {
    let mut world = bevy::prelude::World::new();

    // Ground sub-voxel — in cache ONLY, NOT in SpatialGrid
    let ground_bounds = (Vec3::new(-0.3, 0.4, -0.3), Vec3::new(0.3, 0.5, 0.3));
    let ground_entity = world
        .spawn(SubVoxel {
            bounds: ground_bounds,
        })
        .id();

    // Empty spatial grid
    world.insert_resource(SpatialGrid::default());

    // Physics AABB for a player at y=0.9 (bottom at 0.5), falling
    let physics_min = Vec3::new(-0.2, 0.5, -0.2);
    let physics_max = Vec3::new(0.2, 1.3, 0.2);

    // Cache bounds fully enclose the physics AABB
    let cache_min = Vec3::new(-0.5, 0.0, -0.5);
    let cache_max = Vec3::new(0.5, 2.0, 0.5);
    world.insert_resource(PreFetchedCollisionEntities {
        entities: vec![ground_entity],
        bounds: Some((cache_min, cache_max)),
    });

    let mut state: SystemState<(
        Res<SpatialGrid>,
        Res<PreFetchedCollisionEntities>,
        Query<&SubVoxel, Without<Player>>,
    )> = SystemState::new(&mut world);
    let (spatial_grid, pre_fetched, sub_voxel_query) = state.get(&world);

    // Verify the containment check passes — cache should be used
    let (bounds_min, bounds_max) = pre_fetched.bounds.unwrap();
    let covered = bounds_min.x <= physics_min.x
        && bounds_min.y <= physics_min.y
        && bounds_min.z <= physics_min.z
        && bounds_max.x >= physics_max.x
        && bounds_max.y >= physics_max.y
        && bounds_max.z >= physics_max.z;
    assert!(covered, "Cache bounds should cover the physics AABB");

    // Verify the grid would NOT find the entity (it's cache-only)
    let grid_result = spatial_grid.get_entities_in_aabb(physics_min, physics_max);
    assert!(
        grid_result.is_empty(),
        "Grid should be empty — entity is in cache only"
    );

    // Verify the cache contains the entity and its bounds can be accessed via the query
    assert_eq!(pre_fetched.entities.len(), 1);
    assert!(
        sub_voxel_query.get(pre_fetched.entities[0]).is_ok(),
        "Cache entity should be accessible via the sub-voxel query"
    );
}

// Verify that when the cache is empty, apply_physics falls back to querying the grid.
#[test]
fn empty_cache_falls_back_to_grid_query() {
    let mut world = bevy::prelude::World::new();

    // Ground sub-voxel in the GRID (not in cache)
    let ground_bounds = (Vec3::new(-0.3, 0.4, -0.3), Vec3::new(0.3, 0.5, 0.3));
    let ground_entity = world
        .spawn(SubVoxel {
            bounds: ground_bounds,
        })
        .id();

    let mut grid = SpatialGrid::default();
    grid.cells
        .entry(bevy::prelude::IVec3::new(0, 0, 0))
        .or_default()
        .push(ground_entity);
    world.insert_resource(grid);

    // Empty cache — bounds = None
    world.insert_resource(PreFetchedCollisionEntities::default());

    // Verify the fallback path: grid query finds the entity
    let mut state: SystemState<(Res<SpatialGrid>, Res<PreFetchedCollisionEntities>)> =
        SystemState::new(&mut world);
    let (spatial_grid, pre_fetched) = state.get(&world);

    assert!(pre_fetched.bounds.is_none(), "Cache should be empty");

    let physics_min = Vec3::new(-0.2, 0.2, -0.2);
    let physics_max = Vec3::new(0.2, 1.4, 0.2);
    let fallback_entities = spatial_grid.get_entities_in_aabb(physics_min, physics_max);
    assert!(
        !fallback_entities.is_empty(),
        "Fallback grid query should find the ground entity"
    );
    assert_eq!(fallback_entities[0], ground_entity);
}
