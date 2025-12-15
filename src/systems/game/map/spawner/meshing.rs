//! Greedy meshing algorithm for chunk-based voxel rendering.
//!
//! This module implements greedy meshing which merges adjacent coplanar faces
//! of the same color into larger quads, dramatically reducing polygon count.

use super::{Face, SUB_VOXEL_COUNT, SUB_VOXEL_SIZE};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use std::collections::{HashMap, HashSet};

/// Occupancy grid for fast neighbor lookups during face culling.
/// Uses a HashSet of sub-voxel global coordinates.
pub struct OccupancyGrid {
    occupied: HashSet<(i32, i32, i32)>,
}

impl OccupancyGrid {
    pub fn new() -> Self {
        Self {
            occupied: HashSet::new(),
        }
    }

    /// Insert an occupied position (voxel coords + sub-voxel coords combined into global sub-voxel coords).
    #[inline]
    pub fn insert(&mut self, x: i32, y: i32, z: i32, sub_x: i32, sub_y: i32, sub_z: i32) {
        // Convert to global sub-voxel coordinates
        let global_x = x * SUB_VOXEL_COUNT + sub_x;
        let global_y = y * SUB_VOXEL_COUNT + sub_y;
        let global_z = z * SUB_VOXEL_COUNT + sub_z;
        self.occupied.insert((global_x, global_y, global_z));
    }

    /// Check if a neighbor exists in the given direction.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn has_neighbor(
        &self,
        x: i32,
        y: i32,
        z: i32,
        sub_x: i32,
        sub_y: i32,
        sub_z: i32,
        face: Face,
    ) -> bool {
        let (dx, dy, dz) = face.offset();
        let global_x = x * SUB_VOXEL_COUNT + sub_x + dx;
        let global_y = y * SUB_VOXEL_COUNT + sub_y + dy;
        let global_z = z * SUB_VOXEL_COUNT + sub_z + dz;
        self.occupied.contains(&(global_x, global_y, global_z))
    }
}

impl Default for OccupancyGrid {
    fn default() -> Self {
        Self::new()
    }
}

/// A visible face with position and color information for greedy meshing.
#[derive(Clone, Copy)]
#[allow(dead_code)]
struct VisibleFace {
    /// Global sub-voxel coordinates
    global_x: i32,
    global_y: i32,
    global_z: i32,
    /// Color index (from palette hash) for merging same-color faces
    color_index: usize,
    /// The actual color
    color: Color,
}

/// Greedy mesher that merges adjacent coplanar faces of the same color.
///
/// Algorithm:
/// 1. For each face direction, collect all visible faces into 2D slices
/// 2. For each slice, find maximal rectangles of same-color faces
/// 3. Emit single quads for merged rectangles instead of individual faces
#[derive(Default)]
pub struct GreedyMesher {
    /// Faces grouped by direction and then by slice depth
    /// Key: (Face direction, slice depth) -> Vec of (u, v, color_index, color)
    /// where u,v are the 2D coordinates within the slice
    slices: HashMap<(Face, i32), Vec<(i32, i32, usize, Color)>>,
}

impl GreedyMesher {
    pub fn new() -> Self {
        Self {
            slices: HashMap::new(),
        }
    }

    /// Add a visible face to the mesher.
    #[inline]
    pub fn add_face(
        &mut self,
        global_x: i32,
        global_y: i32,
        global_z: i32,
        face: Face,
        color_index: usize,
        color: Color,
    ) {
        // Convert 3D coordinates to 2D slice coordinates based on face direction
        let (depth, u, v) = match face {
            Face::PosX | Face::NegX => (global_x, global_y, global_z),
            Face::PosY | Face::NegY => (global_y, global_x, global_z),
            Face::PosZ | Face::NegZ => (global_z, global_x, global_y),
        };

        self.slices
            .entry((face, depth))
            .or_default()
            .push((u, v, color_index, color));
    }

    /// Build greedy-meshed geometry into a ChunkMeshBuilder (full detail, LOD 0).
    pub fn build_into(&self, builder: &mut ChunkMeshBuilder) {
        for ((face, depth), faces) in &self.slices {
            Self::mesh_slice(builder, *face, *depth, faces);
        }
    }

    /// Mesh a single 2D slice using greedy meshing.
    fn mesh_slice(
        builder: &mut ChunkMeshBuilder,
        face: Face,
        depth: i32,
        faces: &[(i32, i32, usize, Color)],
    ) {
        if faces.is_empty() {
            return;
        }

        // Build a 2D grid of the slice
        // Key: (u, v) -> (color_index, color, used)
        let mut grid: HashMap<(i32, i32), (usize, Color, bool)> = HashMap::new();
        for (u, v, color_index, color) in faces {
            grid.insert((*u, *v), (*color_index, *color, false));
        }

        // Find all unique (u, v) positions and sort them for consistent iteration
        let mut positions: Vec<(i32, i32)> = grid.keys().copied().collect();
        positions.sort();

        // Greedy rectangle finding
        for (start_u, start_v) in positions {
            // Skip if already used
            if let Some((_, _, used)) = grid.get(&(start_u, start_v)) {
                if *used {
                    continue;
                }
            } else {
                continue;
            }

            let (color_index, color, _) = *grid.get(&(start_u, start_v)).unwrap();

            // Expand in U direction as far as possible with same color
            let mut end_u = start_u;
            while let Some((ci, _, used)) = grid.get(&(end_u + 1, start_v)) {
                if *used || *ci != color_index {
                    break;
                }
                end_u += 1;
            }

            // Expand in V direction as far as possible, checking entire U span has same color
            let mut end_v = start_v;
            'v_expand: loop {
                let next_v = end_v + 1;
                // Check all cells in the U span at next_v
                for u in start_u..=end_u {
                    match grid.get(&(u, next_v)) {
                        Some((ci, _, used)) if !*used && *ci == color_index => {}
                        _ => break 'v_expand,
                    }
                }
                end_v = next_v;
            }

            // Mark all cells in the rectangle as used
            for u in start_u..=end_u {
                for v in start_v..=end_v {
                    if let Some(cell) = grid.get_mut(&(u, v)) {
                        cell.2 = true;
                    }
                }
            }

            // Emit the merged quad
            let width = (end_u - start_u + 1) as f32 * SUB_VOXEL_SIZE;
            let height = (end_v - start_v + 1) as f32 * SUB_VOXEL_SIZE;

            // Calculate center position of the merged quad
            let (center_x, center_y, center_z) = Self::slice_to_world(
                face,
                depth,
                start_u as f32 + (end_u - start_u) as f32 / 2.0,
                start_v as f32 + (end_v - start_v) as f32 / 2.0,
            );

            let center = Vec3::new(center_x, center_y, center_z);
            builder.add_quad(center, face, width, height, color);
        }
    }

    /// Convert slice coordinates back to world coordinates.
    #[inline]
    fn slice_to_world(face: Face, depth: i32, u: f32, v: f32) -> (f32, f32, f32) {
        let offset = -0.5 + SUB_VOXEL_SIZE * 0.5;
        let d = depth as f32 / SUB_VOXEL_COUNT as f32 + offset;
        let u_world = u / SUB_VOXEL_COUNT as f32 + offset;
        let v_world = v / SUB_VOXEL_COUNT as f32 + offset;

        match face {
            Face::PosX => (d + SUB_VOXEL_SIZE / 2.0, u_world, v_world),
            Face::NegX => (d - SUB_VOXEL_SIZE / 2.0, u_world, v_world),
            Face::PosY => (u_world, d + SUB_VOXEL_SIZE / 2.0, v_world),
            Face::NegY => (u_world, d - SUB_VOXEL_SIZE / 2.0, v_world),
            Face::PosZ => (u_world, v_world, d + SUB_VOXEL_SIZE / 2.0),
            Face::NegZ => (u_world, v_world, d - SUB_VOXEL_SIZE / 2.0),
        }
    }

    /// Build greedy-meshed geometry at a specific LOD level.
    /// LOD 0 = full detail, LOD 1 = 1/2 resolution, LOD 2 = 1/4, LOD 3 = 1/8
    pub fn build_lod(&self, builder: &mut ChunkMeshBuilder, lod_level: usize) {
        let sample_rate = 1 << lod_level; // 1, 2, 4, 8 for LOD 0-3
        let lod_voxel_size = SUB_VOXEL_SIZE * sample_rate as f32;

        for ((face, depth), faces) in &self.slices {
            // Only process slices at LOD intervals
            if depth % sample_rate != 0 {
                continue;
            }

            Self::mesh_slice_lod(builder, *face, *depth, faces, sample_rate, lod_voxel_size);
        }
    }

    /// Mesh a single 2D slice at a specific LOD level.
    fn mesh_slice_lod(
        builder: &mut ChunkMeshBuilder,
        face: Face,
        depth: i32,
        faces: &[(i32, i32, usize, Color)],
        sample_rate: i32,
        lod_voxel_size: f32,
    ) {
        if faces.is_empty() {
            return;
        }

        // Downsample: group faces into LOD cells
        // Key: (lod_u, lod_v) -> (color_index, color, used)
        let mut grid: HashMap<(i32, i32), (usize, Color, bool)> = HashMap::new();

        for (u, v, color_index, color) in faces {
            let lod_u = u / sample_rate;
            let lod_v = v / sample_rate;

            // First face in each LOD cell determines color
            grid.entry((lod_u, lod_v))
                .or_insert((*color_index, *color, false));
        }

        // Find all unique positions and sort them
        let mut positions: Vec<(i32, i32)> = grid.keys().copied().collect();
        positions.sort();

        // Greedy rectangle finding (same algorithm as full-res)
        for (start_u, start_v) in positions {
            if let Some((_, _, used)) = grid.get(&(start_u, start_v)) {
                if *used {
                    continue;
                }
            } else {
                continue;
            }

            let (color_index, color, _) = *grid.get(&(start_u, start_v)).unwrap();

            // Expand in U direction
            let mut end_u = start_u;
            while let Some((ci, _, used)) = grid.get(&(end_u + 1, start_v)) {
                if *used || *ci != color_index {
                    break;
                }
                end_u += 1;
            }

            // Expand in V direction
            let mut end_v = start_v;
            'v_expand: loop {
                let next_v = end_v + 1;
                for u in start_u..=end_u {
                    match grid.get(&(u, next_v)) {
                        Some((ci, _, used)) if !*used && *ci == color_index => {}
                        _ => break 'v_expand,
                    }
                }
                end_v = next_v;
            }

            // Mark as used
            for u in start_u..=end_u {
                for v in start_v..=end_v {
                    if let Some(cell) = grid.get_mut(&(u, v)) {
                        cell.2 = true;
                    }
                }
            }

            // Emit the merged quad at LOD scale
            let width = (end_u - start_u + 1) as f32 * lod_voxel_size;
            let height = (end_v - start_v + 1) as f32 * lod_voxel_size;

            // Calculate center position using LOD-scaled coordinates
            let (center_x, center_y, center_z) = Self::slice_to_world_lod(
                face,
                depth,
                start_u as f32 * sample_rate as f32
                    + (end_u - start_u) as f32 * sample_rate as f32 / 2.0,
                start_v as f32 * sample_rate as f32
                    + (end_v - start_v) as f32 * sample_rate as f32 / 2.0,
            );

            let center = Vec3::new(center_x, center_y, center_z);
            builder.add_quad(center, face, width, height, color);
        }
    }

    /// Convert slice coordinates back to world coordinates (for LOD meshes).
    #[inline]
    fn slice_to_world_lod(face: Face, depth: i32, u: f32, v: f32) -> (f32, f32, f32) {
        let offset = -0.5 + SUB_VOXEL_SIZE * 0.5;
        let d = depth as f32 / SUB_VOXEL_COUNT as f32 + offset;
        let u_world = u / SUB_VOXEL_COUNT as f32 + offset;
        let v_world = v / SUB_VOXEL_COUNT as f32 + offset;

        match face {
            Face::PosX => (d + SUB_VOXEL_SIZE / 2.0, u_world, v_world),
            Face::NegX => (d - SUB_VOXEL_SIZE / 2.0, u_world, v_world),
            Face::PosY => (u_world, d + SUB_VOXEL_SIZE / 2.0, v_world),
            Face::NegY => (u_world, d - SUB_VOXEL_SIZE / 2.0, v_world),
            Face::PosZ => (u_world, v_world, d + SUB_VOXEL_SIZE / 2.0),
            Face::NegZ => (u_world, v_world, d - SUB_VOXEL_SIZE / 2.0),
        }
    }
}

/// Builder for constructing chunk meshes from multiple cubes.
/// Combines all sub-voxels in a chunk into a single mesh with vertex colors.
/// Supports hidden face culling to reduce triangle count.
#[derive(Default)]
pub struct ChunkMeshBuilder {
    pub positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl ChunkMeshBuilder {
    /// Add a single face to the mesh (uniform size).
    #[inline]
    #[allow(dead_code)]
    pub fn add_face(&mut self, position: Vec3, size: f32, face: Face, color: Color) {
        self.add_quad(position, face, size, size, color);
    }

    /// Add a quad with variable width and height for greedy meshing.
    /// Width is in the first axis of the face plane, height is in the second.
    #[inline]
    pub fn add_quad(&mut self, position: Vec3, face: Face, width: f32, height: f32, color: Color) {
        let half_w = width / 2.0;
        let half_h = height / 2.0;
        let base_index = self.positions.len() as u32;
        let color_array = color.to_linear().to_f32_array();
        let normal = face.normal();

        // Generate 4 vertices for the quad based on face direction
        // Width and height map differently based on face orientation
        let vertices: [[f32; 3]; 4] = match face {
            Face::PosZ => [
                [position.x - half_w, position.y - half_h, position.z],
                [position.x + half_w, position.y - half_h, position.z],
                [position.x + half_w, position.y + half_h, position.z],
                [position.x - half_w, position.y + half_h, position.z],
            ],
            Face::NegZ => [
                [position.x + half_w, position.y - half_h, position.z],
                [position.x - half_w, position.y - half_h, position.z],
                [position.x - half_w, position.y + half_h, position.z],
                [position.x + half_w, position.y + half_h, position.z],
            ],
            Face::PosX => [
                [position.x, position.y - half_w, position.z + half_h],
                [position.x, position.y - half_w, position.z - half_h],
                [position.x, position.y + half_w, position.z - half_h],
                [position.x, position.y + half_w, position.z + half_h],
            ],
            Face::NegX => [
                [position.x, position.y - half_w, position.z - half_h],
                [position.x, position.y - half_w, position.z + half_h],
                [position.x, position.y + half_w, position.z + half_h],
                [position.x, position.y + half_w, position.z - half_h],
            ],
            Face::PosY => [
                [position.x - half_w, position.y, position.z + half_h],
                [position.x + half_w, position.y, position.z + half_h],
                [position.x + half_w, position.y, position.z - half_h],
                [position.x - half_w, position.y, position.z - half_h],
            ],
            Face::NegY => [
                [position.x - half_w, position.y, position.z - half_h],
                [position.x + half_w, position.y, position.z - half_h],
                [position.x + half_w, position.y, position.z + half_h],
                [position.x - half_w, position.y, position.z + half_h],
            ],
        };

        self.positions.extend_from_slice(&vertices);
        self.normals.extend_from_slice(&[normal; 4]);

        // Scale UVs based on quad dimensions for proper texture tiling
        let uv_scale_w = width / SUB_VOXEL_SIZE;
        let uv_scale_h = height / SUB_VOXEL_SIZE;
        self.uvs.extend_from_slice(&[
            [0.0, uv_scale_h],
            [uv_scale_w, uv_scale_h],
            [uv_scale_w, 0.0],
            [0.0, 0.0],
        ]);

        for _ in 0..4 {
            self.colors.push(color_array);
        }

        // Two triangles for the quad
        self.indices.extend_from_slice(&[
            base_index,
            base_index + 1,
            base_index + 2,
            base_index + 2,
            base_index + 3,
            base_index,
        ]);
    }

    /// Add a cube with hidden face culling.
    /// Only adds faces where the neighbor array indicates no adjacent sub-voxel.
    /// neighbors order: [+X, -X, +Y, -Y, +Z, -Z]
    #[inline]
    #[allow(dead_code)]
    pub fn add_cube_culled(
        &mut self,
        position: Vec3,
        size: f32,
        color: Color,
        neighbors: [bool; 6],
    ) {
        if !neighbors[0] {
            self.add_face(position, size, Face::PosX, color);
        }
        if !neighbors[1] {
            self.add_face(position, size, Face::NegX, color);
        }
        if !neighbors[2] {
            self.add_face(position, size, Face::PosY, color);
        }
        if !neighbors[3] {
            self.add_face(position, size, Face::NegY, color);
        }
        if !neighbors[4] {
            self.add_face(position, size, Face::PosZ, color);
        }
        if !neighbors[5] {
            self.add_face(position, size, Face::NegZ, color);
        }
    }

    /// Add a cube to the mesh at the given position with the given color (no culling).
    #[allow(dead_code)]
    pub fn add_cube(&mut self, position: Vec3, size: f32, color: Color) {
        let half = size / 2.0;
        let base_index = self.positions.len() as u32;

        // Convert color to linear RGBA
        let color_array = color.to_linear().to_f32_array();

        // 8 vertices of the cube
        let vertices = [
            // Front face (facing +Z)
            [position.x - half, position.y - half, position.z + half], // 0: bottom-left
            [position.x + half, position.y - half, position.z + half], // 1: bottom-right
            [position.x + half, position.y + half, position.z + half], // 2: top-right
            [position.x - half, position.y + half, position.z + half], // 3: top-left
            // Back face (facing -Z)
            [position.x + half, position.y - half, position.z - half], // 4: bottom-left
            [position.x - half, position.y - half, position.z - half], // 5: bottom-right
            [position.x - half, position.y + half, position.z - half], // 6: top-right
            [position.x + half, position.y + half, position.z - half], // 7: top-left
            // Right face (facing +X)
            [position.x + half, position.y - half, position.z + half], // 8
            [position.x + half, position.y - half, position.z - half], // 9
            [position.x + half, position.y + half, position.z - half], // 10
            [position.x + half, position.y + half, position.z + half], // 11
            // Left face (facing -X)
            [position.x - half, position.y - half, position.z - half], // 12
            [position.x - half, position.y - half, position.z + half], // 13
            [position.x - half, position.y + half, position.z + half], // 14
            [position.x - half, position.y + half, position.z - half], // 15
            // Top face (facing +Y)
            [position.x - half, position.y + half, position.z + half], // 16
            [position.x + half, position.y + half, position.z + half], // 17
            [position.x + half, position.y + half, position.z - half], // 18
            [position.x - half, position.y + half, position.z - half], // 19
            // Bottom face (facing -Y)
            [position.x - half, position.y - half, position.z - half], // 20
            [position.x + half, position.y - half, position.z - half], // 21
            [position.x + half, position.y - half, position.z + half], // 22
            [position.x - half, position.y - half, position.z + half], // 23
        ];

        let normals = [
            // Front face
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            // Back face
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            // Right face
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            // Left face
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            // Top face
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            // Bottom face
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
        ];

        let uvs = [
            // Front
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0],
            [0.0, 0.0],
            // Back
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0],
            [0.0, 0.0],
            // Right
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0],
            [0.0, 0.0],
            // Left
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0],
            [0.0, 0.0],
            // Top
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0],
            [0.0, 0.0],
            // Bottom
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0],
            [0.0, 0.0],
        ];

        // Add vertices
        self.positions.extend_from_slice(&vertices);
        self.normals.extend_from_slice(&normals);
        self.uvs.extend_from_slice(&uvs);

        // Add colors for all 24 vertices
        for _ in 0..24 {
            self.colors.push(color_array);
        }

        // Add indices for 6 faces (2 triangles each = 36 indices)
        #[rustfmt::skip]
        let face_indices: [u32; 36] = [
            // Front
            0, 1, 2, 2, 3, 0,
            // Back
            4, 5, 6, 6, 7, 4,
            // Right
            8, 9, 10, 10, 11, 8,
            // Left
            12, 13, 14, 14, 15, 12,
            // Top
            16, 17, 18, 18, 19, 16,
            // Bottom
            20, 21, 22, 22, 23, 20,
        ];

        for idx in face_indices {
            self.indices.push(base_index + idx);
        }
    }

    /// Build the final mesh from accumulated geometry.
    pub fn build(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_COLOR,
            VertexAttributeValues::Float32x4(self.colors),
        );
        mesh.insert_indices(Indices::U32(self.indices));

        mesh
    }

    /// Check if the builder has any geometry.
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// Get the number of quads in the builder (for statistics).
    /// Each quad has 4 vertices.
    pub fn quad_count(&self) -> usize {
        self.positions.len() / 4
    }
}

/// Pre-generated material palette for efficient voxel rendering.
///
/// Instead of creating a unique material for each sub-voxel (which would be millions
/// of materials in a large world), we pre-create a fixed palette and hash positions
/// to palette indices. This enables GPU batching and reduces memory usage by 99.99%.
#[derive(Resource, Clone)]
pub struct VoxelMaterialPalette {
    pub materials: Vec<Handle<StandardMaterial>>,
}

impl VoxelMaterialPalette {
    /// Number of unique materials in the palette.
    /// 64 provides good visual variety while keeping material count low.
    pub const PALETTE_SIZE: usize = 64;

    /// Create a new material palette with pre-generated colors.
    pub fn new(materials_asset: &mut Assets<StandardMaterial>) -> Self {
        let materials: Vec<_> = (0..Self::PALETTE_SIZE)
            .map(|i| {
                let t = i as f32 / Self::PALETTE_SIZE as f32;
                // Generate visually distinct colors across the palette
                let color = Color::srgb(
                    0.2 + t * 0.6,
                    0.3 + ((t * std::f32::consts::PI * 2.0).sin() * 0.5 + 0.5) * 0.4,
                    0.4 + ((t * std::f32::consts::PI * 3.0).cos() * 0.5 + 0.5) * 0.4,
                );
                materials_asset.add(color)
            })
            .collect();

        Self { materials }
    }

    /// Get material index for a sub-voxel position using spatial hashing.
    /// Uses prime number multiplication for good hash distribution.
    #[inline]
    pub fn get_material_index(x: i32, y: i32, z: i32, sub_x: i32, sub_y: i32, sub_z: i32) -> usize {
        // Spatial hash using prime numbers for good distribution
        let hash = (x.wrapping_mul(73856093))
            ^ (y.wrapping_mul(19349663))
            ^ (z.wrapping_mul(83492791))
            ^ (sub_x.wrapping_mul(15485863))
            ^ (sub_y.wrapping_mul(32452843))
            ^ (sub_z.wrapping_mul(49979687));
        (hash.unsigned_abs() as usize) % Self::PALETTE_SIZE
    }

    /// Get a material handle for the given sub-voxel position.
    #[inline]
    pub fn get_material(
        &self,
        x: i32,
        y: i32,
        z: i32,
        sub_x: i32,
        sub_y: i32,
        sub_z: i32,
    ) -> Handle<StandardMaterial> {
        let index = Self::get_material_index(x, y, z, sub_x, sub_y, sub_z);
        self.materials[index].clone()
    }
}
