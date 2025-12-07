# Voxel Rendering Engine Optimizations

## Overview

This document outlines performance optimizations for the voxel rendering system. The current implementation creates individual entities and materials for each sub-voxel, which causes significant performance issues at scale.

**Status**: 📋 Planned  
**Priority**: High  
**Estimated Total Impact**: 10-100x performance improvement  
**Last Updated**: 2025-12-07

---

## Current Architecture Analysis

### Files Affected
- `src/systems/game/map/spawner.rs` - Game world voxel spawning
- `src/editor/renderer.rs` - Editor viewport rendering

### Current Implementation

```rust
// Each sub-voxel creates a unique material
let color = Color::srgb(
    0.2 + (x as f32 * 0.2) + (sub_x as f32 * 0.01),
    0.3 + (z as f32 * 0.15) + (sub_z as f32 * 0.01),
    0.4 + (y as f32 * 0.2) + (sub_y as f32 * 0.01),
);
let material = materials.add(color);

// Each sub-voxel is a separate entity
commands.spawn((
    Mesh3d(mesh.clone()),
    MeshMaterial3d(material),
    Transform::from_xyz(sub_x_pos, sub_y_pos, sub_z_pos),
    SubVoxel { bounds },
));
```

### Performance Metrics (Worst Case)

| Metric | Current | Target |
|--------|---------|--------|
| World Size | 64×64×64 voxels | Same |
| Sub-voxels per voxel | 512 (8³) | Same |
| Total sub-voxels | 134,217,728 | Same |
| Materials | 134M (1 per sub-voxel) | 64-256 |
| Entities | 134M | ~4,096 chunks |
| Draw calls | 134M | ~4,096 |

---

## Optimization Tiers

## Tier 1: Material Palette (High Impact, Low Effort)

### Status: 📋 Not Started

### Problem

Every sub-voxel creates a unique `StandardMaterial` based on its position. This:
- Consumes massive GPU memory
- Prevents draw call batching
- Causes shader state changes

### Solution

Pre-create a fixed palette of materials and hash positions to palette indices.

### Implementation

#### Step 1: Create Material Palette Resource

**File**: `src/systems/game/map/spawner.rs`

```rust
/// Pre-generated material palette for efficient voxel rendering.
/// Materials are reused based on position hashing to enable GPU batching.
#[derive(Resource)]
pub struct VoxelMaterialPalette {
    pub materials: Vec<Handle<StandardMaterial>>,
}

impl VoxelMaterialPalette {
    pub const PALETTE_SIZE: usize = 64;

    pub fn new(materials_asset: &mut Assets<StandardMaterial>) -> Self {
        let materials: Vec<_> = (0..Self::PALETTE_SIZE)
            .map(|i| {
                let t = i as f32 / Self::PALETTE_SIZE as f32;
                let color = Color::srgb(
                    0.2 + t * 0.6,
                    0.3 + ((t * 2.0).sin() * 0.5 + 0.5) * 0.4,
                    0.4 + ((t * 3.0).cos() * 0.5 + 0.5) * 0.4,
                );
                materials_asset.add(color)
            })
            .collect();

        Self { materials }
    }

    /// Get material index for a sub-voxel position using spatial hashing.
    #[inline]
    pub fn get_material_index(x: i32, y: i32, z: i32, sub_x: i32, sub_y: i32, sub_z: i32) -> usize {
        // Simple spatial hash that preserves visual variation
        let hash = (x.wrapping_mul(73856093))
            ^ (y.wrapping_mul(19349663))
            ^ (z.wrapping_mul(83492791))
            ^ (sub_x.wrapping_mul(15485863))
            ^ (sub_y.wrapping_mul(32452843))
            ^ (sub_z.wrapping_mul(49979687));
        (hash.unsigned_abs() as usize) % Self::PALETTE_SIZE
    }

    #[inline]
    pub fn get_material(&self, x: i32, y: i32, z: i32, sub_x: i32, sub_y: i32, sub_z: i32) -> Handle<StandardMaterial> {
        let index = Self::get_material_index(x, y, z, sub_x, sub_y, sub_z);
        self.materials[index].clone()
    }
}
```

#### Step 2: Initialize Palette at Startup

**File**: `src/systems/game/map/spawner.rs`

```rust
// In spawn_map_system, before spawning voxels:
let material_palette = VoxelMaterialPalette::new(materials.as_mut());
commands.insert_resource(material_palette.clone());
```

#### Step 3: Update Sub-Voxel Spawning

**File**: `src/systems/game/map/spawner.rs`

```rust
fn spawn_sub_voxel(
    ctx: &mut VoxelSpawnContext,
    palette: &VoxelMaterialPalette,  // Add parameter
    x: i32, y: i32, z: i32,
    sub_x: i32, sub_y: i32, sub_z: i32,
) {
    // Replace material creation with palette lookup
    let sub_voxel_material = palette.get_material(x, y, z, sub_x, sub_y, sub_z);
    
    // ... rest of function unchanged
}
```

#### Step 4: Apply Same Changes to Editor Renderer

**File**: `src/editor/renderer.rs`

Apply the same palette pattern for consistency.

### Expected Results

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Materials | 134M | 64 | 99.99% reduction |
| GPU memory (materials) | ~16 GB | ~8 KB | 99.99% reduction |
| Shader state changes | Per sub-voxel | Per 2M sub-voxels | ~2M× fewer |

### Testing

- [ ] Visual comparison: Colors should still vary spatially
- [ ] Performance benchmark: Measure material creation time
- [ ] Memory profiling: Verify reduced GPU memory usage

---

## Tier 2: GPU Instancing (High Impact, Medium Effort)

### Status: 📋 Not Started

### Problem

Each sub-voxel is a separate draw call even when using the same mesh and material.

### Solution

Use Bevy's automatic GPU instancing by ensuring sub-voxels with the same material are batched.

### Implementation

#### Step 1: Enable Instancing (Bevy Default)

Bevy automatically instances entities with identical `Mesh3d` and `MeshMaterial3d`. The material palette from Tier 1 enables this.

#### Step 2: Sort Sub-Voxels by Material Index

```rust
fn spawn_voxels(ctx: &mut VoxelSpawnContext, map: &MapData, progress: &mut MapLoadProgress) {
    // Collect all sub-voxels with their material indices
    let mut sub_voxels: Vec<(i32, i32, i32, i32, i32, i32, usize)> = Vec::new();
    
    for voxel_data in &map.world.voxels {
        let (x, y, z) = voxel_data.pos;
        let pattern = voxel_data.pattern.unwrap_or(SubVoxelPattern::Full);
        let geometry = pattern.geometry_with_rotation(voxel_data.rotation_state);
        
        for (sub_x, sub_y, sub_z) in geometry.occupied_positions() {
            let mat_idx = VoxelMaterialPalette::get_material_index(x, y, z, sub_x, sub_y, sub_z);
            sub_voxels.push((x, y, z, sub_x, sub_y, sub_z, mat_idx));
        }
    }
    
    // Sort by material index for optimal batching
    sub_voxels.sort_by_key(|v| v.6);
    
    // Spawn in sorted order
    for (x, y, z, sub_x, sub_y, sub_z, _) in sub_voxels {
        spawn_sub_voxel(ctx, &palette, x, y, z, sub_x, sub_y, sub_z);
    }
}
```

### Expected Results

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Draw calls | 134M | ~64 (1 per material) | 99.99% reduction |
| GPU utilization | Low (CPU bound) | High | Variable |

### Testing

- [ ] Enable Bevy's render statistics to verify instancing
- [ ] Profile frame time reduction
- [ ] Test with various world sizes

---

## Tier 3: Chunk-Based Meshing (Highest Impact, High Effort)

### Status: 📋 Not Started

### Problem

Millions of entities cause massive ECS overhead regardless of instancing.

### Solution

Combine sub-voxels into chunk meshes (16³ or 32³). Each chunk becomes a single entity with a merged mesh.

### Implementation

#### Step 1: Define Chunk System

```rust
pub const CHUNK_SIZE: i32 = 16;

#[derive(Component)]
pub struct VoxelChunk {
    pub chunk_pos: IVec3,
}

#[derive(Default)]
pub struct ChunkMeshBuilder {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl ChunkMeshBuilder {
    pub fn add_cube(&mut self, position: Vec3, size: f32, color: Color) {
        // Add 8 vertices and 36 indices for a cube
        // ... implementation
    }

    pub fn build(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}
```

#### Step 2: Group Sub-Voxels by Chunk

```rust
fn spawn_voxels_chunked(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    map: &MapData,
) {
    // Group sub-voxels by chunk
    let mut chunks: HashMap<IVec3, ChunkMeshBuilder> = HashMap::new();
    
    for voxel_data in &map.world.voxels {
        let (x, y, z) = voxel_data.pos;
        let chunk_pos = IVec3::new(
            x.div_euclid(CHUNK_SIZE),
            y.div_euclid(CHUNK_SIZE),
            z.div_euclid(CHUNK_SIZE),
        );
        
        let builder = chunks.entry(chunk_pos).or_default();
        
        let pattern = voxel_data.pattern.unwrap_or(SubVoxelPattern::Full);
        let geometry = pattern.geometry_with_rotation(voxel_data.rotation_state);
        
        for (sub_x, sub_y, sub_z) in geometry.occupied_positions() {
            let world_pos = calculate_sub_voxel_pos(x, y, z, sub_x, sub_y, sub_z);
            let color = calculate_color(x, y, z, sub_x, sub_y, sub_z);
            builder.add_cube(world_pos, SUB_VOXEL_SIZE, color);
        }
    }
    
    // Spawn chunk entities
    let chunk_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });
    
    for (chunk_pos, builder) in chunks {
        let mesh = meshes.add(builder.build());
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(chunk_material.clone()),
            Transform::default(),
            VoxelChunk { chunk_pos },
        ));
    }
}
```

#### Step 3: Update Collision System

The `SpatialGrid` already handles collision efficiently. Ensure it's populated correctly from chunk data.

### Expected Results

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Entities | 134M | ~4,096 | 99.99% reduction |
| ECS overhead | Massive | Minimal | 1000×+ faster |
| Memory (entities) | ~16 GB | ~1 MB | 99.99% reduction |

### Testing

- [ ] Verify visual correctness with chunk boundaries
- [ ] Benchmark entity spawn time
- [ ] Test chunk update when voxels change

---

## Tier 4: Hidden Face Culling (Medium Impact, Medium Effort)

### Status: 📋 Not Started

### Problem

Interior faces between adjacent sub-voxels are rendered but never visible.

### Solution

During mesh generation, check adjacent sub-voxels and skip faces that are hidden.

### Implementation

```rust
impl ChunkMeshBuilder {
    pub fn add_cube_with_neighbors(
        &mut self,
        position: Vec3,
        size: f32,
        color: Color,
        neighbors: [bool; 6], // +X, -X, +Y, -Y, +Z, -Z
    ) {
        // Only add faces where neighbor is empty (false)
        if !neighbors[0] { self.add_face(position, size, Face::PosX, color); }
        if !neighbors[1] { self.add_face(position, size, Face::NegX, color); }
        if !neighbors[2] { self.add_face(position, size, Face::PosY, color); }
        if !neighbors[3] { self.add_face(position, size, Face::NegY, color); }
        if !neighbors[4] { self.add_face(position, size, Face::PosZ, color); }
        if !neighbors[5] { self.add_face(position, size, Face::NegZ, color); }
    }
}
```

### Expected Results

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Triangles (solid chunk) | 12 per cube | ~2 per cube | 83% reduction |
| GPU vertex processing | Full | Surfaces only | 60-90% reduction |

---

## Tier 5: Greedy Meshing (High Impact, High Effort)

### Status: 📋 Not Started

### Problem

Even with face culling, we generate many small quads for large flat surfaces.

### Solution

Merge adjacent coplanar faces of the same color into larger quads.

### Algorithm

```
For each axis (X, Y, Z):
    For each slice perpendicular to axis:
        Create 2D mask of visible faces
        While mask has faces:
            Find rectangle of same-color faces
            Emit single quad for rectangle
            Clear rectangle from mask
```

### Expected Results

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Quads (flat floor) | 1 per sub-voxel | 1 per region | 90%+ reduction |
| Vertices | O(n³) | O(n²) for surfaces | Massive |

---

## Tier 6: LOD System (Variable Impact)

### Status: 📋 Not Started

### Problem

Distant voxels don't need full sub-voxel detail.

### Solution

Generate multiple mesh LODs per chunk, switch based on camera distance.

### Implementation

```rust
#[derive(Component)]
pub struct ChunkLOD {
    pub lod_meshes: [Handle<Mesh>; 4], // Full, Half, Quarter, Eighth
    pub current_lod: usize,
}

fn update_chunk_lods(
    camera: Query<&Transform, With<Camera>>,
    mut chunks: Query<(&Transform, &mut ChunkLOD, &mut Mesh3d)>,
) {
    let camera_pos = camera.single().translation;
    
    for (transform, mut lod, mut mesh) in chunks.iter_mut() {
        let distance = camera_pos.distance(transform.translation);
        let new_lod = match distance {
            d if d < 20.0 => 0,
            d if d < 50.0 => 1,
            d if d < 100.0 => 2,
            _ => 3,
        };
        
        if new_lod != lod.current_lod {
            lod.current_lod = new_lod;
            mesh.0 = lod.lod_meshes[new_lod].clone();
        }
    }
}
```

---

## Implementation Order

| Priority | Optimization | Effort | Impact | Dependencies |
|----------|--------------|--------|--------|--------------|
| 1 | Material Palette | Low | High | None |
| 2 | GPU Instancing | Low | High | Material Palette |
| 3 | Chunk-Based Meshing | High | Highest | None (replaces 1-2) |
| 4 | Hidden Face Culling | Medium | Medium | Chunk Meshing |
| 5 | Greedy Meshing | High | High | Face Culling |
| 6 | LOD System | Medium | Variable | Chunk Meshing |

### Recommended Path

**Option A: Incremental (Lower Risk)**
1. Implement Material Palette (1-2 hours)
2. Verify instancing works (30 minutes)
3. Profile and decide if chunks needed

**Option B: Full Rewrite (Higher Impact)**
1. Implement Chunk-Based Meshing directly
2. Add Face Culling during implementation
3. Add Greedy Meshing as enhancement

---

## Testing Plan

### Performance Benchmarks

```rust
#[cfg(test)]
mod benchmarks {
    // Benchmark: Time to spawn 1000 voxels
    // Benchmark: Frame time with 10K/100K/1M sub-voxels
    // Benchmark: Memory usage at various scales
}
```

### Visual Tests

- [ ] Colors maintain spatial variation
- [ ] No visual artifacts at chunk boundaries
- [ ] Correct rendering of all voxel patterns
- [ ] Rotation states render correctly

### Regression Tests

- [ ] Collision detection still works
- [ ] Spatial grid populated correctly
- [ ] Editor and game render consistently

---

## Success Metrics

| Metric | Current | Target | Method |
|--------|---------|--------|--------|
| Spawn time (1K voxels) | ~500ms | <50ms | Benchmark |
| Frame time (100K sub-voxels) | ~32ms | <8ms | Profiler |
| GPU memory (materials) | ~1GB | <10MB | GPU profiler |
| Entity count | O(n³) | O(chunks) | ECS stats |

---

## Rollback Plan

Each optimization is isolated:

1. **Material Palette**: Revert `VoxelMaterialPalette`, restore per-voxel material creation
2. **Instancing**: No code change needed, just sorting removal
3. **Chunk Meshing**: Restore individual entity spawning

---

## References

- [Bevy Rendering Architecture](https://bevyengine.org/learn/book/gpu-driven-rendering/)
- [Greedy Meshing Algorithm](https://0fps.net/2012/06/30/meshing-in-a-minecraft-game/)
- [Voxel Engine Optimization Techniques](https://tomcc.github.io/2014/08/31/visibility-1.html)
- [Physics Analysis](../developer-guide/systems/physics-analysis.md) - Related spatial grid optimization
