# Investigation: Game Binary Graphics Lag vs Map Editor

**Date:** 2026-03-15 21:41  
**Reporter:** Carlos Tejada  
**Status:** Complete  

---

## Summary

The `adrakestory` game binary exhibits noticeably laggy/stuttery graphics while the `map_editor` binary runs smoothly under identical hardware conditions. Investigation identified four root causes, all exclusive to the game binary.

---

## Environment

- **Project:** A Drake's Story (Rust / Bevy)
- **Affected binary:** `adrakestory` (release build)
- **Unaffected binary:** `map_editor` (release build)
- **OS:** macOS
- **Reproduction:** Consistent, every run

---

## Investigation Method

1. Compared entry points: `src/main.rs` vs `src/bin/map_editor/main.rs`
2. Diffed plugin registrations and system schedules between the two binaries
3. Traced per-frame system costs through `GameSystemSet` chain
4. Analyzed Bevy asset mutation semantics for `Assets::get_mut()`
5. Reviewed camera interpolation math and frame-rate dependency

---

## Findings

### Finding 1 — Unconditional `materials.get_mut()` every frame (CRITICAL)

**File:** `src/systems/game/occlusion.rs` — `update_occlusion_uniforms()` (~line 270)

`update_occlusion_uniforms` calls `materials.get_mut(&material_handle.0)` unconditionally on every frame and overwrites the uniform struct with the current values, even when nothing has changed. In Bevy, `Assets::get_mut()` **always marks the asset as changed**, triggering:

- Re-preparation of the `OcclusionMaterial` bind groups
- GPU re-upload of material uniform buffers every frame
- Re-batching of all voxel chunks using this material

This occurs at 60+ fps regardless of whether the player or camera moved. The map editor does not use `OcclusionMaterial` and is entirely unaffected.

---

### Finding 2 — Interior detection BFS flood-fill creates periodic frame spikes (HIGH)

**File:** `src/systems/game/interior_detection.rs` — `detect_interior_system()` (~line 73)

When `OcclusionMode` is `RegionBased` or `Hybrid`, a BFS flood-fill over up to `MAX_REGION_SIZE` (1,000) voxels executes every `region_update_interval` frames (default: 10). At 60 fps this fires approximately every 167ms. The cache-rebuild path additionally iterates all cells in the `SpatialGrid`.

Frame spikes from this system can range from 5–50ms depending on map complexity, causing visible stutter. The map editor has no equivalent system.

---

### Finding 3 — Frame-rate-dependent camera lerp causes persistent visual lag (MEDIUM)

**File:** `src/systems/game/camera.rs` — `follow_player_camera()` (~line 43)

```rust
camera_transform.translation = camera_transform.translation.lerp(
    target_position,
    game_camera.follow_speed * time.delta_secs(),  // follow_speed = 5.0
);
```

At 60 fps: lerp factor ≈ `5.0 × 0.0167 = 0.083`. The camera closes only ~8% of the remaining gap per frame, taking 25+ frames (~420ms) to reach 99% convergence. This creates a permanently trailing camera that feels "floaty" and laggy even when the game is running at full FPS. The map editor camera moves directly with no trailing interpolation.

The same issue exists in `rotate_camera()` which uses `slerp` with the same delta-dependent factor.

---

### Finding 4 — `update_chunk_lods` iterates all chunks every frame (LOW)

**File:** `src/systems/game/map/spawner/mod.rs` — `update_chunk_lods()` (~line 339)

Every frame, `update_chunk_lods` queries all `VoxelChunk` entities and computes the camera distance for each one to determine the LOD level. While the mesh swap is guarded by a `if new_lod != lod.current_lod` check, the per-frame distance computation and mutable query iteration adds overhead that scales with map size. The map editor has no LOD system.

---

## Root Cause Summary

| # | Root Cause | Location | Severity | Present in Editor |
|---|-----------|----------|----------|-------------------|
| 1 | `materials.get_mut()` forces GPU re-upload every frame | `occlusion.rs` | Critical | No |
| 2 | BFS flood-fill fires every 10 frames creating spikes | `interior_detection.rs` | High | No |
| 3 | Camera lerp factor too low — visible trailing lag | `camera.rs` | Medium | No |
| 4 | All chunks iterated for LOD every frame | `spawner/mod.rs` | Low | No |

---

## Recommended Fixes

### Fix 1 — Cache uniform values; skip `get_mut()` when unchanged
In `update_occlusion_uniforms`, store the last-written `OcclusionUniforms` in a `Local` resource and only call `materials.get_mut()` when any value has actually changed.

### Fix 2 — Increase `region_update_interval` and guard mode
Raise the default `region_update_interval` from 10 to 60 frames. Additionally, skip `detect_interior_system` entirely when `OcclusionMode` is `ShaderBased` (it already returns early, but the mode default should prefer `ShaderBased` over `Hybrid`).

### Fix 3 — Use frame-rate-independent exponential decay for camera
Replace:
```rust
camera_transform.translation.lerp(target, follow_speed * delta)
```
With:
```rust
let alpha = 1.0 - (-follow_speed * delta).exp();
camera_transform.translation.lerp(target, alpha)
```
Or simply increase `follow_speed` to `15.0–20.0`.

### Fix 4 — Throttle LOD updates
Run `update_chunk_lods` only when camera has moved beyond a threshold distance, or throttle to every N frames using a `Local<u32>` frame counter.

---

## Related Bugs

- [2026-03-15-2141-occlusion-material-gpu-reupload-every-frame.md](../bugs/2026-03-15-2141-occlusion-material-gpu-reupload-every-frame.md)
- [2026-03-15-2141-interior-detection-frame-spikes.md](../bugs/2026-03-15-2141-interior-detection-frame-spikes.md)
- [2026-03-15-2141-camera-lerp-visual-lag.md](../bugs/2026-03-15-2141-camera-lerp-visual-lag.md)
- [2026-03-15-2141-lod-all-chunks-iterated-every-frame.md](../bugs/2026-03-15-2141-lod-all-chunks-iterated-every-frame.md)
