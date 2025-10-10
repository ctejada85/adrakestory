//! Map spawning system that instantiates loaded map data into the game world.

use super::super::components::{CollisionBox, GameCamera, Player, SubVoxel, Voxel};
use super::super::resources::{GameInitialized, SpatialGrid};
use super::format::{EntityType, MapData, SubVoxelPattern};
use super::loader::{LoadProgress, LoadedMapData, MapLoadProgress};
use bevy::prelude::*;

const SUB_VOXEL_COUNT: i32 = 8; // 8x8x8 sub-voxels per voxel
const SUB_VOXEL_SIZE: f32 = 1.0 / SUB_VOXEL_COUNT as f32;

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
    mut spatial_grid: ResMut<SpatialGrid>,
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

    let map = &map_data.map;

    // Create sub-voxel mesh
    let sub_voxel_mesh = meshes.add(Cuboid::new(SUB_VOXEL_SIZE, SUB_VOXEL_SIZE, SUB_VOXEL_SIZE));

    // Stage 4: Spawn voxels (60-90%)
    progress.update(LoadProgress::SpawningVoxels(0.0));
    spawn_voxels(
        &mut commands,
        &mut spatial_grid,
        &sub_voxel_mesh,
        &mut materials,
        map,
        &mut progress,
    );
    progress.update(LoadProgress::SpawningVoxels(1.0));

    // Stage 5: Spawn entities (90-95%)
    progress.update(LoadProgress::SpawningEntities(0.0));
    spawn_entities(
        &mut commands,
        &mut meshes,
        &mut materials,
        map,
        &mut progress,
    );
    progress.update(LoadProgress::SpawningEntities(1.0));

    // Stage 6: Setup lighting (95-97%)
    progress.update(LoadProgress::Finalizing(0.0));
    spawn_lighting(&mut commands, map);

    // Stage 7: Setup camera (97-100%)
    progress.update(LoadProgress::Finalizing(0.5));
    spawn_camera(&mut commands, map);

    // Complete
    progress.update(LoadProgress::Finalizing(1.0));
    progress.update(LoadProgress::Complete);
}

/// Spawn all voxels from the map data.
fn spawn_voxels(
    commands: &mut Commands,
    spatial_grid: &mut SpatialGrid,
    sub_voxel_mesh: &Handle<Mesh>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map: &MapData,
    progress: &mut MapLoadProgress,
) {
    let total_voxels = map.world.voxels.len();

    for (index, voxel_data) in map.world.voxels.iter().enumerate() {
        // Update progress
        let voxel_progress = (index as f32) / (total_voxels as f32);
        progress.update(LoadProgress::SpawningVoxels(voxel_progress));

        let (x, y, z) = voxel_data.pos;

        // Spawn parent voxel marker
        commands.spawn(Voxel);

        // Determine which pattern to use
        let pattern = voxel_data.pattern.unwrap_or(SubVoxelPattern::Full);

        match pattern {
            SubVoxelPattern::Full => {
                spawn_full_voxel_sub_voxels(
                    commands,
                    spatial_grid,
                    sub_voxel_mesh,
                    materials,
                    x,
                    y,
                    z,
                );
            }
            SubVoxelPattern::Platform => {
                spawn_platform_sub_voxels(
                    commands,
                    spatial_grid,
                    sub_voxel_mesh,
                    materials,
                    x,
                    y,
                    z,
                );
            }
            SubVoxelPattern::Staircase => {
                spawn_staircase_sub_voxels(
                    commands,
                    spatial_grid,
                    sub_voxel_mesh,
                    materials,
                    x,
                    y,
                    z,
                );
            }
            SubVoxelPattern::Pillar => {
                spawn_pillar_sub_voxels(commands, spatial_grid, sub_voxel_mesh, materials, x, y, z);
            }
        }
    }
}

/// Spawn entities from the map data.
fn spawn_entities(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map: &MapData,
    progress: &mut MapLoadProgress,
) {
    let total_entities = map.entities.len();

    for (index, entity_data) in map.entities.iter().enumerate() {
        // Update progress
        let entity_progress = (index as f32) / (total_entities as f32);
        progress.update(LoadProgress::SpawningEntities(entity_progress));

        let (x, y, z) = entity_data.position;

        match entity_data.entity_type {
            EntityType::PlayerSpawn => {
                spawn_player(commands, meshes, materials, Vec3::new(x, y, z));
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

/// Spawn the player entity.
fn spawn_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) {
    let player_radius = 0.3;
    let player_mesh = meshes.add(Sphere::new(player_radius));
    let player_material = materials.add(Color::srgb(0.8, 0.2, 0.2));

    commands.spawn((
        Mesh3d(player_mesh),
        MeshMaterial3d(player_material),
        Transform::from_translation(position),
        Player {
            speed: 3.0,
            velocity: Vec3::ZERO,
            is_grounded: true,
            radius: player_radius,
        },
    ));

    // Create collision box (invisible by default)
    let collision_box_mesh = meshes.add(Cuboid::new(
        player_radius * 2.0,
        player_radius * 2.0,
        player_radius * 2.0,
    ));
    let collision_box_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 1.0, 0.0, 0.3),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn((
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

    // Spawn directional light if configured
    if let Some(dir_light) = &lighting.directional_light {
        let (dx, dy, dz) = dir_light.direction;
        let direction = Vec3::new(dx, dy, dz).normalize();

        commands.spawn((
            DirectionalLight {
                illuminance: dir_light.illuminance,
                color: Color::srgb(dir_light.color.0, dir_light.color.1, dir_light.color.2),
                ..default()
            },
            Transform::from_rotation(Quat::from_rotation_arc(Vec3::NEG_Z, direction)),
        ));
    }

    // Note: Ambient lighting is typically handled by Bevy's environment settings
    // For now, we'll just log it
    info!("Map ambient intensity: {}", lighting.ambient_intensity);
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

    commands.spawn((
        Camera3d::default(),
        camera_transform,
        GameCamera {
            original_rotation,
            target_rotation: original_rotation,
            rotation_speed: 5.0,
        },
    ));
}

// Sub-voxel spawning functions (same as in world_generation.rs)

fn spawn_staircase_sub_voxels(
    commands: &mut Commands,
    spatial_grid: &mut SpatialGrid,
    sub_voxel_mesh: &Handle<Mesh>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    x: i32,
    y: i32,
    z: i32,
) {
    for step in 0..SUB_VOXEL_COUNT {
        let step_height = step + 1;

        for sub_x in step..(step + 1) {
            for sub_y in 0..step_height {
                for sub_z in 0..SUB_VOXEL_COUNT {
                    spawn_sub_voxel(
                        commands,
                        spatial_grid,
                        sub_voxel_mesh,
                        materials,
                        x,
                        y,
                        z,
                        sub_x,
                        sub_y,
                        sub_z,
                    );
                }
            }
        }
    }
}

fn spawn_platform_sub_voxels(
    commands: &mut Commands,
    spatial_grid: &mut SpatialGrid,
    sub_voxel_mesh: &Handle<Mesh>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    x: i32,
    y: i32,
    z: i32,
) {
    for sub_x in 0..SUB_VOXEL_COUNT {
        for sub_y in 0..1 {
            for sub_z in 0..SUB_VOXEL_COUNT {
                spawn_sub_voxel(
                    commands,
                    spatial_grid,
                    sub_voxel_mesh,
                    materials,
                    x,
                    y,
                    z,
                    sub_x,
                    sub_y,
                    sub_z,
                );
            }
        }
    }
}

fn spawn_pillar_sub_voxels(
    commands: &mut Commands,
    spatial_grid: &mut SpatialGrid,
    sub_voxel_mesh: &Handle<Mesh>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    x: i32,
    y: i32,
    z: i32,
) {
    let pillar_count = 2;
    let pillar_start = 3;

    for sub_x in pillar_start..(pillar_start + pillar_count) {
        for sub_y in pillar_start..(pillar_start + pillar_count) {
            for sub_z in pillar_start..(pillar_start + pillar_count) {
                spawn_sub_voxel(
                    commands,
                    spatial_grid,
                    sub_voxel_mesh,
                    materials,
                    x,
                    y,
                    z,
                    sub_x,
                    sub_y,
                    sub_z,
                );
            }
        }
    }
}

fn spawn_full_voxel_sub_voxels(
    commands: &mut Commands,
    spatial_grid: &mut SpatialGrid,
    sub_voxel_mesh: &Handle<Mesh>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    x: i32,
    y: i32,
    z: i32,
) {
    for sub_x in 0..SUB_VOXEL_COUNT {
        for sub_y in 0..SUB_VOXEL_COUNT {
            for sub_z in 0..SUB_VOXEL_COUNT {
                spawn_sub_voxel(
                    commands,
                    spatial_grid,
                    sub_voxel_mesh,
                    materials,
                    x,
                    y,
                    z,
                    sub_x,
                    sub_y,
                    sub_z,
                );
            }
        }
    }
}

fn spawn_sub_voxel(
    commands: &mut Commands,
    spatial_grid: &mut SpatialGrid,
    sub_voxel_mesh: &Handle<Mesh>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
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
    let sub_voxel_material = materials.add(color);

    let offset = -0.5 + SUB_VOXEL_SIZE * 0.5;
    let sub_x_pos = x as f32 + offset + (sub_x as f32 * SUB_VOXEL_SIZE);
    let sub_y_pos = y as f32 + offset + (sub_y as f32 * SUB_VOXEL_SIZE);
    let sub_z_pos = z as f32 + offset + (sub_z as f32 * SUB_VOXEL_SIZE);

    let sub_voxel_entity = commands
        .spawn((
            Mesh3d(sub_voxel_mesh.clone()),
            MeshMaterial3d(sub_voxel_material),
            Transform::from_xyz(sub_x_pos, sub_y_pos, sub_z_pos),
            SubVoxel {
                parent_x: x,
                parent_y: y,
                parent_z: z,
                sub_x,
                sub_y,
                sub_z,
            },
        ))
        .id();

    let sub_voxel_world_pos = Vec3::new(sub_x_pos, sub_y_pos, sub_z_pos);
    let grid_coords = SpatialGrid::world_to_grid_coords(sub_voxel_world_pos);
    spatial_grid
        .cells
        .entry(grid_coords)
        .or_default()
        .push(sub_voxel_entity);
}
