//! Hot reload functionality for runtime map reloading.
//!
//! This module provides file system watching to automatically detect
//! when map files are modified and trigger a reload in the running game.

mod notifications;
mod reload_handler;
mod state;
mod systems;

pub use notifications::{show_reload_notification, update_reload_notifications};
pub use reload_handler::{handle_map_reload, restore_player_position, PendingPlayerState};
pub use state::HotReloadState;
pub use systems::{
    cleanup_hot_reload, handle_hot_reload_toggle, handle_reload_hotkey, poll_hot_reload,
    setup_hot_reload_on_enter,
};

use bevy::prelude::*;
use std::path::PathBuf;

/// Event sent when a map reload is triggered
#[derive(Event)]
pub struct MapReloadEvent {
    /// Path to the map file to reload
    pub path: PathBuf,
}

/// Event sent when map reload completes
#[derive(Event)]
pub struct MapReloadedEvent {
    /// Whether the reload was successful
    pub success: bool,
    /// Message describing the result
    pub message: String,
}

/// Resource to pass the map path from main.rs to hot reload system
/// This avoids referencing CommandLineMapPath which is in the binary crate
#[derive(Resource, Default)]
pub struct MapPathForHotReload(pub Option<PathBuf>);
