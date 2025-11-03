//! Character rotation system that smoothly rotates the character model to face the movement direction.
//!
//! This module handles:
//! - Smooth interpolation of character rotation with easing
//! - Updating the visual character model (child entity) rotation
//! - Shortest path rotation algorithm to avoid spinning the long way

use super::components::Player;
use bevy::prelude::*;
use std::f32::consts::PI;

/// Ease-in-out cubic easing function.
/// Starts slow, accelerates quickly in the middle, then decelerates at the end.
///
/// # Arguments
/// * `t` - Progress value between 0.0 and 1.0
///
/// # Returns
/// Eased value between 0.0 and 1.0
fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let f = -2.0 * t + 2.0;
        1.0 - f * f * f / 2.0
    }
}

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

        // Only rotate if there's a significant difference
        if angle_diff.abs() > 0.001 {
            // Calculate base rotation step
            let rotation_step = player.rotation_speed * delta;
            
            // Calculate progress towards target (0 to 1)
            // This represents how close we are to the target
            let distance_ratio = (angle_diff.abs() / PI).min(1.0);
            
            // Apply easing to the speed multiplier
            // When far from target (distance_ratio high), use eased value for acceleration
            // This creates the "start slow, then quick" effect
            let speed_multiplier = ease_in_out_cubic(1.0 - distance_ratio) + 0.5;
            
            // Calculate rotation amount with eased speed
            let eased_step = rotation_step * speed_multiplier;
            let rotation_amount = if angle_diff.abs() <= eased_step {
                angle_diff // Snap to target if close enough
            } else {
                angle_diff.signum() * eased_step
            };
            
            // Update current rotation
            player.current_rotation += rotation_amount;
            
            // Normalize current rotation to [0, 2*PI] range
            player.current_rotation = player.current_rotation.rem_euclid(2.0 * PI);
        }

        // Find and update the character model child entity
        for &child in children.iter() {
            if let Ok(mut transform) = transform_query.get_mut(child) {
                // Apply Y-axis rotation to the character model
                transform.rotation = Quat::from_rotation_y(player.current_rotation);
            }
        }
    }
}