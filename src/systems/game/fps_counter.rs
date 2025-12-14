//! FPS counter overlay for in-game performance monitoring.
//!
//! This module provides an FPS counter that can be toggled with F3.

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

/// Plugin that adds FPS counter functionality to the game.
pub struct FpsCounterPlugin;

impl Plugin for FpsCounterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .init_resource::<FpsCounterState>()
            .add_systems(Startup, setup_fps_counter)
            .add_systems(Update, (toggle_fps_counter, update_fps_counter));
    }
}

/// Resource to track FPS counter visibility state.
#[derive(Resource)]
pub struct FpsCounterState {
    pub visible: bool,
}

impl Default for FpsCounterState {
    fn default() -> Self {
        Self { visible: false }
    }
}

/// Marker component for the FPS counter text.
#[derive(Component)]
pub struct FpsText;

/// Sets up the FPS counter UI elements (initially hidden).
fn setup_fps_counter(mut commands: Commands) {
    commands.spawn((
        Text::new("FPS: --"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(0.0, 1.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        Visibility::Hidden,
        FpsText,
    ));
}

/// System that toggles FPS counter visibility with F3.
fn toggle_fps_counter(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<FpsCounterState>,
    mut fps_query: Query<&mut Visibility, With<FpsText>>,
) {
    if keyboard_input.just_pressed(KeyCode::F3) {
        state.visible = !state.visible;
        for mut visibility in &mut fps_query {
            *visibility = if state.visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
        info!(
            "FPS counter {}",
            if state.visible { "enabled" } else { "disabled" }
        );
    }
}

/// System that updates the FPS counter text.
fn update_fps_counter(
    diagnostics: Res<DiagnosticsStore>,
    state: Res<FpsCounterState>,
    mut fps_query: Query<&mut Text, With<FpsText>>,
) {
    // Only update if visible to save performance
    if !state.visible {
        return;
    }

    if let Some(fps) = diagnostics
        .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
    {
        for mut text in &mut fps_query {
            **text = format!("FPS: {:.0}", fps);
        }
    }
}
