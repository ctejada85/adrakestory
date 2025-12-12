# Voxel Occlusion Transparency System

## Overview

Implement a system that makes voxels transparent or invisible when they occlude (block the view of) the player character. This is a common feature in isometric and 3D games that ensures players can always see their character when walking under structures, bridges, roofs, or other overhead voxels.

**Status**: 📋 Planned  
**Priority**: Medium  
**Estimated Effort**: 3-5 days  
**Approach**: Shader-Based Occlusion  
**Last Updated**: 2025-12-12

---

## Problem Statement

When the player character walks under voxel structures (buildings, bridges, overhangs, etc.), the overhead voxels block the camera's view of the character. This creates a poor gameplay experience where players cannot see what they are doing or where they are going.

### Current Behavior
- Voxels are rendered at full opacity regardless of player position
- Overhead voxels completely obscure the player character
- Players lose visual feedback when navigating under structures

### Desired Behavior
- Voxels directly above/between the camera and player should fade to transparent
- Smooth transition between opaque and transparent states
- Character and nearby floor voxels remain visible
- Transparent voxels should still cast shadows (optional)

---

## Architecture Analysis

### Affected Files
- `src/systems/game/map/spawner.rs` - Chunk spawning (use OcclusionMaterial)
- `src/systems/game/mod.rs` - Module registration
- `src/systems/game/camera.rs` - Camera position data (read only)
- `src/main.rs` - Plugin registration
- New: `src/systems/game/occlusion.rs` - Custom material and uniform update system
- New: `assets/shaders/occlusion_material.wgsl` - Custom WGSL shader

### Current Rendering Architecture

The game uses a chunk-based rendering system:
1. **VoxelChunk** - Contains chunk position and center point
2. **ChunkLOD** - Handles Level of Detail mesh switching
3. **Single Material** - All chunks share one `StandardMaterial` via `MeshMaterial3d`

### Challenge: Shared Material

Currently, all chunks use a shared material for GPU efficiency:
```rust
ctx.commands.spawn((
    Mesh3d(lod_meshes[0].clone()),
    MeshMaterial3d(ctx.chunk_material.clone()),  // Shared!
    Transform::default(),
    VoxelChunk { chunk_pos, center: chunk_center },
    ChunkLOD { lod_meshes, current_lod: 0 },
));
```

This shared material approach conflicts with per-chunk transparency. We need to implement one of these solutions:
1. **Per-chunk materials** (more memory, simpler logic)
2. **Shader-based occlusion** (more complex, better performance)
3. **Mesh-level alpha** (vertex colors with alpha channel)

---

## Approach Comparison: Per-Chunk Materials vs Shader-Based Occlusion

### Per-Chunk Materials

Each chunk gets its own `StandardMaterial` instance, allowing individual alpha control through Bevy's built-in material system.

| Aspect | Details |
|--------|---------|
| **Implementation** | Clone base material per chunk, update `base_color.alpha` at runtime |
| **Complexity** | Low - Uses existing Bevy APIs |
| **Memory** | ~64-128 bytes per material × chunk count |
| **CPU Cost** | O(n) material updates per frame for changing chunks |
| **GPU Cost** | Breaks GPU instancing - each chunk is a separate draw call |

#### Pros
- ✅ **Simple implementation** - No custom shaders, uses Bevy's `StandardMaterial`
- ✅ **Easy to debug** - Can inspect material alpha in Bevy's inspector
- ✅ **Flexible** - Can apply different effects per chunk (tint, emission, etc.)
- ✅ **Predictable behavior** - Well-documented Bevy material system
- ✅ **Quick iteration** - Fast to implement and modify

#### Cons
- ❌ **Breaks GPU batching** - Each chunk becomes a separate draw call
- ❌ **Memory overhead** - Duplicates material data for each chunk
- ❌ **Chunk-granularity only** - Cannot fade individual voxels within a chunk
- ❌ **CPU overhead** - Must update material assets each frame for transitions
- ❌ **Sorting issues** - Transparent materials require depth sorting

---

### Shader-Based Occlusion

A custom shader receives player/camera positions as uniforms and calculates per-pixel transparency in the fragment shader.

| Aspect | Details |
|--------|---------|
| **Implementation** | Custom WGSL shader extending Bevy's PBR pipeline |
| **Complexity** | High - Requires shader programming and pipeline knowledge |
| **Memory** | Minimal - Single material with uniform buffer |
| **CPU Cost** | O(1) - Just update uniform buffer once per frame |
| **GPU Cost** | Small per-fragment cost, but maintains instancing |

#### Pros
- ✅ **Single material** - All chunks share one material, enables GPU instancing
- ✅ **Per-pixel precision** - Smooth gradients, not limited to chunk boundaries
- ✅ **Minimal CPU overhead** - Only update uniforms, no material mutations
- ✅ **Better scaling** - Performance independent of chunk count
- ✅ **Advanced effects** - Can implement soft edges, distance falloff, dithering
- ✅ **No sorting issues** - Can use alpha-to-coverage or dithering to avoid transparency sorting

#### Cons
- ❌ **Complex implementation** - Requires WGSL shader knowledge
- ❌ **Bevy PBR integration** - Must extend/replace standard material shader
- ❌ **Harder to debug** - Shader debugging is more difficult
- ❌ **Maintenance burden** - Custom shaders may break with Bevy updates
- ❌ **Initial development time** - Higher upfront cost

---

### Head-to-Head Comparison

| Factor | Per-Chunk Materials | Shader-Based | Winner |
|--------|---------------------|--------------|--------|
| **Implementation Time** | 1-2 days | 3-5 days | Per-Chunk |
| **Long-term Maintenance** | Low | Medium | Per-Chunk |
| **Performance (small maps)** | Good | Good | Tie |
| **Performance (large maps)** | Degrades | Constant | **Shader** |
| **Visual Quality** | Chunk-level | Pixel-level | **Shader** |
| **Memory Efficiency** | Poor | Excellent | **Shader** |
| **Draw Call Efficiency** | Poor (breaks batching) | Excellent | **Shader** |
| **Bevy Compatibility** | Native | Custom | Per-Chunk |
| **Future Extensibility** | Limited | High | **Shader** |

---

### Recommendation

| Scenario | Recommended Approach |
|----------|---------------------|
| **Prototype / MVP** | Per-Chunk Materials |
| **Small maps (<500 chunks)** | Per-Chunk Materials |
| **Large maps (1000+ chunks)** | Shader-Based |
| **Performance-critical** | Shader-Based |
| **Limited shader experience** | Per-Chunk Materials |
| **Long-term production game** | **Shader-Based** |

### Long-Term Best Choice: **Shader-Based Occlusion**

For a production game with plans to scale, **shader-based occlusion is the better long-term choice** because:

1. **Scalability** - Performance remains constant regardless of chunk count
2. **Visual Quality** - Per-pixel transparency looks significantly better than chunk-level fading
3. **GPU Efficiency** - Maintains instancing, critical for voxel games with many chunks
4. **Future Features** - Easy to add soft edges, dithering, x-ray effects, etc.

However, **start with per-chunk materials** as an MVP to validate the gameplay feel, then migrate to shader-based once the feature is proven valuable.

### Hybrid Approach (Recommended Path)

1. **Phase 1**: Implement per-chunk materials (1-2 days)
   - Validate gameplay and visual design
   - Establish the occlusion detection logic
   
2. **Phase 2**: Migrate to shader-based (3-4 days)
   - Port occlusion logic to WGSL shader
   - Keep detection system, just change how transparency is applied
   - Add per-pixel effects (soft edges, dithering)

This approach gives quick results while building toward the optimal solution.

---

## Proposed Solution: Shader-Based Occlusion

### Rationale

For long-term scalability and visual quality, we'll implement shader-based occlusion that:
- Maintains the single shared material (preserves GPU instancing)
- Calculates transparency per-pixel in the fragment shader
- Provides smooth, high-quality fading effects
- Scales to any number of chunks without performance degradation

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Occlusion System                         │
├─────────────────────────────────────────────────────────────┤
│  1. Track player and camera positions                       │
│  2. Update shader uniforms once per frame                   │
│  3. GPU calculates per-pixel occlusion in fragment shader   │
│  4. Smooth alpha based on world position relative to player │
└─────────────────────────────────────────────────────────────┘

┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   Camera    │────▶│  Uniform Buffer  │────▶│  Custom Shader  │
│  Position   │     │  (once/frame)    │     │  (per-pixel)    │
└─────────────┘     └──────────────────┘     └─────────────────┘
       │                                              │
       ▼                                              ▼
┌─────────────┐                              ┌─────────────────┐
│   Player    │─────────────────────────────▶│  Fragment Alpha │
│  Position   │                              │  Calculation    │
└─────────────┘                              └─────────────────┘
```

### Shader Logic Overview

```wgsl
// In fragment shader:
// 1. Get fragment world position
// 2. Check if fragment is above player Y
// 3. Calculate distance from camera-player ray (XZ plane)
// 4. Apply smooth falloff for alpha
// 5. Output alpha-blended or dithered result
```

---

## Implementation Steps

### Phase 1: Custom Material Definition

#### Step 1.1: Create Occlusion Material Asset

**File:** `assets/shaders/occlusion_material.wgsl`

```wgsl
#import bevy_pbr::{
    mesh_functions,
    view_transformations::position_world_to_clip,
    pbr_types,
    pbr_functions,
    pbr_bindings,
    mesh_view_bindings::view,
}

// Custom uniforms for occlusion
struct OcclusionUniforms {
    player_position: vec3<f32>,
    camera_position: vec3<f32>,
    min_alpha: f32,
    occlusion_radius: f32,
    height_threshold: f32,
    falloff_softness: f32,
}

@group(2) @binding(100)
var<uniform> occlusion: OcclusionUniforms;

// Vertex output structure
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    #ifdef VERTEX_COLORS
    @location(3) color: vec4<f32>,
    #endif
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    
    let world_position = mesh_functions::mesh_position_local_to_world(
        vertex.position
    );
    
    out.clip_position = position_world_to_clip(world_position.xyz);
    out.world_position = world_position.xyz;
    out.world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal);
    out.uv = vertex.uv;
    
    #ifdef VERTEX_COLORS
    out.color = vertex.color;
    #endif
    
    return out;
}

// Calculate distance from point to ray in XZ plane
fn point_to_ray_distance_xz(point: vec3<f32>, ray_origin: vec3<f32>, ray_dir: vec3<f32>) -> f32 {
    let point_xz = vec2<f32>(point.x, point.z);
    let origin_xz = vec2<f32>(ray_origin.x, ray_origin.z);
    let dir_xz = normalize(vec2<f32>(ray_dir.x, ray_dir.z));
    
    // Handle near-vertical rays
    if length(dir_xz) < 0.001 {
        return length(point_xz - origin_xz);
    }
    
    let to_point = point_xz - origin_xz;
    let projection = dot(to_point, dir_xz);
    let closest_point = origin_xz + dir_xz * projection;
    return length(point_xz - closest_point);
}

// Calculate occlusion alpha for a world position
fn calculate_occlusion_alpha(world_pos: vec3<f32>) -> f32 {
    let player_y = occlusion.player_position.y;
    let fragment_y = world_pos.y;
    
    // Only apply occlusion to fragments above the player
    if fragment_y <= player_y + occlusion.height_threshold {
        return 1.0;
    }
    
    // Calculate ray from camera to player
    let ray_direction = normalize(occlusion.player_position - occlusion.camera_position);
    
    // Distance from fragment to camera-player ray (XZ plane)
    let horizontal_distance = point_to_ray_distance_xz(
        world_pos,
        occlusion.camera_position,
        ray_direction
    );
    
    // Check if within occlusion radius
    if horizontal_distance >= occlusion.occlusion_radius {
        return 1.0;
    }
    
    // Smooth falloff based on distance
    let distance_factor = horizontal_distance / occlusion.occlusion_radius;
    let height_factor = smoothstep(
        player_y + occlusion.height_threshold,
        player_y + occlusion.height_threshold + occlusion.falloff_softness,
        fragment_y
    );
    
    // Combine factors for final alpha
    let base_alpha = mix(occlusion.min_alpha, 1.0, distance_factor);
    return mix(1.0, base_alpha, height_factor);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get base color from vertex colors or material
    #ifdef VERTEX_COLORS
    var base_color = in.color;
    #else
    var base_color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    #endif
    
    // Calculate occlusion alpha
    let occlusion_alpha = calculate_occlusion_alpha(in.world_position);
    
    // Apply basic lighting (simplified PBR)
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let ndotl = max(dot(in.world_normal, light_dir), 0.0);
    let ambient = 0.3;
    let diffuse = ndotl * 0.7;
    
    var final_color = base_color.rgb * (ambient + diffuse);
    
    // Apply occlusion transparency
    return vec4<f32>(final_color, base_color.a * occlusion_alpha);
}
```

#### Step 1.2: Create Custom Material in Rust

**File:** `src/systems/game/occlusion.rs`

```rust
//! Voxel occlusion transparency system using custom shaders.
//!
//! This module provides a custom material that makes voxels transparent
//! when they block the camera's view of the player character.

use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    pbr::{MaterialPipeline, MaterialPipelineKey},
    reflect::TypePath,
};

use super::components::{GameCamera, Player};

/// Shader handle for the occlusion material
pub const OCCLUSION_SHADER_HANDLE: Handle<Shader> = 
    Handle::weak_from_u128(0x8a7b6c5d4e3f2a1b_0c9d8e7f6a5b4c3d);

/// Custom material for voxel occlusion transparency.
/// This material calculates per-pixel transparency based on
/// the fragment's position relative to the player and camera.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct OcclusionMaterial {
    /// Base color (typically white, actual colors come from vertex colors)
    #[uniform(0)]
    pub base_color: LinearRgba,
    
    /// Occlusion uniforms
    #[uniform(100)]
    pub occlusion_uniforms: OcclusionUniforms,
}

/// Uniform buffer for occlusion parameters
#[derive(Clone, Copy, Default, ShaderType)]
pub struct OcclusionUniforms {
    /// Player world position
    pub player_position: Vec3,
    /// Camera world position  
    pub camera_position: Vec3,
    /// Minimum alpha for fully occluded voxels (0.0 = invisible)
    pub min_alpha: f32,
    /// Horizontal radius for occlusion check
    pub occlusion_radius: f32,
    /// Only affect voxels this much above player
    pub height_threshold: f32,
    /// Softness of the vertical falloff
    pub falloff_softness: f32,
}

impl Default for OcclusionMaterial {
    fn default() -> Self {
        Self {
            base_color: LinearRgba::WHITE,
            occlusion_uniforms: OcclusionUniforms {
                player_position: Vec3::ZERO,
                camera_position: Vec3::new(0.0, 10.0, 10.0),
                min_alpha: 0.15,
                occlusion_radius: 3.0,
                height_threshold: 0.5,
                falloff_softness: 2.0,
            },
        }
    }
}

impl Material for OcclusionMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/occlusion_material.wgsl".into()
    }
    
    fn vertex_shader() -> ShaderRef {
        "shaders/occlusion_material.wgsl".into()
    }
    
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
    
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        // Enable vertex colors
        descriptor.vertex.shader_defs.push("VERTEX_COLORS".into());
        Ok(())
    }
}

/// Resource to store the occlusion material handle
#[derive(Resource)]
pub struct OcclusionMaterialHandle(pub Handle<OcclusionMaterial>);

/// Configuration for the occlusion system (exposed for runtime tweaking)
#[derive(Resource)]
pub struct OcclusionConfig {
    /// Minimum alpha for occluded voxels
    pub min_alpha: f32,
    /// Horizontal occlusion radius
    pub occlusion_radius: f32,
    /// Height threshold above player
    pub height_threshold: f32,
    /// Vertical falloff softness
    pub falloff_softness: f32,
    /// Whether occlusion is enabled
    pub enabled: bool,
}

impl Default for OcclusionConfig {
    fn default() -> Self {
        Self {
            min_alpha: 0.15,
            occlusion_radius: 3.0,
            height_threshold: 0.5,
            falloff_softness: 2.0,
            enabled: true,
        }
    }
}

/// System to update occlusion material uniforms each frame.
/// This is O(1) - just updating a uniform buffer, not per-chunk work.
pub fn update_occlusion_uniforms(
    config: Res<OcclusionConfig>,
    camera_query: Query<&Transform, With<GameCamera>>,
    player_query: Query<&Transform, With<Player>>,
    material_handle: Option<Res<OcclusionMaterialHandle>>,
    mut materials: ResMut<Assets<OcclusionMaterial>>,
) {
    // Skip if disabled or resources not ready
    if !config.enabled {
        return;
    }
    
    let Some(material_handle) = material_handle else {
        return;
    };
    
    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };
    
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    
    // Update the material's uniform buffer
    if let Some(material) = materials.get_mut(&material_handle.0) {
        material.occlusion_uniforms = OcclusionUniforms {
            player_position: player_transform.translation,
            camera_position: camera_transform.translation,
            min_alpha: config.min_alpha,
            occlusion_radius: config.occlusion_radius,
            height_threshold: config.height_threshold,
            falloff_softness: config.falloff_softness,
        };
    }
}

/// Plugin to set up the occlusion system
pub struct OcclusionPlugin;

impl Plugin for OcclusionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<OcclusionMaterial>::default())
            .insert_resource(OcclusionConfig::default())
            .add_systems(Update, update_occlusion_uniforms);
    }
}
```

### Phase 2: Integrate with Chunk Spawning

#### Step 2.1: Create Occlusion Material at Map Load

**File:** `src/systems/game/map/spawner.rs`

Modify the chunk spawning to use the occlusion material:

```rust
use super::super::occlusion::{OcclusionMaterial, OcclusionMaterialHandle};

/// Create the occlusion material for all chunks
fn create_occlusion_material(
    materials: &mut Assets<OcclusionMaterial>,
) -> Handle<OcclusionMaterial> {
    materials.add(OcclusionMaterial::default())
}

// In spawn_voxels_chunked, use OcclusionMaterial instead of StandardMaterial:
fn spawn_voxels_chunked(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    occlusion_materials: &mut Assets<OcclusionMaterial>,
    // ... other params
) {
    // Create shared occlusion material (single material for all chunks!)
    let chunk_material = create_occlusion_material(occlusion_materials);
    
    // Store handle as resource for uniform updates
    commands.insert_resource(OcclusionMaterialHandle(chunk_material.clone()));
    
    // Spawn chunks with the shared material
    for (chunk_pos, mesher) in chunk_meshers.into_iter() {
        // ... build mesh ...
        
        commands.spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(chunk_material.clone()),  // Shared material!
            Transform::default(),
            VoxelChunk { chunk_pos, center: chunk_center },
            ChunkLOD { lod_meshes, current_lod: 0 },
        ));
    }
}
```

#### Step 2.2: Register the Plugin

**File:** `src/main.rs`

```rust
use crate::systems::game::occlusion::OcclusionPlugin;

fn main() {
    App::new()
        // ... other plugins ...
        .add_plugins(OcclusionPlugin)
        // ... rest of setup ...
        .run();
}
```

### Phase 3: Register Module

#### Step 3.1: Update Module Exports

**File:** `src/systems/game/mod.rs`

```rust
pub mod components;
pub mod gamepad;
pub mod hot_reload;
pub mod resources;
pub mod systems;

// New focused modules
mod camera;
pub mod character;
mod character_rotation;
mod collision;
mod input;
pub mod map;
pub mod occlusion;  // NEW - public for plugin access
mod physics;
mod player_movement;
```

### Phase 4: Advanced Shader Features (Optional)

#### Step 4.1: Dithered Transparency

For better performance, add screen-space dithering instead of alpha blending:

```wgsl
// Bayer matrix for 4x4 dithering
const BAYER_MATRIX: array<f32, 16> = array<f32, 16>(
    0.0/16.0,  8.0/16.0,  2.0/16.0, 10.0/16.0,
    12.0/16.0, 4.0/16.0, 14.0/16.0,  6.0/16.0,
    3.0/16.0, 11.0/16.0,  1.0/16.0,  9.0/16.0,
    15.0/16.0, 7.0/16.0, 13.0/16.0,  5.0/16.0
);

fn dither_alpha(screen_pos: vec2<f32>, alpha: f32) -> bool {
    let x = u32(screen_pos.x) % 4u;
    let y = u32(screen_pos.y) % 4u;
    let threshold = BAYER_MATRIX[y * 4u + x];
    return alpha > threshold;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // ... calculate occlusion_alpha ...
    
    // Dithered discard for better performance
    if !dither_alpha(in.clip_position.xy, occlusion_alpha) {
        discard;
    }
    
    return vec4<f32>(final_color, 1.0);  // Fully opaque (dithering handles transparency)
}
```

#### Step 4.2: Soft Edge Falloff

Add smoother edges to the occlusion zone:

```wgsl
fn calculate_occlusion_alpha(world_pos: vec3<f32>) -> f32 {
    // ... existing code ...
    
    // Add radial softness at the edge
    let edge_softness = 0.5;
    let soft_distance = smoothstep(
        occlusion.occlusion_radius - edge_softness,
        occlusion.occlusion_radius,
        horizontal_distance
    );
    
    return mix(occlusion.min_alpha, 1.0, soft_distance * height_factor);
}
```

### Phase 5: Debug Visualization

#### Step 5.1: Debug Gizmos System

**File:** `src/systems/game/occlusion.rs`

```rust
/// Debug system to visualize occlusion zone (toggle with key)
pub fn debug_draw_occlusion_zone(
    config: Res<OcclusionConfig>,
    mut gizmos: Gizmos,
    camera_query: Query<&Transform, With<GameCamera>>,
    player_query: Query<&Transform, With<Player>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut show_debug: Local<bool>,
) {
    // Toggle debug view with F3
    if keyboard.just_pressed(KeyCode::F3) {
        *show_debug = !*show_debug;
    }
    
    if !*show_debug {
        return;
    }
    
    let Ok(camera) = camera_query.get_single() else { return };
    let Ok(player) = player_query.get_single() else { return };
    
    // Draw ray from camera to player
    gizmos.line(
        camera.translation,
        player.translation,
        Color::srgba(1.0, 1.0, 0.0, 0.8),
    );
    
    // Draw occlusion cylinder above player
    let cylinder_center = player.translation + Vec3::Y * (config.height_threshold + 2.0);
    gizmos.circle(
        Isometry3d::new(cylinder_center, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        config.occlusion_radius,
        Color::srgba(1.0, 0.0, 0.0, 0.5),
    );
    
    // Draw height threshold line
    let threshold_y = player.translation.y + config.height_threshold;
    gizmos.line(
        Vec3::new(player.translation.x - 2.0, threshold_y, player.translation.z),
        Vec3::new(player.translation.x + 2.0, threshold_y, player.translation.z),
        Color::srgba(0.0, 1.0, 0.0, 0.8),
    );
}
```

---

## Alternative Approaches

### Alternative 1: Shader-Based Occlusion

Instead of per-chunk materials, use a custom shader that:
1. Receives player position as a uniform
2. Calculates occlusion in the fragment shader
3. Applies transparency based on fragment position relative to player

**Pros:**
- Single material for all chunks (GPU efficient)
- Smooth per-pixel transparency
- No material updates needed

**Cons:**
- Requires custom shader (more complex)
- May not work well with Bevy's PBR pipeline
- Harder to debug

### Alternative 2: Render Order / Cutout

Instead of transparency, simply don't render occluding voxels:
1. Set `Visibility::Hidden` on occluding chunks
2. Use a cutout circle around the player

**Pros:**
- Simplest implementation
- No alpha blending overhead

**Cons:**
- Jarring visual effect (chunks pop in/out)
- Loses spatial context

### Alternative 3: Slice View

Render only voxels at or below the player's Y level:

**Pros:**
- Clean visual separation
- Common in building games (Rimworld, Prison Architect)

**Cons:**
- May not fit game's visual style
- Loses sense of 3D structure above player

---

## Testing Plan

### Unit Tests

1. **Ray-point distance calculation** - Verify `calculate_point_to_ray_distance_xz`
2. **Alpha interpolation** - Test smooth transitions

### Integration Tests

1. **Chunk occlusion detection** - Walk under structures, verify chunks fade
2. **Performance** - Measure frame time with many chunks
3. **Edge cases**:
   - Player at map edge
   - Very tall structures
   - Moving camera rotation

### Manual Testing Scenarios

1. Walk under a single-voxel bridge
2. Walk under a multi-story building
3. Rotate camera while under structure
4. Enter and exit caves/tunnels
5. Stand at the edge of an overhang

---

## Performance Considerations

### Memory Impact

- **Single shared material**: Minimal overhead (~256 bytes for uniforms)
- **No per-chunk materials**: Maintains current memory efficiency

### CPU Impact

- Uniform update: O(1) per frame - just copy 6 floats
- No per-chunk iteration required
- Expected: <0.1ms total

### GPU Impact

- Small per-fragment shader cost for occlusion calculation
- Maintains GPU instancing (all chunks batched)
- Alpha blending overhead (or use dithering to avoid)
- No sorting required with dithered approach

### Optimizations

1. **Dithered transparency** - Avoid alpha blending entirely with screen-space dithering
2. **Early fragment discard** - Skip occlusion math for fragments clearly not in occlusion zone
3. **Uniform batching** - All occlusion data in single uniform buffer
4. **LOD-aware** - Could simplify occlusion for distant LOD levels

---

## Future Enhancements

1. **Per-voxel transparency** - Fade individual voxels instead of whole chunks
2. **X-ray effect** - Show wireframe instead of transparency
3. **Configurable per map** - Some maps may want different occlusion behavior
4. **Editor support** - Add occlusion preview in map editor
5. **Shadow handling** - Transparent voxels could still cast shadows

---

## Checklist

- [ ] Create shader file `assets/shaders/occlusion_material.wgsl`
- [ ] Create `OcclusionMaterial` custom material type
- [ ] Create `OcclusionUniforms` struct for shader data
- [ ] Implement `update_occlusion_uniforms` system
- [ ] Create `OcclusionPlugin` for easy integration
- [ ] Modify chunk spawning to use `OcclusionMaterial`
- [ ] Store `OcclusionMaterialHandle` as resource
- [ ] Register occlusion module in `mod.rs`
- [ ] Add `OcclusionPlugin` to app
- [ ] Add `OcclusionConfig` resource for runtime tweaking
- [ ] Implement dithered transparency option (Phase 4)
- [ ] Add debug visualization (F3 toggle)
- [ ] Test with various map layouts
- [ ] Performance profiling (should be O(1) per frame)
- [ ] Documentation update

---

## References

- Bevy Alpha Blending: https://docs.rs/bevy/latest/bevy/pbr/struct.StandardMaterial.html
- Isometric Game Occlusion Techniques: Common in games like Divinity: Original Sin, Diablo series
- Three.js Transparency: Used as reference for smooth transitions
