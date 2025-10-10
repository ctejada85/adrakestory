//! Systems for the loading screen.

use super::components::{LoadingScreenUI, LoadingText, ProgressBarFill};
use crate::systems::game::map::MapLoadProgress;
use bevy::prelude::*;

/// Setup the loading screen UI.
pub fn setup_loading_screen(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
            LoadingScreenUI,
        ))
        .with_children(|parent| {
            // Loading title
            parent.spawn((
                Text::new("Loading Map..."),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Progress bar container
            parent
                .spawn((
                    Node {
                        width: Val::Px(400.0),
                        height: Val::Px(30.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor(Color::srgb(0.5, 0.5, 0.5)),
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                ))
                .with_children(|parent| {
                    // Progress bar fill
                    parent.spawn((
                        Node {
                            width: Val::Percent(0.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.8, 0.3)),
                        ProgressBarFill,
                    ));
                });

            // Loading status text
            parent.spawn((
                Text::new("Initializing..."),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
                LoadingText,
            ));
        });
}

/// Update the loading progress bar and text.
pub fn update_loading_progress(
    progress: Res<MapLoadProgress>,
    mut fill_query: Query<&mut Node, With<ProgressBarFill>>,
    mut text_query: Query<&mut Text, With<LoadingText>>,
) {
    if let Some(current_progress) = &progress.current {
        // Update progress bar
        for mut node in &mut fill_query {
            node.width = Val::Percent(progress.percentage() * 100.0);
        }

        // Update status text
        for mut text in &mut text_query {
            text.0 = current_progress.description();
        }
    }
}

/// Cleanup the loading screen.
pub fn cleanup_loading_screen(mut commands: Commands, query: Query<Entity, With<LoadingScreenUI>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
