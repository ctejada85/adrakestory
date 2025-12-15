//! Map reload handling and state preservation.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::{MapReloadEvent, MapReloadedEvent};
use crate::systems::game::components::{GameCamera, Npc, Player, SubVoxel};
use crate::systems::game::map::loader::MapLoadProgress;
use crate::systems::game::map::spawner::VoxelChunk;
use crate::systems::game::map::{LoadedMapData, MapLoader};
use crate::systems::game::resources::{GameInitialized, SpatialGrid};

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

/// Bundle of queries for state to preserve during reload
#[derive(SystemParam)]
pub struct ReloadStateQueries<'w, 's> {
    /// Query for player state to preserve (position and rotation)
    pub player: Query<'w, 's, (&'static Transform, &'static Player)>,
    /// Query for camera state to preserve
    pub camera: Query<'w, 's, (&'static Transform, &'static GameCamera)>,
}

/// Bundle of queries for entities to despawn during reload
#[derive(SystemParam)]
pub struct ReloadDespawnQueries<'w, 's> {
    pub chunks: Query<'w, 's, Entity, With<VoxelChunk>>,
    pub players: Query<'w, 's, Entity, With<Player>>,
    pub npcs: Query<'w, 's, Entity, With<Npc>>,
    pub subvoxels: Query<'w, 's, Entity, With<SubVoxel>>,
    pub directional_lights: Query<'w, 's, Entity, With<DirectionalLight>>,
    pub cameras: Query<'w, 's, Entity, With<GameCamera>>,
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
