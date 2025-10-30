//! Map spawning system that instantiates loaded map data into the game world.

use super::super::character::CharacterModel;
use super::super::components::{CollisionBox, GameCamera, Player, SubVoxel, Voxel};
use super::super::resources::{GameInitialized, SpatialGrid};
use super::format::{EntityType, MapData, SubVoxelPattern};
use super::loader::{LoadProgress, LoadedMapData, MapLoadProgress};
use bevy::gltf::GltfAssetLabel;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;

const SUB_VOXEL_COUNT: i32 = 8; // 8x8x8 sub-voxels per voxel
const SUB_VOXEL_SIZE: f32 = 1.0 / SUB_VOXEL_COUNT as f32;

/// Context for spawning voxels and sub-voxels.
struct VoxelSpawnContext<'w, 's, 'a> {
    commands: Commands<'w, 's>,
    spatial_grid: &'a mut SpatialGrid,
    sub_voxel_mesh: &'a Handle<Mesh>,
    materials: &'a mut Assets<StandardMaterial>,
}

/// Context for spawning entities.
struct EntitySpawnContext<'w, 's, 'a> {
    commands: Commands<'w, 's>,
    meshes: &'a mut Assets<Mesh>,
    materials: &'a mut Assets<StandardMaterial>,
    asset_server: &'a AssetServer,
}

/// System that spawns a loaded map into the game world.
///
/// This system reads the LoadedMapData resource and creates all the
/// necessary entities (voxels, sub-voxels, entities, lighting, camera).
pub fn spawn_map_system(
    mut commands: Commands,
    map_data: Res<LoadedMapData>,
    mut progress: ResMut<MapLoadProgress>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    game_initialized: Option<Res<GameInitialized>>,
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

    // Create sub-voxel mesh
    let sub_voxel_mesh = meshes.add(Cuboid::new(SUB_VOXEL_SIZE, SUB_VOXEL_SIZE, SUB_VOXEL_SIZE));

    // Stage 4: Spawn voxels (60-90%)
    progress.update(LoadProgress::SpawningVoxels(0.0));
    commands = {
        let mut voxel_ctx = VoxelSpawnContext {
            commands,
            spatial_grid: &mut spatial_grid,
            sub_voxel_mesh: &sub_voxel_mesh,
            materials: materials.as_mut(),
        };
        spawn_voxels(&mut voxel_ctx, map, &mut progress);
        voxel_ctx.commands
    };
    progress.update(LoadProgress::SpawningVoxels(1.0));

    // Stage 5: Spawn entities (90-95%)
    progress.update(LoadProgress::SpawningEntities(0.0));
    commands = {
        let mut entity_ctx = EntitySpawnContext {
            commands,
            meshes: meshes.as_mut(),
            materials: materials.as_mut(),
            asset_server: &asset_server,
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

/// Spawn all voxels from the map data.
fn spawn_voxels(ctx: &mut VoxelSpawnContext, map: &MapData, progress: &mut MapLoadProgress) {
    let total_voxels = map.world.voxels.len();

    for (index, voxel_data) in map.world.voxels.iter().enumerate() {
        // Update progress
        let voxel_progress = (index as f32) / (total_voxels as f32);
        progress.update(LoadProgress::SpawningVoxels(voxel_progress));

        let (x, y, z) = voxel_data.pos;

        // Spawn parent voxel marker
        ctx.commands.spawn(Voxel);

        // Determine which pattern to use
        let pattern = voxel_data.pattern.unwrap_or(SubVoxelPattern::Full);

        // Get the geometry for this pattern with rotation applied
        let geometry = pattern.geometry_with_rotation(voxel_data.rotation_state);

        // Spawn all occupied sub-voxels from the geometry
        for (sub_x, sub_y, sub_z) in geometry.occupied_positions() {
            spawn_sub_voxel(ctx, x, y, z, sub_x, sub_y, sub_z);
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
        }
    }
}

/// Spawn the player entity with a 3D character model.
///
/// This function creates:
/// 1. A player entity with physics components (no visible mesh)
/// 2. A GLB character model as a child entity for visuals
/// 3. An invisible collision box for debugging
///
/// The physics collision uses a sphere collider (radius: 0.3) which is kept
/// separate from the visual model for flexibility and performance.
fn spawn_player(ctx: &mut EntitySpawnContext, position: Vec3) {
    let player_radius = 0.3;

    // Load the character model (GLB file) with explicit scene specification
    // Using GltfAssetLabel::Scene(0) to load the first (default) scene from the GLB file
    let character_scene: Handle<Scene> = ctx.asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("characters/base_basic_pbr.glb")
    );

    info!("Loading character model: characters/base_basic_pbr.glb#Scene0");

    // Spawn the main player entity (parent) with physics components
    // No visible mesh - the GLB model will be the visual representation
    let player_entity = ctx
        .commands
        .spawn((
            Transform::from_translation(position),
            Visibility::default(),
            Player {
                speed: 3.0,
                velocity: Vec3::ZERO,
                is_grounded: true,
                radius: player_radius,
            },
            CharacterModel::new(character_scene.clone()),
        ))
        .id();

    // Spawn the character model as a child entity
    // The model scale and offset can be adjusted here if needed
    ctx.commands.spawn((
        SceneRoot(character_scene),
        Transform::from_scale(Vec3::splat(1.0)),
    )).set_parent(player_entity);

    info!("Spawned player with character model at position: {:?}", position);

    // Create collision box (invisible by default, for debugging)
    let collision_box_mesh = ctx.meshes.add(Cuboid::new(
        player_radius * 2.0,
        player_radius * 2.0,
        player_radius * 2.0,
    ));
    let collision_box_material = ctx.materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 1.0, 0.0, 0.3),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    ctx.commands.spawn((
        Mesh3d(collision_box_mesh),
        MeshMaterial3d(collision_box_material),
        Transform::from_translation(position),
        Visibility::Hidden,
        CollisionBox,
    ));
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
            follow_speed: 5.0, // Medium responsiveness
            target_position: look_at_point, // Initially look at the map's look_at point
        },
    ));
}

/// Spawn a single sub-voxel at the specified position.
fn spawn_sub_voxel(
    ctx: &mut VoxelSpawnContext,
    x: i32,
    y: i32,
    z: i32,
    sub_x: i32,
    sub_y: i32,
    sub_z: i32,
) {
    let color = Color::srgb(
        0.2 + (x as f32 * 0.2) + (sub_x as f32 * 0.01),
        0.3 + (z as f32 * 0.15) + (sub_z as f32 * 0.01),
        0.4 + (y as f32 * 0.2) + (sub_y as f32 * 0.01),
    );
    let sub_voxel_material = ctx.materials.add(color);

    let offset = -0.5 + SUB_VOXEL_SIZE * 0.5;
    let sub_x_pos = x as f32 + offset + (sub_x as f32 * SUB_VOXEL_SIZE);
    let sub_y_pos = y as f32 + offset + (sub_y as f32 * SUB_VOXEL_SIZE);
    let sub_z_pos = z as f32 + offset + (sub_z as f32 * SUB_VOXEL_SIZE);

    // Calculate and cache bounds at spawn time for efficient collision detection
    let center = Vec3::new(sub_x_pos, sub_y_pos, sub_z_pos);
    let half_size = SUB_VOXEL_SIZE / 2.0;
    let bounds = (
        center - Vec3::splat(half_size),
        center + Vec3::splat(half_size),
    );

    let sub_voxel_entity = ctx
        .commands
        .spawn((
            Mesh3d(ctx.sub_voxel_mesh.clone()),
            MeshMaterial3d(sub_voxel_material),
            Transform::from_xyz(sub_x_pos, sub_y_pos, sub_z_pos),
            SubVoxel { bounds },
        ))
        .id();

    let sub_voxel_world_pos = Vec3::new(sub_x_pos, sub_y_pos, sub_z_pos);
    let grid_coords = SpatialGrid::world_to_grid_coords(sub_voxel_world_pos);
    ctx.spatial_grid
        .cells
        .entry(grid_coords)
        .or_default()
        .push(sub_voxel_entity);
}
