//! Camera control system for the game.
//!
//! This module handles:
//! - Camera following the player with smooth interpolation
//! - Camera rotation around the player
//! - Smooth interpolation between rotation states
//! - Delete key input for temporary rotation

use super::components::{GameCamera, Player};
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
/// The camera can be temporarily rotated 90 degrees to the left by holding
/// the Delete key. When released, it smoothly returns to its original position.
///
/// The rotation is performed around the player's position (target_position),
/// which is updated by the follow_player_camera system. The camera's position
/// and rotation are both updated to maintain the view of the player.
pub fn rotate_camera(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<(&mut GameCamera, &mut Transform)>,
) {
    if let Ok((mut game_camera, mut transform)) = camera_query.get_single_mut() {
        // Use the player's position as the rotation center
        let center = game_camera.target_position;

        // Check if Delete key is pressed
        if keyboard_input.pressed(KeyCode::Delete) {
            // Rotate 90 degrees to the left around the world Y-axis
            let rotation_offset = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
            game_camera.target_rotation = rotation_offset * game_camera.original_rotation;
        } else {
            // Return to original rotation
            game_camera.target_rotation = game_camera.original_rotation;
        }

        // Smoothly interpolate rotation
        let new_rotation = transform.rotation.slerp(
            game_camera.target_rotation,
            game_camera.rotation_speed * time.delta_secs(),
        );

        // Calculate how much we rotated
        let rotation_delta = new_rotation * transform.rotation.inverse();

        // Rotate the camera position around the center point
        let offset = transform.translation - center;
        let rotated_offset = rotation_delta * offset;

        transform.translation = center + rotated_offset;
        transform.rotation = new_rotation;
    }
}
