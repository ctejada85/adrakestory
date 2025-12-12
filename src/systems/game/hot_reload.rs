//! Hot reload functionality for runtime map reloading.
//!
//! This module provides file system watching to automatically detect
//! when map files are modified and trigger a reload in the running game.

use bevy::prelude::*;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::components::{GameCamera, Npc, Player, SubVoxel};
use super::map::loader::MapLoadProgress;
use super::map::spawner::VoxelChunk;
use super::map::{LoadedMapData, MapLoader};
use super::resources::{GameInitialized, SpatialGrid};

/// Resource to track hot reload state
#[derive(Resource)]
pub struct HotReloadState {
    /// File watcher instance (wrapped for thread safety)
    watcher: Option<Arc<Mutex<RecommendedWatcher>>>,
    /// Channel receiver for file events (wrapped for thread safety)
    receiver: Option<Arc<Mutex<Receiver<Result<Event, notify::Error>>>>>,
    /// Path being watched
    watched_path: Option<PathBuf>,
    /// Last reload time (for debouncing)
    last_reload: Instant,
    /// Whether hot reload is enabled
    pub enabled: bool,
}

impl Default for HotReloadState {
    fn default() -> Self {
        Self {
            watcher: None,
            receiver: None,
            watched_path: None,
            last_reload: Instant::now(),
            enabled: true,
        }
    }
}

impl HotReloadState {
    /// Start watching a map file for changes
    pub fn watch_file(&mut self, path: PathBuf) -> Result<(), String> {
        // Stop any existing watcher
        self.stop_watching();

        let (sender, receiver) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = sender.send(res);
            },
            Config::default().with_poll_interval(Duration::from_millis(500)),
        )
        .map_err(|e| format!("Failed to create file watcher: {}", e))?;

        // Watch the parent directory since some editors do atomic saves
        // (write to temp file, then rename)
        let watch_path = path.parent().unwrap_or(&path).to_path_buf();

        watcher
            .watch(&watch_path, RecursiveMode::NonRecursive)
            .map_err(|e| format!("Failed to watch directory {:?}: {}", watch_path, e))?;

        self.watcher = Some(Arc::new(Mutex::new(watcher)));
        self.receiver = Some(Arc::new(Mutex::new(receiver)));
        self.watched_path = Some(path.clone());

        info!("Hot reload: watching {:?}", path);
        Ok(())
    }

    /// Stop watching for file changes
    pub fn stop_watching(&mut self) {
        if let Some(watcher_arc) = self.watcher.take() {
            if let Ok(mut watcher) = watcher_arc.lock() {
                if let Some(path) = &self.watched_path {
                    let watch_path = path.parent().unwrap_or(path);
                    let _ = watcher.unwatch(watch_path);
                }
            }
        }
        self.receiver = None;
        self.watched_path = None;
        info!("Hot reload: stopped watching");
    }

    /// Check for file changes (called each frame)
    /// Returns the path if a reload should be triggered
    pub fn poll_changes(&mut self) -> Option<PathBuf> {
        const DEBOUNCE_MS: u128 = 200;

        if !self.enabled {
            return None;
        }

        let receiver_arc = self.receiver.as_ref()?;
        let watched_path = self.watched_path.as_ref()?;

        // Try to lock the receiver
        let receiver = match receiver_arc.try_lock() {
            Ok(r) => r,
            Err(_) => return None, // Another thread has the lock, skip this frame
        };

        // Drain all pending events
        let mut should_reload = false;
        while let Ok(event_result) = receiver.try_recv() {
            if let Ok(event) = event_result {
                // Check if this event affects our watched file
                let affects_our_file = event.paths.iter().any(|p| {
                    // Check if the path matches our file (handle both exact match and canonicalized paths)
                    p == watched_path
                        || p.file_name() == watched_path.file_name()
                            && p.parent() == watched_path.parent()
                });

                if affects_our_file {
                    match event.kind {
                        EventKind::Modify(_) | EventKind::Create(_) => {
                            should_reload = true;
                            debug!("Hot reload: detected {:?} on {:?}", event.kind, event.paths);
                        }
                        _ => {}
                    }
                }
            }
        }

        // Drop the lock before potentially returning
        drop(receiver);

        // Debounce: only reload if enough time has passed since last reload
        if should_reload && self.last_reload.elapsed().as_millis() > DEBOUNCE_MS {
            self.last_reload = Instant::now();
            info!("Hot reload: triggering reload for {:?}", watched_path);
            return Some(watched_path.clone());
        }

        None
    }

    /// Get the currently watched path
    pub fn watched_path(&self) -> Option<&PathBuf> {
        self.watched_path.as_ref()
    }

    /// Check if currently watching a file
    pub fn is_watching(&self) -> bool {
        self.watcher.is_some() && self.watched_path.is_some()
    }
}

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

/// Resource to store player position during reload
/// This allows restoring the player to their previous position after map respawn
#[derive(Resource)]
pub struct PendingPlayerPosition(pub Option<Vec3>);

/// System to poll for file changes and trigger reload events
pub fn poll_hot_reload(
    mut hot_reload: ResMut<HotReloadState>,
    mut reload_events: EventWriter<MapReloadEvent>,
) {
    if let Some(path) = hot_reload.poll_changes() {
        reload_events.send(MapReloadEvent { path });
    }
}

/// System to handle manual reload hotkey (F5 in game)
/// Allows the player to manually trigger a map reload while testing
pub fn handle_reload_hotkey(
    keyboard: Res<ButtonInput<KeyCode>>,
    hot_reload: Res<HotReloadState>,
    mut reload_events: EventWriter<MapReloadEvent>,
) {
    // F5 triggers manual reload (same as editor Play shortcut, but in-game)
    if keyboard.just_pressed(KeyCode::F5) {
        if let Some(path) = hot_reload.watched_path() {
            info!("Manual reload triggered via F5");
            reload_events.send(MapReloadEvent { path: path.clone() });
        } else {
            info!("F5 pressed but no map is being watched for hot reload");
        }
    }
}

/// System to handle map reload events
/// Despawns existing map entities, loads new map data, and triggers respawn
pub fn handle_map_reload(
    mut commands: Commands,
    mut reload_events: EventReader<MapReloadEvent>,
    mut reloaded_events: EventWriter<MapReloadedEvent>,
    mut progress: ResMut<MapLoadProgress>,
    // Query for player position to preserve
    player_query: Query<&Transform, With<Player>>,
    // Entities to despawn during reload
    chunks_query: Query<Entity, With<VoxelChunk>>,
    player_entities: Query<Entity, With<Player>>,
    npc_entities: Query<Entity, With<Npc>>,
    subvoxel_query: Query<Entity, With<SubVoxel>>,
    directional_lights: Query<Entity, With<DirectionalLight>>,
    cameras_query: Query<Entity, With<GameCamera>>,
) {
    for event in reload_events.read() {
        info!("Hot reload: reloading map from {:?}", event.path);

        // Store player position before despawning
        let player_pos = player_query.get_single().ok().map(|t| t.translation);
        info!("Hot reload: saving player position {:?}", player_pos);

        // Try to load the new map
        progress.clear();
        let map_result =
            MapLoader::load_from_file(event.path.to_string_lossy().as_ref(), &mut progress);

        match map_result {
            Ok(map) => {
                info!("Hot reload: successfully parsed map, despawning old entities...");

                // Count entities for logging
                let chunk_count = chunks_query.iter().count();
                let subvoxel_count = subvoxel_query.iter().count();

                // Despawn all existing map entities
                for entity in chunks_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                for entity in player_entities.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                for entity in npc_entities.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                for entity in subvoxel_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                for entity in directional_lights.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                for entity in cameras_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }

                info!(
                    "Hot reload: despawned {} chunks, {} sub-voxels",
                    chunk_count, subvoxel_count
                );

                // Clear the spatial grid
                commands.remove_resource::<SpatialGrid>();

                // Reset GameInitialized so spawn_map_system will run again
                commands.insert_resource(GameInitialized(false));

                // Insert new map data (spawn_map_system will handle spawning)
                commands.insert_resource(LoadedMapData { map });

                // Store player position for restoration after spawn
                commands.insert_resource(PendingPlayerPosition(player_pos));

                reloaded_events.send(MapReloadedEvent {
                    success: true,
                    message: "Map reloaded successfully".to_string(),
                });

                info!("Hot reload: reload complete, map will respawn");
            }
            Err(e) => {
                warn!("Hot reload failed: {}", e);
                reloaded_events.send(MapReloadedEvent {
                    success: false,
                    message: format!("Reload failed: {}", e),
                });
            }
        }
    }
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

/// Resource to pass the map path from main.rs to hot reload system
/// This avoids referencing CommandLineMapPath which is in the binary crate
#[derive(Resource, Default)]
pub struct MapPathForHotReload(pub Option<PathBuf>);

/// Component for reload notification UI displayed in-game
#[derive(Component)]
pub struct ReloadNotification {
    /// Time when the notification was spawned (elapsed seconds)
    pub spawn_time: f32,
    /// Duration in seconds before the notification is despawned
    pub duration: f32,
}

/// System to restore player position after reload
/// Runs after spawn_map_system when PendingPlayerPosition exists
pub fn restore_player_position(
    mut commands: Commands,
    pending_pos: Option<Res<PendingPlayerPosition>>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    if let Some(pending) = pending_pos {
        if let Some(saved_pos) = pending.0 {
            if let Ok(mut transform) = player_query.get_single_mut() {
                info!("Hot reload: restoring player position to {:?}", saved_pos);
                transform.translation = saved_pos;
            }
        }
        // Clean up the pending position resource
        commands.remove_resource::<PendingPlayerPosition>();
    }
}

/// System to show a notification when map reload completes
/// Spawns a text notification that fades out over time
pub fn show_reload_notification(
    mut commands: Commands,
    mut reloaded_events: EventReader<MapReloadedEvent>,
    time: Res<Time>,
) {
    for event in reloaded_events.read() {
        let color = if event.success {
            Color::srgba(0.2, 0.9, 0.2, 1.0) // Green for success
        } else {
            Color::srgba(0.9, 0.2, 0.2, 1.0) // Red for failure
        };

        info!("Showing reload notification: {}", event.message);

        // Spawn notification text with UI components
        commands.spawn((
            Text::new(&event.message),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(color),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(60.0),
                left: Val::Percent(50.0),
                // Center the text horizontally
                justify_self: JustifySelf::Center,
                ..default()
            },
            ReloadNotification {
                spawn_time: time.elapsed_secs(),
                duration: 2.5,
            },
        ));
    }
}

/// System to fade out and despawn reload notifications
/// Notifications fade out during the last 30% of their duration
pub fn update_reload_notifications(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &ReloadNotification, &mut TextColor)>,
) {
    for (entity, notification, mut text_color) in query.iter_mut() {
        let elapsed = time.elapsed_secs() - notification.spawn_time;

        if elapsed > notification.duration {
            // Notification expired, despawn it
            commands.entity(entity).despawn();
        } else if elapsed > notification.duration * 0.7 {
            // Fade out during last 30% of duration
            let fade_progress =
                (elapsed - notification.duration * 0.7) / (notification.duration * 0.3);
            let alpha = 1.0 - fade_progress;
            text_color.0 = text_color.0.with_alpha(alpha);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hot_reload_state_default() {
        let state = HotReloadState::default();
        assert!(state.enabled);
        assert!(!state.is_watching());
        assert!(state.watched_path().is_none());
    }

    #[test]
    fn test_debounce_timing() {
        let mut state = HotReloadState::default();
        // Set last_reload to now
        state.last_reload = Instant::now();

        // Immediate poll should not trigger (debounce)
        assert!(state.poll_changes().is_none());
    }
}
