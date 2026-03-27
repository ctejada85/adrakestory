# Requirements — Fence Rotation Field Handling

**Source:** Map Format Analysis Investigation — 2026-03-22  
**Bug:** `docs/bugs/fence-rotation-ignored/bug.md`  
**Status:** Final — all questions resolved 2026-03-26

---

## 1. Overview

`Fence` voxels are spawned via a neighbour-aware path in `spawn_voxels_chunked()`
(`src/systems/game/map/spawner/chunks.rs:104–115`) that calls
`pattern.fence_geometry_with_neighbors(neighbors)` directly. This function does not
accept an orientation parameter. The `voxel_data.rotation` field is never consulted
for fence voxels; `geometry_with_rotation()` — which applies the orientation matrix
— is only reached by the `else` branch for non-fence patterns.

The fix applies the orientation matrix to the neighbour-generated fence geometry in
the spawner, after the neighbour detection step. Neighbour detection remains
world-axis-aligned. Collision bounds and `SpatialGrid` insertion must reflect the
rotated geometry.

---

## 2. Functional Requirements

### 2.1 Rotation Applied to Fence Geometry

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1.1 | The spawner must apply the voxel's orientation matrix to the geometry produced by `fence_geometry_with_neighbors()`. The application must use `apply_orientation_matrix()` from `src/systems/game/map/format/rotation.rs`. | Phase 1 |
| FR-2.1.2 | A `Fence` voxel with `rotation: None` must produce geometry and collision bounds identical to the current un-rotated behaviour. | Phase 1 |
| FR-2.1.3 | A `Fence` voxel with `rotation: Some(i)` must produce geometry equivalent to the neighbour-connected fence geometry with the orientation matrix at index `i` applied. | Phase 1 |

### 2.2 Neighbour Detection

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.2.1 | Neighbour detection must remain world-axis-aligned. The four positions queried by the spawner (`x±1, y, z` and `x, y, z±1`) do not rotate with the fence voxel's orientation matrix. | Phase 1 |

### 2.3 Collision Bounds and SpatialGrid

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.3.1 | `SubVoxel.bounds` for fence sub-voxels must be computed from the rotated geometry when `rotation != None`. The spawner must not use bounds derived from the un-rotated fence geometry when a rotation is present. | Phase 1 |
| FR-2.3.2 | Rotated fence sub-voxels must be inserted into `SpatialGrid` using their rotated world-space AABB. | Phase 1 |

### 2.4 Backward Compatibility

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.4.1 | Existing map files that contain `Fence` voxels with `rotation: None` must load and render identically before and after this fix. | Phase 1 |
| FR-2.4.2 | Existing map files that contain `Fence` voxels with `rotation: Some(_)` must now render with the orientation matrix applied. This is an intentional visual change for previously broken data. | Phase 1 |

### 2.5 Documentation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.5.1 | `docs/api/map-format-spec.md` must be updated to document that `rotation` is fully supported for `Fence` voxels and that the orientation matrix is applied after neighbour-aware geometry generation. | Phase 1 |

---

## 3. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-3.1 | All orientation matrix arithmetic must use integer types only — no `f32` or floating-point. | Phase 1 |
| NFR-3.2 | `SubVoxelGeometry::fence_with_connections()` must not be modified. | Phase 1 |
| NFR-3.3 | `SubVoxelPattern::fence_geometry_with_neighbors()` must not be modified. | Phase 1 |
| NFR-3.4 | `apply_orientation_matrix()` must not be modified. | Phase 1 |
| NFR-3.5 | `SpatialGrid` requires no structural changes; the rotated AABB is inserted using the existing API. | Phase 1 |
| NFR-3.6 | Both binaries (`adrakestory` and `map_editor`) must compile without error or warning. | Phase 1 |
| NFR-3.7 | Existing unit tests in `validation.rs`, `patterns.rs`, and `chunks.rs` must continue to pass. New tests must cover the acceptance criteria in `ticket.md`. | Phase 1 |

---

## 4. Phase Scoping

### Phase 1 — MVP

- Spawner applies orientation matrix to fence geometry after neighbour detection.
- Neighbour detection remains world-axis-aligned.
- `SubVoxel.bounds` and `SpatialGrid` insertion use rotated AABB.
- `docs/api/map-format-spec.md` updated to document rotation support.
- All existing tests pass; new tests cover geometry correctness and collision bounds.

---

## 5. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | Fix 1 (`map-format-multi-axis-rotation`) is complete and merged. `orientations: Vec<OrientationMatrix>` on `MapData` and `rotation: Option<usize>` on `VoxelData` are available. |
| 2 | `apply_orientation_matrix()` is available in `src/systems/game/map/format/rotation.rs` and works correctly on any `SubVoxelGeometry` bit array. |
| 3 | Fix 2 (`staircase-double-rotation`) is complete (commit `4874885`). The loader pipeline — `migrate_legacy_rotations() → normalise_staircase_variants() → validate_map()` — is stable. |
| 4 | Neighbour detection queries axis-aligned world positions and does not depend on voxel orientation. Rotating a fence post does not change which adjacent positions are queried for connectivity. |
| 5 | `SubVoxel.bounds` are pre-computed at spawn time from geometry. The existing sub-voxel loop in the spawner computes bounds from `geometry.occupied_positions()`; providing rotated geometry to that variable is sufficient — the loop itself needs no change. |
| 6 | `SpatialGrid` accepts AABB insertion; inserting a rotated AABB does not require structural changes to `SpatialGrid`. |

---

## 6. Open Questions

*All questions resolved.*

| # | Question | Resolution |
|---|----------|------------|
| 1 | Should neighbour detection axes rotate with the fence geometry? | **No — world-axis-aligned** (FR-2.2.1). Rotation affects the visual/collision geometry of the post; connectivity queries remain in world space. |
| 2 | Does the sub-voxel bounds loop in the spawner need modification? | **No.** The existing loop computes bounds from `geometry.occupied_positions()`. Providing rotated geometry to the `geometry` variable is sufficient; the loop is unchanged. |

---

## 7. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | Fix 1 — Orientation matrix system (`map-format-multi-axis-rotation`) | **Done** (commit `eda90e3`) | Team |
| 2 | Fix 2 — Staircase normalisation (`staircase-double-rotation`) | **Done** (commit `4874885`) | Team |

---

## 8. Reference: Example RON Snippets

| Type | Example | Notes |
|------|---------|-------|
| Fence, no rotation (valid) | `(pos:(3,0,5), voxel_type:Stone, pattern:Some(Fence), rotation:None)` | Neighbour-connected geometry; no rotation applied |
| Fence with rotation | `(pos:(3,0,5), voxel_type:Stone, pattern:Some(Fence), rotation:Some(0))` | Neighbour-connected geometry with orientation matrix at index 0 applied |

---

*Created: 2026-03-26*  
*Source: `docs/investigations/2026-03-22-1427-map-format-analysis.md` — Finding 3*
