//! Chunk mesh builder for constructing meshes from voxel geometry.

use super::{Face, SUB_VOXEL_SIZE};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;

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
