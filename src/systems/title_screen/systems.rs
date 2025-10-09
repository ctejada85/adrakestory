use super::components::{MenuButton, ScalableText, TitleScreenBackground, TitleScreenUI};
use super::resources::{SelectedMenuIndex, TitleScreenFadeTimer};
use crate::states::GameState;
use bevy::prelude::*;
use bevy::window::WindowResized;

const NORMAL_BUTTON: Color = Color::srgba(0.15, 0.15, 0.15, 0.0);
const HOVERED_BUTTON: Color = Color::srgba(1.0, 0.8, 0.2, 0.3);
const PRESSED_BUTTON: Color = Color::srgba(1.0, 0.8, 0.2, 0.5);

pub fn setup_title_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Insert fade timer and menu selection
    commands.insert_resource(TitleScreenFadeTimer::new());
    commands.insert_resource(SelectedMenuIndex::default());

    // Root UI node
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
            TitleScreenUI,
        ))
        .with_children(|parent| {
            // Background image
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ImageNode::new(asset_server.load("textures/title_background.png")),
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
                TitleScreenBackground,
            ));

            // Title
            parent.spawn((
                Text::new("Adrakestory"),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgba(0.9, 0.9, 0.9, 0.0)),
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
                    create_menu_button(parent, "New Game", MenuButton::NewGame);
                    create_menu_button(parent, "Continue", MenuButton::Continue);
                    create_menu_button(parent, "Settings", MenuButton::Settings);
                    create_menu_button(parent, "Exit", MenuButton::Exit);
                });
        });
}

fn create_menu_button(parent: &mut ChildBuilder, text: &str, button_type: MenuButton) {
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
            BackgroundColor(Color::NONE),
            button_type,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(text),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::srgba(0.9, 0.9, 0.9, 0.0)),
                ScalableText::new(30.0, 1.0),
            ));
        });
}

pub fn button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &MenuButton),
        Changed<Interaction>,
    >,
    mut selected: ResMut<SelectedMenuIndex>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, mut color, button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                match button {
                    MenuButton::NewGame => {
                        info!("Starting new game...");
                        next_state.set(GameState::InGame);
                    }
                    MenuButton::Continue => {
                        info!("Continue not implemented yet");
                    }
                    MenuButton::Settings => {
                        info!("Opening settings...");
                        next_state.set(GameState::Settings);
                    }
                    MenuButton::Exit => {
                        info!("Exiting game...");
                        exit.send(AppExit::Success);
                    }
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                // Update selected index when hovering
                let button_index = match button {
                    MenuButton::NewGame => 0,
                    MenuButton::Continue => 1,
                    MenuButton::Settings => 2,
                    MenuButton::Exit => 3,
                };
                selected.index = button_index;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub fn fade_in_title_screen(
    time: Res<Time>,
    mut timer: ResMut<TitleScreenFadeTimer>,
    mut bg_query: Query<&mut BackgroundColor, With<TitleScreenBackground>>,
    mut text_query: Query<&mut TextColor, (Without<Parent>, Without<MenuButton>)>,
    button_query: Query<(&Children, &MenuButton), Without<TitleScreenUI>>,
    mut button_text_query: Query<&mut TextColor, With<Parent>>,
) {
    timer.timer.tick(time.delta());
    let alpha = timer.timer.fraction();

    // Fade in background image
    for mut bg in &mut bg_query {
        bg.0.set_alpha(alpha);
    }

    // Fade in title text
    for mut text_color in &mut text_query {
        text_color.0.set_alpha(alpha);
    }

    // Fade in button text
    for (children, _) in &button_query {
        for &child in children.iter() {
            if let Ok(mut text_color) = button_text_query.get_mut(child) {
                text_color.0.set_alpha(alpha);
            }
        }
    }
}

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

pub fn keyboard_navigation(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedMenuIndex>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    // Navigate menu with arrow keys
    if keyboard_input.just_pressed(KeyCode::ArrowUp) && selected.index > 0 {
        selected.index -= 1;
    }
    if keyboard_input.just_pressed(KeyCode::ArrowDown) && selected.index < selected.total - 1 {
        selected.index += 1;
    }

    // Select option with Enter
    if keyboard_input.just_pressed(KeyCode::Enter) {
        match selected.index {
            0 => {
                info!("Starting new game...");
                next_state.set(GameState::InGame);
            }
            1 => {
                info!("Continue not implemented yet");
            }
            2 => {
                info!("Opening settings...");
                next_state.set(GameState::Settings);
            }
            3 => {
                info!("Exiting game...");
                exit.send(AppExit::Success);
            }
            _ => {}
        }
    }
}

pub fn update_selected_button_visual(
    selected: Res<SelectedMenuIndex>,
    mut button_query: Query<(&MenuButton, &mut BackgroundColor, &Interaction)>,
) {
    let buttons: Vec<(usize, &MenuButton)> = vec![
        (0, &MenuButton::NewGame),
        (1, &MenuButton::Continue),
        (2, &MenuButton::Settings),
        (3, &MenuButton::Exit),
    ];

    for (button, mut bg_color, interaction) in &mut button_query {
        // Find index of current button
        let button_index = buttons
            .iter()
            .find(|(_, b)| {
                matches!(
                    (button, *b),
                    (MenuButton::NewGame, MenuButton::NewGame)
                        | (MenuButton::Continue, MenuButton::Continue)
                        | (MenuButton::Settings, MenuButton::Settings)
                        | (MenuButton::Exit, MenuButton::Exit)
                )
            })
            .map(|(i, _)| *i);

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

pub fn cleanup_title_screen(mut commands: Commands, query: Query<Entity, With<TitleScreenUI>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<TitleScreenFadeTimer>();
    commands.remove_resource::<SelectedMenuIndex>();
}
