//! Voxel rendering system for the map editor.
//!
//! This module handles spawning and despawning voxel meshes in the 3D viewport
//! when maps are loaded or modified.
//!
//! ## Optimizations (Tiers 3-5)
//!
//! The editor uses optimized rendering similar to the game:
//! - **Tier 3: Chunk-Based Meshing** - Groups sub-voxels into 16³ chunks with merged meshes
//! - **Tier 4: Hidden Face Culling** - Only renders faces not adjacent to other voxels
//! - **Tier 5: Greedy Meshing** - Merges adjacent same-color faces into larger quads
//! - **Frustum Culling** - Chunks outside camera view are not rendered
//!
//! Note: LOD (Tier 6) is disabled for the editor since full detail is needed when editing.

use crate::editor::state::EditorState;
use crate::editor::tools::UpdateSelectionHighlights;
use crate::systems::game::map::format::{EntityType, SubVoxelPattern};
use crate::systems::game::map::spawner::{
    ChunkMeshBuilder, Face, GreedyMesher, OccupancyGrid, VoxelMaterialPalette, CHUNK_SIZE,
    SUB_VOXEL_COUNT, SUB_VOXEL_SIZE,
};
use bevy::math::Vec3A;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use std::collections::HashMap;

/// Marker component for chunk entities spawned by the editor
#[derive(Component)]
pub struct EditorChunk {
    /// The chunk position in chunk coordinates
    pub chunk_pos: IVec3,
}

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

/// Resource to cache the chunk material (uses vertex colors)
#[derive(Resource)]
pub struct EditorChunkMaterial(pub Handle<StandardMaterial>);

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

/// Calculate color for a sub-voxel based on its position.
/// Uses the same hash-based coloring as the material palette for consistency.
#[inline]
fn get_sub_voxel_color(x: i32, y: i32, z: i32, sub_x: i32, sub_y: i32, sub_z: i32) -> Color {
    let index = VoxelMaterialPalette::get_material_index(x, y, z, sub_x, sub_y, sub_z);
    let t = index as f32 / VoxelMaterialPalette::PALETTE_SIZE as f32;
    Color::srgb(
        0.2 + t * 0.6,
        0.3 + ((t * std::f32::consts::PI * 2.0).sin() * 0.5 + 0.5) * 0.4,
        0.4 + ((t * std::f32::consts::PI * 3.0).cos() * 0.5 + 0.5) * 0.4,
    )
}

/// Calculate world position for a sub-voxel.
#[inline]
fn calculate_sub_voxel_pos(x: i32, y: i32, z: i32, sub_x: i32, sub_y: i32, sub_z: i32) -> Vec3 {
    let offset = -0.5 + SUB_VOXEL_SIZE * 0.5;
    Vec3::new(
        x as f32 + offset + (sub_x as f32 * SUB_VOXEL_SIZE),
        y as f32 + offset + (sub_y as f32 * SUB_VOXEL_SIZE),
        z as f32 + offset + (sub_z as f32 * SUB_VOXEL_SIZE),
    )
}

/// System to render the map when requested using optimized chunk-based meshing.
///
/// This uses the same optimizations as the game renderer (except LOD):
/// - Chunk-based meshing (Tier 3)
/// - Hidden face culling (Tier 4)
/// - Greedy meshing (Tier 5)
pub fn render_map_system(
    mut commands: Commands,
    mut render_events: EventReader<RenderMapEvent>,
    editor_state: Res<EditorState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_chunks: Query<Entity, With<EditorChunk>>,
    chunk_material_res: Option<Res<EditorChunkMaterial>>,
) {
    // Only render if we received an event
    if render_events.read().count() == 0 {
        return;
    }

    let total_voxels = editor_state.current_map.world.voxels.len();
    info!("Rendering map with {} voxels (optimized)", total_voxels);

    // Despawn all existing editor chunks
    for entity in existing_chunks.iter() {
        commands.entity(entity).despawn();
    }

    // Get or create chunk material (uses vertex colors)
    let chunk_material = if let Some(ref m) = chunk_material_res {
        m.0.clone()
    } else {
        let new_material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            ..default()
        });
        commands.insert_resource(EditorChunkMaterial(new_material.clone()));
        new_material
    };

    // Early return for empty maps
    if total_voxels == 0 {
        info!("Map rendering complete (empty map)");
        return;
    }

    // ========== TIER 4: Build Occupancy Grid for Hidden Face Culling ==========
    let mut occupancy = OccupancyGrid::new();

    // Collect all sub-voxel data for subsequent passes
    let mut all_sub_voxels: Vec<(i32, i32, i32, i32, i32, i32, Vec3, usize, Color)> = Vec::new();

    for voxel_data in &editor_state.current_map.world.voxels {
        let (x, y, z) = voxel_data.pos;
        let pattern = voxel_data.pattern.unwrap_or(SubVoxelPattern::Full);
        let geometry = pattern.geometry_with_rotation(voxel_data.rotation_state);

        for (sub_x, sub_y, sub_z) in geometry.occupied_positions() {
            // Add to occupancy grid for neighbor lookups
            occupancy.insert(x, y, z, sub_x, sub_y, sub_z);

            let world_pos = calculate_sub_voxel_pos(x, y, z, sub_x, sub_y, sub_z);
            let color_index =
                VoxelMaterialPalette::get_material_index(x, y, z, sub_x, sub_y, sub_z);
            let color = get_sub_voxel_color(x, y, z, sub_x, sub_y, sub_z);
            all_sub_voxels.push((x, y, z, sub_x, sub_y, sub_z, world_pos, color_index, color));
        }
    }

    // ========== TIER 3 & 5: Chunk-Based Meshing with Greedy Meshing ==========
    // Group visible faces into per-chunk greedy meshers
    let mut chunk_meshers: HashMap<IVec3, GreedyMesher> = HashMap::new();

    for (x, y, z, sub_x, sub_y, sub_z, world_pos, color_index, color) in all_sub_voxels {
        // Determine which chunk this sub-voxel belongs to
        let chunk_pos = IVec3::new(
            (world_pos.x / CHUNK_SIZE as f32).floor() as i32,
            (world_pos.y / CHUNK_SIZE as f32).floor() as i32,
            (world_pos.z / CHUNK_SIZE as f32).floor() as i32,
        );

        // Global sub-voxel coordinates for the greedy mesher
        let global_x = x * SUB_VOXEL_COUNT + sub_x;
        let global_y = y * SUB_VOXEL_COUNT + sub_y;
        let global_z = z * SUB_VOXEL_COUNT + sub_z;

        let mesher = chunk_meshers.entry(chunk_pos).or_default();

        // TIER 4: Check each face and add visible ones to the mesher
        let faces = [
            Face::PosX,
            Face::NegX,
            Face::PosY,
            Face::NegY,
            Face::PosZ,
            Face::NegZ,
        ];
        for face in faces {
            if !occupancy.has_neighbor(x, y, z, sub_x, sub_y, sub_z, face) {
                mesher.add_face(global_x, global_y, global_z, face, color_index, color);
            }
        }
    }

    // ========== Build Meshes and Spawn Chunks (Full Detail Only) ==========
    let total_chunks = chunk_meshers.len();
    let mut total_quads = 0usize;

    for (chunk_pos, mesher) in chunk_meshers {
        // Build full-detail mesh
        let mut builder = ChunkMeshBuilder::default();
        mesher.build_into(&mut builder);

        if builder.is_empty() {
            continue;
        }

        // Count quads for stats
        total_quads += builder.quad_count();

        // Create mesh and spawn chunk entity
        let mesh = meshes.add(builder.build());

        // Calculate chunk bounds for frustum culling
        // Chunks are positioned at their world coordinates, so AABB center is at chunk center
        let chunk_center = Vec3::new(
            (chunk_pos.x as f32 + 0.5) * CHUNK_SIZE as f32,
            (chunk_pos.y as f32 + 0.5) * CHUNK_SIZE as f32,
            (chunk_pos.z as f32 + 0.5) * CHUNK_SIZE as f32,
        );
        let half_extent = Vec3::splat(CHUNK_SIZE as f32 / 2.0);

        // Spawn chunk with explicit AABB for proper frustum culling
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(chunk_material.clone()),
            Transform::default(),
            EditorChunk { chunk_pos },
            // Explicit AABB enables Bevy's automatic frustum culling
            Aabb {
                center: Vec3A::from(chunk_center),
                half_extents: Vec3A::from(half_extent),
            },
            // Ensure visibility component is present for culling to work
            Visibility::default(),
        ));
    }

    info!(
        "Map rendering complete: {} chunks, {} quads (greedy meshing enabled)",
        total_chunks, total_quads
    );
}

/// Event sent when entities should be re-rendered
#[derive(Event)]
pub struct RenderEntitiesEvent;

/// System to render entity markers in the viewport
pub fn render_entities_system(
    mut commands: Commands,
    mut render_events: EventReader<RenderMapEvent>,
    mut selection_events: EventReader<UpdateSelectionHighlights>,
    editor_state: Res<EditorState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_markers: Query<Entity, With<EditorEntityMarker>>,
) {
    // Only render if we received a render event or selection changed
    let render_count = render_events.read().count();
    let selection_count = selection_events.read().count();
    if render_count == 0 && selection_count == 0 {
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
        // Snap to grid center for consistent alignment with voxels
        // Entity positions should be at integer coordinates (grid cell centers)
        let position = Vec3::new(x.round(), y.round(), z.round());

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

        // Spawn the entity marker with AABB for frustum culling
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(position),
            EditorEntityMarker {
                entity_index: index,
            },
            // AABB for frustum culling (sphere bounds)
            Aabb {
                center: Vec3A::ZERO, // Local space center
                half_extents: Vec3A::splat(size),
            },
            Visibility::default(),
        ));
    }

    info!("Entity marker rendering complete");
}
