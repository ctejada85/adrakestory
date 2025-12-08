//! Player movement system with collision detection.
//!
//! This module handles:
//! - WASD/Arrow key input for movement
//! - Gamepad left stick for movement
//! - Space bar / A button for jumping
//! - Collision detection during movement

use super::collision::check_sub_voxel_collision;
use super::components::{Player, SubVoxel};
use super::gamepad::{InputSource, PlayerInput};
use super::resources::SpatialGrid;
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

/// System that handles player movement based on unified input (keyboard or gamepad).
///
/// This system:
/// - Reads from PlayerInput resource for movement direction and jump
/// - Handles both keyboard (WASD) and gamepad (left stick) input
/// - Applies collision detection
/// - Updates player position and grounded state
///
/// Movement is adjusted for the camera rotation:
/// - Forward (+Y input) moves in +X direction
/// - Back (-Y input) moves in -X direction
/// - Left (-X input) moves in -Z direction
/// - Right (+X input) moves in +Z direction
pub fn move_player(
    time: Res<Time>,
    player_input: Res<PlayerInput>,
    spatial_grid: Res<SpatialGrid>,
    sub_voxel_query: Query<&SubVoxel, Without<Player>>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
) {
    if let Ok((mut player, mut transform)) = player_query.get_single_mut() {
        // Clamp delta time to prevent physics issues when window regains focus
        let delta = time.delta_secs().min(0.1);

        // Convert 2D input to 3D movement direction
        // PlayerInput.movement: x = left/right, y = forward/back
        // Game world: x = forward/back, z = left/right
        let direction = Vec3::new(player_input.movement.y, 0.0, player_input.movement.x);

        // Jump - use just_pressed for responsive jumping
        if player_input.jump_just_pressed && player.is_grounded {
            player.velocity.y = 8.0;
            player.is_grounded = false;
        }

        if direction.length() > 0.0 {
            // For analog input, preserve the magnitude for variable speed
            let magnitude = direction.length().min(1.0);
            let normalized_dir = direction.normalize();

            // Calculate new target rotation based on movement direction
            // The character model faces right by default, so we subtract π/2 (90°) to align it
            let new_target = normalized_dir.z.atan2(-normalized_dir.x) - FRAC_PI_2;

            // Use a larger threshold for gamepad to prevent constant rotation resets
            // from small analog stick variations. Keyboard input is always normalized
            // so it can use a smaller threshold.
            let rotation_threshold = match player_input.input_source {
                InputSource::Gamepad => 0.15, // ~8.6 degrees - prevents jitter from analog input
                InputSource::KeyboardMouse => 0.01, // ~0.6 degrees - responsive for digital input
            };

            // Check if target rotation changed significantly
            if (new_target - player.target_rotation).abs() > rotation_threshold {
                // Target changed - reset easing
                player.start_rotation = player.current_rotation;
                player.rotation_elapsed = 0.0;
                player.target_rotation = new_target;
            }

            let current_pos = transform.translation;
            // Apply magnitude for analog movement (stick pushed halfway = half speed)
            let move_delta = normalized_dir * magnitude * player.speed * delta;

            // Calculate current floor Y position (bottom of player sphere)
            let mut current_floor_y = current_pos.y - player.radius;

            let new_x = current_pos.x + move_delta.x;
            let new_z = current_pos.z + move_delta.z;

            // Optimization: Try diagonal movement first when moving in both axes
            let moving_diagonally = move_delta.x != 0.0 && move_delta.z != 0.0;

            if moving_diagonally {
                let diagonal_collision = check_sub_voxel_collision(
                    &spatial_grid,
                    &sub_voxel_query,
                    new_x,
                    current_pos.y,
                    new_z,
                    player.radius,
                    current_floor_y,
                );

                if !diagonal_collision.has_collision {
                    transform.translation.x = new_x;
                    transform.translation.z = new_z;
                } else if diagonal_collision.can_step_up && player.is_grounded {
                    transform.translation.x = new_x;
                    transform.translation.z = new_z;
                    transform.translation.y =
                        current_floor_y + diagonal_collision.step_up_height + player.radius;
                    player.velocity.y = 0.0;
                } else {
                    apply_axis_movement(
                        &spatial_grid,
                        &sub_voxel_query,
                        &mut transform,
                        &mut player,
                        current_pos,
                        new_x,
                        new_z,
                        &mut current_floor_y,
                    );
                }
            } else {
                apply_axis_movement(
                    &spatial_grid,
                    &sub_voxel_query,
                    &mut transform,
                    &mut player,
                    current_pos,
                    new_x,
                    new_z,
                    &mut current_floor_y,
                );
            }
        }
    }
}

/// Applies movement checking each axis individually for wall sliding behavior.
#[inline]
#[allow(clippy::too_many_arguments)]
fn apply_axis_movement(
    spatial_grid: &SpatialGrid,
    sub_voxel_query: &Query<&SubVoxel, Without<Player>>,
    transform: &mut Transform,
    player: &mut Player,
    current_pos: Vec3,
    new_x: f32,
    new_z: f32,
    current_floor_y: &mut f32,
) {
    // Try moving on X axis
    if new_x != current_pos.x {
        let x_collision = check_sub_voxel_collision(
            spatial_grid,
            sub_voxel_query,
            new_x,
            current_pos.y,
            current_pos.z,
            player.radius,
            *current_floor_y,
        );

        if !x_collision.has_collision {
            // No collision, move freely
            transform.translation.x = new_x;
        } else if x_collision.can_step_up && player.is_grounded {
            // Step-up collision - move horizontally and adjust height
            transform.translation.x = new_x;
            transform.translation.y = *current_floor_y + x_collision.step_up_height + player.radius;
            // Update current_floor_y for subsequent collision checks (critical for stairs)
            *current_floor_y = transform.translation.y - player.radius;
            // Reset vertical velocity to prevent falling after step-up
            player.velocity.y = 0.0;
        }
        // else: blocking collision, don't move
    }

    // Try moving on Z axis
    if new_z != current_pos.z {
        let z_collision = check_sub_voxel_collision(
            spatial_grid,
            sub_voxel_query,
            transform.translation.x,
            transform.translation.y,
            new_z,
            player.radius,
            *current_floor_y,
        );

        if !z_collision.has_collision {
            // No collision, move freely
            transform.translation.z = new_z;
        } else if z_collision.can_step_up && player.is_grounded {
            // Step-up collision - move horizontally and adjust height
            transform.translation.z = new_z;
            transform.translation.y = *current_floor_y + z_collision.step_up_height + player.radius;
            // Reset vertical velocity to prevent falling after step-up
            player.velocity.y = 0.0;
        }
        // else: blocking collision, don't move
    }
}
