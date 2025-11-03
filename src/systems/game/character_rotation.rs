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
/// This system uses a fixed-duration rotation approach where all rotations
/// (45°, 90°, 180°, etc.) take the same amount of time. The easing is applied
/// to the progress (0.0 to 1.0) from start angle to target angle.
///
/// Key features:
/// - Fixed duration for all rotations (0.2 seconds by default)
/// - Progress-based easing (ease-in-out cubic)
/// - Shortest path rotation algorithm
/// - Easing resets when target changes (handled in player_movement system)
pub fn rotate_character_model(
    time: Res<Time>,
    mut player_query: Query<(&mut Player, &Children)>,
    mut transform_query: Query<&mut Transform, With<SceneRoot>>,
) {
    for (mut player, children) in player_query.iter_mut() {
        // Calculate the angle difference from start to target (shortest path)
        let mut angle_diff = player.target_rotation - player.start_rotation;

        // Normalize angle difference to [-PI, PI] for shortest path rotation
        while angle_diff > PI {
            angle_diff -= 2.0 * PI;
        }
        while angle_diff < -PI {
            angle_diff += 2.0 * PI;
        }

        // Only rotate if there's a significant difference
        if angle_diff.abs() > 0.001 {
            // Update elapsed time
            player.rotation_elapsed += time.delta_secs();
            
            // Calculate progress (0.0 to 1.0) clamped to max 1.0
            let progress = (player.rotation_elapsed / player.rotation_duration).min(1.0);
            
            // Apply easing to progress
            let eased_progress = ease_in_out_cubic(progress);
            
            // Lerp from start_rotation to target_rotation using eased progress
            player.current_rotation = player.start_rotation + (angle_diff * eased_progress);
            
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