use super::components::{PauseMenuRoot, QuitButton, ResumeButton, ScalableText};
use super::resources::SelectedPauseMenuIndex;
use crate::states::GameState;
use crate::systems::game::gamepad::{get_menu_gamepad_input, ActiveGamepad, GamepadSettings};
use bevy::prelude::*;
use bevy::window::WindowResized;

const NORMAL_BUTTON: Color = Color::srgba(0.15, 0.15, 0.15, 0.0);
const HOVERED_BUTTON: Color = Color::srgba(1.0, 0.8, 0.2, 0.3);
const PRESSED_BUTTON: Color = Color::srgba(1.0, 0.8, 0.2, 0.5);

/// Spawns the pause menu UI
pub fn setup_pause_menu(mut commands: Commands) {
    // Insert selected menu index resource
    commands.insert_resource(SelectedPauseMenuIndex::default());

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
                ScalableText::new(80.0, 1.0),
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
                                ScalableText::new(30.0, 1.0),
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
                                ScalableText::new(30.0, 1.0),
                            ));
                        });
                });
        });
}

/// Handles ESC key or B button to resume game from pause menu
pub fn pause_menu_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    active_gamepad: Res<ActiveGamepad>,
    gamepad_query: Query<&Gamepad>,
    settings: Res<GamepadSettings>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Get gamepad input (we use back/B button to resume)
    let (_gp_up, _gp_down, _gp_select, gp_back) =
        get_menu_gamepad_input(&active_gamepad, &gamepad_query, &settings);

    // Also check Start button to unpause
    let gp_start = if let Some(gamepad_entity) = active_gamepad.0 {
        if let Ok(gamepad) = gamepad_query.get(gamepad_entity) {
            gamepad.just_pressed(bevy::input::gamepad::GamepadButton::Start)
        } else {
            false
        }
    } else {
        false
    };

    if keyboard_input.just_pressed(KeyCode::Escape) || gp_back || gp_start {
        next_state.set(GameState::InGame);
    }
}

/// Handles keyboard and gamepad navigation for the pause menu
pub fn keyboard_navigation(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    active_gamepad: Res<ActiveGamepad>,
    gamepad_query: Query<&Gamepad>,
    settings: Res<GamepadSettings>,
    mut selected: ResMut<SelectedPauseMenuIndex>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    // Get gamepad input
    let (gp_up, gp_down, gp_select, _gp_back) =
        get_menu_gamepad_input(&active_gamepad, &gamepad_query, &settings);

    // Navigate menu with arrow keys or gamepad D-pad
    if (keyboard_input.just_pressed(KeyCode::ArrowUp) || gp_up) && selected.index > 0 {
        selected.index -= 1;
    }
    if (keyboard_input.just_pressed(KeyCode::ArrowDown) || gp_down)
        && selected.index < selected.total - 1
    {
        selected.index += 1;
    }

    // Select option with Enter or A button
    if keyboard_input.just_pressed(KeyCode::Enter) || gp_select {
        match selected.index {
            0 => {
                // Resume
                next_state.set(GameState::InGame);
            }
            1 => {
                // Quit
                exit.send(AppExit::Success);
            }
            _ => {}
        }
    }
}

/// Updates the visual appearance of buttons based on keyboard selection
type PauseMenuButtonQueryItem<'a> = (
    Option<&'a ResumeButton>,
    Option<&'a QuitButton>,
    Mut<'a, BackgroundColor>,
    &'a Interaction,
);

pub fn update_selected_button_visual(
    selected: Res<SelectedPauseMenuIndex>,
    mut button_query: Query<PauseMenuButtonQueryItem, With<Button>>,
) {
    for (is_resume, is_quit, mut bg_color, interaction) in &mut button_query {
        // Determine button index
        let button_index = if is_resume.is_some() {
            Some(0)
        } else if is_quit.is_some() {
            Some(1)
        } else {
            None
        };

        if let Some(idx) = button_index {
            // Only apply keyboard selection color if not being hovered/pressed by mouse
            if *interaction == Interaction::None {
                if idx == selected.index {
                    *bg_color = HOVERED_BUTTON.into();
                } else {
                    *bg_color = NORMAL_BUTTON.into();
                }
            }
        }
    }
}

/// Handles button interaction for Resume and Quit
type PauseMenuButtonInteractionQueryItem<'a> = (
    &'a Interaction,
    Mut<'a, BackgroundColor>,
    Option<&'a ResumeButton>,
    Option<&'a QuitButton>,
);

pub fn pause_menu_button_interaction(
    mut interaction_query: Query<
        PauseMenuButtonInteractionQueryItem,
        (Changed<Interaction>, With<Button>),
    >,
    mut selected: ResMut<SelectedPauseMenuIndex>,
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
                // Update selected index when hovering
                if is_resume.is_some() {
                    selected.index = 0;
                } else if is_quit.is_some() {
                    selected.index = 1;
                }
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

/// Scales text elements based on window size to maintain proportions
pub fn scale_text_on_resize(
    mut resize_events: EventReader<WindowResized>,
    mut text_query: Query<(&ScalableText, &mut TextFont)>,
) {
    for event in resize_events.read() {
        let scale = (event.width + event.height) / 2000.0;

        for (scalable, mut font) in &mut text_query {
            font.font_size = scalable.base_size * scale * scalable.scale_factor;
        }
    }
}

/// Cleans up the pause menu UI
pub fn cleanup_pause_menu(mut commands: Commands, root_query: Query<Entity, With<PauseMenuRoot>>) {
    for entity in &root_query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<SelectedPauseMenuIndex>();
}
