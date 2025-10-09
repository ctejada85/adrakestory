//! Player movement system with collision detection and step-up mechanics.
//!
//! This module handles:
//! - WASD/Arrow key input for movement
//! - Space bar for jumping
//! - Collision detection during movement
//! - Automatic step-up for small obstacles

use super::collision::{check_sub_voxel_collision, get_step_up_height, PlayerCollision};
use super::components::{Player, SubVoxel};
use super::resources::SpatialGrid;
use bevy::prelude::*;

const SUB_VOXEL_SIZE: f32 = 1.0 / 8.0;

/// System that handles player movement based on keyboard input.
///
/// This system:
/// - Processes WASD/Arrow keys for directional movement
/// - Handles Space bar for jumping
/// - Applies collision detection
/// - Implements step-up mechanics for small obstacles
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
            let max_step_height = SUB_VOXEL_SIZE; // Can step up 1 sub-voxel height

            // Try moving on X axis
            let new_x = current_pos.x + move_delta.x;
            if check_sub_voxel_collision(
                &spatial_grid,
                &sub_voxel_query,
                new_x,
                current_pos.y,
                current_pos.z,
                player.radius,
            ) {
                // Check if we can step up
                let player_collision = PlayerCollision {
                    pos: Vec3::new(new_x, current_pos.y, current_pos.z),
                    radius: player.radius,
                    current_y: current_pos.y,
                };
                if let Some(step_height) = get_step_up_height(
                    &spatial_grid,
                    &sub_voxel_query,
                    &player_collision,
                    max_step_height,
                ) {
                    // Only step up if the height increase is reasonable (within one step)
                    let height_increase = step_height - current_pos.y;
                    if height_increase > 0.001 && height_increase <= max_step_height + 0.001 {
                        transform.translation.x = new_x;
                        transform.translation.y = step_height;
                        player.is_grounded = true;
                        player.velocity.y = 0.0;
                    }
                }
                // If step-up failed, don't move (blocked)
            } else {
                transform.translation.x = new_x;
            }

            // Try moving on Z axis
            let new_z = current_pos.z + move_delta.z;
            if check_sub_voxel_collision(
                &spatial_grid,
                &sub_voxel_query,
                transform.translation.x,
                transform.translation.y,
                new_z,
                player.radius,
            ) {
                // Check if we can step up
                let player_collision = PlayerCollision {
                    pos: Vec3::new(transform.translation.x, transform.translation.y, new_z),
                    radius: player.radius,
                    current_y: transform.translation.y,
                };
                if let Some(step_height) = get_step_up_height(
                    &spatial_grid,
                    &sub_voxel_query,
                    &player_collision,
                    max_step_height,
                ) {
                    // Only step up if the height increase is reasonable (within one step)
                    let height_increase = step_height - transform.translation.y;
                    if height_increase > 0.001 && height_increase <= max_step_height + 0.001 {
                        transform.translation.z = new_z;
                        transform.translation.y = step_height;
                        player.is_grounded = true;
                        player.velocity.y = 0.0;
                    }
                }
                // If step-up failed, don't move (blocked)
            } else {
                transform.translation.z = new_z;
            }
        }
    }
}
