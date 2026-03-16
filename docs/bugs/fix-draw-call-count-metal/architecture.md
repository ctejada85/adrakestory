# Architecture — Fix Draw-Call Count via Frustum Culling on Chunk Entities

**Bug:** `docs/bugs/2026-03-16-1213-p2-draw-call-count-metal-backend.md`  
**Requirements:** `docs/bugs/fix-draw-call-count-metal/requirements.md`  
**Status:** Draft

---

## 1. Problem Statement

Every `VoxelChunk` entity is submitted to the GPU as a separate draw command each frame. Bevy's draw-call batching cannot merge chunks because each has a unique mesh asset (`Handle<Mesh>`). With 100–200 visible chunks, macOS issues 100–200 Metal draw commands per frame, costing 6–10 ms of CPU overhead per frame on Apple Silicon and Intel GPUs.

Bevy's built-in frustum culling system can suppress draw calls for entities outside the camera frustum, but only when an `Aabb` component is present on the entity. Without `Aabb`, Bevy assumes the entity is always visible. No `Aabb` is currently attached to `VoxelChunk` entities in the game spawner.

The **editor already solves this** in `src/editor/renderer.rs` (lines 250–272), attaching an explicit `Aabb` to each `EditorChunk`. The game spawner needs the same treatment.

---

## 2. Current Architecture

### 2.1 Chunk Spawn (Problematic Path)

```
spawn_chunk() — chunks.rs:231–259
  commands.spawn((
    Mesh3d(lod_meshes[0]),      ← unique handle → no Bevy batching
    MeshMaterial3d(mat),        ← shared handle (same asset ID)
    Transform::default(),       ← identity; mesh vertices in world space
    VoxelChunk { center, .. },
    ChunkLOD { lod_meshes, .. },
    // ← NO Aabb here
  ))
```

### 2.2 Frame-Time Draw-Call Cost (macOS vs Windows)

```
Per frame, for N visible chunks:
  Metal  (macOS)  : N × ~0.04 ms  → 200 chunks = ~8 ms overhead
  Vulkan (Windows): N × ~0.007 ms → 200 chunks = ~1.4 ms overhead
```

### 2.3 Bevy Frustum Culling Requirements

Bevy suppresses a `Mesh3d` draw call when:
1. The entity has an `Aabb` component (local-space bounding box), **AND**
2. The `Aabb` transformed to world space falls entirely outside the camera frustum

Without `Aabb`, Bevy assumes infinite extent and never culls the entity.

### 2.4 Editor Reference Implementation

```rust
// src/editor/renderer.rs — lines 250–272 (already correct)
commands.spawn((
    Mesh3d(mesh),
    MeshMaterial3d(chunk_material.clone()),
    Transform::default(),
    EditorChunk { chunk_pos },
    Aabb {
        center: Vec3A::from(chunk_center),         // world center of this 16×16×16 chunk
        half_extents: Vec3A::from(half_extent),    // Vec3::splat(8.0)
    },
    Visibility::default(),
));
```

---

## 3. Target Architecture — Phase 1 (Frustum Culling)

### 3.1 Fix: Add Aabb to VoxelChunk Spawn

```
spawn_chunk() — chunks.rs (after fix)
  commands.spawn((
    Mesh3d(lod_meshes[0]),
    MeshMaterial3d(mat),
    Transform::default(),
    VoxelChunk { center: chunk_center, .. },
    ChunkLOD { lod_meshes, .. },
    Aabb {                                          ← NEW
      center: Vec3A::from(chunk_center),
      half_extents: Vec3A::splat(CHUNK_SIZE as f32 / 2.0),
    },
  ))
```

Both spawn paths in `chunks.rs` receive the `Aabb` — the `OcclusionMaterial` path and the `StandardMaterial` path (controlled by `OcclusionConfig`).

### 3.2 Aabb Geometry

| Property | Value | Notes |
|----------|-------|-------|
| `center` | `Vec3A::from(chunk_center)` | `chunk_center` is already computed before spawn |
| `half_extents` | `Vec3A::splat(8.0)` | `CHUNK_SIZE (16) / 2 = 8.0` world units |
| Frame cost | O(1) — constant formula | No mesh vertex analysis needed |
| Changes on LOD swap | None | `Aabb` is geometry-independent; same bounds at all LOD levels |

### 3.3 Why Aabb Does Not Change on LOD Swap

`update_chunk_lods` (mod.rs:411–448) replaces `mesh.0` with a different LOD handle at runtime. All LOD meshes cover the same spatial extent (same voxels, different detail). The `Aabb` represents the chunk's spatial footprint, not its mesh complexity, so it remains constant across LOD transitions.

### 3.4 Culling Benefit

```
Before fix: N draw calls regardless of camera direction
After fix:  K draw calls where K = chunks within camera frustum

At 90° horizontal FOV, roughly 50–60% of a large map may be outside the frustum.
Estimated reduction: 200 draws → ~80–100 draws when camera faces open area.
Estimated frame saving: ~3–5 ms on macOS (Metal).
```

### 3.5 Remaining Draw-Call Overhead

Frustum culling does not merge remaining in-frustum draw calls. 80–100 Metal draw calls still cost ~3–4 ms. Reducing this further requires mesh merging (Option A from the bug report) — a separate, larger effort.

---

## 4. No-Change Items

- `update_chunk_lods` — no changes; `Aabb` is static
- `VoxelChunk` struct — no new fields needed
- `ChunkLOD` struct — no changes
- `meshing/` — greedy mesher unchanged
- `OcclusionConfig` / occlusion material — unchanged
- `SpatialGrid` / collision — unchanged

---

## 5. Change Scope

| File | Change |
|------|--------|
| `src/systems/game/map/spawner/chunks.rs` | Add `use bevy::render::primitives::Aabb;` import; add `Aabb` component to both spawn paths |
| `src/systems/game/map/spawner/tests.rs` (new or existing) | Add unit test asserting `Aabb.half_extents == Vec3A::splat(CHUNK_SIZE as f32 / 2.0)` |

---

## 6. Test Strategy

| Test | Location | Assertion |
|------|----------|-----------|
| `chunk_aabb_half_extents_match_chunk_size` | `spawner/tests.rs` | `Vec3A::splat(CHUNK_SIZE as f32 / 2.0) == Vec3A::splat(8.0)` |
| `chunk_aabb_center_matches_chunk_center` | `spawner/tests.rs` | Given a known `chunk_center`, `Aabb.center == Vec3A::from(chunk_center)` |

Both tests are pure (no `bevy::App` required) — they verify the math formula, not Bevy internals.

---

## Appendix A — Code Templates

### A.1 Import to Add in `chunks.rs`

```rust
use bevy::render::primitives::Aabb;
```

### A.2 Aabb Component in Both Spawn Paths

```rust
// Add to the spawn tuple in BOTH the OcclusionMaterial and StandardMaterial paths:
Aabb {
    center: Vec3A::from(chunk_center),
    half_extents: Vec3A::splat(CHUNK_SIZE as f32 / 2.0),
},
```

### A.3 Unit Tests

```rust
use bevy::math::Vec3A;
use super::{CHUNK_SIZE};
use bevy::render::primitives::Aabb;

#[test]
fn chunk_aabb_half_extents_match_chunk_size() {
    let half = Vec3A::splat(CHUNK_SIZE as f32 / 2.0);
    assert_eq!(half, Vec3A::splat(8.0));
}

#[test]
fn chunk_aabb_center_matches_chunk_center() {
    let chunk_center = bevy::math::Vec3::new(8.0, 8.0, 8.0); // first chunk (0,0,0)
    let aabb = Aabb {
        center: Vec3A::from(chunk_center),
        half_extents: Vec3A::splat(CHUNK_SIZE as f32 / 2.0),
    };
    assert_eq!(aabb.center, Vec3A::new(8.0, 8.0, 8.0));
    assert_eq!(aabb.half_extents, Vec3A::splat(8.0));
}
```
