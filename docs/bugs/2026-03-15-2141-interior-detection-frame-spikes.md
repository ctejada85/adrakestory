# Bug: Interior Detection BFS Flood-Fill Creates Periodic Frame Spikes

**Date:** 2026-03-15  
**Severity:** High  
**Status:** Open  
**Component:** Occlusion / Interior Detection  

---

## Description

The `detect_interior_system` performs a BFS flood-fill over up to 1,000 voxels every 10 frames to detect ceiling regions for room-based occlusion. At 60 fps this fires every ~167ms and can take 5–50ms depending on map complexity, creating visible periodic stutter in the game. The system also conditionally rebuilds a full voxel occupancy `HashSet` by iterating the entire `SpatialGrid`, compounding the cost.

---

## Actual Behavior

- Every 10 frames, a BFS flood-fill executes across up to `MAX_REGION_SIZE` (1,000) voxel positions
- When the spatial grid entity count changes (e.g. after hot reload), the occupancy cache is fully rebuilt by iterating every cell in the `SpatialGrid`
- This produces frame-time spikes visible as stutter approximately every 167ms at 60 fps
- The default `OcclusionMode` is `Hybrid`, which enables this path

---

## Expected Behavior

- Interior detection should not cause perceptible frame spikes during normal gameplay
- Cache rebuilds should not occur during steady-state gameplay (no map changes)
- The default occlusion mode should prefer the cheaper `ShaderBased` path

---

## Root Cause Analysis

**File:** `src/systems/game/interior_detection.rs`  
**Function:** `detect_interior_system()`  
**Approximate line:** 73

```rust
// Runs every region_update_interval frames (default: 10)
interior_state.frames_since_update += 1;
if interior_state.frames_since_update < config.region_update_interval {
    return;
}

// Potentially rebuilds entire occupancy cache
let current_entity_count = spatial_grid.cells.values().map(|v| v.len()).sum();
if interior_state.occupied_voxels_cache.is_some()
    && interior_state.cache_entity_count == current_entity_count
{
    // Use cached version
} else {
    // Rebuild: iterates all SubVoxel entities
    let new_cache = build_occupied_voxel_set(&spatial_grid, &sub_voxels);
    // ...
}
// Then: BFS flood-fill up to MAX_REGION_SIZE (1000) voxels
```

Two compounding issues:
1. `region_update_interval = 10` is too low — fires 6 times per second at 60 fps
2. Entity count is a fragile cache-invalidation key — any change forces full rebuild

**File:** `src/systems/game/occlusion.rs`  
The default `OcclusionConfig` uses `OcclusionMode::Hybrid`, which routes through the region detection path. `ShaderBased` mode would skip `detect_interior_system` entirely.

---

## Steps to Reproduce

1. Run: `cargo run --release`
2. Enable the FPS counter with `F3`
3. Walk the player around the default map
4. Observe periodic FPS drops occurring roughly every 167ms (every 10 frames at 60 fps)
5. Drops are more pronounced in areas with ceiling voxels (interiors)

---

## Suggested Fix

**Short-term:** Increase `region_update_interval` to 60 frames (~1 second):
```rust
// src/systems/game/occlusion.rs
pub region_update_interval: u32 = 60,
```

**Medium-term:** Change the default mode to `ShaderBased` (skips flood-fill entirely):
```rust
pub mode: OcclusionMode = OcclusionMode::ShaderBased,
```

**Long-term:** Replace entity-count-based cache invalidation with a proper change-detection approach (e.g., listen to `Added<SubVoxel>` and `Removed<SubVoxel>` events).

---

## Related

- Investigation: [2026-03-15-2141-game-binary-graphics-lag.md](../investigations/2026-03-15-2141-game-binary-graphics-lag.md)
- Bug: [2026-03-15-2141-occlusion-material-gpu-reupload-every-frame.md](2026-03-15-2141-occlusion-material-gpu-reupload-every-frame.md)
