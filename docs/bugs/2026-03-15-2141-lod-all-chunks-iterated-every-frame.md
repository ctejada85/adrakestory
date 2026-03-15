# Bug: LOD System Iterates All Chunks Every Frame

**Date:** 2026-03-15  
**Severity:** Low  
**Status:** Open  
**Component:** Rendering / LOD  

---

## Description

The `update_chunk_lods` system runs every frame and computes the camera-to-chunk distance for every `VoxelChunk` entity in the scene to determine the appropriate LOD level. While mesh swaps are guarded by a change check, the unbounded per-frame iteration scales linearly with map size and adds unnecessary CPU overhead during steady-state gameplay where most chunks haven't changed LOD.

---

## Actual Behavior

- Every frame: `camera_pos.distance(chunk.center)` is computed for all chunks
- The `chunks.iter_mut()` query runs unconditionally each frame
- On large maps with many chunks, this accumulates measurable CPU overhead every frame
- No throttling or early-exit mechanism exists

---

## Expected Behavior

- LOD recalculation should only occur when the camera has moved a meaningful distance
- Updates should be throttled so they do not run on every single frame
- CPU cost should be near-zero when the camera is stationary

---

## Root Cause Analysis

**File:** `src/systems/game/map/spawner/mod.rs`  
**Function:** `update_chunk_lods()`  
**Approximate line:** 339

```rust
pub fn update_chunk_lods(
    camera_query: Query<&Transform, With<Camera3d>>,
    mut chunks: Query<(&VoxelChunk, &mut ChunkLOD, &mut Mesh3d)>,
) {
    let camera_pos = camera_transform.translation;

    // Runs for EVERY chunk EVERY frame
    for (chunk, mut lod, mut mesh) in chunks.iter_mut() {
        let distance = camera_pos.distance(chunk.center);
        let new_lod = LOD_DISTANCES
            .iter()
            .position(|&threshold| distance < threshold)
            .unwrap_or(LOD_LEVELS - 1);

        if new_lod != lod.current_lod {
            lod.current_lod = new_lod;
            mesh.0 = lod.lod_meshes[new_lod].clone();
        }
    }
}
```

There is no distance-moved threshold check before iterating. The system is registered in `GameSystemSet::Visual` which runs every frame as part of the chained system set in `Update`.

---

## Steps to Reproduce

1. Run: `cargo run --release`
2. Load a large map with many chunks
3. Profile with `RUST_LOG=trace cargo run --release` or a CPU profiler
4. Observe `update_chunk_lods` appearing in every frame's trace even when the camera is stationary

---

## Suggested Fix

**Option A — Distance threshold guard (recommended):**
```rust
pub fn update_chunk_lods(
    camera_query: Query<&Transform, With<Camera3d>>,
    mut chunks: Query<(&VoxelChunk, &mut ChunkLOD, &mut Mesh3d)>,
    mut last_camera_pos: Local<Vec3>,
) {
    let Ok(camera_transform) = camera_query.get_single() else { return };
    let camera_pos = camera_transform.translation;

    // Skip if camera hasn't moved more than 0.5 units
    if camera_pos.distance(*last_camera_pos) < 0.5 {
        return;
    }
    *last_camera_pos = camera_pos;

    for (chunk, mut lod, mut mesh) in chunks.iter_mut() {
        // ... existing logic ...
    }
}
```

**Option B — Frame counter throttle:**
Run the system only every N frames using a `Local<u32>` counter (every 5–10 frames is sufficient given LOD transitions are not time-critical).

---

## Related

- Investigation: [2026-03-15-2141-game-binary-graphics-lag.md](../investigations/2026-03-15-2141-game-binary-graphics-lag.md)
