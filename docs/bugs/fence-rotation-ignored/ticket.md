# Fix: Fence Rotation Field Silently Ignored

**Date:** 2026-03-26  
**Severity:** Medium (p2)  
**Component:** Map format — `Fence` / `src/systems/game/map/spawner/chunks.rs`

---

## Story

As a level designer, I want a `Fence` voxel with a `rotation` value to render with the orientation matrix applied so that fence posts and rails orient correctly and the `rotation` field has the same meaning for fences as it does for every other pattern.

---

## Description

The spawner branches on `pattern.is_fence()` before resolving the orientation matrix. The fence path calls `pattern.fence_geometry_with_neighbors(neighbors)` directly — which does not accept an orientation parameter — and never reads `voxel_data.rotation`. Any `rotation: Option<usize>` stored on a `Fence` voxel is silently discarded at spawn time.

The fix applies the orientation matrix to the neighbour-generated fence geometry in the spawner, after the neighbour detection step. Neighbour detection remains world-axis-aligned (the adjacent positions queried do not rotate with the fence). Collision bounds (`SubVoxel.bounds`) and `SpatialGrid` insertion automatically reflect the rotated geometry because the existing sub-voxel loop derives bounds from `geometry.occupied_positions()`.

See `bug.md` for the full bug description and root cause analysis.

---

## Acceptance Criteria

1. Loading a map with `pattern: Some(Fence), rotation: Some(i)` produces geometry equivalent to the neighbour-connected fence geometry with the orientation matrix at index `i` applied via `apply_orientation_matrix()`.
2. Loading a map with `pattern: Some(Fence), rotation: None` produces geometry identical to the pre-fix behaviour — the `rotation: None` path must be strictly backward-compatible.
3. Neighbour detection queries `(x±1, y, z)` and `(x, y, z±1)` in world space regardless of the fence voxel's rotation matrix.
4. `SubVoxel.bounds` for a rotated fence voxel are computed from the rotated geometry. A collision test against a rotated fence must use the rotated AABB, not the un-rotated AABB.
5. `SubVoxelGeometry::fence_with_connections()` and `SubVoxelPattern::fence_geometry_with_neighbors()` are not modified.
6. `docs/api/map-format-spec.md` documents that `rotation` is fully supported for `Fence` voxels and that the orientation matrix is applied after neighbour-aware geometry generation.
7. `cargo test` passes with no failures; new unit tests cover: fence with rotation applied, fence with `rotation: None` unchanged, and collision bounds reflect rotated geometry.
8. Both `adrakestory` and `map_editor` binaries compile without error or new warning.

---

## Non-Functional Requirements

- All orientation matrix arithmetic must use integer types only — no `f32` or floating-point.
- `SubVoxelGeometry::fence_with_connections()` must not be modified.
- `SubVoxelPattern::fence_geometry_with_neighbors()` must not be modified.
- `apply_orientation_matrix()` must not be modified.
- `SpatialGrid` requires no structural changes.
- Changes must compile for both the `adrakestory` and `map_editor` binaries.

---

## Tasks

1. In `src/systems/game/map/spawner/chunks.rs:104–115`, update the fence branch to apply the orientation matrix after `fence_geometry_with_neighbors()`: resolve `voxel_data.rotation.and_then(|i| map.orientations.get(i))` and, if `Some(matrix)`, call `apply_orientation_matrix(fence_geo, matrix)`; otherwise use `fence_geo` unchanged.
2. Update `docs/api/map-format-spec.md` to document that `rotation` is fully supported for `Fence` voxels and that the matrix is applied after neighbour-aware geometry generation, with neighbour detection remaining world-axis-aligned.
3. Write unit tests covering all acceptance criteria: fence geometry with rotation applied, `rotation: None` backward compat, and collision bounds derived from rotated geometry.
4. Run `cargo test` and `cargo clippy`; fix any failures or warnings.
