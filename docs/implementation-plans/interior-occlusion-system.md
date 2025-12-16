# Interior Occlusion System

## Overview

Implement a system that detects when the player enters an interior space (house, room, cave, etc.) and automatically hides or fades the voxels forming the ceiling/roof of that space. Unlike the current per-pixel shader-based occlusion, this approach identifies **connected horizontal regions** above the player and hides them as a unit.

**Status**: 📋 Planned  
**Priority**: Medium  
**Estimated Effort**: 5-8 days  
**Last Updated**: 2025-12-16

---

## Problem Statement

### Current Behavior

The existing `OcclusionMaterial` shader provides per-pixel transparency based on distance from the camera-player ray. While effective for small overhangs, it has limitations:

1. **Radius-based**: Only affects voxels within a fixed horizontal radius
2. **No room awareness**: Doesn't understand architectural boundaries
3. **Partial transparency**: Shows all ceiling voxels as semi-transparent instead of fully hiding them
4. **No shadow handling**: Shadows from transparent voxels are still visible

### Desired Behavior

When the player enters an interior:
1. Detect that the player is "under" a ceiling structure
2. Identify all voxels that belong to that ceiling/roof region (flood-fill horizontally)
3. Hide those voxels completely (and optionally their shadows)
4. Restore visibility when the player exits

This creates a "cutaway" effect common in isometric games like The Sims, Divinity: Original Sin, and Baldur's Gate 3.

---

## Approach Comparison

### Approach 1: Region-Based Occlusion (Flood-Fill)

**Concept**: When player is under a voxel, flood-fill horizontally from that voxel to find all connected ceiling voxels at the same Y level (±tolerance), then hide the entire region.

```
┌─────────────────────────────────────────────┐
│     Before: Player enters building          │
│     ████████████████ (roof visible)        │
│     █              █                        │
│     █    Player    █                        │
│     █      ↓       █                        │
│     ████████████████ (floor)               │
├─────────────────────────────────────────────┤
│     After: Roof region detected and hidden  │
│     ................. (roof hidden)        │
│     █              █                        │
│     █    Player    █ (walls visible)       │
│     █              █                        │
│     ████████████████ (floor visible)       │
└─────────────────────────────────────────────┘
```

**Algorithm**:
1. Cast ray upward from player position
2. If ray hits voxel within threshold distance → player is "inside"
3. Flood-fill from hit voxel in XZ plane to find connected ceiling region
4. Mark all voxels in region as hidden
5. Update when player moves to new region or exits

**Pros**:
- ✅ Natural "room" detection based on actual geometry
- ✅ Entire ceiling disappears at once (clean visual)
- ✅ Works with irregular room shapes
- ✅ Can handle multi-level buildings (check multiple Y levels)
- ✅ Caches region data for performance

**Cons**:
- ❌ Flood-fill has computational cost (mitigated by caching)
- ❌ Needs careful definition of "connected" (diagonal? gaps?)
- ❌ Large open roofs could hide too much
- ❌ Requires CPU-side voxel data access

**Implementation Complexity**: Medium-High

---

### Approach 2: Interior Zones (Pre-Defined Regions)

**Concept**: Define interior zones in the map format. When player enters a zone, hide associated voxels.

```ron
// In map RON file
interior_zones: [
    InteriorZone {
        bounds: Aabb { min: (10, 0, 5), max: (20, 5, 15) },
        ceiling_y: 4,
        name: "house_main_room",
    },
]
```

**Algorithm**:
1. Check if player position is inside any interior zone
2. If inside, hide all voxels above `ceiling_y` within zone bounds
3. Exit zone → restore visibility

**Pros**:
- ✅ Designer control over exactly what's hidden
- ✅ Very fast runtime (simple AABB check)
- ✅ Can have different behaviors per zone
- ✅ Works with any geometry complexity

**Cons**:
- ❌ Manual setup required for each interior
- ❌ Map format changes needed
- ❌ Editor support needed for zone placement
- ❌ Doesn't adapt to dynamic geometry changes

**Implementation Complexity**: Medium (map format + editor + runtime)

---

### Approach 3: Depth-Based Ceiling Detection

**Concept**: Use depth information from the camera's perspective. Voxels that are between the camera and a "floor" plane under the player are potential ceilings.

**Algorithm**:
1. Define invisible floor plane at player's feet
2. In shader: if fragment is above player AND between camera and floor plane → fade/hide
3. Use depth buffer or analytical calculation

**Pros**:
- ✅ Entirely GPU-based (fast)
- ✅ No pre-defined regions needed
- ✅ Smooth transitions possible
- ✅ Works with any camera angle

**Cons**:
- ❌ Complex shader math for proper depth calculation
- ❌ Doesn't respect architectural boundaries
- ❌ May hide walls that shouldn't be hidden
- ❌ Difficult to handle shadows separately

**Implementation Complexity**: High (shader complexity)

---

### Approach 4: Hybrid Shader + Region Detection

**Concept**: Combine the current shader-based approach with CPU-side region detection.

1. **CPU detects** if player is in an interior (ray cast + flood-fill)
2. **CPU calculates** the bounding region of the ceiling
3. **Shader receives** region bounds as uniforms
4. **Shader hides** fragments within the region bounds

```wgsl
struct InteriorRegion {
    min_bounds: vec3<f32>,
    max_bounds: vec3<f32>,
    is_active: u32,
}

@fragment
fn fragment(in: VertexOutput) -> FragmentOutput {
    // Check if fragment is in active interior region
    if interior.is_active == 1u {
        if in_bounds(in.world_position, interior.min_bounds, interior.max_bounds) {
            discard; // Or set alpha = 0
        }
    }
    // ... rest of rendering
}
```

**Pros**:
- ✅ Best of both worlds: smart detection + fast rendering
- ✅ Region bounds can be cached until player moves significantly
- ✅ Shader handles the actual hiding (efficient)
- ✅ Can fall back to current approach when not in interior

**Cons**:
- ❌ Requires both CPU and GPU work
- ❌ More complexity than pure approaches
- ❌ Region updates need synchronization

**Implementation Complexity**: Medium-High

---

### Approach 5: Voxel Tagging with Structure IDs

**Concept**: Each voxel can have an optional `structure_id`. All voxels with the same ID form a "structure" (building, room). When player is under any voxel of a structure, hide all ceiling voxels of that structure.

```ron
voxels: [
    Voxel { pos: (5, 2, 5), pattern: Full, structure_id: Some(1) }, // roof
    Voxel { pos: (5, 0, 5), pattern: Full, structure_id: Some(1) }, // floor
    Voxel { pos: (4, 1, 5), pattern: Full, structure_id: Some(1) }, // wall
]
```

**Algorithm**:
1. When player enters voxel space, check if any voxel above has `structure_id`
2. If yes, collect all voxels with same `structure_id` that are "ceiling" (above player Y)
3. Hide those voxels

**Pros**:
- ✅ Explicit designer control
- ✅ Can group non-contiguous voxels (split-level buildings)
- ✅ Fast lookup if indexed by structure_id
- ✅ Integrates with map format naturally

**Cons**:
- ❌ Requires tagging every voxel (tedious)
- ❌ Map format changes
- ❌ Editor tools needed for efficient tagging

**Implementation Complexity**: Medium

---

## Recommendation

### Primary Recommendation: **Hybrid Shader + Region Detection** (Approach 4)

This approach provides the best balance of:
- **Automatic detection** (no manual zone setup)
- **Performance** (GPU-based hiding)
- **Architectural awareness** (flood-fill respects room boundaries)
- **Configurability** (fall back to current shader when outside interiors)

### Implementation Strategy

1. **Phase 1**: Add region detection system (CPU-side flood-fill)
2. **Phase 2**: Extend shader with region bounds uniform
3. **Phase 3**: Add shadow handling for hidden regions
4. **Phase 4**: Configuration system for occlusion mode selection

---

## Occlusion Mode Configuration

Add configuration to select occlusion technique:

```rust
/// Occlusion technique for handling overhead voxels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OcclusionMode {
    /// No occlusion - voxels always visible
    None,
    
    /// Current shader-based: per-pixel transparency based on camera-player ray
    /// Best for: Outdoor areas, small overhangs
    #[default]
    ShaderBased,
    
    /// Region-based: detect connected ceiling regions and hide entirely
    /// Best for: Buildings, rooms, caves
    RegionBased,
    
    /// Hybrid: Use region detection when inside, shader-based when outside
    /// Best for: Mixed environments
    Hybrid,
    
    /// Pre-defined zones: Use map-defined interior zones
    /// Best for: Designer-controlled experiences
    ZoneBased,
}

#[derive(Resource)]
pub struct OcclusionConfig {
    // ... existing fields ...
    
    /// Which occlusion technique to use
    pub mode: OcclusionMode,
    
    /// Height threshold for "inside" detection (how close ceiling must be)
    pub interior_height_threshold: f32,
    
    /// Whether to hide shadows of occluded voxels
    pub hide_shadows: bool,
    
    /// Update frequency for region detection (frames between updates)
    pub region_update_interval: u32,
}
```

---

## Detailed Implementation Plan

### Phase 1: Interior Detection System

**File**: `src/systems/game/interior_detection.rs`

```rust
/// Represents a detected interior region
#[derive(Debug, Clone)]
pub struct InteriorRegion {
    /// AABB bounds of the ceiling region
    pub bounds: Aabb,
    /// Y level of the ceiling
    pub ceiling_y: f32,
    /// Set of chunk positions affected
    pub affected_chunks: HashSet<IVec3>,
    /// Entity IDs of voxels to hide (if tracking individually)
    pub hidden_voxels: Vec<Entity>,
}

/// Resource tracking current interior state
#[derive(Resource, Default)]
pub struct InteriorState {
    /// Currently detected interior region (None if player is outside)
    pub current_region: Option<InteriorRegion>,
    /// Last player position used for detection (avoid recalculating every frame)
    pub last_detection_pos: Vec3,
    /// Frame counter for throttled updates
    pub frames_since_update: u32,
}

/// System to detect if player is inside an interior
pub fn detect_interior_system(
    config: Res<OcclusionConfig>,
    player_query: Query<&Transform, With<Player>>,
    spatial_grid: Res<SpatialGrid>,
    sub_voxels: Query<(&SubVoxel, &Transform)>,
    mut interior_state: ResMut<InteriorState>,
) {
    // Throttle updates based on config
    interior_state.frames_since_update += 1;
    if interior_state.frames_since_update < config.region_update_interval {
        return;
    }
    
    // Only run for region-based or hybrid modes
    if !matches!(config.mode, OcclusionMode::RegionBased | OcclusionMode::Hybrid) {
        return;
    }
    
    let Ok(player_transform) = player_query.get_single() else { return };
    let player_pos = player_transform.translation;
    
    // Skip if player hasn't moved significantly
    if player_pos.distance(interior_state.last_detection_pos) < 0.5 {
        return;
    }
    
    interior_state.frames_since_update = 0;
    interior_state.last_detection_pos = player_pos;
    
    // Cast ray upward from player to find ceiling
    let ceiling_hit = find_ceiling_above(
        player_pos,
        config.interior_height_threshold,
        &spatial_grid,
        &sub_voxels,
    );
    
    if let Some((ceiling_pos, ceiling_y)) = ceiling_hit {
        // Flood-fill to find connected ceiling region
        let region = flood_fill_ceiling_region(
            ceiling_pos,
            ceiling_y,
            &spatial_grid,
            &sub_voxels,
        );
        interior_state.current_region = Some(region);
    } else {
        interior_state.current_region = None;
    }
}

/// Find ceiling voxel directly above player within threshold
fn find_ceiling_above(
    player_pos: Vec3,
    max_height: f32,
    spatial_grid: &SpatialGrid,
    sub_voxels: &Query<(&SubVoxel, &Transform)>,
) -> Option<(IVec3, f32)> {
    // Ray cast upward from player position
    // Check voxels in column above player
    // Return first hit within threshold
    todo!()
}

/// Flood-fill to find all connected ceiling voxels at same Y level
fn flood_fill_ceiling_region(
    start_pos: IVec3,
    ceiling_y: f32,
    spatial_grid: &SpatialGrid,
    sub_voxels: &Query<(&SubVoxel, &Transform)>,
) -> InteriorRegion {
    // BFS/DFS from start position
    // Expand in XZ plane to find all connected voxels at ceiling_y (±tolerance)
    // Track bounds and affected chunks
    todo!()
}
```

### Phase 2: Shader Extension for Region Bounds

**File**: `assets/shaders/occlusion_material.wgsl`

Add region-based occlusion to existing shader:

```wgsl
struct OcclusionUniforms {
    // ... existing fields ...
    
    // Interior region bounds (active when w > 0)
    region_min: vec4<f32>,  // xyz = min bounds, w = unused
    region_max: vec4<f32>,  // xyz = max bounds, w = is_active (1.0 = active)
}

fn in_interior_region(world_pos: vec3<f32>) -> bool {
    if occlusion.region_max.w < 0.5 {
        return false;  // No active region
    }
    
    return world_pos.x >= occlusion.region_min.x &&
           world_pos.x <= occlusion.region_max.x &&
           world_pos.y >= occlusion.region_min.y &&
           world_pos.y <= occlusion.region_max.y &&
           world_pos.z >= occlusion.region_min.z &&
           world_pos.z <= occlusion.region_max.z;
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    // Check region-based occlusion first
    if in_interior_region(in.world_position.xyz) {
        discard;  // Completely hide voxels in interior region
    }
    
    // ... rest of existing shader code ...
}
```

### Phase 3: Shadow Handling

Two approaches for shadows:

#### Option A: Dual Material System

- Use `OcclusionMaterial` for visible rendering
- Use separate shadow-only material that also checks region bounds
- Requires Bevy shadow material customization

#### Option B: Shadow Map Masking

- Render interior region to a mask texture
- In shadow pass, skip fragments that are in the mask
- More complex but more flexible

#### Option C: Visibility Component (Simpler)

- When in interior, set `Visibility::Hidden` on affected chunk entities
- Shadows automatically disappear with visibility
- Requires per-chunk tracking (breaks single-material optimization)

**Recommendation**: Start with Option C (visibility-based) for simplicity, optimize later if needed.

### Phase 4: Configuration System

**File**: `src/systems/game/occlusion.rs`

Extend `OcclusionConfig`:

```rust
impl Default for OcclusionConfig {
    fn default() -> Self {
        Self {
            // ... existing ...
            mode: OcclusionMode::Hybrid,
            interior_height_threshold: 8.0,  // Max ceiling height to trigger interior mode
            hide_shadows: true,
            region_update_interval: 10,  // Update every 10 frames (~6 times/sec at 60fps)
        }
    }
}
```

---

## Alternative Consideration: Slice/Layer View

Some games (The Sims, Dwarf Fortress, Prison Architect) use a **slice view** where everything above a certain Y level is hidden:

```
Layer view controls:
- Page Up: Raise visible layer
- Page Down: Lower visible layer
- Home: Show all layers

When layer is set, all voxels above that Y are hidden.
```

**Pros**:
- Very simple to implement (single Y threshold)
- Gives player control
- Works perfectly for multi-story buildings

**Cons**:
- Requires player interaction
- Not automatic
- May hide things player wants to see

**Could be added as additional mode**: `OcclusionMode::LayerSlice`

---

## Testing Plan

### Unit Tests

1. **Flood-fill algorithm**: Test with various room shapes
2. **Ray casting**: Verify ceiling detection accuracy
3. **Region bounds**: Test edge cases (player at boundary)

### Integration Tests

1. Walk into simple box room → ceiling hides
2. Walk out of room → ceiling appears
3. Multi-room building → only current room ceiling hides
4. Irregular room shape → correct region detected
5. Performance with large regions

### Manual Testing Scenarios

1. **Simple house**: 4 walls + roof
2. **Multi-room building**: Hallway connecting rooms
3. **Multi-story**: Upper floor becomes ceiling for lower floor
4. **Cave**: Irregular ceiling shape
5. **Open courtyard**: Building with open center (shouldn't hide sky)
6. **Overlapping structures**: Multiple ceilings at different heights

---

## Performance Considerations

### Flood-Fill Cost

- **Worst case**: O(n) where n = voxels in region
- **Mitigation**: 
  - Cache region until player moves significantly
  - Limit max region size
  - Use spatial partitioning for faster neighbor lookup

### Shader Cost

- **Additional per-fragment**: ~5 ALU ops for bounds check
- **Negligible** compared to PBR lighting calculations

### Memory

- `InteriorRegion`: ~200-500 bytes depending on voxel count
- Single region cached, minimal overhead

### Recommended Limits

```rust
const MAX_REGION_SIZE: usize = 10000;  // Max voxels in flood-fill
const MIN_UPDATE_INTERVAL: u32 = 5;    // Minimum frames between updates
const POSITION_THRESHOLD: f32 = 0.5;   // Player movement threshold for re-detection
```

---

## Future Enhancements

1. **Multiple active regions**: Support nested buildings
2. **Gradual fade**: Smooth transition when entering/exiting
3. **Wall transparency**: Optionally fade walls facing camera
4. **Minimap integration**: Show room outlines on minimap
5. **Editor preview**: Show interior detection in map editor
6. **Per-map config**: Different occlusion modes per map
7. **Animated transition**: Roof "slides away" animation

---

## Checklist

### Phase 1: Interior Detection
- [ ] Create `interior_detection.rs` module
- [ ] Implement `InteriorRegion` struct
- [ ] Implement `InteriorState` resource
- [ ] Implement upward ray casting for ceiling detection
- [ ] Implement flood-fill algorithm for region detection
- [ ] Add `detect_interior_system` to game systems
- [ ] Add throttling based on config

### Phase 2: Shader Extension
- [ ] Add region bounds to `OcclusionUniforms`
- [ ] Update shader with `in_interior_region` function
- [ ] Add discard for region-based hiding
- [ ] Update `update_occlusion_uniforms` to include region bounds

### Phase 3: Shadow Handling
- [ ] Evaluate shadow handling approaches
- [ ] Implement chosen approach (visibility-based recommended)
- [ ] Test shadow behavior in interiors

### Phase 4: Configuration
- [ ] Add `OcclusionMode` enum
- [ ] Extend `OcclusionConfig` with new fields
- [ ] Add runtime switching between modes
- [ ] Update debug visualization for regions

### Testing & Polish
- [ ] Write unit tests for flood-fill
- [ ] Write integration tests for interior detection
- [ ] Manual testing with various room configurations
- [ ] Performance profiling
- [ ] Documentation update

---

## References

- **Baldur's Gate 3**: Uses region-based roof hiding with smooth transitions
- **The Sims**: Layer-based visibility with manual control
- **Divinity: Original Sin**: Combination of automatic detection and designer zones
- **Isometric Game Design**: https://www.gamedeveloper.com/design/the-art-of-isometric-level-design

---

## Summary Table

| Approach | Complexity | Performance | Designer Control | Automatic | Recommended For |
|----------|------------|-------------|------------------|-----------|-----------------|
| Flood-Fill (1) | Medium-High | Good (cached) | None | Yes | Procedural/simple buildings |
| Pre-defined Zones (2) | Medium | Excellent | Full | No | Hand-crafted levels |
| Depth-Based (3) | High | Excellent | None | Yes | Not recommended |
| **Hybrid (4)** | Medium-High | Good | Some | Yes | **General use** |
| Structure Tags (5) | Medium | Good | Full | Partial | Complex buildings |

**Final Recommendation**: Implement **Hybrid (Approach 4)** with **Flood-Fill detection** + **Shader-based hiding**, keeping the current `OcclusionMode` configurable so existing shader-based approach remains available.
