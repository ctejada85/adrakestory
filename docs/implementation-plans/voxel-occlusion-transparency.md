# Voxel Occlusion Transparency System

## Overview

Implement a system that makes voxels transparent or invisible when they occlude (block the view of) the player character. This is a common feature in isometric and 3D games that ensures players can always see their character when walking under structures, bridges, roofs, or other overhead voxels.

**Status**: 📋 Planned  
**Priority**: Medium  
**Estimated Effort**: 3-5 days  
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
- `src/systems/game/map/spawner.rs` - Chunk spawning and material creation
- `src/systems/game/components.rs` - New components for occlusion tracking
- `src/systems/game/mod.rs` - Module registration
- `src/systems/game/camera.rs` - Camera position data (read only)
- New: `src/systems/game/occlusion.rs` - Occlusion detection and transparency system

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

## Proposed Solution: Per-Chunk Material Instances

### Rationale

While per-chunk materials use more memory, the current chunk count is manageable (typically <1000 chunks). This approach:
- Works with Bevy's existing material system
- Allows smooth per-chunk alpha transitions
- Is simpler to implement than custom shaders
- Can be optimized later with instancing if needed

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Occlusion System                         │
├─────────────────────────────────────────────────────────────┤
│  1. Calculate occlusion ray (camera → player)               │
│  2. Find chunks intersecting the ray                        │
│  3. Mark chunks for transparency                            │
│  4. Smoothly animate alpha values                           │
│  5. Update chunk material alpha                             │
└─────────────────────────────────────────────────────────────┘

┌─────────────┐     ┌──────────────┐     ┌─────────────────┐
│   Camera    │────▶│   Occlusion  │────▶│  Chunk Materials │
│  Position   │     │    Check     │     │  (Alpha Update)  │
└─────────────┘     └──────────────┘     └─────────────────┘
       │                   │
       ▼                   ▼
┌─────────────┐     ┌──────────────┐
│   Player    │────▶│   Ray Cast   │
│  Position   │     │   Through    │
└─────────────┘     │   Chunks     │
                    └──────────────┘
```

---

## Implementation Steps

### Phase 1: Per-Chunk Material System

#### Step 1.1: Add Occlusion Component

**File:** `src/systems/game/components.rs`

```rust
/// Component for tracking chunk occlusion state.
/// Attached to VoxelChunk entities to manage transparency.
#[derive(Component)]
pub struct ChunkOcclusion {
    /// Current alpha value (0.0 = invisible, 1.0 = opaque)
    pub current_alpha: f32,
    /// Target alpha value for smooth transitions
    pub target_alpha: f32,
    /// Handle to this chunk's unique material
    pub material_handle: Handle<StandardMaterial>,
}

impl Default for ChunkOcclusion {
    fn default() -> Self {
        Self {
            current_alpha: 1.0,
            target_alpha: 1.0,
            material_handle: Handle::default(),
        }
    }
}
```

#### Step 1.2: Modify Chunk Spawning

**File:** `src/systems/game/map/spawner.rs`

Replace the shared material with per-chunk materials:

```rust
// Create a unique material for this chunk (cloned from base)
let chunk_material = ctx.materials.add(StandardMaterial {
    base_color: Color::WHITE,
    alpha_mode: AlphaMode::Blend,  // Enable transparency
    ..ctx.base_material.clone()
});

ctx.commands.spawn((
    Mesh3d(lod_meshes[0].clone()),
    MeshMaterial3d(chunk_material.clone()),
    Transform::default(),
    VoxelChunk { chunk_pos, center: chunk_center },
    ChunkLOD { lod_meshes, current_lod: 0 },
    ChunkOcclusion {
        current_alpha: 1.0,
        target_alpha: 1.0,
        material_handle: chunk_material,
    },
));
```

### Phase 2: Occlusion Detection System

#### Step 2.1: Create Occlusion Module

**File:** `src/systems/game/occlusion.rs`

```rust
//! Voxel occlusion transparency system.
//!
//! This module handles making voxels transparent when they block
//! the camera's view of the player character.

use super::components::{ChunkOcclusion, GameCamera, Player};
use super::map::spawner::VoxelChunk;
use bevy::prelude::*;

/// Configuration for the occlusion system
pub struct OcclusionConfig {
    /// Minimum alpha for occluding chunks (0.0 = invisible)
    pub min_alpha: f32,
    /// Speed of alpha transitions (higher = faster fade)
    pub transition_speed: f32,
    /// Vertical threshold - only check chunks above player
    pub height_threshold: f32,
    /// Horizontal radius for occlusion check
    pub occlusion_radius: f32,
}

impl Default for OcclusionConfig {
    fn default() -> Self {
        Self {
            min_alpha: 0.2,        // Semi-transparent, not fully invisible
            transition_speed: 8.0, // Fast but smooth transition
            height_threshold: 0.5, // Only chunks 0.5+ units above player
            occlusion_radius: 2.0, // Check chunks within 2 unit radius
        }
    }
}

impl Resource for OcclusionConfig {}

/// System to detect which chunks are occluding the player.
/// Runs every frame to update chunk target alpha values.
pub fn detect_chunk_occlusion(
    config: Res<OcclusionConfig>,
    camera_query: Query<&Transform, With<GameCamera>>,
    player_query: Query<&Transform, With<Player>>,
    mut chunk_query: Query<(&VoxelChunk, &mut ChunkOcclusion)>,
) {
    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let camera_pos = camera_transform.translation;
    let player_pos = player_transform.translation;

    // Calculate the ray direction from camera to player
    let ray_direction = (player_pos - camera_pos).normalize();

    for (chunk, mut occlusion) in chunk_query.iter_mut() {
        // Check if chunk is between camera and player (vertically)
        let chunk_y = chunk.center.y;
        let player_y = player_pos.y;

        // Only consider chunks above the player
        if chunk_y <= player_y + config.height_threshold {
            occlusion.target_alpha = 1.0;
            continue;
        }

        // Check horizontal proximity to the camera-player line
        let horizontal_distance = calculate_point_to_ray_distance_xz(
            chunk.center,
            camera_pos,
            ray_direction,
        );

        if horizontal_distance < config.occlusion_radius {
            // Chunk is occluding - fade to transparent
            // Closer chunks become more transparent
            let alpha = config.min_alpha + 
                (horizontal_distance / config.occlusion_radius) * (1.0 - config.min_alpha);
            occlusion.target_alpha = alpha.clamp(config.min_alpha, 1.0);
        } else {
            // Chunk is not occluding - restore opacity
            occlusion.target_alpha = 1.0;
        }
    }
}

/// Calculate the distance from a point to a ray in the XZ plane (ignoring Y).
fn calculate_point_to_ray_distance_xz(
    point: Vec3,
    ray_origin: Vec3,
    ray_direction: Vec3,
) -> f32 {
    // Project to XZ plane
    let point_xz = Vec2::new(point.x, point.z);
    let origin_xz = Vec2::new(ray_origin.x, ray_origin.z);
    let dir_xz = Vec2::new(ray_direction.x, ray_direction.z).normalize_or_zero();

    if dir_xz.length_squared() < 0.001 {
        // Ray is nearly vertical - use simple distance
        return (point_xz - origin_xz).length();
    }

    // Calculate perpendicular distance from point to ray line
    let to_point = point_xz - origin_xz;
    let projection = to_point.dot(dir_xz);
    let closest_point = origin_xz + dir_xz * projection;
    (point_xz - closest_point).length()
}

/// System to smoothly animate chunk alpha values.
/// Interpolates current_alpha toward target_alpha each frame.
pub fn animate_chunk_alpha(
    time: Res<Time>,
    config: Res<OcclusionConfig>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut chunk_query: Query<&mut ChunkOcclusion>,
) {
    let delta = time.delta_secs();

    for mut occlusion in chunk_query.iter_mut() {
        // Skip if already at target
        if (occlusion.current_alpha - occlusion.target_alpha).abs() < 0.001 {
            continue;
        }

        // Smoothly interpolate toward target
        occlusion.current_alpha = occlusion.current_alpha
            + (occlusion.target_alpha - occlusion.current_alpha) 
            * config.transition_speed * delta;

        // Update the material's alpha
        if let Some(material) = materials.get_mut(&occlusion.material_handle) {
            material.base_color = material.base_color.with_alpha(occlusion.current_alpha);
        }
    }
}
```

#### Step 2.2: Register Occlusion Module

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
mod occlusion;  // NEW
mod physics;
mod player_movement;
```

#### Step 2.3: Add Systems to Game Loop

**File:** `src/main.rs` or where systems are registered

```rust
// Add occlusion systems to the Update schedule
app.insert_resource(OcclusionConfig::default())
   .add_systems(
       Update,
       (
           detect_chunk_occlusion,
           animate_chunk_alpha,
       )
       .chain()
       .run_if(in_state(GameState::Playing)),
   );
```

### Phase 3: Visual Refinements

#### Step 3.1: Improve Transparency Look

For better visual quality, consider these material settings:

```rust
let chunk_material = ctx.materials.add(StandardMaterial {
    base_color: Color::WHITE,
    alpha_mode: AlphaMode::Blend,
    // Keep shadows even when transparent
    cull_mode: None,  // Render both faces when transparent
    // Adjust depth handling for transparency
    depth_bias: -0.1,
    ..default()
});
```

#### Step 3.2: Optional - Dithered Transparency

For better performance (avoid alpha blending overhead), use dithered transparency:

```rust
alpha_mode: AlphaMode::Mask(0.5),  // Binary transparency with dithering
```

This requires a custom shader to implement proper screen-space dithering.

### Phase 4: Configuration & Debugging

#### Step 4.1: Add Debug Visualization (Optional)

```rust
/// Debug system to visualize occlusion rays
pub fn debug_draw_occlusion_rays(
    mut gizmos: Gizmos,
    camera_query: Query<&Transform, With<GameCamera>>,
    player_query: Query<&Transform, With<Player>>,
    chunk_query: Query<(&VoxelChunk, &ChunkOcclusion)>,
) {
    if let (Ok(camera), Ok(player)) = (camera_query.get_single(), player_query.get_single()) {
        // Draw ray from camera to player
        gizmos.line(
            camera.translation,
            player.translation,
            Color::srgba(1.0, 1.0, 0.0, 0.5),
        );

        // Highlight occluded chunks
        for (chunk, occlusion) in chunk_query.iter() {
            if occlusion.target_alpha < 1.0 {
                gizmos.sphere(
                    chunk.center,
                    0.5,
                    Color::srgba(1.0, 0.0, 0.0, 0.3),
                );
            }
        }
    }
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

- **Per-chunk materials**: ~64 bytes per chunk × 1000 chunks = 64KB additional
- Acceptable for current map sizes

### CPU Impact

- Occlusion detection: O(chunks) per frame
- Alpha animation: O(occluded_chunks) per frame
- Expected: <0.5ms total with 1000 chunks

### GPU Impact

- Alpha blending has some overhead
- Consider dithered alpha for better performance
- Transparent chunks render after opaque (sorting cost)

### Optimizations

1. **Spatial partitioning** - Only check chunks near player
2. **Caching** - Skip chunks that haven't changed
3. **LOD-aware** - Disable occlusion for distant LOD chunks
4. **Batch updates** - Update materials only when alpha changes significantly

---

## Future Enhancements

1. **Per-voxel transparency** - Fade individual voxels instead of whole chunks
2. **X-ray effect** - Show wireframe instead of transparency
3. **Configurable per map** - Some maps may want different occlusion behavior
4. **Editor support** - Add occlusion preview in map editor
5. **Shadow handling** - Transparent voxels could still cast shadows

---

## Checklist

- [ ] Add `ChunkOcclusion` component to `components.rs`
- [ ] Create `occlusion.rs` module
- [ ] Implement `detect_chunk_occlusion` system
- [ ] Implement `animate_chunk_alpha` system
- [ ] Modify chunk spawning for per-chunk materials
- [ ] Register occlusion systems in game loop
- [ ] Add `OcclusionConfig` resource
- [ ] Test with various map layouts
- [ ] Performance profiling
- [ ] Documentation update

---

## References

- Bevy Alpha Blending: https://docs.rs/bevy/latest/bevy/pbr/struct.StandardMaterial.html
- Isometric Game Occlusion Techniques: Common in games like Divinity: Original Sin, Diablo series
- Three.js Transparency: Used as reference for smooth transitions
