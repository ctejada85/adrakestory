use bevy::prelude::*;

/// Resource to track if the pause menu is currently open (optional, for extensibility)
#[derive(Resource, Default)]
pub struct PauseMenuState {
    pub is_open: bool,
}
