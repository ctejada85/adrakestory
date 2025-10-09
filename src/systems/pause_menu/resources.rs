use bevy::prelude::*;

/// Resource to track the selected menu index for keyboard navigation
#[derive(Resource)]
pub struct SelectedPauseMenuIndex {
    pub index: usize,
    pub total: usize,
}

impl Default for SelectedPauseMenuIndex {
    fn default() -> Self {
        Self {
            index: 0,
            total: 2, // Resume and Quit
        }
    }
}
