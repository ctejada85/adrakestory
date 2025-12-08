//! Voxel rendering system for the map editor.
//!
//! This module handles spawning and despawning voxel meshes in the 3D viewport
//! when maps are loaded or modified.

use crate::editor::state::EditorState;
use crate::systems::game::map::format::{EntityType, SubVoxelPattern};
use crate::systems::game::map::spawner::VoxelMaterialPalette;
use bevy::prelude::*;

const SUB_VOXEL_COUNT: i32 = 8; // 8x8x8 sub-voxels per voxel
const SUB_VOXEL_SIZE: f32 = 1.0 / SUB_VOXEL_COUNT as f32;

/// Marker component for voxels spawned by the editor
#[derive(Component)]
pub struct EditorVoxel;

/// Marker component for entity indicators spawned by the editor
#[derive(Component)]
pub struct EditorEntityMarker {
    pub entity_index: usize,
}

/// Resource to track if the map needs to be re-rendered
#[derive(Resource, Default)]
pub struct MapRenderState {
    pub needs_render: bool,
    pub last_voxel_count: usize,
    pub last_entity_count: usize,
}

/// Resource to cache the editor's material palette
#[derive(Resource)]
pub struct EditorMaterialPalette(pub VoxelMaterialPalette);

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
    let current_entity_count = editor_state.current_map.entities.len();

    // Check if voxel or entity count changed
    if current_voxel_count != render_state.last_voxel_count
        || current_entity_count != render_state.last_entity_count
    {
        render_state.needs_render = true;
        render_state.last_voxel_count = current_voxel_count;
        render_state.last_entity_count = current_entity_count;
        render_events.send(RenderMapEvent);
        info!(
            "Map changed, triggering re-render ({} voxels, {} entities)",
            current_voxel_count, current_entity_count
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
    palette_res: Option<Res<EditorMaterialPalette>>,
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

    // Get or create material palette
    let palette = if let Some(ref p) = palette_res {
        p.0.clone()
    } else {
        let new_palette = VoxelMaterialPalette::new(materials.as_mut());
        commands.insert_resource(EditorMaterialPalette(new_palette.clone()));
        new_palette
    };

    // Collect all sub-voxels with their material indices for sorting
    // This enables GPU instancing by spawning same-material entities consecutively
    let mut sub_voxels: Vec<(i32, i32, i32, i32, i32, i32, usize)> = Vec::new();

    for voxel_data in &editor_state.current_map.world.voxels {
        let (x, y, z) = voxel_data.pos;
        let pattern = voxel_data.pattern.unwrap_or(SubVoxelPattern::Full);

        // Debug: Log rotation state for each voxel
        if let Some(rot) = voxel_data.rotation_state {
            debug!(
                "Rendering voxel at {:?} with rotation: {:?} angle {}",
                voxel_data.pos, rot.axis, rot.angle
            );
        }

        // Get the geometry for this pattern with rotation applied
        let geometry = pattern.geometry_with_rotation(voxel_data.rotation_state);

        // Collect all occupied sub-voxels with their material indices
        for (sub_x, sub_y, sub_z) in geometry.occupied_positions() {
            let mat_idx = VoxelMaterialPalette::get_material_index(x, y, z, sub_x, sub_y, sub_z);
            sub_voxels.push((x, y, z, sub_x, sub_y, sub_z, mat_idx));
        }
    }

    // Sort by material index for optimal GPU instancing
    sub_voxels.sort_unstable_by_key(|v| v.6);

    // Spawn sub-voxels in sorted order
    for (x, y, z, sub_x, sub_y, sub_z, _) in sub_voxels {
        spawn_sub_voxel(
            &mut commands,
            &sub_voxel_mesh,
            &palette,
            x,
            y,
            z,
            sub_x,
            sub_y,
            sub_z,
        );
    }

    info!("Map rendering complete");
}

/// Spawn a single sub-voxel
#[allow(clippy::too_many_arguments)]
fn spawn_sub_voxel(
    commands: &mut Commands,
    mesh: &Handle<Mesh>,
    palette: &VoxelMaterialPalette,
    x: i32,
    y: i32,
    z: i32,
    sub_x: i32,
    sub_y: i32,
    sub_z: i32,
) {
    // Use material palette instead of creating a new material per sub-voxel
    let material = palette.get_material(x, y, z, sub_x, sub_y, sub_z);

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

/// Event sent when entities should be re-rendered
#[derive(Event)]
pub struct RenderEntitiesEvent;

/// System to render entity markers in the viewport
pub fn render_entities_system(
    mut commands: Commands,
    mut render_events: EventReader<RenderMapEvent>,
    editor_state: Res<EditorState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_markers: Query<Entity, With<EditorEntityMarker>>,
) {
    // Only render if we received an event
    if render_events.read().count() == 0 {
        return;
    }

    // Despawn existing entity markers
    for entity in existing_markers.iter() {
        commands.entity(entity).despawn_recursive();
    }

    info!(
        "Rendering {} entity markers",
        editor_state.current_map.entities.len()
    );

    // Spawn markers for each entity
    for (index, entity_data) in editor_state.current_map.entities.iter().enumerate() {
        let (x, y, z) = entity_data.position;
        let position = Vec3::new(x, y, z);

        // Get color and size based on entity type
        let (color, size) = match entity_data.entity_type {
            EntityType::PlayerSpawn => (Color::srgba(0.0, 1.0, 0.0, 0.8), 0.4),
            EntityType::Npc => (Color::srgba(0.0, 0.5, 1.0, 0.8), 0.35),
            EntityType::Enemy => (Color::srgba(1.0, 0.0, 0.0, 0.8), 0.35),
            EntityType::Item => (Color::srgba(1.0, 1.0, 0.0, 0.8), 0.25),
            EntityType::Trigger => (Color::srgba(1.0, 0.0, 1.0, 0.5), 0.5),
        };

        // Check if this entity is selected
        let is_selected = editor_state.selected_entities.contains(&index);
        let final_color = if is_selected {
            // Make selected entities brighter/more saturated
            Color::srgba(
                (color.to_srgba().red + 0.3).min(1.0),
                (color.to_srgba().green + 0.3).min(1.0),
                (color.to_srgba().blue + 0.3).min(1.0),
                1.0,
            )
        } else {
            color
        };

        // Create sphere mesh for marker
        let mesh = meshes.add(Sphere::new(size));
        let material = materials.add(StandardMaterial {
            base_color: final_color,
            alpha_mode: AlphaMode::Blend,
            unlit: true, // Make markers always visible regardless of lighting
            ..default()
        });

        // Spawn the entity marker
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(position),
            EditorEntityMarker {
                entity_index: index,
            },
        ));
    }

    info!("Entity marker rendering complete");
}
