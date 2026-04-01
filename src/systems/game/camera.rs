//! Camera control system for the game.
//!
//! This module handles:
//! - Camera following the player with smooth interpolation
//! - Camera rotation around the player (keyboard only - Delete key)
//! - Smooth interpolation between rotation states
//! - Note: Gamepad right stick controls character facing direction, not camera

use super::components::{GameCamera, Player};
use super::gamepad::{InputSource, PlayerInput};
use crate::diagnostics::FrameProfiler;
use crate::profile_scope;
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
    player_transform: Single<&Transform, With<Player>>,
    camera: Single<(&mut GameCamera, &mut Transform), Without<Player>>,
    profiler: Option<Res<FrameProfiler>>,
) {
    profile_scope!(profiler, "follow_player_camera");
    let player_position = player_transform.translation;
    let (mut game_camera, mut camera_transform) = camera.into_inner();

    // Update the target position for rotation system
    game_camera.target_position = player_position;

    // Calculate the offset in world space by rotating it with the camera's current rotation
    let rotated_offset = camera_transform.rotation * game_camera.follow_offset;

    // Calculate target camera position
    let target_position = player_position + rotated_offset;

    // Smoothly interpolate camera position using frame-rate-independent exponential decay.
    // alpha = 1 - exp(-speed * delta) gives identical convergence time at any frame rate.
    let alpha = 1.0 - (-game_camera.follow_speed * time.delta_secs()).exp();
    camera_transform.translation = camera_transform.translation.lerp(target_position, alpha);
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
    camera: Single<(&mut GameCamera, &mut Transform)>,
) {
    let (mut game_camera, mut transform) = camera.into_inner();
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

    // Smoothly interpolate rotation for keyboard (Delete key) using frame-rate-independent
    // exponential decay. alpha = 1 - exp(-speed * delta) gives identical convergence time
    // at any frame rate.
    if player_input.input_source == InputSource::KeyboardMouse {
        let alpha = 1.0 - (-game_camera.rotation_speed * delta).exp();
        let new_rotation = transform.rotation.slerp(game_camera.target_rotation, alpha);

        let rotation_delta = new_rotation * transform.rotation.inverse();

        let offset = transform.translation - center;
        let rotated_offset = rotation_delta * offset;

        transform.translation = center + rotated_offset;
        transform.rotation = new_rotation;
    }
}
