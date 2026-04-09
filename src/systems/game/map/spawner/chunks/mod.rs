//! Chunk-based voxel spawning with greedy meshing.

use super::super::super::components::SubVoxel;
use super::super::super::occlusion::{OcclusionMaterial, ShadowQuality};
use super::super::super::resources::SpatialGrid;
use super::super::format::{
    apply_orientation_matrix, world_dir_to_local, MapData, SubVoxelPattern,
};
use super::super::loader::{LoadProgress, MapLoadProgress};
use super::meshing::{ChunkMeshBuilder, GreedyMesher, OccupancyGrid, VoxelMaterialPalette};
use super::{ChunkLOD, Face, VoxelChunk, CHUNK_SIZE, LOD_LEVELS, SUB_VOXEL_COUNT, SUB_VOXEL_SIZE};
use bevy::camera::primitives::Aabb;
use bevy::light::NotShadowCaster;
use bevy::math::Vec3A;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

/// (voxel_x, voxel_y, voxel_z, sub_x, sub_y, sub_z, world_pos, color_index, color)
type SubVoxelEntry = (i32, i32, i32, i32, i32, i32, Vec3, usize, Color);

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
    /// Shadow quality applied at chunk spawn time (inserts `NotShadowCaster` for `CharactersOnly`).
    pub shadow_quality: ShadowQuality,
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
    let mut all_sub_voxels: Vec<SubVoxelEntry> = Vec::new();

    // Build a set of fence positions for neighbor lookups
    let fence_positions: HashSet<(i32, i32, i32)> = map
        .world
        .voxels
        .iter()
        .filter(|v| v.pattern.is_some_and(|p| p.is_fence()))
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

        // Determine which pattern to use
        let pattern = voxel_data.pattern.unwrap_or(SubVoxelPattern::Full);

        // For fence patterns, check neighbors and generate context-aware geometry
        let geometry = if pattern.is_fence() {
            // Look up this voxel's orientation once; used for both neighbour mapping and geometry.
            let orientation = voxel_data.rotation.and_then(|i| map.orientations.get(i));

            // World-axis neighbour directions paired with their neighbour positions.
            // Includes Y neighbours so that rotated fences (e.g. oriented as wall panels)
            // can connect to fences stacked above or below them.
            let world_dirs: [([i32; 3], (i32, i32, i32)); 6] = [
                ([-1, 0, 0], (x - 1, y, z)), // world −X
                ([1, 0, 0], (x + 1, y, z)),  // world +X
                ([0, 0, -1], (x, y, z - 1)), // world −Z
                ([0, 0, 1], (x, y, z + 1)),  // world +Z
                ([0, -1, 0], (x, y - 1, z)), // world −Y
                ([0, 1, 0], (x, y + 1, z)),  // world +Y
            ];

            // Map each world direction into the fence's local frame (Mᵀ × d).
            // fence_geometry_with_neighbors expects (neg_x, pos_x, neg_z, pos_z) in LOCAL space.
            let mut local_neg_x = false;
            let mut local_pos_x = false;
            let mut local_neg_z = false;
            let mut local_pos_z = false;

            for (world_dir, neighbor_pos) in &world_dirs {
                if fence_positions.contains(neighbor_pos) {
                    match world_dir_to_local(orientation, *world_dir) {
                        [-1, 0, 0] => local_neg_x = true,
                        [1, 0, 0] => local_pos_x = true,
                        [0, 0, -1] => local_neg_z = true,
                        [0, 0, 1] => local_pos_z = true,
                        _ => {} // diagonal or unexpected direction, ignore
                    }
                }
            }

            let fence_geo = pattern.fence_geometry_with_neighbors((
                local_neg_x,
                local_pos_x,
                local_neg_z,
                local_pos_z,
            ));
            // Rotate the locally-correct geometry into world space.
            if let Some(matrix) = orientation {
                apply_orientation_matrix(fence_geo, matrix)
            } else {
                fence_geo
            }
        } else {
            let orientation = voxel_data.rotation.and_then(|i| map.orientations.get(i));
            pattern.geometry_with_rotation(orientation)
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
        // Note: PosY (top) face is ALWAYS rendered regardless of neighbor,
        // because the interior occlusion system may hide voxels above,
        // and we need the floor to be visible when the ceiling is hidden.
        let faces = [
            Face::PosX,
            Face::NegX,
            Face::PosY,
            Face::NegY,
            Face::PosZ,
            Face::NegZ,
        ];
        for face in faces {
            // Always render top faces (PosY) to handle interior occlusion
            let should_render =
                face == Face::PosY || !occupancy.has_neighbor(x, y, z, sub_x, sub_y, sub_z, face);
            if should_render {
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
                let mut entity = ctx.commands.spawn((
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
                    Aabb {
                        center: Vec3A::from(chunk_center),
                        half_extents: Vec3A::splat(CHUNK_SIZE as f32 / 2.0),
                    },
                ));
                if ctx.shadow_quality == ShadowQuality::CharactersOnly {
                    entity.insert(NotShadowCaster);
                }
            }
            ChunkMaterial::Standard(mat) => {
                let mut entity = ctx.commands.spawn((
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
                    Aabb {
                        center: Vec3A::from(chunk_center),
                        half_extents: Vec3A::splat(CHUNK_SIZE as f32 / 2.0),
                    },
                ));
                if ctx.shadow_quality == ShadowQuality::CharactersOnly {
                    entity.insert(NotShadowCaster);
                }
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

#[cfg(test)]
mod tests;
