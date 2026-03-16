//! In-game settings screen.
//!
//! Provides `SettingsPlugin` which registers the settings menu for `GameState::Settings`.
//! Accessible from both the title screen and the pause menu.
//! All `OcclusionConfig` fields are exposed with live-apply controls.
//! Settings are saved to `settings.ron` on exit and loaded on startup.

mod components;
pub mod resources;
mod systems;

pub use components::{BackButton, SettingId, SettingRow, SettingValueDisplay, SettingsMenuRoot};
pub use resources::{SelectedSettingsIndex, SettingsOrigin};

use crate::states::GameState;
use bevy::prelude::*;
use systems::{
    cleanup_settings_menu, load_settings, save_settings, settings_back_button, settings_input,
    setup_settings_menu, update_settings_visual,
};

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SettingsOrigin>()
            .add_systems(Startup, load_settings)
            .add_systems(OnEnter(GameState::Settings), setup_settings_menu)
            .add_systems(
                OnExit(GameState::Settings),
                (cleanup_settings_menu, save_settings),
            )
            .add_systems(
                Update,
                (settings_input, update_settings_visual, settings_back_button)
                    .chain()
                    .run_if(in_state(GameState::Settings)),
            );
    }
}
