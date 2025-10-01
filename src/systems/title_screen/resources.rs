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