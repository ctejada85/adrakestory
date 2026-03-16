# Requirements — Fix Draw-Call Count via Frustum Culling on Chunk Entities

**Source:** Bug report `docs/bugs/2026-03-16-1213-p2-draw-call-count-metal-backend.md` — 2026-03-16  
**Status:** Draft

---

## 1. Overview

Each `VoxelChunk` entity in the game is submitted as a separate GPU draw call every frame, regardless of whether the chunk is visible to the camera. On macOS (Metal backend), each draw call carries ~0.03–0.05 ms of CPU overhead, so 100–200 visible chunks cost 6–10 ms per frame — comparable to the entire 8.3 ms budget at 120 FPS. On Windows (Vulkan/DX12), per-draw overhead is 5–10× lower, so the problem does not manifest.

The fix is to add an explicit `Aabb` component to each `VoxelChunk` entity at spawn time. Bevy's built-in frustum culling system suppresses draw calls for any entity with `Aabb` that falls entirely outside the camera frustum. This is a proven low-risk pattern: the map **editor** already applies identical `Aabb` components to its own chunk entities in `src/editor/renderer.rs`.

---

## 2. Problem Scope

This ticket covers frustum culling only (Option B from the bug report). Mesh merging or GPU instancing (Option A) is a larger architectural change documented separately and is not part of this scope.

---

## 3. Functional Requirements

### 3.1 Frustum Culling

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1.1 | Every `VoxelChunk` entity spawned in `chunks.rs` must carry a `bevy::render::primitives::Aabb` component | Phase 1 |
| FR-3.1.2 | The `Aabb.center` must be the world-space center of the chunk, matching `VoxelChunk.center` | Phase 1 |
| FR-3.1.3 | The `Aabb.half_extents` must be `Vec3A::splat(CHUNK_SIZE as f32 / 2.0)` (8.0 in each axis) | Phase 1 |
| FR-3.1.4 | The `Aabb` must be set at spawn time, not computed retroactively each frame | Phase 1 |
| FR-3.1.5 | Chunks whose `Aabb` falls entirely outside the camera frustum must not generate a Metal/Vulkan draw command that frame | Phase 1 |

### 3.2 Consistency with Editor

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.2.1 | The game chunk `Aabb` calculation must use the same formula as `src/editor/renderer.rs` (`center = chunk_center`, `half_extents = CHUNK_SIZE / 2.0`) | Phase 1 |
| FR-3.2.2 | The `Aabb` import (`bevy::render::primitives::Aabb`) must be added to `chunks.rs` | Phase 1 |

### 3.3 Correctness

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.3.1 | LOD mesh swaps via `update_chunk_lods` must continue to work correctly — the `Aabb` is static and does not need to change when the active LOD mesh changes | Phase 1 |
| FR-3.3.2 | Hot reload must re-spawn chunks with correct `Aabb` values (no special handling needed — re-spawn path calls the same spawn function) | Phase 1 |
| FR-3.3.3 | Both `OcclusionMaterial` and `StandardMaterial` spawn paths in `chunks.rs` must receive the `Aabb` component | Phase 1 |

---

## 4. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-4.1 | The fix must not change visible rendering output for chunks that ARE within the frustum | Phase 1 |
| NFR-4.2 | Spawn-time cost of computing the `Aabb` must be O(1) per chunk (constant formula, no mesh analysis) | Phase 1 |
| NFR-4.3 | The fix must not break the existing LOD system, spatial grid, or collision systems | Phase 1 |
| NFR-4.4 | A unit test must assert that `Aabb.half_extents == Vec3A::splat(CHUNK_SIZE as f32 / 2.0)` | Phase 1 |

---

## 5. Phase Scoping

### Phase 1 — Frustum Culling

- Add `Aabb` component to both spawn paths in `chunks.rs`
- Add `use bevy::render::primitives::Aabb;` import
- Add unit test asserting `Aabb` dimensions match chunk world size
- Verify LOD swaps still work with explicit `Aabb`

---

## 6. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | `VoxelChunk` mesh vertices encode absolute world positions; `Transform` is the identity. The `Aabb` center therefore equals the world-space chunk center |
| 2 | Bevy 0.15's frustum culling system suppresses draw calls (not just visibility) for culled entities with `Aabb` |
| 3 | Mesh merging / GPU instancing (Option A) is a separate ticket and is not in scope here |
| 4 | `CHUNK_SIZE = 16` world units — this constant must not change as part of this fix |
| 5 | The editor's `EditorChunk` spawn in `src/editor/renderer.rs` is already correct and serves as the reference implementation |

---

## 7. Open Questions

| # | Question | Status | Owner |
|---|----------|--------|-------|
| 1 | Should a unit test spin up a full `bevy::App` to verify the `Aabb` is attached to spawned entities, or is a pure math test sufficient? | **Open** | Engineer |

---

*Created: 2026-03-16*  
*Source: `docs/bugs/2026-03-16-1213-p2-draw-call-count-metal-backend.md`*
