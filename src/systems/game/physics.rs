//! Physics systems for gravity and collision response.
//!
//! This module handles:
//! - Applying gravity to the player
//! - Updating player position based on velocity
//! - Ground collision detection
//! - Setting grounded state

use super::collision::get_sub_voxel_bounds;
use super::components::{Player, SubVoxel};
use bevy::prelude::*;

const GRAVITY: f32 = -32.0;

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
/// The system checks all sub-voxels below the player to find the highest
/// collision point, ensuring the player lands on the correct surface.
pub fn apply_physics(
    time: Res<Time>,
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

        let mut hit_ground = false;
        let mut highest_collision_y = f32::MIN;

        // Check collision with sub-voxels below
        for sub_voxel in sub_voxel_query.iter() {
            let (min, max) = get_sub_voxel_bounds(sub_voxel);

            // Only check sub-voxels that are below the player's bottom
            // This ensures we check all sub-voxels that could support the player
            if max.y > new_y + player.radius {
                continue;
            }

            // Check if player sphere is above this sub-voxel horizontally
            let player_x = transform.translation.x;
            let player_z = transform.translation.z;

            // Check horizontal overlap
            let horizontal_overlap = player_x + player.radius > min.x
                && player_x - player.radius < max.x
                && player_z + player.radius > min.z
                && player_z - player.radius < max.z;

            if horizontal_overlap && player.velocity.y <= 0.0 {
                // Check if player's bottom would go through the top of this sub-voxel
                // Player was above, and would now be at or below the top
                if current_bottom >= max.y && player_bottom <= max.y {
                    highest_collision_y = highest_collision_y.max(max.y);
                    hit_ground = true;
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
