//! Physics systems for gravity and collision response.
//!
//! This module handles:
//! - Applying gravity to the player
//! - Updating player position based on velocity
//! - Ground collision detection
//! - Setting grounded state

use super::collision::get_sub_voxel_bounds;
use super::components::{Player, SubVoxel};
use super::resources::SpatialGrid;
use bevy::prelude::*;

const GRAVITY: f32 = -32.0;
const GROUND_DETECTION_EPSILON: f32 = 0.001;

/// System that applies gravity to the player's velocity.
///
/// Gravity is applied as a constant downward acceleration.
/// Delta time is clamped to prevent physics issues when the window
/// regains focus after being minimized.
pub fn apply_gravity(time: Res<Time>, mut player_query: Query<&mut Player>) {
    if let Ok(mut player) = player_query.get_single_mut() {
        // Clamp delta time to prevent physics issues
        let delta = time.delta_secs().min(0.1);
        player.velocity.y += GRAVITY * delta;
    }
}

/// System that applies physics to the player, including velocity and ground collision.
///
/// This system:
/// - Updates player position based on velocity
/// - Detects collisions with the ground (sub-voxels below the player)
/// - Stops downward movement when hitting the ground
/// - Sets the player's grounded state
///
/// Uses spatial grid optimization to only check nearby sub-voxels instead of
/// iterating through all sub-voxels in the world, providing significant
/// performance improvements in large worlds.
pub fn apply_physics(
    time: Res<Time>,
    spatial_grid: Res<SpatialGrid>,
    sub_voxel_query: Query<&SubVoxel, Without<Player>>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
) {
    if let Ok((mut player, mut transform)) = player_query.get_single_mut() {
        // Clamp delta time to prevent physics issues
        let delta = time.delta_secs().min(0.1);

        // Apply velocity
        let new_y = transform.translation.y + player.velocity.y * delta;
        let player_bottom = new_y - player.radius;
        let current_bottom = transform.translation.y - player.radius;

        // Extract player position (loop-invariant values)
        let player_x = transform.translation.x;
        let player_z = transform.translation.z;
        let player_radius = player.radius;

        let mut hit_ground = false;
        let mut highest_collision_y = f32::MIN;

        // Use spatial grid to get only nearby sub-voxels
        // This reduces checks from O(n) to O(k) where k << n
        let player_min = Vec3::new(
            player_x - player_radius,
            new_y - player_radius,
            player_z - player_radius,
        );
        let player_max = Vec3::new(
            player_x + player_radius,
            new_y + player_radius,
            player_z + player_radius,
        );

        let relevant_entities = spatial_grid.get_entities_in_aabb(player_min, player_max);

        // Check collision with nearby sub-voxels only
        for entity in relevant_entities {
            if let Ok(sub_voxel) = sub_voxel_query.get(entity) {
                let (min, max) = get_sub_voxel_bounds(sub_voxel);

                // Only check sub-voxels that are below the player's bottom
                // This ensures we check all sub-voxels that could support the player
                if max.y > new_y + player.radius {
                    continue;
                }

                // Check horizontal overlap
                let horizontal_overlap = player_x + player_radius > min.x
                    && player_x - player_radius < max.x
                    && player_z + player_radius > min.z
                    && player_z - player_radius < max.z;

                if horizontal_overlap && player.velocity.y <= 0.0 {
                    // Check if player's bottom would go through the top of this sub-voxel
                    // Player was above (or very close due to floating-point errors), and would now be at or below the top
                    // Use epsilon tolerance to handle floating-point precision issues after step-ups
                    if current_bottom >= max.y - GROUND_DETECTION_EPSILON && player_bottom <= max.y
                    {
                        highest_collision_y = highest_collision_y.max(max.y);
                        hit_ground = true;
                    }
                }
            }
        }

        if hit_ground {
            transform.translation.y = highest_collision_y + player.radius;
            player.velocity.y = 0.0;
            player.is_grounded = true;
        } else {
            transform.translation.y = new_y;
            player.is_grounded = false;
        }
    }
}
