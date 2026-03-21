use bevy::prelude::*;

/// Tracks where the settings screen was entered from, for correct back-navigation.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum SettingsOrigin {
    #[default]
    TitleScreen,
    Paused,
}

/// Keyboard/gamepad navigation state for the settings screen.
#[derive(Resource)]
pub struct SelectedSettingsIndex {
    pub index: usize,
    pub total: usize,
}

impl Default for SelectedSettingsIndex {
    fn default() -> Self {
        Self {
            index: 0,
            total: 14, // 13 settings + 1 Back button
        }
    }
}
