//! Hot reload functionality for runtime map reloading.
//!
//! This module provides file system watching to automatically detect
//! when map files are modified and trigger a reload in the running game.

use bevy::ecs::system::SystemParam;
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

/// Resource to store player and camera state during reload
/// This allows restoring the player and camera to their previous state after map respawn
#[derive(Resource)]
pub struct PendingPlayerState {
    /// Player's position before reload
    pub position: Option<Vec3>,
    /// Player's rotation values before reload (target_rotation, current_rotation)
    pub rotation: Option<(f32, f32)>,
    /// Camera's transform before reload
    pub camera_transform: Option<Transform>,
    /// Camera's target_position before reload
    pub camera_target_position: Option<Vec3>,
}

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

/// Bundle of queries for state to preserve during reload
#[derive(SystemParam)]
pub struct ReloadStateQueries<'w, 's> {
    /// Query for player state to preserve (position and rotation)
    player: Query<'w, 's, (&'static Transform, &'static Player)>,
    /// Query for camera state to preserve
    camera: Query<'w, 's, (&'static Transform, &'static GameCamera)>,
}

/// Bundle of queries for entities to despawn during reload
#[derive(SystemParam)]
pub struct ReloadDespawnQueries<'w, 's> {
    chunks: Query<'w, 's, Entity, With<VoxelChunk>>,
    players: Query<'w, 's, Entity, With<Player>>,
    npcs: Query<'w, 's, Entity, With<Npc>>,
    subvoxels: Query<'w, 's, Entity, With<SubVoxel>>,
    directional_lights: Query<'w, 's, Entity, With<DirectionalLight>>,
    cameras: Query<'w, 's, Entity, With<GameCamera>>,
}

/// System to handle map reload events
/// Despawns existing map entities, loads new map data, and triggers respawn
pub fn handle_map_reload(
    mut commands: Commands,
    mut reload_events: EventReader<MapReloadEvent>,
    mut reloaded_events: EventWriter<MapReloadedEvent>,
    mut progress: ResMut<MapLoadProgress>,
    state_queries: ReloadStateQueries,
    despawn_queries: ReloadDespawnQueries,
) {
    for event in reload_events.read() {
        info!("Hot reload: reloading map from {:?}", event.path);

        // Store player position and rotation before despawning
        let player_state = state_queries
            .player
            .get_single()
            .ok()
            .map(|(t, p)| (t.translation, p.target_rotation, p.current_rotation));
        info!("Hot reload: saving player state {:?}", player_state);

        // Store camera state before despawning
        let camera_state = state_queries
            .camera
            .get_single()
            .ok()
            .map(|(t, c)| (*t, c.target_position));
        info!(
            "Hot reload: saving camera state {:?}",
            camera_state.as_ref().map(|(t, _)| t.translation)
        );

        // Try to load the new map
        progress.clear();
        let map_result =
            MapLoader::load_from_file(event.path.to_string_lossy().as_ref(), &mut progress);

        match map_result {
            Ok(map) => {
                info!("Hot reload: successfully parsed map, despawning old entities...");

                // Count entities for logging
                let chunk_count = despawn_queries.chunks.iter().count();
                let subvoxel_count = despawn_queries.subvoxels.iter().count();

                // Despawn all existing map entities
                for entity in despawn_queries.chunks.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                for entity in despawn_queries.players.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                for entity in despawn_queries.npcs.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                for entity in despawn_queries.subvoxels.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                for entity in despawn_queries.directional_lights.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                for entity in despawn_queries.cameras.iter() {
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

                // Store player state for restoration after spawn
                let (position, rotation) =
                    if let Some((pos, target_rot, current_rot)) = player_state {
                        (Some(pos), Some((target_rot, current_rot)))
                    } else {
                        (None, None)
                    };
                let (camera_transform, camera_target_position) =
                    if let Some((transform, target_pos)) = camera_state {
                        (Some(transform), Some(target_pos))
                    } else {
                        (None, None)
                    };
                commands.insert_resource(PendingPlayerState {
                    position,
                    rotation,
                    camera_transform,
                    camera_target_position,
                });

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

/// System to restore player position and rotation after reload
/// Runs after spawn_map_system when PendingPlayerState exists
pub fn restore_player_position(
    mut commands: Commands,
    pending_state: Option<Res<PendingPlayerState>>,
    mut player_query: Query<(&mut Transform, &mut Player)>,
    mut camera_query: Query<(&mut Transform, &mut GameCamera), Without<Player>>,
) {
    if let Some(pending) = pending_state {
        if let Ok((mut transform, mut player)) = player_query.get_single_mut() {
            // Restore position
            if let Some(saved_pos) = pending.position {
                info!("Hot reload: restoring player position to {:?}", saved_pos);
                transform.translation = saved_pos;
            }
            // Restore rotation
            if let Some((target_rot, current_rot)) = pending.rotation {
                info!("Hot reload: restoring player rotation to {:?}", current_rot);
                player.target_rotation = target_rot;
                player.current_rotation = current_rot;
                player.start_rotation = current_rot;
                player.rotation_elapsed = player.rotation_duration; // Mark rotation as complete
            }
        }

        // Restore camera state
        if let Ok((mut cam_transform, mut game_camera)) = camera_query.get_single_mut() {
            if let Some(saved_transform) = pending.camera_transform {
                info!(
                    "Hot reload: restoring camera position to {:?}",
                    saved_transform.translation
                );
                *cam_transform = saved_transform;
            }
            if let Some(saved_target_pos) = pending.camera_target_position {
                info!(
                    "Hot reload: restoring camera target_position to {:?}",
                    saved_target_pos
                );
                game_camera.target_position = saved_target_pos;
            }
        }

        // Clean up the pending state resource
        commands.remove_resource::<PendingPlayerState>();
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
        let mut state = HotReloadState {
            last_reload: Instant::now(),
            ..Default::default()
        };

        // Immediate poll should not trigger (debounce)
        assert!(state.poll_changes().is_none());
    }
}
