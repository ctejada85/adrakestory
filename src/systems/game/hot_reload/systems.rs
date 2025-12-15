//! Hot reload system functions.

use super::state::HotReloadState;
use super::{MapPathForHotReload, MapReloadEvent, MapReloadedEvent};
use bevy::prelude::*;

/// System to poll for file changes and trigger reload events
pub fn poll_hot_reload(
    mut hot_reload: ResMut<HotReloadState>,
    mut reload_events: EventWriter<MapReloadEvent>,
) {
    if let Some(path) = hot_reload.poll_changes() {
        reload_events.send(MapReloadEvent { path });
    }
}

/// System to handle manual reload hotkey (F5 or Ctrl+R in game)
/// Allows the player to manually trigger a map reload while testing
pub fn handle_reload_hotkey(
    keyboard: Res<ButtonInput<KeyCode>>,
    hot_reload: Res<HotReloadState>,
    mut reload_events: EventWriter<MapReloadEvent>,
) {
    // F5 or Ctrl+R triggers manual reload
    let f5_pressed = keyboard.just_pressed(KeyCode::F5);
    let ctrl_r_pressed = (keyboard.pressed(KeyCode::ControlLeft)
        || keyboard.pressed(KeyCode::ControlRight))
        && keyboard.just_pressed(KeyCode::KeyR);

    if f5_pressed || ctrl_r_pressed {
        if let Some(path) = hot_reload.watched_path() {
            let key = if f5_pressed { "F5" } else { "Ctrl+R" };
            info!("Manual reload triggered via {}", key);
            reload_events.send(MapReloadEvent { path: path.clone() });
        } else {
            info!("Reload hotkey pressed but no map is being watched for hot reload");
        }
    }
}

/// System to toggle hot reload on/off with Ctrl+H
/// Provides a way to temporarily disable automatic reloading
pub fn handle_hot_reload_toggle(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut hot_reload: ResMut<HotReloadState>,
    mut reloaded_events: EventWriter<MapReloadedEvent>,
) {
    let ctrl_h_pressed = (keyboard.pressed(KeyCode::ControlLeft)
        || keyboard.pressed(KeyCode::ControlRight))
        && keyboard.just_pressed(KeyCode::KeyH);

    if ctrl_h_pressed {
        hot_reload.enabled = !hot_reload.enabled;
        let status = if hot_reload.enabled {
            "enabled"
        } else {
            "disabled"
        };
        info!("Hot reload {}", status);

        // Show notification
        reloaded_events.send(MapReloadedEvent {
            success: true,
            message: format!("Hot reload {}", status),
        });
    }
}

/// System to cleanup hot reload when exiting InGame state
pub fn cleanup_hot_reload(mut hot_reload: ResMut<HotReloadState>) {
    info!("Cleaning up hot reload file watcher");
    hot_reload.stop_watching();
}

/// System to setup hot reload watcher when entering InGame state
/// Takes the map path from CommandLineMapPath resource
pub fn setup_hot_reload_on_enter(
    mut hot_reload: ResMut<HotReloadState>,
    cli_map_path: Option<Res<MapPathForHotReload>>,
) {
    // Only setup if a map path was specified
    if let Some(map_path) = cli_map_path {
        if let Some(path) = &map_path.0 {
            if let Err(e) = hot_reload.watch_file(path.clone()) {
                warn!("Failed to setup hot reload: {}", e);
            }
        }
    }
}
