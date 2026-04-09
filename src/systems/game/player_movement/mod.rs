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

use super::collision::{
    check_sub_voxel_collision, CollisionParams, STEP_UP_TOLERANCE, SUB_VOXEL_SIZE,
};
use super::components::{Player, SubVoxel};
use super::gamepad::{InputSource, PlayerInput};
use super::resources::{PreFetchedCollisionEntities, SpatialGrid};
use crate::diagnostics::FrameProfiler;
use crate::profile_scope;
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
    spatial_grid: Option<Res<SpatialGrid>>,
    mut pre_fetched: ResMut<PreFetchedCollisionEntities>,
    sub_voxel_query: Query<&SubVoxel, Without<Player>>,
    player: Single<(&mut Player, &mut Transform)>,
    profiler: Option<Res<FrameProfiler>>,
) {
    profile_scope!(profiler, "move_player");
    // Clear the cache so a stale slice from the previous frame is never read by apply_physics
    pre_fetched.entities.clear();
    pre_fetched.bounds = None;
    // SpatialGrid is removed during hot reload between despawn and respawn frames.
    let Some(spatial_grid) = spatial_grid else {
        return;
    };
    let (mut player, mut transform) = player.into_inner();
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

        // Pre-fetch all nearby entities once with a widened AABB that covers:
        // - current position and destination in XZ (expanded by abs(move_delta))
        // - step-up height upward in Y (so the same slice covers the step-up re-check)
        let prefetch_min = Vec3::new(
            current_pos.x - player.radius - move_delta.x.abs(),
            current_pos.y - player.half_height,
            current_pos.z - player.radius - move_delta.z.abs(),
        );
        let prefetch_max = Vec3::new(
            current_pos.x + player.radius + move_delta.x.abs(),
            current_pos.y + player.half_height + SUB_VOXEL_SIZE + STEP_UP_TOLERANCE,
            current_pos.z + player.radius + move_delta.z.abs(),
        );
        let prefetched_entities = spatial_grid.get_entities_in_aabb(prefetch_min, prefetch_max);

        // Share the pre-fetched slice with apply_physics (runs later in the same frame).
        // apply_physics checks bounds containment before using it.
        pre_fetched.entities.clone_from(&prefetched_entities);
        pre_fetched.bounds = Some((prefetch_min, prefetch_max));

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
                Some(&prefetched_entities),
            );

            if !diagonal_collision.has_collision {
                transform.translation.x = new_x;
                transform.translation.z = new_z;
            } else if diagonal_collision.can_step_up && player.is_grounded {
                transform.translation.x = new_x;
                transform.translation.z = new_z;
                transform.translation.y = diagonal_collision.new_y;
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
                    &prefetched_entities,
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
                &prefetched_entities,
            );
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
    prefetched: &[Entity],
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
            Some(prefetched),
        );

        if !x_collision.has_collision {
            // No collision, move freely
            transform.translation.x = new_x;
        } else if x_collision.can_step_up && player.is_grounded {
            // Step-up collision - move horizontally and adjust height
            transform.translation.x = new_x;
            transform.translation.y = x_collision.new_y;
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
            Some(prefetched),
        );

        if !z_collision.has_collision {
            // No collision, move freely
            transform.translation.z = new_z;
        } else if z_collision.can_step_up && player.is_grounded {
            // Step-up collision - move horizontally and adjust height
            transform.translation.z = new_z;
            transform.translation.y = z_collision.new_y;
            // Reset vertical velocity to prevent falling after step-up
            player.velocity.y = 0.0;
        }
        // else: blocking collision, don't move
    }
}

#[cfg(test)]
mod tests;
