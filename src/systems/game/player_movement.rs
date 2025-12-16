//! Player movement system with collision detection.
//!
//! This module handles:
//! - WASD keys / Gamepad left stick for movement
//! - Arrow keys / Gamepad right stick for character facing direction
//! - Space bar / A button for jumping
//! - Collision detection during movement
//!
//! Character Facing Behavior:
//! - If look direction input is active (arrow keys or right stick), character faces that direction
//! - Otherwise, character faces the direction of movement

use super::collision::{check_sub_voxel_collision, CollisionParams};
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

        // Handle character facing direction
        // If right stick (look_direction) is being used, face that direction
        // Otherwise, face the movement direction (if moving)
        let look_dir = if player_input.look_direction.length() > 0.01 {
            // Right stick is active - use it for facing direction
            Vec3::new(
                player_input.look_direction.y,
                0.0,
                player_input.look_direction.x,
            )
        } else if direction.length() > 0.0 {
            // No right stick input - face movement direction
            direction
        } else {
            Vec3::ZERO
        };

        // Update rotation if we have a look direction
        if look_dir.length() > 0.0 {
            let normalized_look = look_dir.normalize();

            // Calculate new target rotation based on look direction
            // The character model faces right by default, so we subtract π/2 (90°) to align it
            let new_target = normalized_look.z.atan2(-normalized_look.x) - FRAC_PI_2;

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
        }

        if direction.length() > 0.0 {
            // For analog input, preserve the magnitude for variable speed
            let magnitude = direction.length().min(1.0);
            let normalized_dir = direction.normalize();

            let current_pos = transform.translation;
            // Apply magnitude for analog movement (stick pushed halfway = half speed)
            let move_delta = normalized_dir * magnitude * player.speed * delta;

            // Calculate current floor Y position (bottom of player cylinder)
            let mut current_floor_y = current_pos.y - player.half_height;

            let new_x = current_pos.x + move_delta.x;
            let new_z = current_pos.z + move_delta.z;

            // Optimization: Try diagonal movement first when moving in both axes
            let moving_diagonally = move_delta.x != 0.0 && move_delta.z != 0.0;

            if moving_diagonally {
                let diagonal_collision = check_sub_voxel_collision(
                    &spatial_grid,
                    &sub_voxel_query,
                    CollisionParams {
                        x: new_x,
                        y: current_pos.y,
                        z: new_z,
                        radius: player.radius,
                        half_height: player.half_height,
                        current_floor_y,
                    },
                );

                if !diagonal_collision.has_collision {
                    transform.translation.x = new_x;
                    transform.translation.z = new_z;
                } else if diagonal_collision.can_step_up && player.is_grounded {
                    transform.translation.x = new_x;
                    transform.translation.z = new_z;
                    transform.translation.y =
                        current_floor_y + diagonal_collision.step_up_height + player.half_height;
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
            CollisionParams {
                x: new_x,
                y: current_pos.y,
                z: current_pos.z,
                radius: player.radius,
                half_height: player.half_height,
                current_floor_y: *current_floor_y,
            },
        );

        if !x_collision.has_collision {
            // No collision, move freely
            transform.translation.x = new_x;
        } else if x_collision.can_step_up && player.is_grounded {
            // Step-up collision - move horizontally and adjust height
            transform.translation.x = new_x;
            transform.translation.y =
                *current_floor_y + x_collision.step_up_height + player.half_height;
            // Update current_floor_y for subsequent collision checks (critical for stairs)
            *current_floor_y = transform.translation.y - player.half_height;
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
            CollisionParams {
                x: transform.translation.x,
                y: transform.translation.y,
                z: new_z,
                radius: player.radius,
                half_height: player.half_height,
                current_floor_y: *current_floor_y,
            },
        );

        if !z_collision.has_collision {
            // No collision, move freely
            transform.translation.z = new_z;
        } else if z_collision.can_step_up && player.is_grounded {
            // Step-up collision - move horizontally and adjust height
            transform.translation.z = new_z;
            transform.translation.y =
                *current_floor_y + z_collision.step_up_height + player.half_height;
            // Reset vertical velocity to prevent falling after step-up
            player.velocity.y = 0.0;
        }
        // else: blocking collision, don't move
    }
}

/// Calculate 3D movement direction from 2D input.
///
/// Converts PlayerInput 2D movement (x = left/right, y = forward/back)
/// to 3D world coordinates (x = forward/back, z = left/right).
///
/// This is a pure function useful for testing input conversion in isolation.
#[inline]
pub fn input_to_world_direction(input: Vec2) -> Vec3 {
    Vec3::new(input.y, 0.0, input.x)
}

/// Calculate the look direction from input, returning the direction vector.
///
/// Returns Vec3::ZERO if no look direction is active.
#[inline]
pub fn calculate_look_direction(look_input: Vec2, movement_dir: Vec3) -> Vec3 {
    if look_input.length() > 0.01 {
        // Right stick is active - use it for facing direction
        Vec3::new(look_input.y, 0.0, look_input.x)
    } else if movement_dir.length() > 0.0 {
        // No right stick input - face movement direction
        movement_dir
    } else {
        Vec3::ZERO
    }
}

/// Calculate target rotation angle from a look direction vector.
///
/// Returns the rotation angle in radians. The character model faces right by default,
/// so we subtract π/2 (90°) to align it with the look direction.
#[inline]
pub fn calculate_target_rotation(look_dir: Vec3) -> f32 {
    let normalized = look_dir.normalize();
    normalized.z.atan2(-normalized.x) - FRAC_PI_2
}

/// Normalize movement input for consistent speed.
///
/// Ensures diagonal movement isn't faster than cardinal movement by
/// normalizing the input vector while preserving analog magnitude.
#[inline]
pub fn normalize_movement(direction: Vec3) -> (Vec3, f32) {
    let magnitude = direction.length().min(1.0);
    if magnitude > 0.0 {
        (direction.normalize(), magnitude)
    } else {
        (Vec3::ZERO, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // input_to_world_direction tests
    #[test]
    fn test_input_forward() {
        let input = Vec2::new(0.0, 1.0); // Forward
        let result = input_to_world_direction(input);
        assert_eq!(result, Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_input_backward() {
        let input = Vec2::new(0.0, -1.0); // Backward
        let result = input_to_world_direction(input);
        assert_eq!(result, Vec3::new(-1.0, 0.0, 0.0));
    }

    #[test]
    fn test_input_left() {
        let input = Vec2::new(-1.0, 0.0); // Left
        let result = input_to_world_direction(input);
        assert_eq!(result, Vec3::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn test_input_right() {
        let input = Vec2::new(1.0, 0.0); // Right
        let result = input_to_world_direction(input);
        assert_eq!(result, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_input_diagonal() {
        let input = Vec2::new(1.0, 1.0); // Forward-Right
        let result = input_to_world_direction(input);
        assert_eq!(result, Vec3::new(1.0, 0.0, 1.0));
    }

    #[test]
    fn test_input_zero() {
        let input = Vec2::ZERO;
        let result = input_to_world_direction(input);
        assert_eq!(result, Vec3::ZERO);
    }

    // normalize_movement tests
    #[test]
    fn test_normalize_zero_input() {
        let (dir, mag) = normalize_movement(Vec3::ZERO);
        assert_eq!(dir, Vec3::ZERO);
        assert_eq!(mag, 0.0);
    }

    #[test]
    fn test_normalize_cardinal_direction() {
        let (dir, mag) = normalize_movement(Vec3::new(1.0, 0.0, 0.0));
        assert!((dir - Vec3::new(1.0, 0.0, 0.0)).length() < 0.001);
        assert!((mag - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_normalize_diagonal_same_speed() {
        // Diagonal input should have same magnitude as cardinal
        let (_, cardinal_mag) = normalize_movement(Vec3::new(1.0, 0.0, 0.0));
        let (_, diagonal_mag) = normalize_movement(Vec3::new(1.0, 0.0, 1.0));
        // Both should be clamped to 1.0
        assert!((cardinal_mag - 1.0).abs() < 0.001);
        assert!((diagonal_mag - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_normalize_diagonal_normalized_direction() {
        let (dir, _) = normalize_movement(Vec3::new(1.0, 0.0, 1.0));
        // Should be normalized
        assert!((dir.length() - 1.0).abs() < 0.001);
        // Components should be equal (45 degree angle)
        assert!((dir.x - dir.z).abs() < 0.001);
    }

    #[test]
    fn test_normalize_partial_analog() {
        // Analog stick pushed halfway
        let (_, mag) = normalize_movement(Vec3::new(0.5, 0.0, 0.0));
        assert!((mag - 0.5).abs() < 0.001);
    }

    // calculate_look_direction tests
    #[test]
    fn test_look_direction_from_stick() {
        let look_input = Vec2::new(1.0, 0.0); // Looking right
        let movement = Vec3::new(1.0, 0.0, 0.0); // Moving forward
        let result = calculate_look_direction(look_input, movement);
        // Should use look input, not movement
        assert_eq!(result, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_look_direction_from_movement() {
        let look_input = Vec2::ZERO; // No look input
        let movement = Vec3::new(1.0, 0.0, 0.0); // Moving forward
        let result = calculate_look_direction(look_input, movement);
        // Should use movement direction
        assert_eq!(result, movement);
    }

    #[test]
    fn test_look_direction_no_input() {
        let look_input = Vec2::ZERO;
        let movement = Vec3::ZERO;
        let result = calculate_look_direction(look_input, movement);
        assert_eq!(result, Vec3::ZERO);
    }

    #[test]
    fn test_look_direction_small_stick_ignored() {
        // Very small stick input (deadzone-like)
        let look_input = Vec2::new(0.005, 0.005);
        let movement = Vec3::new(1.0, 0.0, 0.0);
        let result = calculate_look_direction(look_input, movement);
        // Should use movement since look_input is too small
        assert_eq!(result, movement);
    }

    // calculate_target_rotation tests
    #[test]
    fn test_rotation_facing_forward() {
        // Looking in +X direction (forward in game)
        let look_dir = Vec3::new(1.0, 0.0, 0.0);
        let rotation = calculate_target_rotation(look_dir);
        // Expected: atan2(0, -1) - PI/2 = PI - PI/2 = PI/2
        let expected = std::f32::consts::FRAC_PI_2;
        assert!((rotation - expected).abs() < 0.001);
    }

    #[test]
    fn test_rotation_facing_backward() {
        // Looking in -X direction (backward in game)
        let look_dir = Vec3::new(-1.0, 0.0, 0.0);
        let rotation = calculate_target_rotation(look_dir);
        // Expected: atan2(0, 1) - PI/2 = 0 - PI/2 = -PI/2
        let expected = -std::f32::consts::FRAC_PI_2;
        assert!((rotation - expected).abs() < 0.001);
    }

    #[test]
    fn test_rotation_facing_right() {
        // Looking in +Z direction (right in game)
        let look_dir = Vec3::new(0.0, 0.0, 1.0);
        let rotation = calculate_target_rotation(look_dir);
        // Expected: atan2(1, 0) - PI/2 = PI/2 - PI/2 = 0
        assert!(rotation.abs() < 0.001);
    }
}
