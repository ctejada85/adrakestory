use super::*;

// CollisionResult tests
#[test]
fn test_collision_result_no_collision() {
    let result = CollisionResult::no_collision();
    assert!(!result.has_collision);
    assert!(!result.can_step_up);
    assert_eq!(result.step_up_height, 0.0);
}

#[test]
fn test_collision_result_blocking() {
    let result = CollisionResult::blocking();
    assert!(result.has_collision);
    assert!(!result.can_step_up);
    assert_eq!(result.step_up_height, 0.0);
}

#[test]
fn test_collision_result_step_up() {
    let result = CollisionResult::step_up(0.125, 1.5);
    assert!(result.has_collision);
    assert!(result.can_step_up);
    assert_eq!(result.step_up_height, 0.125);
    assert_eq!(result.new_y, 1.5);
}

// CollisionParams tests
#[test]
fn test_collision_params_creation() {
    let params = CollisionParams {
        x: 1.0,
        y: 2.0,
        z: 3.0,
        radius: 0.2,
        half_height: 0.4,
        current_floor_y: 1.6,
    };
    assert_eq!(params.x, 1.0);
    assert_eq!(params.y, 2.0);
    assert_eq!(params.z, 3.0);
    assert_eq!(params.radius, 0.2);
    assert_eq!(params.half_height, 0.4);
    assert_eq!(params.current_floor_y, 1.6);
}

// check_sub_voxel_collision prefetch tests (using SystemState to access queries)
#[test]
fn prefetched_empty_slice_skips_grid_query() {
    use super::super::resources::SpatialGrid;
    use bevy::ecs::system::SystemState;
    use bevy::prelude::IVec3;

    let mut world = bevy::prelude::World::new();

    // Spawn a sub-voxel that would block movement if the grid were queried
    let bounds = (Vec3::new(-0.1, 0.2, -0.1), Vec3::new(0.1, 0.8, 0.1));
    let entity = world.spawn(SubVoxel { bounds }).id();

    // Place entity in the grid cell the player AABB would hit
    let mut grid = SpatialGrid::default();
    grid.cells
        .entry(IVec3::new(0, 0, 0))
        .or_default()
        .push(entity);
    world.insert_resource(grid);

    let params = CollisionParams {
        x: 0.0,
        y: 0.8,
        z: 0.0,
        radius: 0.2,
        half_height: 0.4,
        current_floor_y: 0.0,
    };

    let mut state: SystemState<(
        bevy::prelude::Res<SpatialGrid>,
        bevy::prelude::Query<&SubVoxel, bevy::prelude::Without<Player>>,
    )> = SystemState::new(&mut world);
    let (spatial_grid, sub_voxel_query) = state.get(&world);

    // Empty prefetched slice — function skips the grid regardless of what's there
    let result = check_sub_voxel_collision(&spatial_grid, &sub_voxel_query, params, Some(&[]));
    assert!(
        !result.has_collision,
        "Expected no collision when prefetched slice is empty (grid bypassed)"
    );
}

#[test]
fn prefetched_none_queries_grid_and_finds_blocking_collision() {
    use super::super::resources::SpatialGrid;
    use bevy::ecs::system::SystemState;
    use bevy::prelude::IVec3;

    let mut world = bevy::prelude::World::new();

    // A tall blocking sub-voxel (obstacle_height > SUB_VOXEL_SIZE + STEP_UP_TOLERANCE)
    let bounds = (Vec3::new(-0.1, 0.2, -0.1), Vec3::new(0.1, 0.8, 0.1));
    let entity = world.spawn(SubVoxel { bounds }).id();

    let mut grid = SpatialGrid::default();
    grid.cells
        .entry(IVec3::new(0, 0, 0))
        .or_default()
        .push(entity);
    world.insert_resource(grid);

    let params = CollisionParams {
        x: 0.0,
        y: 0.8,
        z: 0.0,
        radius: 0.2,
        half_height: 0.4,
        current_floor_y: 0.0,
    };

    let mut state: SystemState<(
        bevy::prelude::Res<SpatialGrid>,
        bevy::prelude::Query<&SubVoxel, bevy::prelude::Without<Player>>,
    )> = SystemState::new(&mut world);
    let (spatial_grid, sub_voxel_query) = state.get(&world);

    // None — function queries the grid and finds the blocking entity
    let result = check_sub_voxel_collision(&spatial_grid, &sub_voxel_query, params, None);
    assert!(
        result.has_collision,
        "Expected collision when prefetched is None and grid contains blocking entity"
    );
    assert!(
        !result.can_step_up,
        "Expected blocking collision (too tall to step up)"
    );
}
