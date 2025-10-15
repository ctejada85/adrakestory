//! Voxel rendering system for the map editor.
//!
//! This module handles spawning and despawning voxel meshes in the 3D viewport
//! when maps are loaded or modified.

use crate::editor::state::EditorState;
use crate::systems::game::map::format::SubVoxelPattern;
use bevy::prelude::*;

const SUB_VOXEL_COUNT: i32 = 8; // 8x8x8 sub-voxels per voxel
const SUB_VOXEL_SIZE: f32 = 1.0 / SUB_VOXEL_COUNT as f32;
/// Marker component for voxels spawned by the editor
#[derive(Component)]
pub struct EditorVoxel;

/// Resource to track if the map needs to be re-rendered
#[derive(Resource, Default)]
pub struct MapRenderState {
    pub needs_render: bool,
    pub last_voxel_count: usize,
}

/// Event sent when the map should be re-rendered
#[derive(Event)]
pub struct RenderMapEvent;

/// System to detect when the map has changed and needs re-rendering
pub fn detect_map_changes(
    editor_state: Res<EditorState>,
    mut render_state: ResMut<MapRenderState>,
    mut render_events: EventWriter<RenderMapEvent>,
) {
    let current_voxel_count = editor_state.current_map.world.voxels.len();

    // Check if voxel count changed
    if current_voxel_count != render_state.last_voxel_count {
        render_state.needs_render = true;
        render_state.last_voxel_count = current_voxel_count;
        render_events.send(RenderMapEvent);
        info!(
            "Map changed, triggering re-render ({} voxels)",
            current_voxel_count
        );
    }
}

/// System to render the map when requested
pub fn render_map_system(
    mut commands: Commands,
    mut render_events: EventReader<RenderMapEvent>,
    editor_state: Res<EditorState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_voxels: Query<Entity, With<EditorVoxel>>,
) {
    // Only render if we received an event
    if render_events.read().count() == 0 {
        return;
    }

    info!(
        "Rendering map with {} voxels",
        editor_state.current_map.world.voxels.len()
    );

    // Despawn all existing editor voxels
    for entity in existing_voxels.iter() {
        commands.entity(entity).despawn();
    }

    // Create sub-voxel mesh (reused for all sub-voxels)
    let sub_voxel_mesh = meshes.add(Cuboid::new(SUB_VOXEL_SIZE, SUB_VOXEL_SIZE, SUB_VOXEL_SIZE));

    // Spawn voxels from the current map
    for voxel_data in &editor_state.current_map.world.voxels {
        let (x, y, z) = voxel_data.pos;
        let pattern = voxel_data.pattern.unwrap_or(SubVoxelPattern::Full);

        match pattern {
            SubVoxelPattern::Full => {
                spawn_full_voxel(&mut commands, &sub_voxel_mesh, &mut materials, x, y, z);
            }
            SubVoxelPattern::Platform => {
                spawn_platform_voxel(&mut commands, &sub_voxel_mesh, &mut materials, x, y, z);
            }
            SubVoxelPattern::Staircase => {
                spawn_staircase_voxel(&mut commands, &sub_voxel_mesh, &mut materials, x, y, z);
            }
            SubVoxelPattern::Pillar => {
                spawn_pillar_voxel(&mut commands, &sub_voxel_mesh, &mut materials, x, y, z);
            }
        }
    }

    info!("Map rendering complete");
}

/// Spawn a full voxel (8x8x8 sub-voxels)
fn spawn_full_voxel(
    commands: &mut Commands,
    mesh: &Handle<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    x: i32,
    y: i32,
    z: i32,
) {
    for sub_x in 0..SUB_VOXEL_COUNT {
        for sub_y in 0..SUB_VOXEL_COUNT {
            for sub_z in 0..SUB_VOXEL_COUNT {
                spawn_sub_voxel(commands, mesh, materials, x, y, z, sub_x, sub_y, sub_z);
            }
        }
    }
}

/// Spawn a platform voxel (8x1x8 sub-voxels)
fn spawn_platform_voxel(
    commands: &mut Commands,
    mesh: &Handle<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    x: i32,
    y: i32,
    z: i32,
) {
    for sub_x in 0..SUB_VOXEL_COUNT {
        for sub_y in 0..1 {
            for sub_z in 0..SUB_VOXEL_COUNT {
                spawn_sub_voxel(commands, mesh, materials, x, y, z, sub_x, sub_y, sub_z);
            }
        }
    }
}

/// Spawn a staircase voxel
fn spawn_staircase_voxel(
    commands: &mut Commands,
    mesh: &Handle<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    x: i32,
    y: i32,
    z: i32,
) {
    for step in 0..SUB_VOXEL_COUNT {
        let step_height = step + 1;
        for sub_x in step..(step + 1) {
            for sub_y in 0..step_height {
                for sub_z in 0..SUB_VOXEL_COUNT {
                    spawn_sub_voxel(commands, mesh, materials, x, y, z, sub_x, sub_y, sub_z);
                }
            }
        }
    }
}

/// Spawn a pillar voxel (2x2x2 centered column)
fn spawn_pillar_voxel(
    commands: &mut Commands,
    mesh: &Handle<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    x: i32,
    y: i32,
    z: i32,
) {
    let pillar_count = 2;
    let pillar_start = 3;

    for sub_x in pillar_start..(pillar_start + pillar_count) {
        for sub_y in pillar_start..(pillar_start + pillar_count) {
            for sub_z in pillar_start..(pillar_start + pillar_count) {
                spawn_sub_voxel(commands, mesh, materials, x, y, z, sub_x, sub_y, sub_z);
            }
        }
    }
}

/// Spawn a single sub-voxel
#[allow(clippy::too_many_arguments)]
fn spawn_sub_voxel(
    commands: &mut Commands,
    mesh: &Handle<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    x: i32,
    y: i32,
    z: i32,
    sub_x: i32,
    sub_y: i32,
    sub_z: i32,
) {
    // Calculate color based on position (same as game)
    let color = Color::srgb(
        0.2 + (x as f32 * 0.2) + (sub_x as f32 * 0.01),
        0.3 + (z as f32 * 0.15) + (sub_z as f32 * 0.01),
        0.4 + (y as f32 * 0.2) + (sub_y as f32 * 0.01),
    );
    let material = materials.add(color);

    // Calculate world position
    let offset = -0.5 + SUB_VOXEL_SIZE * 0.5;
    let world_x = x as f32 + offset + (sub_x as f32 * SUB_VOXEL_SIZE);
    let world_y = y as f32 + offset + (sub_y as f32 * SUB_VOXEL_SIZE);
    let world_z = z as f32 + offset + (sub_z as f32 * SUB_VOXEL_SIZE);

    // Spawn the sub-voxel
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material),
        Transform::from_xyz(world_x, world_y, world_z),
        EditorVoxel, // Mark as editor voxel for cleanup
    ));
}
