use bevy::prelude::*;

#[derive(Resource)]
pub struct TitleScreenFadeTimer {
    pub timer: Timer,
}

impl TitleScreenFadeTimer {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(0.2, TimerMode::Once),
        }
    }
}

#[derive(Resource)]
pub struct SelectedMenuIndex {
    pub index: usize,
    pub total: usize,
}

impl Default for SelectedMenuIndex {
    fn default() -> Self {
        Self {
            index: 0,
            total: 4, // NewGame, Continue, Settings, Exit
        }
    }
}