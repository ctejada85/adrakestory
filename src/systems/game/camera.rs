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

            // Smoothly interpolate camera position using frame-rate-independent exponential decay.
            // alpha = 1 - exp(-speed * delta) gives identical convergence time at any frame rate.
            let alpha = 1.0 - (-game_camera.follow_speed * time.delta_secs()).exp();
            camera_transform.translation =
                camera_transform.translation.lerp(target_position, alpha);
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
}

/// Compute the frame-rate-independent lerp/slerp alpha for a given speed and delta time.
///
/// Uses exponential decay: `alpha = 1 - exp(-speed * delta)`.
/// Guarantees identical convergence time regardless of frame rate.
pub(crate) fn lerp_alpha(speed: f32, delta: f32) -> f32 {
    1.0 - (-speed * delta).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exponential decay formula must produce the same fraction of remaining distance
    /// per unit time regardless of frame rate. Verified by checking convergence after 1 second
    /// across 30, 60, and 120 fps step sizes.
    #[test]
    fn exponential_alpha_is_frame_rate_independent() {
        let speed = 15.0_f32;
        let total_seconds = 1.0_f32;

        // Simulate 1 second of lerp at different frame rates; record remaining error.
        let remaining_at_fps = |fps: u32| -> f32 {
            let delta = 1.0 / fps as f32;
            let steps = fps;
            let mut pos = 0.0_f32;
            let target = 1.0_f32;
            for _ in 0..steps {
                let alpha = lerp_alpha(speed, delta);
                pos = pos + (target - pos) * alpha;
            }
            (target - pos).abs()
        };

        let rem_30 = remaining_at_fps(30);
        let rem_60 = remaining_at_fps(60);
        let rem_120 = remaining_at_fps(120);

        // All three should converge to the same remaining error within 1% of each other.
        assert!(
            (rem_30 - rem_60).abs() < 0.01,
            "30 fps ({rem_30:.6}) and 60 fps ({rem_60:.6}) convergence differ by more than 1%"
        );
        assert!(
            (rem_60 - rem_120).abs() < 0.01,
            "60 fps ({rem_60:.6}) and 120 fps ({rem_120:.6}) convergence differ by more than 1%"
        );

        // Sanity check: after 1 second at speed=15, almost all distance should be covered.
        let _ = total_seconds; // used implicitly via fps * 1 second above
        assert!(rem_60 < 0.01, "Expected >99% convergence in 1s, got {rem_60:.6} remaining");
    }

    #[test]
    fn exponential_alpha_approaches_1_at_large_delta() {
        // A very large delta (e.g. after focus loss) should snap the camera near-instantly.
        let alpha = lerp_alpha(15.0, 10.0);
        assert!(alpha > 0.999, "Expected alpha ≈ 1.0 for large delta, got {alpha}");
        assert!(alpha <= 1.0, "alpha must never exceed 1.0, got {alpha}");
    }

    #[test]
    fn exponential_alpha_approaches_0_at_tiny_delta() {
        // An extremely small delta should cause almost no movement.
        let alpha = lerp_alpha(15.0, 0.0001);
        assert!(alpha < 0.01, "Expected alpha ≈ 0.0 for tiny delta, got {alpha}");
        assert!(alpha >= 0.0, "alpha must be non-negative, got {alpha}");
    }

    #[test]
    fn linear_approximation_differs_from_exponential_at_low_fps() {
        // At 30 fps the old linear approximation (speed * delta) diverges from the
        // correct exponential formula. This test documents that divergence.
        let speed = 15.0_f32;
        let delta_30fps = 1.0 / 30.0_f32;

        let linear_alpha = speed * delta_30fps;   // old (wrong) formula
        let exp_alpha = lerp_alpha(speed, delta_30fps); // new (correct) formula

        // Linear approximation overstates alpha at low fps (t > 0.1 is a large approximation error).
        assert!(
            (linear_alpha - exp_alpha).abs() > 0.05,
            "Expected measurable divergence at 30 fps: linear={linear_alpha:.4}, exp={exp_alpha:.4}"
        );
        // The exponential result must still be in (0, 1).
        assert!(exp_alpha > 0.0 && exp_alpha < 1.0);
    }
}
