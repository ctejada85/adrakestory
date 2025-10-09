//! Player movement system with collision detection.
//!
//! This module handles:
//! - WASD/Arrow key input for movement
//! - Space bar for jumping
//! - Collision detection during movement

use super::collision::check_sub_voxel_collision;
use super::components::{Player, SubVoxel};
use super::resources::SpatialGrid;
use bevy::prelude::*;

/// System that handles player movement based on keyboard input.
///
/// This system:
/// - Processes WASD/Arrow keys for directional movement
/// - Handles Space bar for jumping
/// - Applies collision detection
/// - Updates player position and grounded state
///
/// Movement is adjusted for the camera rotation:
/// - W/Up moves in +X direction
/// - S/Down moves in -X direction
/// - A/Left moves in -Z direction
/// - D/Right moves in +Z direction
pub fn move_player(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    spatial_grid: Res<SpatialGrid>,
    sub_voxel_query: Query<&SubVoxel, Without<Player>>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
) {
    if let Ok((mut player, mut transform)) = player_query.get_single_mut() {
        // Clamp delta time to prevent physics issues when window regains focus
        let delta = time.delta_secs().min(0.1);

        let mut direction = Vec3::ZERO;

        // Adjusted for camera rotation: up moves in +X, right moves in -Z
        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            direction.x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.z += 1.0;
        }

        // Jump
        if keyboard_input.just_pressed(KeyCode::Space) && player.is_grounded {
            player.velocity.y = 8.0;
            player.is_grounded = false;
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();

            let current_pos = transform.translation;
            let move_delta = direction * player.speed * delta;

            // Calculate current floor Y position (bottom of player sphere)
            let current_floor_y = current_pos.y - player.radius;

            // Try moving on X axis
            let new_x = current_pos.x + move_delta.x;
            let x_collision = check_sub_voxel_collision(
                &spatial_grid,
                &sub_voxel_query,
                new_x,
                current_pos.y,
                current_pos.z,
                player.radius,
                current_floor_y,
            );

            if !x_collision.has_collision {
                // No collision, move freely
                transform.translation.x = new_x;
            } else if x_collision.can_step_up && player.is_grounded {
                // Step-up collision - move horizontally and adjust height
                transform.translation.x = new_x;
                transform.translation.y =
                    current_floor_y + x_collision.step_up_height + player.radius;
            }
            // else: blocking collision, don't move

            // Try moving on Z axis
            let new_z = current_pos.z + move_delta.z;
            let z_collision = check_sub_voxel_collision(
                &spatial_grid,
                &sub_voxel_query,
                transform.translation.x,
                transform.translation.y,
                new_z,
                player.radius,
                current_floor_y,
            );

            if !z_collision.has_collision {
                // No collision, move freely
                transform.translation.z = new_z;
            } else if z_collision.can_step_up && player.is_grounded {
                // Step-up collision - move horizontally and adjust height
                transform.translation.z = new_z;
                transform.translation.y =
                    current_floor_y + z_collision.step_up_height + player.radius;
            }
            // else: blocking collision, don't move
        }
    }
}
