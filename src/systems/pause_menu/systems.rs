use super::components::{PauseMenuRoot, QuitButton, ResumeButton};
use crate::states::GameState;
use bevy::prelude::*;

const NORMAL_BUTTON: Color = Color::srgba(0.15, 0.15, 0.15, 0.0);
const HOVERED_BUTTON: Color = Color::srgba(1.0, 0.8, 0.2, 0.3);
const PRESSED_BUTTON: Color = Color::srgba(1.0, 0.8, 0.2, 0.5);

/// Spawns the pause menu UI
pub fn setup_pause_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            PauseMenuRoot,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Paused"),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgba(0.9, 0.9, 0.9, 1.0)),
                Node {
                    margin: UiRect::all(Val::Vw(5.0)),
                    ..default()
                },
            ));

            // Button container
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Vh(2.0),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|parent| {
                    // Resume button
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Vw(20.0),
                                height: Val::Vh(8.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(NORMAL_BUTTON),
                            ResumeButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Resume"),
                                TextFont {
                                    font_size: 30.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.9, 0.9, 0.9, 1.0)),
                            ));
                        });

                    // Quit button
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Vw(20.0),
                                height: Val::Vh(8.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(NORMAL_BUTTON),
                            QuitButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Quit"),
                                TextFont {
                                    font_size: 30.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.9, 0.9, 0.9, 1.0)),
                            ));
                        });
                });
        });
}

/// Handles ESC key to resume game from pause menu
pub fn pause_menu_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::InGame);
    }
}

/// Handles button interaction for Resume and Quit
pub fn pause_menu_button_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&ResumeButton>,
            Option<&QuitButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, mut color, is_resume, is_quit) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                if is_resume.is_some() {
                    next_state.set(GameState::InGame);
                } else if is_quit.is_some() {
                    exit.send(AppExit::Success);
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

/// Cleans up the pause menu UI
pub fn cleanup_pause_menu(mut commands: Commands, root_query: Query<Entity, With<PauseMenuRoot>>) {
    for entity in &root_query {
        commands.entity(entity).despawn_recursive();
    }
}
