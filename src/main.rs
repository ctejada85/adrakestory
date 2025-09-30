use bevy::prelude::*;

mod components;
mod resources;
mod states;
mod systems;

use states::GameState;
use systems::intro_animation::{setup_intro, animate_intro, cleanup_intro};
use systems::title_screen::{setup_title_screen, button_interaction, cleanup_title_screen};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::IntroAnimation), setup_intro)
        .add_systems(Update, animate_intro.run_if(in_state(GameState::IntroAnimation)))
        .add_systems(OnExit(GameState::IntroAnimation), cleanup_intro)
        .add_systems(OnEnter(GameState::TitleScreen), setup_title_screen)
        .add_systems(Update, button_interaction.run_if(in_state(GameState::TitleScreen)))
        .add_systems(OnExit(GameState::TitleScreen), cleanup_title_screen)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}