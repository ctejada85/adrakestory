# Bug: Occlusion Material Forces GPU Re-Upload Every Frame

**Date:** 2026-03-15  
**Severity:** Critical  
**Status:** Open  
**Component:** Occlusion / Rendering  

---

## Description

The `update_occlusion_uniforms` system calls `materials.get_mut()` unconditionally on every frame. In Bevy, `Assets::get_mut()` always marks the asset as changed regardless of whether the data was actually modified. This triggers a full GPU re-upload of the `OcclusionMaterial` bind groups every frame, causing unnecessary rendering overhead and degraded frame rate in the game binary.

---

## Actual Behavior

- `OcclusionMaterial` is marked dirty every frame
- Bevy re-prepares material bind groups and re-uploads uniform buffers to the GPU on every render tick
- All voxel chunk meshes using this material are re-batched unnecessarily
- The game binary runs at noticeably lower frame rate / higher GPU utilization than the map editor

---

## Expected Behavior

- Material uniforms are only re-uploaded when the player position, camera position, or occlusion config actually changes
- GPU work is proportional to what has changed, not unconditionally performed every frame
- Frame rate of the game binary is comparable to the map editor under equivalent scene complexity

---

## Root Cause Analysis

**File:** `src/systems/game/occlusion.rs`  
**Function:** `update_occlusion_uniforms()`  
**Approximate line:** 270

```rust
if let Some(material) = materials.get_mut(&material_handle.0) {
    // Overwrites uniforms every frame even if values are identical
    material.extension.occlusion_uniforms = OcclusionUniforms {
        player_position: player_pos,
        camera_position: camera_pos,
        // ...
    };
}
```

Bevy's `Assets<T>::get_mut()` internally calls `self.mark_changed()` on the asset, which sets the asset's change-detection tick. The render world then sees the asset as modified and re-extracts/re-prepares the material. Because this happens unconditionally every frame, the cost is constant rather than event-driven.

---

## Steps to Reproduce

1. Run the game binary in release mode: `cargo run --release`
2. Enable GPU frame profiling (e.g., with Bevy's `bevy/trace` feature or an external profiler)
3. Observe `OcclusionMaterial` bind group preparation occurring every frame
4. Alternatively: compare FPS counter (F3) in game vs map editor on the same map

---

## Suggested Fix

Cache the last-written values in a `Local` and skip `get_mut()` when nothing has changed:

```rust
pub fn update_occlusion_uniforms(
    // ...
    mut last_uniforms: Local<Option<OcclusionUniforms>>,
) {
    // ... compute new_uniforms ...

    if last_uniforms.as_ref() != Some(&new_uniforms) {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.extension.occlusion_uniforms = new_uniforms;
        }
        *last_uniforms = Some(new_uniforms);
    }
}
```

This requires `OcclusionUniforms` to derive `PartialEq`.

---

## Related

- Investigation: [2026-03-15-2141-game-binary-graphics-lag.md](../investigations/2026-03-15-2141-game-binary-graphics-lag.md)
