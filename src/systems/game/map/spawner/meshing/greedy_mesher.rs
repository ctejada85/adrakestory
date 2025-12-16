//! Greedy meshing algorithm for merging adjacent coplanar faces.

use super::{mesh_builder::ChunkMeshBuilder, Face, SUB_VOXEL_COUNT, SUB_VOXEL_SIZE};
use bevy::prelude::*;
use std::collections::HashMap;

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
