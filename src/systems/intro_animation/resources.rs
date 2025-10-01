use bevy::prelude::*;

#[derive(Resource)]
pub struct IntroAnimationTimer {
    pub timer: Timer,
    pub phase: IntroPhase,
}

#[derive(Debug, PartialEq)]
pub enum IntroPhase {
    FadeIn,
    Display,
    FadeOut,
}

impl IntroAnimationTimer {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(0.2, TimerMode::Once),
            phase: IntroPhase::FadeIn,
        }
    }
}