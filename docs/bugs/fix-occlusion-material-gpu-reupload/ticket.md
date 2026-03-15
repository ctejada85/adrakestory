# Fix: Occlusion Material GPU Re-Upload Every Frame

**Date:** 2026-03-15  
**Severity:** Critical (P1)  
**Component:** Occlusion / Rendering  

---

## Story

As a player, I want the game to run at a smooth and consistent frame rate so that movement and camera control feel responsive, even in scenes with many voxel chunks.

---

## Description

The `update_occlusion_uniforms` system calls `Assets::get_mut()` unconditionally every frame, which marks the `OcclusionMaterial` asset as changed regardless of whether any uniform values have actually changed. This causes Bevy's render world to re-prepare GPU bind groups and re-upload the uniform buffer on every tick — even when the player and camera are completely still.

The fix splits `OcclusionUniforms` into two private Rust-side sub-structs (`StaticOcclusionUniforms` for config-driven fields and `DynamicOcclusionUniforms` for positional fields), caches each independently using Bevy `Local` state, and uses Bevy change detection (`Ref<Transform>`, `is_changed()`) to skip sub-struct recomputation on static frames. `get_mut()` is only called when at least one sub-struct cache differs from the newly computed value. The GPU-facing `OcclusionUniforms` struct, its shader binding, and the WGSL shader are left unchanged.

---

## Acceptance Criteria

1. The game binary no longer calls `Assets::get_mut()` on `OcclusionMaterial` on frames where player position, camera position, and `OcclusionConfig` have not changed.
2. `OcclusionUniforms` derives `PartialEq`.
3. Two private structs, `StaticOcclusionUniforms` and `DynamicOcclusionUniforms`, exist in `occlusion.rs` and together cover all fields of `OcclusionUniforms`.
4. `StaticOcclusionUniforms` is only recomputed when `OcclusionConfig` is marked changed by Bevy.
5. `DynamicOcclusionUniforms` is only recomputed when the camera transform, player transform, or `InteriorState` is marked changed by Bevy.
6. `get_mut()` is called only when at least one sub-struct cache value differs from the newly computed value (safety fallback on top of Bevy change detection).
7. On the first frame after the material handle becomes available, uniforms are written unconditionally regardless of cache state.
8. The GPU-facing `OcclusionUniforms` struct, its `#[uniform(100)]` binding, and the WGSL shader are unchanged.
9. All existing early-return conditions (occlusion disabled, mode is `None`, material handle unavailable) are preserved.
11. All unit tests for cache hit, cache miss, static/dynamic independence, and first-frame write pass.

---

## Non-Functional Requirements

- GPU bind group preparation for `OcclusionMaterial` must not occur on frames where all uniform inputs are unchanged.
- No per-frame heap allocation is introduced; all caches are stack-resident `Local` values holding `Copy` types.
- The system's time complexity remains O(1) — no iteration over entities or assets is added.
- Frame rate of the game binary under a static scene must be comparable to the map editor under equivalent scene complexity after the fix is applied.
- The changes must not affect the map editor binary, which does not use `OcclusionMaterial`.
- `StaticOcclusionUniforms` and `DynamicOcclusionUniforms` are private to `occlusion.rs` and do not appear in any public API.

---

## Tasks

1. Add `PartialEq` to the `#[derive(...)]` list of `OcclusionUniforms`.
2. Define `StaticOcclusionUniforms` (private, `Clone, Copy, PartialEq`) with all `OcclusionConfig`-driven fields.
3. Define `DynamicOcclusionUniforms` (private, `Clone, Copy, PartialEq`) with all per-frame positional fields.
4. Change the camera and player query parameters in `update_occlusion_uniforms` from `Query<&Transform, ...>` to `Query<Ref<Transform>, ...>`.
5. Replace the system's `Local<Option<OcclusionUniforms>>` parameter with two separate caches: `Local<Option<StaticOcclusionUniforms>>` and `Local<Option<DynamicOcclusionUniforms>>`.
6. Gate static field computation on `config.is_changed()` or empty cache; gate dynamic field computation on transform/interior change detection or empty cache.
7. Implement the dirty check: skip `get_mut()` when both sub-struct caches match the newly computed values.
8. Assemble the full `OcclusionUniforms` from both sub-structs and write to the material only when dirty.
9. Update both sub-struct caches after a successful write.
10. Write a unit test verifying the cache hit path: given identical inputs on a second system call, `get_mut()` is not triggered.
11. Write a unit test verifying the cache miss path: when a uniform field changes, `get_mut()` is triggered and the material is updated.
12. Write a unit test verifying static/dynamic cache independence: a config change only dirties the static cache; a transform change only dirties the dynamic cache.
13. Write a unit test verifying first-frame unconditional write: with empty caches, uniforms are always written regardless of change detection.
14. Verify with the FPS counter (F3) that frame rate under a static scene is no longer degraded compared to the map editor.
