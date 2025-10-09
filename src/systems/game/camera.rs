//! Camera control system for the game.
//!
//! This module handles:
//! - Camera rotation around the world center
//! - Smooth interpolation between rotation states
//! - Delete key input for temporary rotation

use super::components::GameCamera;
use bevy::prelude::*;

/// System that handles camera rotation based on keyboard input.
///
/// The camera can be temporarily rotated 90 degrees to the left by holding
/// the Delete key. When released, it smoothly returns to its original position.
///
/// The rotation is performed around a fixed center point (1.5, 0.0, 1.5),
/// which is the center of the game world. The camera's position and rotation
/// are both updated to maintain the view of the center point.
pub fn rotate_camera(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<(&mut GameCamera, &mut Transform)>,
) {
    if let Ok((mut game_camera, mut transform)) = camera_query.get_single_mut() {
        let center = Vec3::new(1.5, 0.0, 1.5);

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
