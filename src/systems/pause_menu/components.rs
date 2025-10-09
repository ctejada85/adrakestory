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
