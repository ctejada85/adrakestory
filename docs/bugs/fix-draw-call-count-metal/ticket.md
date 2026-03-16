# Fix: One Draw Call Per Chunk Overwhelms Metal Backend at High Chunk Counts

**Date:** 2026-03-16  
**Severity:** High  
**Component:** Chunk spawner / Rendering pipeline

---

## Story

As a player on macOS, I want the game to render large maps at a stable frame rate so that camera panning and exploration feel smooth even when many voxel chunks are visible.

---

## Description

`VoxelChunk` entities are spawned without an `Aabb` component, so Bevy's frustum culling never suppresses their draw calls. All 100–200 chunks are submitted to Metal every frame, regardless of whether they are within the camera's view. Metal's per-draw overhead (~0.04 ms) makes this 6–10 ms of pure CPU cost per frame on macOS. The fix is to add an explicit `Aabb` component to each chunk at spawn time, identical to what the map editor already does for its own `EditorChunk` entities in `src/editor/renderer.rs`. No other system (LOD, collision, occlusion) is affected.

---

## Acceptance Criteria

1. Every `VoxelChunk` entity spawned by `chunks.rs` has an `Aabb` component at the time of spawning.
2. `Aabb.center` equals the world-space center of the chunk (`chunk_center` computed at spawn time).
3. `Aabb.half_extents` equals `Vec3A::splat(CHUNK_SIZE as f32 / 2.0)` (8.0 in each axis for a 16-voxel chunk).
4. Both spawn paths in `chunks.rs` (OcclusionMaterial and StandardMaterial) include the `Aabb`.
5. `update_chunk_lods` continues to swap LOD mesh handles correctly after the change — no regression in LOD behaviour.
6. Hot reload re-spawns chunks with correct `Aabb` values without requiring special handling.
7. Unit test `chunk_aabb_half_extents_match_chunk_size` passes.
8. Unit test `chunk_aabb_center_matches_chunk_center` passes.
9. `cargo test --lib`, `cargo clippy --lib`, and `cargo build --release` all pass with no new errors or warnings.

---

## Non-Functional Requirements

- The `Aabb` must be computed with a constant formula at spawn time — no per-frame recalculation and no mesh vertex analysis.
- The fix must not affect rendering output for chunks that are within the camera frustum.
- The fix must not modify `VoxelChunk`, `ChunkLOD`, or any collision/spatial grid system.
- The `Aabb` formula must match the editor implementation in `src/editor/renderer.rs` exactly.

---

## Tasks

1. Read `docs/developer-guide/coding-guardrails.md`, `coding-style.md`, and `architecture.md`.
2. Add `use bevy::render::primitives::Aabb;` to the imports in `src/systems/game/map/spawner/chunks.rs` (see architecture Appendix A.1).
3. Add the `Aabb` component to the `OcclusionMaterial` spawn path in `chunks.rs` (see architecture Appendix A.2).
4. Add the `Aabb` component to the `StandardMaterial` spawn path in `chunks.rs` (see architecture Appendix A.2).
5. Add unit tests `chunk_aabb_half_extents_match_chunk_size` and `chunk_aabb_center_matches_chunk_center` to the spawner test file (see architecture Appendix A.3).
6. Run `cargo test --lib`, `cargo clippy --lib`, `cargo build --release`; fix any issues.
7. Manually verify: build release, load a large map, rotate the camera so chunks go off-screen — confirm no visual artefacts on-screen.
8. Commit: `fix(spawner): add Aabb to VoxelChunk entities to enable frustum culling`.
