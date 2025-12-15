//! Map spawning system that instantiates loaded map data into the game world.
//!
//! This module coordinates the spawning of all map elements including:
//! - Voxel chunks with greedy meshing and LOD
//! - Entities (players, NPCs, items, etc.)
//! - Lighting (directional and ambient)
//! - Camera setup

mod chunks;
mod entities;
mod meshing;

pub use chunks::{spawn_voxels_chunked, ChunkMaterial, ChunkSpawnContext};
pub use entities::{spawn_light_source, spawn_npc, spawn_player, EntitySpawnContext};
pub use meshing::{ChunkMeshBuilder, GreedyMesher, OccupancyGrid, VoxelMaterialPalette};

use super::super::components::GameCamera;
use super::super::occlusion::{
    create_occlusion_material, OcclusionConfig, OcclusionMaterialHandle,
};
use super::super::resources::{GameInitialized, SpatialGrid};
use super::format::{EntityType, MapData};
use super::loader::{LoadProgress, LoadedMapData, MapLoadProgress};
use bevy::ecs::system::SystemParam;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;

/// Number of sub-voxels per voxel axis (8x8x8 = 512 sub-voxels per voxel)
pub const SUB_VOXEL_COUNT: i32 = 8;
/// Size of a single sub-voxel in world units
pub const SUB_VOXEL_SIZE: f32 = 1.0 / SUB_VOXEL_COUNT as f32;

/// Chunk size in voxels (16x16x16 voxels per chunk)
pub const CHUNK_SIZE: i32 = 16;

/// Number of LOD levels (0 = full detail, 3 = lowest detail)
pub const LOD_LEVELS: usize = 4;

/// Distance thresholds for LOD switching (in world units)
pub const LOD_DISTANCES: [f32; 4] = [50.0, 100.0, 200.0, f32::MAX];

/// Bundled asset resources for map spawning.
#[derive(SystemParam)]
pub struct SpawnAssets<'w> {
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub occlusion_materials: ResMut<'w, Assets<super::super::occlusion::OcclusionMaterial>>,
    pub asset_server: Res<'w, AssetServer>,
}

/// Marker component for chunk entities
#[derive(Component)]
pub struct VoxelChunk {
    /// The chunk position for potential future use (chunk updates, unloading)
    #[allow(dead_code)]
    pub chunk_pos: IVec3,
    /// Center of the chunk in world coordinates for LOD distance calculation
    pub center: Vec3,
}

/// LOD component for chunks with multiple detail levels.
/// Each chunk has 4 mesh LODs that are swapped based on camera distance.
#[derive(Component)]
pub struct ChunkLOD {
    /// Mesh handles for each LOD level (0 = highest detail, 3 = lowest)
    pub lod_meshes: [Handle<Mesh>; LOD_LEVELS],
    /// Current active LOD level
    pub current_lod: usize,
}

/// Face direction for hidden face culling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Face {
    /// +X direction
    PosX,
    /// -X direction
    NegX,
    /// +Y direction (up)
    PosY,
    /// -Y direction (down)
    NegY,
    /// +Z direction
    PosZ,
    /// -Z direction
    NegZ,
}

impl Face {
    /// Returns the normal vector for this face.
    #[inline]
    pub fn normal(self) -> [f32; 3] {
        match self {
            Face::PosX => [1.0, 0.0, 0.0],
            Face::NegX => [-1.0, 0.0, 0.0],
            Face::PosY => [0.0, 1.0, 0.0],
            Face::NegY => [0.0, -1.0, 0.0],
            Face::PosZ => [0.0, 0.0, 1.0],
            Face::NegZ => [0.0, 0.0, -1.0],
        }
    }

    /// Returns the neighbor offset for this face direction.
    #[inline]
    pub fn offset(self) -> (i32, i32, i32) {
        match self {
            Face::PosX => (1, 0, 0),
            Face::NegX => (-1, 0, 0),
            Face::PosY => (0, 1, 0),
            Face::NegY => (0, -1, 0),
            Face::PosZ => (0, 0, 1),
            Face::NegZ => (0, 0, -1),
        }
    }
}

/// System that spawns a loaded map into the game world.
///
/// This system reads the LoadedMapData resource and creates all the
/// necessary entities (voxels, sub-voxels, entities, lighting, camera).
///
/// Uses chunk-based meshing to combine sub-voxels into larger meshes,
/// dramatically reducing entity count and improving performance.
pub fn spawn_map_system(
    mut commands: Commands,
    map_data: Res<LoadedMapData>,
    mut progress: ResMut<MapLoadProgress>,
    mut assets: SpawnAssets,
    game_initialized: Option<Res<GameInitialized>>,
    occlusion_config: Res<OcclusionConfig>,
) {
    // If game is already initialized, don't spawn again
    if let Some(initialized) = game_initialized {
        if initialized.0 {
            return;
        }
    }

    // Mark game as initialized
    commands.insert_resource(GameInitialized(true));

    // Initialize SpatialGrid
    let mut spatial_grid = SpatialGrid::default();

    let map = &map_data.map;

    // Create material based on occlusion config
    // When occlusion is disabled, use StandardMaterial for proper PBR lighting
    let chunk_material = if occlusion_config.enabled {
        // Occlusion material with custom shader for transparency
        let occlusion_mat =
            create_occlusion_material(assets.occlusion_materials.as_mut(), occlusion_config.technique);
        commands.insert_resource(OcclusionMaterialHandle(occlusion_mat.clone()));
        ChunkMaterial::Occlusion(occlusion_mat)
    } else {
        // Standard PBR material with vertex colors
        let standard_mat = assets.materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.9,
            metallic: 0.0,
            reflectance: 0.1,
            ..default()
        });
        ChunkMaterial::Standard(standard_mat)
    };

    // Stage 4: Spawn voxels using chunk-based meshing (60-90%)
    progress.update(LoadProgress::SpawningVoxels(0.0));
    commands = {
        let mut chunk_ctx = ChunkSpawnContext {
            commands,
            spatial_grid: &mut spatial_grid,
            meshes: assets.meshes.as_mut(),
            chunk_material,
        };
        spawn_voxels_chunked(&mut chunk_ctx, map, &mut progress);
        chunk_ctx.commands
    };
    progress.update(LoadProgress::SpawningVoxels(1.0));

    // Stage 5: Spawn entities (90-95%)
    progress.update(LoadProgress::SpawningEntities(0.0));
    commands = {
        let mut entity_ctx = EntitySpawnContext {
            commands,
            meshes: assets.meshes.as_mut(),
            materials: assets.materials.as_mut(),
            asset_server: &assets.asset_server,
        };
        spawn_entities(&mut entity_ctx, map, &mut progress);
        entity_ctx.commands
    };
    progress.update(LoadProgress::SpawningEntities(1.0));

    // Stage 6: Setup lighting (95-97%)
    progress.update(LoadProgress::Finalizing(0.0));
    spawn_lighting(&mut commands, map);

    // Stage 7: Setup camera (97-100%)
    progress.update(LoadProgress::Finalizing(0.5));
    spawn_camera(&mut commands, map);

    // Insert the spatial grid as a resource
    commands.insert_resource(spatial_grid);

    // Complete
    progress.update(LoadProgress::Finalizing(1.0));
    progress.update(LoadProgress::Complete);
}

/// System that updates chunk LOD levels based on camera distance.
/// Runs each frame to swap mesh detail based on how far chunks are from the camera.
pub fn update_chunk_lods(
    camera_query: Query<&Transform, With<Camera3d>>,
    mut chunks: Query<(&VoxelChunk, &mut ChunkLOD, &mut Mesh3d)>,
) {
    // Get camera position, or early return if no camera
    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };
    let camera_pos = camera_transform.translation;

    for (chunk, mut lod, mut mesh) in chunks.iter_mut() {
        let distance = camera_pos.distance(chunk.center);

        // Determine new LOD level based on distance thresholds
        let new_lod = LOD_DISTANCES
            .iter()
            .position(|&threshold| distance < threshold)
            .unwrap_or(LOD_LEVELS - 1);

        // Only update if LOD changed
        if new_lod != lod.current_lod {
            lod.current_lod = new_lod;
            mesh.0 = lod.lod_meshes[new_lod].clone();
        }
    }
}

/// Spawn entities from the map data.
fn spawn_entities(ctx: &mut EntitySpawnContext, map: &MapData, progress: &mut MapLoadProgress) {
    let total_entities = map.entities.len();

    for (index, entity_data) in map.entities.iter().enumerate() {
        // Update progress
        let entity_progress = (index as f32) / (total_entities as f32);
        progress.update(LoadProgress::SpawningEntities(entity_progress));

        let (x, y, z) = entity_data.position;

        match entity_data.entity_type {
            EntityType::PlayerSpawn => {
                spawn_player(ctx, Vec3::new(x, y, z));
            }
            EntityType::Npc => {
                spawn_npc(ctx, Vec3::new(x, y, z), &entity_data.properties);
            }
            EntityType::Enemy => {
                // TODO: Implement enemy spawning
                info!("Enemy spawn at ({}, {}, {}) - not yet implemented", x, y, z);
            }
            EntityType::Item => {
                // TODO: Implement item spawning
                info!("Item spawn at ({}, {}, {}) - not yet implemented", x, y, z);
            }
            EntityType::Trigger => {
                // TODO: Implement trigger spawning
                info!(
                    "Trigger spawn at ({}, {}, {}) - not yet implemented",
                    x, y, z
                );
            }
            EntityType::LightSource => {
                spawn_light_source(ctx, Vec3::new(x, y, z), &entity_data.properties);
            }
        }
    }
}

/// Spawn lighting from the map data.
fn spawn_lighting(commands: &mut Commands, map: &MapData) {
    let lighting = &map.lighting;

    // Spawn directional light if configured with high-quality shadows
    if let Some(dir_light) = &lighting.directional_light {
        let (dx, dy, dz) = dir_light.direction;
        let direction = Vec3::new(dx, dy, dz).normalize();

        let cascade_shadow_config = CascadeShadowConfigBuilder {
            num_cascades: 4,
            first_cascade_far_bound: 4.0,
            maximum_distance: 100.0,
            ..default()
        }
        .build();

        commands.spawn((
            DirectionalLight {
                illuminance: dir_light.illuminance,
                color: Color::srgb(dir_light.color.0, dir_light.color.1, dir_light.color.2),
                shadows_enabled: true,
                shadow_depth_bias: 0.02,
                shadow_normal_bias: 1.8,
            },
            cascade_shadow_config,
            Transform::from_rotation(Quat::from_rotation_arc(Vec3::NEG_Z, direction)),
        ));
    }

    // Spawn ambient light using map-defined intensity
    // Convert 0.0-1.0 intensity to brightness (scale by 1000 for Bevy's lighting system)
    let ambient_brightness = lighting.ambient_intensity * 1000.0;
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: ambient_brightness,
    });

    info!(
        "Spawned ambient light with brightness: {}",
        ambient_brightness
    );
}

/// Spawn camera from the map data.
fn spawn_camera(commands: &mut Commands, map: &MapData) {
    let camera = &map.camera;
    let (px, py, pz) = camera.position;
    let (lx, ly, lz) = camera.look_at;

    let mut camera_transform =
        Transform::from_xyz(px, py, pz).looking_at(Vec3::new(lx, ly, lz), Vec3::Y);

    // Apply rotation offset
    if camera.rotation_offset != 0.0 {
        camera_transform.rotate_around(
            Vec3::new(lx, ly, lz),
            Quat::from_rotation_y(camera.rotation_offset),
        );
    }

    let original_rotation = camera_transform.rotation;

    // Calculate initial follow offset from camera position to look_at point
    let look_at_point = Vec3::new(lx, ly, lz);
    let initial_offset = camera_transform.translation - look_at_point;

    // Transform offset to local camera space (inverse of camera rotation)
    let follow_offset = camera_transform.rotation.inverse() * initial_offset;

    commands.spawn((
        Camera3d::default(),
        camera_transform,
        GameCamera {
            original_rotation,
            target_rotation: original_rotation,
            rotation_speed: 5.0,
            follow_offset,
            follow_speed: 5.0,              // Medium responsiveness
            target_position: look_at_point, // Initially look at the map's look_at point
        },
    ));
}
