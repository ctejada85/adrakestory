# Bug: VoxelChunk Meshes Missing `NotShadowCaster` — All Chunks Rendered into 4 Shadow Depth Passes Every Frame

**Date:** 2026-03-16
**Priority:** p2
**Severity:** High
**Status:** Open
**Component:** `src/systems/game/map/spawner/chunks.rs`

---

## Description

VoxelChunk mesh entities are spawned without the `NotShadowCaster` component. Because the directional light has `shadows_enabled: true` with 4 shadow cascades, Bevy renders every chunk mesh into up to 4 separate shadow depth maps per frame regardless of whether the chunks are visible to the player camera. GPU work scales linearly with total map voxel count, causing the game to slow down as more voxels are placed even when none are on screen.

---

## Actual Behavior

- All VoxelChunk mesh entities participate in 4 cascaded shadow depth passes every frame
- Shadow rendering cost scales with total chunk mesh complexity (quad count), not screen-visible quad count
- Performance degrades proportionally as the map grows, even when the player's view is empty
- On macOS/Metal, per-draw-call shadow overhead is higher than on Vulkan/DX12, amplifying the effect

---

## Expected Behavior

- Voxel chunk meshes should opt out of shadow casting (`NotShadowCaster`)
- GPU work per frame should be proportional to visible geometry, not total map size
- Adding voxels off-screen should have negligible per-frame GPU impact

---

## Root Cause Analysis

**File:** `src/systems/game/map/spawner/chunks.rs`
**Function:** `spawn_voxels_chunked()`
**Approximate line:** 232 (occlusion material path), 252 (standard material path)

Both chunk spawn paths omit `NotShadowCaster`:

```rust
// Both spawn paths — neither includes NotShadowCaster
ctx.commands.spawn((
    Mesh3d(lod_meshes[0].clone()),
    MeshMaterial3d(mat.clone()),
    Transform::default(),
    VoxelChunk { chunk_pos, center: chunk_center },
    ChunkLOD { lod_meshes, current_lod: 0 },
    Aabb { center, half_extents },
    // ← NotShadowCaster absent
));
```

In Bevy 0.15, any `Mesh3d` entity without `NotShadowCaster` is unconditionally included in all active shadow depth passes. The directional light is spawned in `spawn_lighting()` (`mod.rs:499–515`) with:

```rust
let cascade_shadow_config = CascadeShadowConfigBuilder {
    num_cascades: 4,
    maximum_distance: 100.0,
    ..default()
}.build();

DirectionalLight { shadows_enabled: true, ... }
```

With 4 cascades covering 100 world-units, all 10 chunk meshes (totalling ~285k quads) are rendered into 4 depth passes every frame. The shadow pass uses the light's orthographic frustum, not the player camera's — so main-camera frustum culling (which works correctly via `Aabb`) does not help here.

---

## Steps to Reproduce

1. `cargo run --release`
2. Load any map with a directional light entry in its `.ron` file
3. Turn the player camera away so no chunks are in view
4. Observe FPS — it will be the same as or worse than when all chunks are visible, because shadow passes still run

---

## Suggested Fix

Add `NotShadowCaster` to both chunk spawn paths:

```rust
// src/systems/game/map/spawner/chunks.rs
use bevy::pbr::NotShadowCaster;

// Occlusion material path (~line 232) AND standard material path (~line 252):
ctx.commands.spawn((
    Mesh3d(lod_meshes[0].clone()),
    MeshMaterial3d(mat.clone()),
    Transform::default(),
    VoxelChunk { chunk_pos, center: chunk_center },
    ChunkLOD { lod_meshes, current_lod: 0 },
    Aabb { center, half_extents },
    NotShadowCaster,  // ← add to both paths
));
```

**Trade-off:** Voxel geometry will no longer cast shadows onto other surfaces. If shadow casting is required for art direction, reduce `num_cascades` to 1 and lower `maximum_distance` to match the actual visible play area.

**Alternative:** If shadows are important, add `NotShadowCaster` only during runtime when the directional light is absent from the map RON data, keeping the default for maps that explicitly define a directional light with shadows.

---

## Related

- Investigation: `docs/investigations/2026-03-16-1310-off-screen-voxel-performance.md`
- Related GPU perf: `docs/bugs/2026-03-16-1213-p2-draw-call-count-metal-backend.md`
