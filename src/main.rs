use bevy::prelude::*;

mod states;
mod systems;

use states::GameState;
use systems::game::systems::{
    apply_gravity, apply_physics, cleanup_game, handle_escape_key, move_player, rotate_camera,
    setup_game, toggle_collision_box, update_collision_box,
};
use systems::intro_animation::systems::{animate_intro, cleanup_intro, setup_intro};
use systems::title_screen::systems::{
    button_interaction, cleanup_title_screen, fade_in_title_screen, keyboard_navigation,
    scale_text_on_resize, setup_title_screen, update_selected_button_visual,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::IntroAnimation), setup_intro)
        .add_systems(
            Update,
            animate_intro.run_if(in_state(GameState::IntroAnimation)),
        )
        .add_systems(OnExit(GameState::IntroAnimation), cleanup_intro)
        .add_systems(OnEnter(GameState::TitleScreen), setup_title_screen)
        .add_systems(
            Update,
            (
                fade_in_title_screen,
                button_interaction,
                keyboard_navigation,
                update_selected_button_visual,
                scale_text_on_resize,
            )
                .run_if(in_state(GameState::TitleScreen)),
        )
        .add_systems(OnExit(GameState::TitleScreen), cleanup_title_screen)
        .add_systems(OnEnter(GameState::InGame), setup_game)
        .add_systems(
            Update,
            (
                move_player,
                rotate_camera,
                toggle_collision_box,
                update_collision_box,
                apply_gravity,
                apply_physics,
                handle_escape_key,
            )
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(OnExit(GameState::InGame), cleanup_game)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
