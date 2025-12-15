//! Chunk-based voxel spawning with greedy meshing.

use super::super::format::{MapData, SubVoxelPattern};
use super::super::loader::{LoadProgress, MapLoadProgress};
use super::super::super::components::{SubVoxel, Voxel};
use super::super::super::occlusion::OcclusionMaterial;
use super::super::super::resources::SpatialGrid;
use super::meshing::{ChunkMeshBuilder, GreedyMesher, OccupancyGrid, VoxelMaterialPalette};
use super::{ChunkLOD, Face, VoxelChunk, CHUNK_SIZE, LOD_LEVELS, SUB_VOXEL_COUNT, SUB_VOXEL_SIZE};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

/// Enum to hold either material type for chunk rendering
#[derive(Clone)]
pub enum ChunkMaterial {
    Occlusion(Handle<OcclusionMaterial>),
    Standard(Handle<StandardMaterial>),
}

/// Context for chunk-based voxel spawning.
pub struct ChunkSpawnContext<'w, 's, 'a> {
    pub commands: Commands<'w, 's>,
    pub spatial_grid: &'a mut SpatialGrid,
    pub meshes: &'a mut Assets<Mesh>,
    pub chunk_material: ChunkMaterial,
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

/// Spawn all voxels using chunk-based meshing with greedy face merging.
///
/// This function:
/// 1. First pass: Collects all occupied sub-voxel positions into an OccupancyGrid
/// 2. Second pass: For each sub-voxel, determine visible faces and add to GreedyMesher
/// 3. Third pass: Greedy mesher merges adjacent same-color faces into larger quads
/// 4. Spawns one entity per chunk with optimized mesh
///
/// Greedy meshing reduces quad count by 90%+ for large flat surfaces.
pub fn spawn_voxels_chunked(
    ctx: &mut ChunkSpawnContext,
    map: &MapData,
    progress: &mut MapLoadProgress,
) {
    let total_voxels = map.world.voxels.len();

    // First pass: Build occupancy grid for neighbor lookups
    progress.update(LoadProgress::SpawningVoxels(0.0));
    let mut occupancy = OccupancyGrid::new();

    // Collect all sub-voxel data for subsequent passes
    let mut all_sub_voxels: Vec<(i32, i32, i32, i32, i32, i32, Vec3, usize, Color)> = Vec::new();

    // Build a set of fence positions for neighbor lookups
    let fence_positions: HashSet<(i32, i32, i32)> = map
        .world
        .voxels
        .iter()
        .filter(|v| v.pattern.map_or(false, |p| p.is_fence()))
        .map(|v| v.pos)
        .collect();

    for (index, voxel_data) in map.world.voxels.iter().enumerate() {
        // Update progress (occupancy collection phase: 0-15%)
        if index % 100 == 0 {
            let voxel_progress = (index as f32) / (total_voxels as f32) * 0.15;
            progress.update(LoadProgress::SpawningVoxels(voxel_progress));
        }

        let (x, y, z) = voxel_data.pos;

        // Spawn parent voxel marker
        ctx.commands.spawn(Voxel);

        // Determine which pattern to use
        let pattern = voxel_data.pattern.unwrap_or(SubVoxelPattern::Full);

        // For fence patterns, check neighbors and generate context-aware geometry
        let geometry = if pattern.is_fence() {
            let neighbors = (
                fence_positions.contains(&(x - 1, y, z)), // neg_x
                fence_positions.contains(&(x + 1, y, z)), // pos_x
                fence_positions.contains(&(x, y, z - 1)), // neg_z
                fence_positions.contains(&(x, y, z + 1)), // pos_z
            );
            pattern.fence_geometry_with_neighbors(neighbors)
        } else {
            pattern.geometry_with_rotation(voxel_data.rotation_state)
        };

        // Add each sub-voxel to occupancy grid and collect data
        for (sub_x, sub_y, sub_z) in geometry.occupied_positions() {
            occupancy.insert(x, y, z, sub_x, sub_y, sub_z);

            let world_pos = calculate_sub_voxel_pos(x, y, z, sub_x, sub_y, sub_z);
            let color_index =
                VoxelMaterialPalette::get_material_index(x, y, z, sub_x, sub_y, sub_z);
            let color = get_sub_voxel_color(x, y, z, sub_x, sub_y, sub_z);
            all_sub_voxels.push((x, y, z, sub_x, sub_y, sub_z, world_pos, color_index, color));
        }
    }

    // Second pass: Collect visible faces into per-chunk greedy meshers
    let mut chunk_meshers: HashMap<IVec3, GreedyMesher> = HashMap::new();
    let mut sub_voxel_positions: Vec<(Vec3, (Vec3, Vec3))> = Vec::new();

    let total_sub_voxels_count = all_sub_voxels.len();
    for (index, (x, y, z, sub_x, sub_y, sub_z, world_pos, color_index, color)) in
        all_sub_voxels.into_iter().enumerate()
    {
        // Update progress (face collection phase: 15-35%)
        if index % 1000 == 0 {
            let build_progress = 0.15 + (index as f32) / (total_sub_voxels_count as f32) * 0.2;
            progress.update(LoadProgress::SpawningVoxels(build_progress));
        }

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

        // Check each face and add visible ones to the mesher
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

        // Calculate bounds for collision detection
        let half_size = SUB_VOXEL_SIZE / 2.0;
        let bounds = (
            world_pos - Vec3::splat(half_size),
            world_pos + Vec3::splat(half_size),
        );
        sub_voxel_positions.push((world_pos, bounds));
    }

    // Third pass: Build greedy meshes with LOD levels and spawn chunk entities
    let total_chunks = chunk_meshers.len();
    let mut total_quads = 0usize;

    for (index, (chunk_pos, mesher)) in chunk_meshers.into_iter().enumerate() {
        // Update progress (mesh building phase: 35-60%)
        let spawn_progress = 0.35 + (index as f32) / (total_chunks as f32) * 0.25;
        progress.update(LoadProgress::SpawningVoxels(spawn_progress));

        // Build LOD 0 (full detail) first to check if chunk has geometry
        let mut builder_lod0 = ChunkMeshBuilder::default();
        mesher.build_into(&mut builder_lod0);

        if builder_lod0.is_empty() {
            continue;
        }

        // Count quads for stats (LOD 0 only)
        total_quads += builder_lod0.positions.len() / 4;

        // Build all LOD levels
        let mut lod_meshes: [Handle<Mesh>; LOD_LEVELS] = Default::default();

        // LOD 0: Use already built full-detail mesh
        lod_meshes[0] = ctx.meshes.add(builder_lod0.build());

        // LOD 1-3: Build progressively lower detail meshes
        for lod_level in 1..LOD_LEVELS {
            let mut builder = ChunkMeshBuilder::default();
            mesher.build_lod(&mut builder, lod_level);

            // If LOD mesh is empty, reuse previous LOD
            if builder.is_empty() {
                lod_meshes[lod_level] = lod_meshes[lod_level - 1].clone();
            } else {
                lod_meshes[lod_level] = ctx.meshes.add(builder.build());
            }
        }

        // Calculate chunk center in world coordinates
        let chunk_center = Vec3::new(
            (chunk_pos.x as f32 + 0.5) * CHUNK_SIZE as f32,
            (chunk_pos.y as f32 + 0.5) * CHUNK_SIZE as f32,
            (chunk_pos.z as f32 + 0.5) * CHUNK_SIZE as f32,
        );

        // Spawn chunk with appropriate material type
        match &ctx.chunk_material {
            ChunkMaterial::Occlusion(mat) => {
                ctx.commands.spawn((
                    Mesh3d(lod_meshes[0].clone()),
                    MeshMaterial3d(mat.clone()),
                    Transform::default(),
                    VoxelChunk {
                        chunk_pos,
                        center: chunk_center,
                    },
                    ChunkLOD {
                        lod_meshes: lod_meshes.clone(),
                        current_lod: 0,
                    },
                ));
            }
            ChunkMaterial::Standard(mat) => {
                ctx.commands.spawn((
                    Mesh3d(lod_meshes[0].clone()),
                    MeshMaterial3d(mat.clone()),
                    Transform::default(),
                    VoxelChunk {
                        chunk_pos,
                        center: chunk_center,
                    },
                    ChunkLOD {
                        lod_meshes,
                        current_lod: 0,
                    },
                ));
            }
        }
    }

    // Spawn invisible collision entities for the spatial grid
    let total_sub_voxels = sub_voxel_positions.len();
    for (index, (world_pos, bounds)) in sub_voxel_positions.into_iter().enumerate() {
        // Update progress (collision setup phase: 60-100%)
        if index % 1000 == 0 {
            let collision_progress = 0.6 + (index as f32) / (total_sub_voxels as f32) * 0.4;
            progress.update(LoadProgress::SpawningVoxels(collision_progress));
        }

        // Spawn invisible entity for collision detection only
        let sub_voxel_entity = ctx.commands.spawn(SubVoxel { bounds }).id();

        // Add to spatial grid
        let grid_coords = SpatialGrid::world_to_grid_coords(world_pos);
        ctx.spatial_grid
            .cells
            .entry(grid_coords)
            .or_default()
            .push(sub_voxel_entity);
    }

    info!(
        "Spawned {} chunks with {} quads ({} collision entities) - greedy meshing enabled",
        total_chunks, total_quads, total_sub_voxels
    );
}
