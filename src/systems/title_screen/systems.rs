use bevy::prelude::*;
use bevy::window::SystemCursorIcon;
use super::components::{TitleScreenUI, TitleScreenBackground, MenuButton};
use super::resources::TitleScreenFadeTimer;
use crate::states::GameState;

const NORMAL_BUTTON: Color = Color::srgba(0.15, 0.15, 0.15, 0.0);
const HOVERED_BUTTON: Color = Color::srgba(1.0, 0.8, 0.2, 0.3);
const PRESSED_BUTTON: Color = Color::srgba(1.0, 0.8, 0.2, 0.5);

pub fn setup_title_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Insert fade timer
    commands.insert_resource(TitleScreenFadeTimer::new());

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
                    margin: UiRect::all(Val::Px(50.0)),
                    ..default()
                },
            ));

            // Button container
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(15.0),
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
                width: Val::Px(250.0),
                height: Val::Px(65.0),
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
            ));
        });
}

pub fn button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &MenuButton),
        Changed<Interaction>,
    >,
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

pub fn cleanup_title_screen(mut commands: Commands, query: Query<Entity, With<TitleScreenUI>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<TitleScreenFadeTimer>();
}