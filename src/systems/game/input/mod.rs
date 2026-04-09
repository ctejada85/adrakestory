//! General input handling systems for the game.
//!
//! This module handles:
//! - Escape key / Start button for pausing the game
//! - Collision box visibility toggle
//! - Collision box position synchronization
//! - Flashlight toggle
//! - Light source synchronization

use super::components::{CollisionBox, FlickerLight, LightSource, Player, PlayerFlashlight};
use super::gamepad::PlayerInput;
use bevy::prelude::*;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode};

/// System that handles pause input (Escape key or Start button).
///
/// When the pause input is triggered, the game transitions to the Paused state,
/// which displays the pause menu.
pub fn handle_escape_key(
    player_input: Res<PlayerInput>,
    mut next_state: ResMut<NextState<crate::states::GameState>>,
) {
    if player_input.pause_just_pressed {
        next_state.set(crate::states::GameState::Paused);
    }
}

/// System that toggles the visibility of the collision box.
///
/// When the 'C' key is pressed, the collision box visibility is toggled
/// between visible and hidden. This is useful for debugging collision detection.
pub fn toggle_collision_box(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut collision_box_query: Query<&mut Visibility, With<CollisionBox>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        for mut visibility in &mut collision_box_query {
            *visibility = match *visibility {
                Visibility::Hidden => Visibility::Visible,
                _ => Visibility::Hidden,
            };
        }
    }
}

/// System that updates the collision box position to match the player.
///
/// This system ensures the collision box visualization stays synchronized
/// with the player's actual position, making it easier to debug collision issues.
pub fn update_collision_box(
    player_transform: Option<Single<&Transform, With<Player>>>,
    mut collision_box_query: Query<&mut Transform, (With<CollisionBox>, Without<Player>)>,
) {
    let Some(player_transform) = player_transform else {
        return;
    };
    for mut box_transform in &mut collision_box_query {
        box_transform.translation = player_transform.translation;
    }
}

/// System that toggles fullscreen mode when Alt+Enter is pressed.
///
/// This system works in any game state and switches between
/// borderless fullscreen and windowed mode.
pub fn toggle_fullscreen(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    let alt_pressed =
        keyboard_input.pressed(KeyCode::AltLeft) || keyboard_input.pressed(KeyCode::AltRight);
    let enter_just_pressed = keyboard_input.just_pressed(KeyCode::Enter);

    if alt_pressed && enter_just_pressed {
        window.mode = match window.mode {
            WindowMode::Windowed => {
                info!("Switching to fullscreen mode");
                WindowMode::BorderlessFullscreen(MonitorSelection::Current)
            }
            _ => {
                info!("Switching to windowed mode");
                WindowMode::Windowed
            }
        };
    }
}

/// System that toggles the flashlight on/off when F key or Y button is pressed.
///
/// The flashlight is a spotlight attached to the player that illuminates
/// the area in front of the character.
pub fn toggle_flashlight(
    player_input: Res<PlayerInput>,
    mut flashlight_query: Query<&mut Visibility, With<PlayerFlashlight>>,
) {
    if player_input.flashlight_toggle_just_pressed {
        for mut visibility in &mut flashlight_query {
            *visibility = match *visibility {
                Visibility::Hidden => {
                    info!("Flashlight ON");
                    Visibility::Visible
                }
                _ => {
                    info!("Flashlight OFF");
                    Visibility::Hidden
                }
            };
        }
    }
}

/// System that rotates the flashlight to match the player's facing direction.
///
/// The flashlight should always point in the direction the character is looking,
/// which is determined by the player's current_rotation (Y-axis rotation).
pub fn update_flashlight_rotation(
    player: Option<Single<&Player>>,
    mut flashlight_query: Query<&mut Transform, With<PlayerFlashlight>>,
) {
    let Some(player) = player else {
        return;
    };

    for mut transform in &mut flashlight_query {
        // Calculate the forward direction based on player's current rotation
        // Player rotation is Y-axis rotation, so forward direction is rotated accordingly
        let forward = Vec3::new(
            player.current_rotation.sin(),
            0.0,
            player.current_rotation.cos(),
        );

        // Point the spotlight in the forward direction, slightly downward
        let target = transform.translation + forward * 10.0 + Vec3::new(0.0, -1.0, 0.0);
        transform.look_at(target, Vec3::Y);
    }
}

/// System that synchronises `LightSource` component values to Bevy's `PointLight`.
///
/// When a `LightSource` component is mutated at runtime (e.g., flickering, gameplay
/// toggles), this system propagates the changes to the corresponding `PointLight`
/// so that Bevy renders the updated light. Runs only when `LightSource` has changed,
/// so there is no overhead on frames where lights are static.
pub fn sync_light_sources(mut query: Query<(&LightSource, &mut PointLight), Changed<LightSource>>) {
    for (light_source, mut point_light) in &mut query {
        point_light.color = light_source.color;
        point_light.intensity = light_source.intensity;
        point_light.range = light_source.range;
        point_light.shadows_enabled = light_source.shadows_enabled;
    }
}

/// Demo system that oscillates the intensity of any [`LightSource`] that also
/// carries a [`FlickerLight`] component.
///
/// Intensity follows a sine wave each frame:
/// ```text
/// intensity = base_intensity + amplitude * sin(elapsed_secs * speed)
/// ```
///
/// Because this system writes `&mut LightSource`, the `Changed<LightSource>`
/// filter on [`sync_light_sources`] fires automatically and propagates the new
/// value to `PointLight` within the same frame (provided `flicker_lights` runs
/// before `sync_light_sources` in the schedule).
///
/// Any light entity can opt in to the flicker effect by adding a
/// [`FlickerLight`] component; removing it stops the effect.
pub fn flicker_lights(time: Res<Time>, mut query: Query<(&mut LightSource, &FlickerLight)>) {
    let t = time.elapsed_secs();
    for (mut light, flicker) in &mut query {
        light.intensity = flicker.base_intensity + flicker.amplitude * (t * flicker.speed).sin();
    }
}

#[cfg(test)]
mod tests;
