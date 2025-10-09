//! World generation and initialization system.
//!
//! This module handles:
//! - Game world initialization
//! - Voxel and sub-voxel spawning
//! - Player entity creation
//! - Camera setup
//! - Lighting setup
//! - Collision box creation

use super::components::{CollisionBox, GameCamera, Player, SubVoxel, Voxel, VoxelType};
use super::resources::{GameInitialized, SpatialGrid, VoxelWorld};
use bevy::prelude::*;

const SUB_VOXEL_COUNT: i32 = 8; // 8x8x8 sub-voxels per voxel
const SUB_VOXEL_SIZE: f32 = 1.0 / SUB_VOXEL_COUNT as f32;

/// System that sets up the game world on first entry to the InGame state.
///
/// This system:
/// - Initializes the voxel world and spatial grid
/// - Spawns sub-voxels for all non-air voxels with different patterns:
///   - Staircases: Progressive height increase
///   - Platforms: Single sub-voxel height
///   - Pillars: Small centered columns
///   - Full blocks: Complete 8x8x8 sub-voxel cubes
/// - Creates the player entity
/// - Sets up the camera with isometric view
/// - Adds directional lighting
/// - Creates the collision box for debugging
///
/// The system only runs once per game session, tracked by the GameInitialized resource.
pub fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_initialized: Option<Res<GameInitialized>>,
) {
    // If game is already initialized, don't run setup again
    if let Some(initialized) = game_initialized {
        if initialized.0 {
            return;
        }
    }

    // Mark game as initialized
    commands.insert_resource(GameInitialized(true));

    let voxel_world = VoxelWorld::default();
    let mut spatial_grid = SpatialGrid::default();

    // Create sub-voxel mesh (1/8 x 1/8 x 1/8 cube)
    let sub_voxel_mesh = meshes.add(Cuboid::new(SUB_VOXEL_SIZE, SUB_VOXEL_SIZE, SUB_VOXEL_SIZE));

    // Render all non-air voxels as sub-voxels
    for x in 0..voxel_world.width {
        for y in 0..voxel_world.height {
            for z in 0..voxel_world.depth {
                if let Some(voxel_type) = voxel_world.get_voxel(x, y, z) {
                    if voxel_type != VoxelType::Air {
                        // Spawn parent voxel marker (for reference, no mesh)
                        commands.spawn(Voxel);

                        // Check if this is a corner pillar voxel at y=1
                        let is_corner_pillar =
                            y == 1 && matches!((x, z), (0, 0) | (0, 3) | (3, 0) | (3, 3));

                        // Check if this is a 1-sub-voxel-height platform
                        let is_step_platform = y == 1 && ((x == 1 && z == 1) || (x == 2 && z == 2));

                        // Check if this is a staircase voxel
                        let is_staircase = y == 1 && x == 2 && z == 1;

                        if is_staircase {
                            spawn_staircase_sub_voxels(
                                &mut commands,
                                &mut spatial_grid,
                                &sub_voxel_mesh,
                                &mut materials,
                                x,
                                y,
                                z,
                            );
                        } else if is_step_platform {
                            spawn_platform_sub_voxels(
                                &mut commands,
                                &mut spatial_grid,
                                &sub_voxel_mesh,
                                &mut materials,
                                x,
                                y,
                                z,
                            );
                        } else if is_corner_pillar {
                            spawn_pillar_sub_voxels(
                                &mut commands,
                                &mut spatial_grid,
                                &sub_voxel_mesh,
                                &mut materials,
                                x,
                                y,
                                z,
                            );
                        } else {
                            spawn_full_voxel_sub_voxels(
                                &mut commands,
                                &mut spatial_grid,
                                &sub_voxel_mesh,
                                &mut materials,
                                x,
                                y,
                                z,
                            );
                        }
                    }
                }
            }
        }
    }

    commands.insert_resource(voxel_world);
    commands.insert_resource(spatial_grid);

    // Create player (sphere) - positioned on top of voxel floor
    let player_radius = 0.3;
    let player_mesh = meshes.add(Sphere::new(player_radius));
    let player_material = materials.add(Color::srgb(0.8, 0.2, 0.2));

    commands.spawn((
        Mesh3d(player_mesh),
        MeshMaterial3d(player_material),
        Transform::from_xyz(1.5, 0.5 + player_radius, 1.5),
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
        Transform::from_xyz(1.5, 0.5 + player_radius, 1.5),
        Visibility::Hidden,
        CollisionBox,
    ));

    // Add light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
    ));

    // Add camera (isometric-style view, tilted 30 degrees and rotated)
    let mut camera_transform =
        Transform::from_xyz(1.5, 8.0, 5.5).looking_at(Vec3::new(1.5, 0.0, 1.5), Vec3::Y);
    camera_transform.rotate_around(
        Vec3::new(1.5, 0.0, 1.5),
        Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
    );

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

/// Spawn sub-voxels for a staircase pattern.
///
/// Creates 8 steps, each 1 sub-voxel wide, with progressively increasing height.
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
        let step_height = step + 1; // Height of this step (1 to 8 sub-voxels)

        for sub_x in step..(step + 1) {
            // Each step is 1 sub-voxel wide in X
            for sub_y in 0..step_height {
                // Height increases with each step
                for sub_z in 0..SUB_VOXEL_COUNT {
                    // Full depth
                    spawn_sub_voxel(
                        commands,
                        spatial_grid,
                        sub_voxel_mesh,
                        materials,
                        SubVoxelSpawnParams {
                            parent_x: x,
                            parent_y: y,
                            parent_z: z,
                            sub_x,
                            sub_y,
                            sub_z,
                        },
                    );
                }
            }
        }
    }
}

/// Spawn sub-voxels for a platform pattern.
///
/// Creates a thin platform that is only 1 sub-voxel tall (8x1x8).
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
            // Only bottom layer
            for sub_z in 0..SUB_VOXEL_COUNT {
                spawn_sub_voxel(
                    commands,
                    spatial_grid,
                    sub_voxel_mesh,
                    materials,
                    SubVoxelSpawnParams {
                        parent_x: x,
                        parent_y: y,
                        parent_z: z,
                        sub_x,
                        sub_y,
                        sub_z,
                    },
                );
            }
        }
    }
}

/// Spawn sub-voxels for a pillar pattern.
///
/// Creates a small centered pillar (2x2x2 sub-voxels).
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
    let pillar_start = 3; // Start at the 3rd sub-voxel position (centered)

    for sub_x in pillar_start..(pillar_start + pillar_count) {
        for sub_y in pillar_start..(pillar_start + pillar_count) {
            for sub_z in pillar_start..(pillar_start + pillar_count) {
                spawn_sub_voxel(
                    commands,
                    spatial_grid,
                    sub_voxel_mesh,
                    materials,
                    SubVoxelSpawnParams {
                        parent_x: x,
                        parent_y: y,
                        parent_z: z,
                        sub_x,
                        sub_y,
                        sub_z,
                    },
                );
            }
        }
    }
}

/// Spawn sub-voxels for a full voxel.
///
/// Creates a complete 8x8x8 cube of sub-voxels.
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
                    SubVoxelSpawnParams {
                        parent_x: x,
                        parent_y: y,
                        parent_z: z,
                        sub_x,
                        sub_y,
                        sub_z,
                    },
                );
            }
        }
    }
}

/// Parameters for spawning a sub-voxel.
struct SubVoxelSpawnParams {
    parent_x: i32,
    parent_y: i32,
    parent_z: i32,
    sub_x: i32,
    sub_y: i32,
    sub_z: i32,
}

/// Helper function to spawn a single sub-voxel entity.
///
/// Creates a sub-voxel with:
/// - Mesh and material
/// - Transform at the correct world position
/// - SubVoxel component with parent and local coordinates
/// - Registration in the spatial grid for efficient collision queries
fn spawn_sub_voxel(
    commands: &mut Commands,
    spatial_grid: &mut SpatialGrid,
    sub_voxel_mesh: &Handle<Mesh>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    params: SubVoxelSpawnParams,
) {
    let x = params.parent_x;
    let y = params.parent_y;
    let z = params.parent_z;
    let sub_x = params.sub_x;
    let sub_y = params.sub_y;
    let sub_z = params.sub_z;
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
