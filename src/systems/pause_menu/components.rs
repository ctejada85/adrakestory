use bevy::prelude::*;

/// Marker for the root node of the pause menu UI
#[derive(Component)]
pub struct PauseMenuRoot;

/// Marker for the Resume button
#[derive(Component)]
pub struct ResumeButton;

/// Marker for the Quit button
#[derive(Component)]
pub struct QuitButton;

/// Component for text that scales with window size
#[derive(Component)]
pub struct ScalableText {
    pub base_size: f32,
    pub scale_factor: f32,
}

impl ScalableText {
    pub fn new(base_size: f32, scale_factor: f32) -> Self {
        Self {
            base_size,
            scale_factor,
        }
    }
}
