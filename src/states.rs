use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    IntroAnimation,
    TitleScreen,
    InGame,
    Settings,
    Paused,
}