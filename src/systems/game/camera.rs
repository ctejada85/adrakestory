//! Camera control system for the game.
//!
//! This module handles:
//! - Camera following the player with smooth interpolation
//! - Camera rotation around the player (keyboard only - Delete key)
//! - Smooth interpolation between rotation states
//! - Note: Gamepad right stick controls character facing direction, not camera

use super::components::{GameCamera, Player};
use super::gamepad::{InputSource, PlayerInput};
use bevy::prelude::*;

/// System that makes the camera smoothly follow the player.
///
/// This system:
/// - Queries the player's current position
/// - Calculates the target camera position based on the follow offset
/// - Applies the camera's current rotation to the offset
/// - Smoothly interpolates the camera position using lerp
/// - Updates the target_position for the rotation system to use
///
/// The follow offset is rotated by the camera's current rotation, so the camera
/// maintains its relative position to the player even when rotated.
pub fn follow_player_camera(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<(&mut GameCamera, &mut Transform), Without<Player>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        if let Ok((mut game_camera, mut camera_transform)) = camera_query.get_single_mut() {
            let player_position = player_transform.translation;

            // Update the target position for rotation system
            game_camera.target_position = player_position;

            // Calculate the offset in world space by rotating it with the camera's current rotation
            let rotated_offset = camera_transform.rotation * game_camera.follow_offset;

            // Calculate target camera position
            let target_position = player_position + rotated_offset;

            // Smoothly interpolate camera position
            camera_transform.translation = camera_transform.translation.lerp(
                target_position,
                game_camera.follow_speed * time.delta_secs(),
            );
        }
    }
}

/// System that handles camera rotation based on keyboard input.
///
/// The camera can be rotated:
/// - Temporarily 90 degrees to the left by holding the Delete key
///
/// Note: The gamepad right stick now controls character facing direction,
/// not camera rotation.
///
/// The rotation is performed around the player's position (target_position),
/// which is updated by the follow_player_camera system.
pub fn rotate_camera(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_input: Res<PlayerInput>,
    mut camera_query: Query<(&mut GameCamera, &mut Transform)>,
) {
    if let Ok((mut game_camera, mut transform)) = camera_query.get_single_mut() {
        let center = game_camera.target_position;
        let delta = time.delta_secs();

        // Handle keyboard camera rotation (Delete key)
        if keyboard_input.pressed(KeyCode::Delete) {
            // Delete key: Rotate 90 degrees to the left around the world Y-axis
            let rotation_offset = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
            game_camera.target_rotation = rotation_offset * game_camera.original_rotation;
        } else {
            // Return to original rotation when not pressing Delete
            game_camera.target_rotation = game_camera.original_rotation;
        }

        // Smoothly interpolate rotation for keyboard (Delete key)
        if player_input.input_source == InputSource::KeyboardMouse {
            let new_rotation = transform.rotation.slerp(
                game_camera.target_rotation,
                game_camera.rotation_speed * delta,
            );

            let rotation_delta = new_rotation * transform.rotation.inverse();

            let offset = transform.translation - center;
            let rotated_offset = rotation_delta * offset;

            transform.translation = center + rotated_offset;
            transform.rotation = new_rotation;
        }
    }
}
