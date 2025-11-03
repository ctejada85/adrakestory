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
use std::f32::consts::FRAC_PI_2;

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
        // Support both regular arrow keys and Fn + arrow keys (PageUp/PageDown/Home/End on macOS)
        if keyboard_input.pressed(KeyCode::KeyW)
            || keyboard_input.pressed(KeyCode::ArrowUp)
            || keyboard_input.pressed(KeyCode::PageUp)
        {
            direction.x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS)
            || keyboard_input.pressed(KeyCode::ArrowDown)
            || keyboard_input.pressed(KeyCode::PageDown)
        {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA)
            || keyboard_input.pressed(KeyCode::ArrowLeft)
            || keyboard_input.pressed(KeyCode::Home)
        {
            direction.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD)
            || keyboard_input.pressed(KeyCode::ArrowRight)
            || keyboard_input.pressed(KeyCode::End)
        {
            direction.z += 1.0;
        }

        // Jump
        if keyboard_input.just_pressed(KeyCode::Space) && player.is_grounded {
            player.velocity.y = 8.0;
            player.is_grounded = false;
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();

            // Calculate new target rotation based on movement direction
            // The character model faces right by default, so we subtract π/2 (90°) to align it
            // W (+X) should face forward, S (-X) backward, A (-Z) left, D (+Z) right
            let new_target = direction.z.atan2(-direction.x) - FRAC_PI_2;

            // Check if target rotation changed (with small threshold for floating point comparison)
            if (new_target - player.target_rotation).abs() > 0.01 {
                // Target changed - reset easing
                player.start_rotation = player.current_rotation;
                player.rotation_elapsed = 0.0;
                player.target_rotation = new_target;
            }

            let current_pos = transform.translation;
            let move_delta = direction * player.speed * delta;

            // Calculate current floor Y position (bottom of player sphere)
            // This will be updated after each step-up to handle stairs correctly
            let mut current_floor_y = current_pos.y - player.radius;

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
                // Update current_floor_y for subsequent collision checks (critical for stairs)
                current_floor_y = transform.translation.y - player.radius;
                // Reset vertical velocity to prevent falling after step-up
                player.velocity.y = 0.0;
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
                // Reset vertical velocity to prevent falling after step-up
                player.velocity.y = 0.0;
                // Note: current_floor_y would be updated here for consistency, but since
                // there are no more collision checks after Z-axis, we omit it to avoid
                // an unused assignment warning
            }
            // else: blocking collision, don't move
        }
    }
}
