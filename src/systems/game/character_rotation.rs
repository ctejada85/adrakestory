//! Character rotation system that smoothly rotates the character model to face the movement direction.
//!
//! This module handles:
//! - Smooth interpolation of character rotation
//! - Updating the visual character model (child entity) rotation
//! - Shortest path rotation algorithm to avoid spinning the long way

use super::components::Player;
use bevy::prelude::*;
use std::f32::consts::PI;

/// System that smoothly rotates the character model to face the movement direction.
///
/// This system:
/// - Queries for Player entities with their rotation state
/// - Finds the character model child entity (SceneRoot)
/// - Smoothly interpolates the current rotation toward the target rotation
/// - Applies the rotation to the character model's Transform
///
/// The rotation uses the shortest path algorithm to ensure the character
/// doesn't spin the long way around when changing direction.
pub fn rotate_character_model(
    time: Res<Time>,
    mut player_query: Query<(&mut Player, &Children)>,
    mut transform_query: Query<&mut Transform, With<SceneRoot>>,
) {
    for (mut player, children) in player_query.iter_mut() {
        let delta = time.delta_secs();

        // Calculate the angle difference
        let mut angle_diff = player.target_rotation - player.current_rotation;

        // Normalize angle difference to [-PI, PI] for shortest path rotation
        // This ensures the character rotates the shortest way around
        while angle_diff > PI {
            angle_diff -= 2.0 * PI;
        }
        while angle_diff < -PI {
            angle_diff += 2.0 * PI;
        }

        // Calculate rotation step based on rotation speed and delta time
        let rotation_step = player.rotation_speed * delta;

        // Clamp the angle difference to prevent overshooting
        let clamped_diff = angle_diff.clamp(-rotation_step, rotation_step);

        // Update current rotation
        player.current_rotation += clamped_diff;

        // Normalize current rotation to [0, 2*PI] range
        player.current_rotation = player.current_rotation.rem_euclid(2.0 * PI);

        // Find and update the character model child entity
        for &child in children.iter() {
            if let Ok(mut transform) = transform_query.get_mut(child) {
                // Apply Y-axis rotation to the character model
                // Note: We use current_rotation directly as it's already in radians
                transform.rotation = Quat::from_rotation_y(player.current_rotation);
            }
        }
    }
}