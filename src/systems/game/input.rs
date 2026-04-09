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
mod tests {
    use super::*;

    // ── sync_light_sources ────────────────────────────────────────────────────

    /// All four fields are propagated from `LightSource` to `PointLight`.
    #[test]
    fn sync_propagates_all_four_fields() {
        let mut app = App::new();
        app.add_systems(Update, sync_light_sources);

        let entity = app
            .world_mut()
            .spawn((
                LightSource {
                    color: Color::srgb(1.0, 0.5, 0.0),
                    intensity: 5_000.0,
                    range: 8.0,
                    shadows_enabled: true,
                },
                PointLight::default(),
            ))
            .id();

        app.update();

        let pl = app.world().get::<PointLight>(entity).unwrap();
        let ls = app.world().get::<LightSource>(entity).unwrap();
        assert_eq!(pl.color, ls.color, "color must match");
        assert_eq!(pl.intensity, 5_000.0, "intensity must match");
        assert_eq!(pl.range, 8.0, "range must match");
        assert!(pl.shadows_enabled, "shadows_enabled must match");
    }

    /// A `PointLight` without a `LightSource` is never touched by the system.
    #[test]
    fn sync_skips_entity_without_light_source() {
        let mut app = App::new();
        app.add_systems(Update, sync_light_sources);

        let entity = app
            .world_mut()
            .spawn(PointLight {
                intensity: 42.0,
                ..default()
            })
            .id();

        app.update();

        let pl = app.world().get::<PointLight>(entity).unwrap();
        assert_eq!(pl.intensity, 42.0, "untouched entity must keep its value");
    }

    /// Mutating `LightSource` after the first frame causes `sync_light_sources`
    /// to pick up the change on the next update.
    #[test]
    fn sync_picks_up_runtime_mutation() {
        let mut app = App::new();
        app.add_systems(Update, sync_light_sources);

        let entity = app
            .world_mut()
            .spawn((LightSource::default(), PointLight::default()))
            .id();

        app.update(); // frame 1 — initial sync

        app.world_mut()
            .get_mut::<LightSource>(entity)
            .unwrap()
            .intensity = 99_999.0;

        app.update(); // frame 2 — Changed<LightSource> fires

        let pl = app.world().get::<PointLight>(entity).unwrap();
        assert_eq!(pl.intensity, 99_999.0);
    }

    /// `shadows_enabled: false` on `LightSource` overrides a `PointLight` that
    /// had shadows enabled.
    #[test]
    fn sync_shadows_disabled_overrides_point_light() {
        let mut app = App::new();
        app.add_systems(Update, sync_light_sources);

        let entity = app
            .world_mut()
            .spawn((
                LightSource {
                    shadows_enabled: false,
                    ..LightSource::default()
                },
                PointLight {
                    shadows_enabled: true,
                    ..default()
                },
            ))
            .id();

        app.update();

        let pl = app.world().get::<PointLight>(entity).unwrap();
        assert!(!pl.shadows_enabled);
    }

    /// `color` field is correctly reflected: a red `LightSource` produces a
    /// red `PointLight`.
    #[test]
    fn sync_color_field_is_reflected() {
        let mut app = App::new();
        app.add_systems(Update, sync_light_sources);

        let red = Color::srgb(1.0, 0.0, 0.0);
        let entity = app
            .world_mut()
            .spawn((
                LightSource {
                    color: red,
                    ..LightSource::default()
                },
                PointLight::default(),
            ))
            .id();

        app.update();

        let pl = app.world().get::<PointLight>(entity).unwrap();
        assert_eq!(pl.color, red);
    }

    // ── flicker_lights ────────────────────────────────────────────────────────

    /// After running `flicker_lights`, intensity stays within
    /// `[base - amplitude, base + amplitude]` on every frame.
    #[test]
    fn flicker_intensity_stays_within_bounds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, flicker_lights);

        let flicker = FlickerLight::default();
        let lo = flicker.base_intensity - flicker.amplitude;
        let hi = flicker.base_intensity + flicker.amplitude;

        let entity = app
            .world_mut()
            .spawn((LightSource::default(), flicker))
            .id();

        for _ in 0..30 {
            app.update();
            let intensity = app.world().get::<LightSource>(entity).unwrap().intensity;
            assert!(
                intensity >= lo && intensity <= hi,
                "intensity {intensity} out of [{lo}, {hi}]"
            );
        }
    }

    /// A `LightSource` without a `FlickerLight` component is not touched by
    /// `flicker_lights`.
    #[test]
    fn flicker_skips_entity_without_flicker_component() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, flicker_lights);

        let entity = app
            .world_mut()
            .spawn(LightSource {
                intensity: 1_234.0,
                ..LightSource::default()
            })
            .id();

        for _ in 0..5 {
            app.update();
        }

        let intensity = app.world().get::<LightSource>(entity).unwrap().intensity;
        assert_eq!(intensity, 1_234.0, "intensity must be unchanged");
    }

    /// `flicker_lights` followed by `sync_light_sources` in the same frame
    /// causes the `PointLight` to reflect the flickered intensity.
    #[test]
    fn flicker_then_sync_round_trip() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // flicker_lights must run before sync_light_sources so the mutation is
        // propagated within the same frame.
        app.add_systems(Update, (flicker_lights, sync_light_sources).chain());

        let entity = app
            .world_mut()
            .spawn((
                LightSource::default(),
                FlickerLight::default(),
                PointLight::default(),
            ))
            .id();

        for _ in 0..10 {
            app.update();
            let ls_intensity = app.world().get::<LightSource>(entity).unwrap().intensity;
            let pl_intensity = app.world().get::<PointLight>(entity).unwrap().intensity;
            assert_eq!(
                pl_intensity, ls_intensity,
                "PointLight must mirror LightSource after flicker+sync"
            );
        }
    }
}
