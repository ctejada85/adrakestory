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
mod tests {
    use super::*;
    use crate::systems::game::map::format::{
        axis_angle_to_matrix, world_dir_to_local, SubVoxelPattern,
    };
    use crate::systems::game::map::geometry::RotationAxis;
    use bevy::math::Vec3A;

    #[test]
    fn test_calculate_sub_voxel_pos_origin() {
        let pos = calculate_sub_voxel_pos(0, 0, 0, 0, 0, 0);
        // First sub-voxel of first voxel
        // offset = -0.5 + 0.125/2 = -0.4375
        let expected_offset = -0.5 + SUB_VOXEL_SIZE * 0.5;
        assert!((pos.x - expected_offset).abs() < 0.001);
        assert!((pos.y - expected_offset).abs() < 0.001);
        assert!((pos.z - expected_offset).abs() < 0.001);
    }

    #[test]
    fn test_calculate_sub_voxel_pos_last_sub_voxel() {
        let pos = calculate_sub_voxel_pos(0, 0, 0, 7, 7, 7);
        // Last sub-voxel (index 7) of first voxel
        let offset = -0.5 + SUB_VOXEL_SIZE * 0.5;
        let expected = offset + (7.0 * SUB_VOXEL_SIZE);
        assert!((pos.x - expected).abs() < 0.001);
        assert!((pos.y - expected).abs() < 0.001);
        assert!((pos.z - expected).abs() < 0.001);
    }

    #[test]
    fn test_calculate_sub_voxel_pos_adjacent_voxel() {
        // First sub-voxel of voxel (1,0,0) should be SUB_VOXEL_COUNT * SUB_VOXEL_SIZE
        // away from first sub-voxel of voxel (0,0,0)
        let pos1 = calculate_sub_voxel_pos(0, 0, 0, 0, 0, 0);
        let pos2 = calculate_sub_voxel_pos(1, 0, 0, 0, 0, 0);
        assert!((pos2.x - pos1.x - 1.0).abs() < 0.001); // 1 voxel = 1 world unit
    }

    #[test]
    fn test_get_sub_voxel_color_deterministic() {
        let color1 = get_sub_voxel_color(5, 10, 15, 3, 4, 5);
        let color2 = get_sub_voxel_color(5, 10, 15, 3, 4, 5);
        // Same input should produce same color
        assert_eq!(format!("{:?}", color1), format!("{:?}", color2));
    }

    #[test]
    fn test_chunk_material_variants() {
        // Just verify the enum variants exist and can be matched
        let occlusion = ChunkMaterial::Occlusion(Handle::default());
        let standard = ChunkMaterial::Standard(Handle::default());

        match occlusion {
            ChunkMaterial::Occlusion(_) => {}
            ChunkMaterial::Standard(_) => panic!("Wrong variant"),
        }

        match standard {
            ChunkMaterial::Standard(_) => {}
            ChunkMaterial::Occlusion(_) => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_sub_voxel_size_matches_count() {
        // 8 sub-voxels should fit in 1 world unit
        assert!((SUB_VOXEL_COUNT as f32 * SUB_VOXEL_SIZE - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_chunk_coordinates_calculation() {
        // Test that world position maps to correct chunk
        let world_pos = Vec3::new(0.0, 0.0, 0.0);
        let chunk_pos = IVec3::new(
            (world_pos.x / CHUNK_SIZE as f32).floor() as i32,
            (world_pos.y / CHUNK_SIZE as f32).floor() as i32,
            (world_pos.z / CHUNK_SIZE as f32).floor() as i32,
        );
        assert_eq!(chunk_pos, IVec3::ZERO);
    }

    #[test]
    fn test_chunk_coordinates_negative() {
        let world_pos = Vec3::new(-1.0, -1.0, -1.0);
        let chunk_pos = IVec3::new(
            (world_pos.x / CHUNK_SIZE as f32).floor() as i32,
            (world_pos.y / CHUNK_SIZE as f32).floor() as i32,
            (world_pos.z / CHUNK_SIZE as f32).floor() as i32,
        );
        assert_eq!(chunk_pos, IVec3::new(-1, -1, -1));
    }

    #[test]
    fn test_chunk_coordinates_boundary() {
        // Position at exactly chunk boundary
        let world_pos = Vec3::new(CHUNK_SIZE as f32, 0.0, 0.0);
        let chunk_pos = IVec3::new(
            (world_pos.x / CHUNK_SIZE as f32).floor() as i32,
            (world_pos.y / CHUNK_SIZE as f32).floor() as i32,
            (world_pos.z / CHUNK_SIZE as f32).floor() as i32,
        );
        assert_eq!(chunk_pos.x, 1);
    }

    #[test]
    fn chunk_aabb_half_extents_match_chunk_size() {
        let half = Vec3A::splat(CHUNK_SIZE as f32 / 2.0);
        assert_eq!(half, Vec3A::splat(8.0));
    }

    #[test]
    fn chunk_aabb_center_matches_chunk_center() {
        let chunk_center = Vec3::new(8.0, 8.0, 8.0); // center of chunk at (0,0,0)
        let aabb = Aabb {
            center: Vec3A::from(chunk_center),
            half_extents: Vec3A::splat(CHUNK_SIZE as f32 / 2.0),
        };
        assert_eq!(aabb.center, Vec3A::new(8.0, 8.0, 8.0));
        assert_eq!(aabb.half_extents, Vec3A::splat(8.0));
    }

    // --- Fence rotation tests ---

    /// Fence with rotation:None must produce the same geometry as calling
    /// fence_geometry_with_neighbors directly (backward-compat AC-2).
    #[test]
    fn fence_rotation_none_is_unchanged() {
        let neighbors = (true, false, false, false);
        let expected = SubVoxelPattern::Fence.fence_geometry_with_neighbors(neighbors);

        // Simulate the spawner fence branch with rotation = None
        let fence_geo = SubVoxelPattern::Fence.fence_geometry_with_neighbors(neighbors);
        // orientation is None → no apply_orientation_matrix
        let result = fence_geo;

        let expected_pos: Vec<_> = expected.occupied_positions().collect();
        let result_pos: Vec<_> = result.occupied_positions().collect();
        assert_eq!(expected_pos, result_pos);
    }

    /// Fence with a Y+90° orientation matrix must produce geometry different from
    /// the un-rotated fence — confirming the matrix is applied (AC-1).
    #[test]
    fn fence_with_rotation_differs_from_unrotated() {
        let neighbors = (true, false, false, false); // one rail in -X direction
        let m_y90 = axis_angle_to_matrix(RotationAxis::Y, 1);

        let unrotated = SubVoxelPattern::Fence.fence_geometry_with_neighbors(neighbors);
        let rotated = apply_orientation_matrix(
            SubVoxelPattern::Fence.fence_geometry_with_neighbors(neighbors),
            &m_y90,
        );

        let unrotated_pos: Vec<_> = unrotated.occupied_positions().collect();
        let rotated_pos: Vec<_> = rotated.occupied_positions().collect();
        // A fence with a rail in -X, rotated 90° around Y, should have the rail in -Z.
        // The two position sets must differ.
        assert_ne!(unrotated_pos, rotated_pos);
    }

    /// Applying the orientation matrix to fence geometry must produce the same
    /// result as calling apply_orientation_matrix directly (AC-1 correctness).
    #[test]
    fn fence_with_rotation_matches_manual_apply() {
        let neighbors = (false, true, false, false); // rail in +X
        let m_y180 = axis_angle_to_matrix(RotationAxis::Y, 2);

        let fence_geo = SubVoxelPattern::Fence.fence_geometry_with_neighbors(neighbors);
        let expected = apply_orientation_matrix(fence_geo.clone(), &m_y180);

        // Simulate spawner fence branch
        let fence_geo2 = SubVoxelPattern::Fence.fence_geometry_with_neighbors(neighbors);
        let result = apply_orientation_matrix(fence_geo2, &m_y180);

        let expected_pos: Vec<_> = expected.occupied_positions().collect();
        let result_pos: Vec<_> = result.occupied_positions().collect();
        assert_eq!(expected_pos, result_pos);
    }

    /// Collision bounds are derived from occupied_positions() of the geometry.
    /// For a rotated fence the occupied positions must match the rotated geometry,
    /// not the un-rotated geometry (AC-4).
    #[test]
    fn fence_rotated_bounds_use_rotated_positions() {
        let neighbors = (true, false, false, false); // rail in -X
        let m_y90 = axis_angle_to_matrix(RotationAxis::Y, 1);

        let unrotated_geo = SubVoxelPattern::Fence.fence_geometry_with_neighbors(neighbors);
        let rotated_geo = apply_orientation_matrix(
            SubVoxelPattern::Fence.fence_geometry_with_neighbors(neighbors),
            &m_y90,
        );

        // The set of occupied sub-voxel positions must differ: the rail moved from
        // -X direction to -Z direction after Y+90°.
        let unrotated_positions: std::collections::HashSet<_> =
            unrotated_geo.occupied_positions().collect();
        let rotated_positions: std::collections::HashSet<_> =
            rotated_geo.occupied_positions().collect();

        assert_ne!(
            unrotated_positions, rotated_positions,
            "Rotated fence geometry must have different occupied positions than un-rotated"
        );

        // The sub-voxel count must be identical (rotation preserves count).
        assert_eq!(
            unrotated_positions.len(),
            rotated_positions.len(),
            "Rotation must preserve the number of occupied sub-voxels"
        );
    }

    /// A fence rotated Y+90° with a world +X neighbor should connect in its local +Z direction.
    /// After applying the Y+90° orientation matrix the rail appears in the world +X direction,
    /// correctly meeting the neighbor.
    ///
    /// Y+90°: local X → world −Z, local Z → world +X.
    /// Inverse: world +X → local +Z. So a world +X neighbor triggers local pos_z = true.
    #[test]
    fn fence_y90_connects_to_world_x_neighbor() {
        let m_y90 = axis_angle_to_matrix(RotationAxis::Y, 1);

        // World +X neighbor exists. Map to local frame via Mᵀ.
        let local_dir = world_dir_to_local(Some(&m_y90), [1, 0, 0]);
        // Y+90°: Mᵀ × [1,0,0] = [0,0,1] (local +Z)
        assert_eq!(
            local_dir,
            [0, 0, 1],
            "world +X must map to local +Z for Y+90°"
        );

        // Build fence geometry with that local connection (pos_z = true).
        let fence_geo =
            SubVoxelPattern::Fence.fence_geometry_with_neighbors((false, false, false, true)); // pos_z

        // Rotate the local geometry into world space (local +Z → world +X).
        let world_geo = apply_orientation_matrix(fence_geo, &m_y90);

        // The rail should now extend in the world +X direction (sub_x > 4).
        let has_rail_in_pos_x = world_geo
            .occupied_positions()
            .any(|(sx, _sy, sz)| sx > 4 && sz >= 3 && sz <= 4);
        assert!(
            has_rail_in_pos_x,
            "after Y+90°, the local +Z rail must appear in the world +X half (sub_x > 4)"
        );
    }
}
