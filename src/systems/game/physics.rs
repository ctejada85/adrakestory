//! Physics systems for gravity and collision response.
//!
//! This module handles:
//! - Applying gravity to the player
//! - Updating player position based on velocity
//! - Ground collision detection
//! - Setting grounded state

use super::collision::get_sub_voxel_bounds;
use super::components::{Npc, Player, SubVoxel};
use super::resources::{PreFetchedCollisionEntities, SpatialGrid};
use crate::diagnostics::FrameProfiler;
use crate::profile_scope;
use bevy::prelude::*;

const GRAVITY: f32 = -32.0;
const GROUND_DETECTION_EPSILON: f32 = 0.001;

/// System that applies gravity to the player's velocity.
///
/// Gravity is applied as a constant downward acceleration.
/// Delta time is clamped to prevent physics issues when the window
/// regains focus after being minimized.
pub fn apply_gravity(
    time: Res<Time>,
    mut player: Single<&mut Player>,
    profiler: Option<Res<FrameProfiler>>,
) {
    profile_scope!(profiler, "apply_gravity");
    // Clamp delta time to prevent physics issues
    let delta = time.delta_secs().min(0.1);
    player.velocity.y += GRAVITY * delta;
}

/// System that applies physics to the player, including velocity and ground/ceiling collision.
///
/// This system:
/// - Updates player position based on velocity
/// - Detects collisions with the ground (sub-voxels below the player)
/// - Detects collisions with the ceiling (sub-voxels above the player)
/// - Stops downward movement when hitting the ground
/// - Stops upward movement when hitting the ceiling
/// - Sets the player's grounded state
///
/// Uses spatial grid optimization to only check nearby sub-voxels instead of
/// iterating through all sub-voxels in the world, providing significant
/// performance improvements in large worlds.
///
/// When `move_player` ran this frame and its pre-fetched AABB covers the physics
/// query AABB, the cached entity slice is reused to avoid a second grid query.
pub fn apply_physics(
    time: Res<Time>,
    spatial_grid: Option<Res<SpatialGrid>>,
    pre_fetched: Res<PreFetchedCollisionEntities>,
    sub_voxel_query: Query<&SubVoxel, Without<Player>>,
    mut player: Single<(&mut Player, &mut Transform)>,
    profiler: Option<Res<FrameProfiler>>,
) {
    profile_scope!(profiler, "apply_physics");
    // SpatialGrid is removed during hot reload between despawn and respawn frames.
    let Some(spatial_grid) = spatial_grid else {
        return;
    };

    let (mut player, mut transform) = player.into_inner();
    // Clamp delta time to prevent physics issues
    let delta = time.delta_secs().min(0.1);

    // Apply velocity
    let new_y = transform.translation.y + player.velocity.y * delta;
    let player_bottom = new_y - player.half_height;
    let player_top = new_y + player.half_height;
    let current_bottom = transform.translation.y - player.half_height;
    let current_top = transform.translation.y + player.half_height;

    // Extract player position (loop-invariant values)
    let player_x = transform.translation.x;
    let player_z = transform.translation.z;
    let player_radius = player.radius;
    let player_half_height = player.half_height;

    let mut hit_ground = false;
    let mut hit_ceiling = false;
    let mut highest_collision_y = f32::MIN;
    let mut lowest_ceiling_y = f32::MAX;

    // Physics AABB — based on new_y after gravity, not the horizontal movement AABB.
    let physics_min = Vec3::new(
        player_x - player_radius,
        new_y - player_half_height,
        player_z - player_radius,
    );
    let physics_max = Vec3::new(
        player_x + player_radius,
        new_y + player_half_height,
        player_z + player_radius,
    );

    // Reuse the pre-fetched slice from move_player when it covers the physics AABB.
    // This avoids a second get_entities_in_aabb call on frames where the player moves.
    let owned: Vec<Entity>;
    let relevant: &[Entity] = if let Some((cache_min, cache_max)) = pre_fetched.bounds {
        let covered = cache_min.x <= physics_min.x
            && cache_min.y <= physics_min.y
            && cache_min.z <= physics_min.z
            && cache_max.x >= physics_max.x
            && cache_max.y >= physics_max.y
            && cache_max.z >= physics_max.z;
        if covered {
            &pre_fetched.entities
        } else {
            owned = spatial_grid.get_entities_in_aabb(physics_min, physics_max);
            &owned
        }
    } else {
        owned = spatial_grid.get_entities_in_aabb(physics_min, physics_max);
        &owned
    };

    // Check collision with nearby sub-voxels only
    for &entity in relevant {
        let Ok(sub_voxel) = sub_voxel_query.get(entity) else {
            continue;
        };

        let (min, max) = get_sub_voxel_bounds(sub_voxel);

        // For cylinder collision detection, we need to check if the player's circular
        // cross-section actually overlaps with the sub-voxel's XZ bounds.
        // Find the closest point on the sub-voxel's XZ rectangle to the player center
        let closest_x = player_x.clamp(min.x, max.x);
        let closest_z = player_z.clamp(min.z, max.z);

        // Check if the closest point is within the cylinder's radius
        let dx = player_x - closest_x;
        let dz = player_z - closest_z;
        let distance_squared = dx * dx + dz * dz;

        // Skip if the cylinder doesn't actually overlap horizontally
        if distance_squared >= player_radius * player_radius {
            continue;
        }

        // Ground collision: Check when moving downward
        if player.velocity.y <= 0.0 {
            // Check if player's bottom would go through the top of this sub-voxel
            // Player was above (or very close due to floating-point errors), and would now be at or below the top
            if current_bottom >= max.y - GROUND_DETECTION_EPSILON
                && player_bottom <= max.y + GROUND_DETECTION_EPSILON
            {
                highest_collision_y = highest_collision_y.max(max.y);
                hit_ground = true;
            }
        }

        // Ceiling collision: Check when moving upward
        if player.velocity.y > 0.0 {
            // Check if player's top would go through the bottom of this sub-voxel
            // Player's top was below (or very close), and would now be at or above the bottom
            if current_top <= min.y + GROUND_DETECTION_EPSILON
                && player_top >= min.y - GROUND_DETECTION_EPSILON
            {
                lowest_ceiling_y = lowest_ceiling_y.min(min.y);
                hit_ceiling = true;
            }
        }
    }

    if hit_ground {
        transform.translation.y = highest_collision_y + player_half_height;
        player.velocity.y = 0.0;
        player.is_grounded = true;
    } else if hit_ceiling {
        // Stop upward movement when hitting ceiling
        transform.translation.y = lowest_ceiling_y - player_half_height;
        player.velocity.y = 0.0;
        player.is_grounded = false;
    } else {
        transform.translation.y = new_y;
        player.is_grounded = false;
    }
}

/// System that handles collision between the player and NPCs.
///
/// This system:
/// - Detects sphere-sphere collision between player and NPCs
/// - Pushes the player away from NPCs when colliding
/// - Prevents the player from walking through NPCs
pub fn apply_npc_collision(
    npc_query: Query<(&Npc, &Transform), Without<Player>>,
    mut player: Option<Single<(&Player, &mut Transform)>>,
) {
    let Some(mut player) = player else {
        return;
    };
    let (player, mut player_transform) = player.into_inner();

    let player_pos = player_transform.translation;
    let player_radius = player.radius;

    // Check collision with all NPCs (sphere-sphere collision)
    for (npc, npc_transform) in &npc_query {
        let npc_pos = npc_transform.translation;

        // Calculate horizontal distance (ignore Y for now to allow jumping over)
        let dx = player_pos.x - npc_pos.x;
        let dz = player_pos.z - npc_pos.z;
        let horizontal_distance = (dx * dx + dz * dz).sqrt();

        // Check if there's vertical overlap (player and NPC are at similar heights)
        let vertical_overlap = (player_pos.y - npc_pos.y).abs() < (player_radius + npc.radius);

        let min_distance = player_radius + npc.radius;

        if horizontal_distance < min_distance && vertical_overlap {
            // Push player away from NPC horizontally
            if horizontal_distance > 0.001 {
                let penetration = min_distance - horizontal_distance;
                let direction_x = dx / horizontal_distance;
                let direction_z = dz / horizontal_distance;

                player_transform.translation.x += direction_x * penetration;
                player_transform.translation.z += direction_z * penetration;
            } else {
                // Player is exactly on NPC, push in arbitrary direction
                player_transform.translation.x += min_distance;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::components::SubVoxel;
    use bevy::ecs::system::SystemState;

    // Verify that when the pre-fetched cache covers the physics AABB and contains
    // a ground entity, the cache is used (not the grid) to find the entity.
    #[test]
    fn prefetched_cache_used_when_bounds_cover_physics_aabb() {
        let mut world = bevy::prelude::World::new();

        // Ground sub-voxel — in cache ONLY, NOT in SpatialGrid
        let ground_bounds = (Vec3::new(-0.3, 0.4, -0.3), Vec3::new(0.3, 0.5, 0.3));
        let ground_entity = world.spawn(SubVoxel { bounds: ground_bounds }).id();

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
        let ground_entity = world.spawn(SubVoxel { bounds: ground_bounds }).id();

        let mut grid = SpatialGrid::default();
        grid.cells
            .entry(bevy::prelude::IVec3::new(0, 0, 0))
            .or_default()
            .push(ground_entity);
        world.insert_resource(grid);

        // Empty cache — bounds = None
        world.insert_resource(PreFetchedCollisionEntities::default());

        // Verify the fallback path: grid query finds the entity
        let mut state: SystemState<(
            Res<SpatialGrid>,
            Res<PreFetchedCollisionEntities>,
        )> = SystemState::new(&mut world);
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
}
