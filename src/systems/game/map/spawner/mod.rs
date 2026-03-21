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
mod shadow_quality;

pub use chunks::{spawn_voxels_chunked, ChunkMaterial, ChunkSpawnContext};
pub use entities::{spawn_light_source, spawn_npc, spawn_player, EntitySpawnContext};
pub use meshing::{ChunkMeshBuilder, GreedyMesher, OccupancyGrid, VoxelMaterialPalette};
pub use shadow_quality::apply_shadow_quality_system;

use bevy::core_pipeline::prepass::DepthPrepass;

use super::super::components::GameCamera;
use super::super::occlusion::{
    create_occlusion_material, OcclusionConfig, OcclusionMaterialHandle, ShadowQuality,
};
use super::super::resources::{GameInitialized, SpatialGrid};
use super::format::{EntityType, MapData};
use super::loader::{LoadProgress, LoadedMapData, MapLoadProgress};
use crate::diagnostics::FrameProfiler;
use crate::profile_scope;
use bevy::ecs::system::SystemParam;
use bevy::light::{CascadeShadowConfig, CascadeShadowConfigBuilder};
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

/// Minimum camera movement (world units) required to trigger a LOD recalculation.
/// Keeps `update_chunk_lods` O(1) when the camera is stationary.
/// LOD transitions span 50–200 world units, so 0.5 units dead zone is imperceptible.
pub const LOD_MOVEMENT_THRESHOLD: f32 = 0.5;

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

/// Runtime configuration for the LOD update system.
#[derive(Resource)]
pub struct LodConfig {
    /// Minimum camera movement (world units) required to trigger a LOD recalculation.
    /// Defaults to [`LOD_MOVEMENT_THRESHOLD`].
    pub movement_threshold: f32,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            movement_threshold: LOD_MOVEMENT_THRESHOLD,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_voxel_count_is_8() {
        assert_eq!(SUB_VOXEL_COUNT, 8);
    }

    #[test]
    fn test_sub_voxel_size_is_eighth() {
        assert!((SUB_VOXEL_SIZE - 0.125).abs() < 0.001);
    }

    #[test]
    fn test_chunk_size_is_16() {
        assert_eq!(CHUNK_SIZE, 16);
    }

    #[test]
    fn test_lod_levels_is_4() {
        assert_eq!(LOD_LEVELS, 4);
    }

    #[test]
    fn test_lod_distances_increasing() {
        for i in 0..LOD_DISTANCES.len() - 1 {
            assert!(
                LOD_DISTANCES[i] < LOD_DISTANCES[i + 1],
                "LOD distances should be increasing"
            );
        }
    }

    #[test]
    fn test_face_normal_pos_x() {
        let normal = Face::PosX.normal();
        assert_eq!(normal, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_face_normal_neg_x() {
        let normal = Face::NegX.normal();
        assert_eq!(normal, [-1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_face_normal_pos_y() {
        let normal = Face::PosY.normal();
        assert_eq!(normal, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_face_normal_neg_y() {
        let normal = Face::NegY.normal();
        assert_eq!(normal, [0.0, -1.0, 0.0]);
    }

    #[test]
    fn test_face_normal_pos_z() {
        let normal = Face::PosZ.normal();
        assert_eq!(normal, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_face_normal_neg_z() {
        let normal = Face::NegZ.normal();
        assert_eq!(normal, [0.0, 0.0, -1.0]);
    }

    #[test]
    fn test_face_offset_pos_x() {
        assert_eq!(Face::PosX.offset(), (1, 0, 0));
    }

    #[test]
    fn test_face_offset_neg_x() {
        assert_eq!(Face::NegX.offset(), (-1, 0, 0));
    }

    #[test]
    fn test_face_offset_pos_y() {
        assert_eq!(Face::PosY.offset(), (0, 1, 0));
    }

    #[test]
    fn test_face_offset_neg_y() {
        assert_eq!(Face::NegY.offset(), (0, -1, 0));
    }

    #[test]
    fn test_face_offset_pos_z() {
        assert_eq!(Face::PosZ.offset(), (0, 0, 1));
    }

    #[test]
    fn test_face_offset_neg_z() {
        assert_eq!(Face::NegZ.offset(), (0, 0, -1));
    }

    #[test]
    fn test_opposite_faces_have_opposite_offsets() {
        let pairs = [(Face::PosX, Face::NegX), (Face::PosY, Face::NegY), (Face::PosZ, Face::NegZ)];

        for (pos, neg) in pairs {
            let (px, py, pz) = pos.offset();
            let (nx, ny, nz) = neg.offset();
            assert_eq!(px + nx, 0);
            assert_eq!(py + ny, 0);
            assert_eq!(pz + nz, 0);
        }
    }

    #[test]
    fn test_opposite_faces_have_opposite_normals() {
        let pairs = [(Face::PosX, Face::NegX), (Face::PosY, Face::NegY), (Face::PosZ, Face::NegZ)];

        for (pos, neg) in pairs {
            let pn = pos.normal();
            let nn = neg.normal();
            assert!((pn[0] + nn[0]).abs() < 0.001);
            assert!((pn[1] + nn[1]).abs() < 0.001);
            assert!((pn[2] + nn[2]).abs() < 0.001);
        }
    }

    // --- LOD movement threshold tests ---

    #[test]
    fn lod_threshold_constant_is_well_below_lod_distances() {
        // Threshold must be much smaller than the smallest LOD transition distance
        // so it never causes a false skip near a LOD boundary.
        assert!(LOD_MOVEMENT_THRESHOLD < LOD_DISTANCES[0] / 10.0);
    }

    #[test]
    fn lod_threshold_guard_skips_when_camera_stationary() {
        let camera_pos = Vec3::new(10.0, 5.0, 20.0);
        let last_pos = camera_pos; // no movement
        assert!(camera_pos.distance(last_pos) < LOD_MOVEMENT_THRESHOLD);
    }

    #[test]
    fn lod_threshold_guard_runs_when_camera_moves_beyond_threshold() {
        let last_pos = Vec3::new(10.0, 5.0, 20.0);
        let camera_pos = last_pos + Vec3::new(1.0, 0.0, 0.0); // 1.0 unit — beyond 0.5
        assert!(camera_pos.distance(last_pos) >= LOD_MOVEMENT_THRESHOLD);
    }

    #[test]
    fn lod_threshold_exact_boundary_skips() {
        // At exactly LOD_MOVEMENT_THRESHOLD distance the strict < guard should skip.
        let last_pos = Vec3::ZERO;
        let camera_pos = Vec3::new(LOD_MOVEMENT_THRESHOLD, 0.0, 0.0);
        assert!(!(camera_pos.distance(last_pos) < LOD_MOVEMENT_THRESHOLD));
    }

    #[test]
    fn lod_threshold_cold_start_passes_at_non_origin_position() {
        // On first frame last_camera_pos = Vec3::ZERO; a typical camera spawn position
        // is far from the origin, so the guard must pass and run the full pass.
        let last_pos = Vec3::ZERO;
        let camera_pos = Vec3::new(50.0, 10.0, 30.0);
        assert!(camera_pos.distance(last_pos) >= LOD_MOVEMENT_THRESHOLD);
    }

    // --- LodConfig resource tests ---

    #[test]
    fn lod_config_default_matches_constant() {
        let config = LodConfig::default();
        assert_eq!(config.movement_threshold, LOD_MOVEMENT_THRESHOLD);
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
    profiler: Option<Res<FrameProfiler>>,
) {
    profile_scope!(profiler, "spawn_map_system");
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
            shadow_quality: occlusion_config.shadow_quality,
        };
        let _p_chunks = profiler
            .as_ref()
            .map(|p| p.scope("spawn_voxels_chunked"));
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
    spawn_lighting(&mut commands, map, &occlusion_config);

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
///
/// Runs each frame but skips the O(N) chunk iteration when the camera has not moved
/// more than [`LOD_MOVEMENT_THRESHOLD`] world units since the last pass AND no new
/// chunks were just spawned. This keeps CPU cost O(1) when the camera is stationary.
pub fn update_chunk_lods(
    camera_query: Query<&Transform, With<Camera3d>>,
    mut chunks: Query<(&VoxelChunk, &mut ChunkLOD, &mut Mesh3d)>,
    new_chunks: Query<(), Added<VoxelChunk>>,
    lod_config: Res<LodConfig>,
    mut last_camera_pos: Local<Vec3>,
    profiler: Option<Res<FrameProfiler>>,
) {
    profile_scope!(profiler, "update_chunk_lods");
    // Get camera position, or early return if no camera
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let camera_pos = camera_transform.translation;

    let camera_moved = camera_pos.distance(*last_camera_pos) >= lod_config.movement_threshold;
    let new_chunks_present = !new_chunks.is_empty();

    // Skip if the camera hasn't moved enough AND no new chunks just spawned.
    if !camera_moved && !new_chunks_present {
        return;
    }
    *last_camera_pos = camera_pos;

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
/// Returns `(shadows_enabled, CascadeShadowConfig)` for each shadow quality level.
///
/// Used both at map-spawn time and by `apply_shadow_quality_system` for runtime changes.
pub fn shadow_params_for_quality(quality: ShadowQuality) -> (bool, CascadeShadowConfig) {
    match quality {
        ShadowQuality::None => (
            false,
            CascadeShadowConfigBuilder::default().build(),
        ),
        ShadowQuality::CharactersOnly | ShadowQuality::Low => (
            true,
            CascadeShadowConfigBuilder {
                num_cascades: 2,
                first_cascade_far_bound: 4.0,
                maximum_distance: 20.0,
                ..default()
            }
            .build(),
        ),
        ShadowQuality::High => (
            true,
            CascadeShadowConfigBuilder {
                num_cascades: 4,
                first_cascade_far_bound: 4.0,
                maximum_distance: 100.0,
                ..default()
            }
            .build(),
        ),
    }
}

fn spawn_lighting(commands: &mut Commands, map: &MapData, config: &OcclusionConfig) {
    let lighting = &map.lighting;

    // Spawn directional light if configured, applying the shadow quality setting
    if let Some(dir_light) = &lighting.directional_light {
        let (dx, dy, dz) = dir_light.direction;
        let direction = Vec3::new(dx, dy, dz).normalize();

        let (shadows_enabled, cascade_shadow_config) =
            shadow_params_for_quality(config.shadow_quality);

        commands.spawn((
            DirectionalLight {
                illuminance: dir_light.illuminance,
                color: Color::srgb(dir_light.color.0, dir_light.color.1, dir_light.color.2),
                shadows_enabled,
                shadow_depth_bias: 0.02,
                shadow_normal_bias: 1.8,
                affects_lightmapped_mesh_diffuse: true,
            },
            cascade_shadow_config,
            Transform::from_rotation(Quat::from_rotation_arc(Vec3::NEG_Z, direction)),
        ));
    }

    // Spawn ambient light using map-defined intensity
    // Convert 0.0-1.0 intensity to brightness (scale by 1000 for Bevy's lighting system)
    let ambient_brightness = lighting.ambient_intensity * 1000.0;
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: ambient_brightness,
        affects_lightmapped_meshes: true,
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
            follow_speed: 15.0,             // Responsive third-person follow
            target_position: look_at_point, // Initially look at the map's look_at point
        },
        DepthPrepass,
    ));
}
