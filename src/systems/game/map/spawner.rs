//! Map spawning system that instantiates loaded map data into the game world.

use super::super::character::CharacterModel;
use super::super::components::{CollisionBox, GameCamera, Player, SubVoxel, Voxel};
use super::super::resources::{GameInitialized, SpatialGrid};
use super::format::{EntityType, MapData, SubVoxelPattern};
use super::loader::{LoadProgress, LoadedMapData, MapLoadProgress};
use bevy::gltf::GltfAssetLabel;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use std::collections::{HashMap, HashSet};

const SUB_VOXEL_COUNT: i32 = 8; // 8x8x8 sub-voxels per voxel
const SUB_VOXEL_SIZE: f32 = 1.0 / SUB_VOXEL_COUNT as f32;

/// Chunk size in voxels (16x16x16 voxels per chunk)
pub const CHUNK_SIZE: i32 = 16;

/// Marker component for chunk entities
#[derive(Component)]
#[allow(dead_code)]
pub struct VoxelChunk {
    /// The chunk position for potential future use (chunk updates, unloading)
    pub chunk_pos: IVec3,
}

/// Face direction for hidden face culling.
#[derive(Clone, Copy, Debug)]
pub enum Face {
    /// +X direction
    PosX,
    /// -X direction
    NegX,
    /// +Y direction (up)
    PosY,
    /// -Y direction (down)
    NegY,
    /// +Z direction
    PosZ,
    /// -Z direction
    NegZ,
}

impl Face {
    /// Returns the normal vector for this face.
    #[inline]
    pub fn normal(self) -> [f32; 3] {
        match self {
            Face::PosX => [1.0, 0.0, 0.0],
            Face::NegX => [-1.0, 0.0, 0.0],
            Face::PosY => [0.0, 1.0, 0.0],
            Face::NegY => [0.0, -1.0, 0.0],
            Face::PosZ => [0.0, 0.0, 1.0],
            Face::NegZ => [0.0, 0.0, -1.0],
        }
    }

    /// Returns the neighbor offset for this face direction.
    #[inline]
    pub fn offset(self) -> (i32, i32, i32) {
        match self {
            Face::PosX => (1, 0, 0),
            Face::NegX => (-1, 0, 0),
            Face::PosY => (0, 1, 0),
            Face::NegY => (0, -1, 0),
            Face::PosZ => (0, 0, 1),
            Face::NegZ => (0, 0, -1),
        }
    }
}

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

/// Builder for constructing chunk meshes from multiple cubes.
/// Combines all sub-voxels in a chunk into a single mesh with vertex colors.
/// Supports hidden face culling to reduce triangle count.
#[derive(Default)]
pub struct ChunkMeshBuilder {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl ChunkMeshBuilder {
    /// Add a single face to the mesh.
    #[inline]
    pub fn add_face(&mut self, position: Vec3, size: f32, face: Face, color: Color) {
        let half = size / 2.0;
        let base_index = self.positions.len() as u32;
        let color_array = color.to_linear().to_f32_array();
        let normal = face.normal();

        // Generate 4 vertices for the face
        let vertices: [[f32; 3]; 4] = match face {
            Face::PosZ => [
                [position.x - half, position.y - half, position.z + half],
                [position.x + half, position.y - half, position.z + half],
                [position.x + half, position.y + half, position.z + half],
                [position.x - half, position.y + half, position.z + half],
            ],
            Face::NegZ => [
                [position.x + half, position.y - half, position.z - half],
                [position.x - half, position.y - half, position.z - half],
                [position.x - half, position.y + half, position.z - half],
                [position.x + half, position.y + half, position.z - half],
            ],
            Face::PosX => [
                [position.x + half, position.y - half, position.z + half],
                [position.x + half, position.y - half, position.z - half],
                [position.x + half, position.y + half, position.z - half],
                [position.x + half, position.y + half, position.z + half],
            ],
            Face::NegX => [
                [position.x - half, position.y - half, position.z - half],
                [position.x - half, position.y - half, position.z + half],
                [position.x - half, position.y + half, position.z + half],
                [position.x - half, position.y + half, position.z - half],
            ],
            Face::PosY => [
                [position.x - half, position.y + half, position.z + half],
                [position.x + half, position.y + half, position.z + half],
                [position.x + half, position.y + half, position.z - half],
                [position.x - half, position.y + half, position.z - half],
            ],
            Face::NegY => [
                [position.x - half, position.y - half, position.z - half],
                [position.x + half, position.y - half, position.z - half],
                [position.x + half, position.y - half, position.z + half],
                [position.x - half, position.y - half, position.z + half],
            ],
        };

        self.positions.extend_from_slice(&vertices);
        self.normals.extend_from_slice(&[normal; 4]);
        self.uvs.extend_from_slice(&[
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0],
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
    pub fn add_cube_culled(&mut self, position: Vec3, size: f32, color: Color, neighbors: [bool; 6]) {
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

/// Context for spawning entities.
struct EntitySpawnContext<'w, 's, 'a> {
    commands: Commands<'w, 's>,
    meshes: &'a mut Assets<Mesh>,
    materials: &'a mut Assets<StandardMaterial>,
    asset_server: &'a AssetServer,
}

/// Context for chunk-based voxel spawning.
struct ChunkSpawnContext<'w, 's, 'a> {
    commands: Commands<'w, 's>,
    spatial_grid: &'a mut SpatialGrid,
    meshes: &'a mut Assets<Mesh>,
    chunk_material: Handle<StandardMaterial>,
}

/// System that spawns a loaded map into the game world.
///
/// This system reads the LoadedMapData resource and creates all the
/// necessary entities (voxels, sub-voxels, entities, lighting, camera).
///
/// Uses chunk-based meshing to combine sub-voxels into larger meshes,
/// dramatically reducing entity count and improving performance.
pub fn spawn_map_system(
    mut commands: Commands,
    map_data: Res<LoadedMapData>,
    mut progress: ResMut<MapLoadProgress>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    game_initialized: Option<Res<GameInitialized>>,
) {
    // If game is already initialized, don't spawn again
    if let Some(initialized) = game_initialized {
        if initialized.0 {
            return;
        }
    }

    // Mark game as initialized
    commands.insert_resource(GameInitialized(true));

    // Initialize SpatialGrid
    let mut spatial_grid = SpatialGrid::default();

    let map = &map_data.map;

    // Create a single material for all chunks (vertex colors provide variation)
    let chunk_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        // Use vertex colors for per-voxel coloring
        ..default()
    });

    // Stage 4: Spawn voxels using chunk-based meshing (60-90%)
    progress.update(LoadProgress::SpawningVoxels(0.0));
    commands = {
        let mut chunk_ctx = ChunkSpawnContext {
            commands,
            spatial_grid: &mut spatial_grid,
            meshes: meshes.as_mut(),
            chunk_material,
        };
        spawn_voxels_chunked(&mut chunk_ctx, map, &mut progress);
        chunk_ctx.commands
    };
    progress.update(LoadProgress::SpawningVoxels(1.0));

    // Stage 5: Spawn entities (90-95%)
    progress.update(LoadProgress::SpawningEntities(0.0));
    commands = {
        let mut entity_ctx = EntitySpawnContext {
            commands,
            meshes: meshes.as_mut(),
            materials: materials.as_mut(),
            asset_server: &asset_server,
        };
        spawn_entities(&mut entity_ctx, map, &mut progress);
        entity_ctx.commands
    };
    progress.update(LoadProgress::SpawningEntities(1.0));

    // Stage 6: Setup lighting (95-97%)
    progress.update(LoadProgress::Finalizing(0.0));
    spawn_lighting(&mut commands, map);

    // Stage 7: Setup camera (97-100%)
    progress.update(LoadProgress::Finalizing(0.5));
    spawn_camera(&mut commands, map);

    // Insert the spatial grid as a resource
    commands.insert_resource(spatial_grid);

    // Complete
    progress.update(LoadProgress::Finalizing(1.0));
    progress.update(LoadProgress::Complete);
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

/// Spawn all voxels using chunk-based meshing with hidden face culling.
///
/// This function:
/// 1. First pass: Collects all occupied sub-voxel positions into an OccupancyGrid
/// 2. Second pass: Groups sub-voxels by chunk (16x16x16), applies face culling,
///    builds a single mesh per chunk with vertex colors
/// 3. Spawns one entity per chunk instead of one entity per sub-voxel
///
/// Hidden face culling removes interior faces between adjacent sub-voxels,
/// reducing triangle count by 60-90% for solid geometry.
fn spawn_voxels_chunked(
    ctx: &mut ChunkSpawnContext,
    map: &MapData,
    progress: &mut MapLoadProgress,
) {
    let total_voxels = map.world.voxels.len();

    // First pass: Build occupancy grid for neighbor lookups
    progress.update(LoadProgress::SpawningVoxels(0.0));
    let mut occupancy = OccupancyGrid::new();
    
    // Collect all sub-voxel data for both passes
    let mut all_sub_voxels: Vec<(i32, i32, i32, i32, i32, i32, Vec3, Color)> = Vec::new();
    
    for (index, voxel_data) in map.world.voxels.iter().enumerate() {
        // Update progress (occupancy collection phase: 0-20%)
        if index % 100 == 0 {
            let voxel_progress = (index as f32) / (total_voxels as f32) * 0.2;
            progress.update(LoadProgress::SpawningVoxels(voxel_progress));
        }

        let (x, y, z) = voxel_data.pos;

        // Spawn parent voxel marker
        ctx.commands.spawn(Voxel);

        // Determine which pattern to use
        let pattern = voxel_data.pattern.unwrap_or(SubVoxelPattern::Full);

        // Get the geometry for this pattern with rotation applied
        let geometry = pattern.geometry_with_rotation(voxel_data.rotation_state);

        // Add each sub-voxel to occupancy grid and collect data
        for (sub_x, sub_y, sub_z) in geometry.occupied_positions() {
            occupancy.insert(x, y, z, sub_x, sub_y, sub_z);
            
            let world_pos = calculate_sub_voxel_pos(x, y, z, sub_x, sub_y, sub_z);
            let color = get_sub_voxel_color(x, y, z, sub_x, sub_y, sub_z);
            all_sub_voxels.push((x, y, z, sub_x, sub_y, sub_z, world_pos, color));
        }
    }

    // Second pass: Build chunk meshes with face culling
    let mut chunks: HashMap<IVec3, ChunkMeshBuilder> = HashMap::new();
    let mut sub_voxel_positions: Vec<(Vec3, (Vec3, Vec3))> = Vec::new();
    
    let total_sub_voxels_count = all_sub_voxels.len();
    for (index, (x, y, z, sub_x, sub_y, sub_z, world_pos, color)) in all_sub_voxels.into_iter().enumerate() {
        // Update progress (mesh building phase: 20-50%)
        if index % 1000 == 0 {
            let build_progress = 0.2 + (index as f32) / (total_sub_voxels_count as f32) * 0.3;
            progress.update(LoadProgress::SpawningVoxels(build_progress));
        }

        // Check all 6 neighbors for face culling
        // neighbors order: [+X, -X, +Y, -Y, +Z, -Z]
        let neighbors = [
            occupancy.has_neighbor(x, y, z, sub_x, sub_y, sub_z, Face::PosX),
            occupancy.has_neighbor(x, y, z, sub_x, sub_y, sub_z, Face::NegX),
            occupancy.has_neighbor(x, y, z, sub_x, sub_y, sub_z, Face::PosY),
            occupancy.has_neighbor(x, y, z, sub_x, sub_y, sub_z, Face::NegY),
            occupancy.has_neighbor(x, y, z, sub_x, sub_y, sub_z, Face::PosZ),
            occupancy.has_neighbor(x, y, z, sub_x, sub_y, sub_z, Face::NegZ),
        ];

        // Determine which chunk this sub-voxel belongs to
        let chunk_pos = IVec3::new(
            (world_pos.x / CHUNK_SIZE as f32).floor() as i32,
            (world_pos.y / CHUNK_SIZE as f32).floor() as i32,
            (world_pos.z / CHUNK_SIZE as f32).floor() as i32,
        );

        // Add cube with culled faces to chunk mesh builder
        let builder = chunks.entry(chunk_pos).or_default();
        builder.add_cube_culled(world_pos, SUB_VOXEL_SIZE, color, neighbors);

        // Calculate bounds for collision detection
        let half_size = SUB_VOXEL_SIZE / 2.0;
        let bounds = (
            world_pos - Vec3::splat(half_size),
            world_pos + Vec3::splat(half_size),
        );
        sub_voxel_positions.push((world_pos, bounds));
    }

    // Spawn chunk entities
    let total_chunks = chunks.len();
    for (index, (chunk_pos, builder)) in chunks.into_iter().enumerate() {
        if builder.is_empty() {
            continue;
        }

        // Update progress (spawning phase)
        let spawn_progress = 0.5 + (index as f32) / (total_chunks as f32) * 0.3;
        progress.update(LoadProgress::SpawningVoxels(spawn_progress));

        let mesh = ctx.meshes.add(builder.build());
        ctx.commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(ctx.chunk_material.clone()),
            Transform::default(),
            VoxelChunk { chunk_pos },
        ));
    }

    // Spawn invisible collision entities for the spatial grid
    // These are needed for physics/collision detection
    let total_sub_voxels = sub_voxel_positions.len();
    for (index, (world_pos, bounds)) in sub_voxel_positions.into_iter().enumerate() {
        // Update progress (collision setup phase)
        if index % 1000 == 0 {
            let collision_progress = 0.8 + (index as f32) / (total_sub_voxels as f32) * 0.2;
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
        "Spawned {} chunks with {} collision entities",
        total_chunks, total_sub_voxels
    );
}

/// Spawn entities from the map data.
fn spawn_entities(ctx: &mut EntitySpawnContext, map: &MapData, progress: &mut MapLoadProgress) {
    let total_entities = map.entities.len();

    for (index, entity_data) in map.entities.iter().enumerate() {
        // Update progress
        let entity_progress = (index as f32) / (total_entities as f32);
        progress.update(LoadProgress::SpawningEntities(entity_progress));

        let (x, y, z) = entity_data.position;

        match entity_data.entity_type {
            EntityType::PlayerSpawn => {
                spawn_player(ctx, Vec3::new(x, y, z));
            }
            EntityType::Enemy => {
                // TODO: Implement enemy spawning
                info!("Enemy spawn at ({}, {}, {}) - not yet implemented", x, y, z);
            }
            EntityType::Item => {
                // TODO: Implement item spawning
                info!("Item spawn at ({}, {}, {}) - not yet implemented", x, y, z);
            }
            EntityType::Trigger => {
                // TODO: Implement trigger spawning
                info!(
                    "Trigger spawn at ({}, {}, {}) - not yet implemented",
                    x, y, z
                );
            }
        }
    }
}

/// Spawn the player entity with a 3D character model.
///
/// This function creates:
/// 1. A player entity with physics components (no visible mesh)
/// 2. A GLB character model as a child entity for visuals
/// 3. An invisible collision box for debugging
///
/// The physics collision uses a sphere collider (radius: 0.3) which is kept
/// separate from the visual model for flexibility and performance.
fn spawn_player(ctx: &mut EntitySpawnContext, position: Vec3) {
    let player_radius = 0.3;

    // Load the character model (GLB file) with explicit scene specification
    // Using GltfAssetLabel::Scene(0) to load the first (default) scene from the GLB file
    let character_scene: Handle<Scene> = ctx
        .asset_server
        .load(GltfAssetLabel::Scene(0).from_asset("characters/base_basic_pbr.glb"));

    info!("Loading character model: characters/base_basic_pbr.glb#Scene0");

    // Spawn the main player entity (parent) with physics components
    // No visible mesh - the GLB model will be the visual representation
    let player_entity = ctx
        .commands
        .spawn((
            Transform::from_translation(position),
            Visibility::default(),
            Player {
                speed: 3.0,
                velocity: Vec3::ZERO,
                is_grounded: true,
                radius: player_radius,
                target_rotation: 0.0,
                current_rotation: 0.0,
                start_rotation: 0.0,
                rotation_elapsed: 0.0,
                rotation_duration: 0.2, // Fixed 0.2 second duration for all rotations
            },
            CharacterModel::new(character_scene.clone()),
        ))
        .id();

    // Spawn the character model as a child entity
    // Scale down to 0.3 and offset down by 0.3 units to align with collision sphere
    ctx.commands
        .spawn((
            SceneRoot(character_scene),
            Transform::from_translation(Vec3::new(0.0, -0.3, 0.0)).with_scale(Vec3::splat(0.5)),
        ))
        .set_parent(player_entity);

    info!(
        "Spawned player with character model at position: {:?}",
        position
    );

    // Create collision box (invisible by default, for debugging)
    let collision_box_mesh = ctx.meshes.add(Cuboid::new(
        player_radius * 2.0,
        player_radius * 2.0,
        player_radius * 2.0,
    ));
    let collision_box_material = ctx.materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 1.0, 0.0, 0.3),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    ctx.commands.spawn((
        Mesh3d(collision_box_mesh),
        MeshMaterial3d(collision_box_material),
        Transform::from_translation(position),
        Visibility::Hidden,
        CollisionBox,
    ));
}

/// Spawn lighting from the map data.
fn spawn_lighting(commands: &mut Commands, map: &MapData) {
    let lighting = &map.lighting;

    // Spawn directional light if configured with high-quality shadows
    if let Some(dir_light) = &lighting.directional_light {
        let (dx, dy, dz) = dir_light.direction;
        let direction = Vec3::new(dx, dy, dz).normalize();

        let cascade_shadow_config = CascadeShadowConfigBuilder {
            num_cascades: 4,
            first_cascade_far_bound: 4.0,
            maximum_distance: 100.0,
            ..default()
        }
        .build();

        commands.spawn((
            DirectionalLight {
                illuminance: dir_light.illuminance,
                color: Color::srgb(dir_light.color.0, dir_light.color.1, dir_light.color.2),
                shadows_enabled: true,
                shadow_depth_bias: 0.02,
                shadow_normal_bias: 1.8,
            },
            cascade_shadow_config,
            Transform::from_rotation(Quat::from_rotation_arc(Vec3::NEG_Z, direction)),
        ));
    }

    // Spawn ambient light using map-defined intensity
    // Convert 0.0-1.0 intensity to brightness (scale by 1000 for Bevy's lighting system)
    let ambient_brightness = lighting.ambient_intensity * 1000.0;
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: ambient_brightness,
    });

    info!(
        "Spawned ambient light with brightness: {}",
        ambient_brightness
    );
}

/// Spawn camera from the map data.
fn spawn_camera(commands: &mut Commands, map: &MapData) {
    let camera = &map.camera;
    let (px, py, pz) = camera.position;
    let (lx, ly, lz) = camera.look_at;

    let mut camera_transform =
        Transform::from_xyz(px, py, pz).looking_at(Vec3::new(lx, ly, lz), Vec3::Y);

    // Apply rotation offset
    if camera.rotation_offset != 0.0 {
        camera_transform.rotate_around(
            Vec3::new(lx, ly, lz),
            Quat::from_rotation_y(camera.rotation_offset),
        );
    }

    let original_rotation = camera_transform.rotation;

    // Calculate initial follow offset from camera position to look_at point
    let look_at_point = Vec3::new(lx, ly, lz);
    let initial_offset = camera_transform.translation - look_at_point;

    // Transform offset to local camera space (inverse of camera rotation)
    let follow_offset = camera_transform.rotation.inverse() * initial_offset;

    commands.spawn((
        Camera3d::default(),
        camera_transform,
        GameCamera {
            original_rotation,
            target_rotation: original_rotation,
            rotation_speed: 5.0,
            follow_offset,
            follow_speed: 5.0,              // Medium responsiveness
            target_position: look_at_point, // Initially look at the map's look_at point
        },
    ));
}
