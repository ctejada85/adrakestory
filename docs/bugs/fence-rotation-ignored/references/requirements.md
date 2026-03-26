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

The result is that any `rotation: Option<usize>` stored on a `Fence` voxel is
silently ignored at spawn time. The field is round-tripped through the file and
editor without effect.

Two fix options are defined:

| Option | Summary |
|--------|---------|
| **A — Warn and document (MVP)** | Emit `warn!()` for fence+rotation, update spec, strip field in editor on save |
| **B — Full rotation support** | Apply the orientation matrix to the neighbour-generated geometry at spawn time; requires deciding whether neighbour detection axes rotate with the geometry |

These requirements cover both options. Phase 1 delivers Option A; Option B is Phase 2.

---

## 2. Functional Requirements

### 2.1 Warning on Fence + Rotation (Option A — Phase 1)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1.1 | `validate_map()` in `src/systems/game/map/validation.rs` must emit `warn!()` for every voxel where `pattern == Some(Fence)` and `rotation == Some(_)`. The warning message must identify the voxel position and state that `rotation` has no effect on `Fence` geometry. | Phase 1 |
| FR-2.1.2 | The warning must not be emitted when `rotation` is `None` on a `Fence` voxel. | Phase 1 |
| FR-2.1.3 | The warning must not cause map loading to fail. The map must continue to load and the fence must render as a neighbour-connected fence (rotation still ignored). | Phase 1 |

### 2.2 Specification Update (Option A — Phase 1)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.2.1 | `docs/api/map-format-spec.md` must document that the `rotation` field is ignored for `Fence` voxels in Phase 1. The exception must be listed in the field description for `VoxelData.rotation` and in the `Fence` pattern section. | Phase 1 |
| FR-2.2.2 | The spec update must recommend that authors leave `rotation: None` on `Fence` voxels and note that a non-`None` value will produce a load-time warning. | Phase 1 |

### 2.3 Editor Sanitisation (Option A — Phase 1)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.3.1 | When the map editor saves a map, it must strip `rotation` from all `Fence` voxels — i.e., set `rotation: None` — before writing the RON file. | Phase 1 |
| FR-2.3.2 | The editor must not expose the rotation tool for `Fence` voxels, or must silently no-op if the rotation tool is applied to a selected fence. | Phase 1 |
| FR-2.3.3 | The editor requirement applies only to the save path and the rotation UI. In-memory representation of a loaded `Fence` voxel with a non-`None` rotation must remain intact until save (to avoid mutating unsaved work silently). | Phase 1 |

### 2.4 Full Rotation Support (Option B — Phase 2)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.4.1 | The spawner must apply the voxel's orientation matrix to the geometry produced by `fence_geometry_with_neighbors()`. The application must use `apply_orientation_matrix()` from `src/systems/game/map/format/rotation.rs`. | Phase 2 |
| FR-2.4.2 | Neighbour detection must remain axis-aligned (world X/Z axes), regardless of the fence voxel's rotation. Rotating a fence post does not change which adjacent positions are queried for connectivity. | Phase 2 |
| FR-2.4.3 | `SubVoxel.bounds` for rotated fence sub-voxels must be recomputed from the rotated geometry at spawn time. The spawner must not use pre-computed bounds derived from the un-rotated fence geometry when a rotation is present. | Phase 2 |
| FR-2.4.4 | Rotated fence sub-voxels must be inserted into `SpatialGrid` using their rotated world-space AABB, not the un-rotated AABB. | Phase 2 |
| FR-2.4.5 | A `Fence` voxel with `rotation: None` must produce geometry and collision bounds identical to the current (un-rotated) behaviour. Phase 2 must be strictly backward-compatible for the `rotation: None` case. | Phase 2 |
| FR-2.4.6 | The Phase 1 `warn!()` in `validate_map()` must be removed when Phase 2 is shipped, since rotation on `Fence` voxels is then a supported operation. The spec must be updated to document rotation support. | Phase 2 |

### 2.5 Backward Compatibility

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.5.1 | Existing map files that contain `Fence` voxels with `rotation: None` must load and render identically before and after Phase 1 and Phase 2. | Phase 1 |
| FR-2.5.2 | Existing map files that contain `Fence` voxels with `rotation: Some(_)` must continue to load without error after Phase 1. The warning (FR-2.1.1) is acceptable; a load failure is not. | Phase 1 |

---

## 3. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-3.1 | All new orientation matrix arithmetic in Phase 2 must use integer types only — no `f32` or floating-point. | Phase 2 |
| NFR-3.2 | The Phase 1 warning path in `validate_map()` must not allocate per-voxel. | Phase 1 |
| NFR-3.3 | `SubVoxelGeometry::fence_with_connections()` must not be modified in Phase 1. | Phase 1 |
| NFR-3.4 | `apply_orientation_matrix()` must not be modified. | Phase 1 + 2 |
| NFR-3.5 | Both binaries (`adrakestory` and `map_editor`) must compile without error or warning after each phase. | Phase 1 + 2 |
| NFR-3.6 | Existing unit tests in `validation.rs`, `patterns.rs`, and `chunks.rs` must continue to pass. New tests must cover the acceptance criteria in `ticket.md`. | Phase 1 |

---

## 4. Phase Scoping

### Phase 1 — MVP (Option A)

- `validate_map()` emits `warn!()` for fence + non-None rotation.
- `docs/api/map-format-spec.md` updated to document the exception.
- Map editor strips `rotation` from `Fence` voxels on save.
- Map editor rotation tool disabled or no-op for selected fence voxels.
- All existing tests pass; new tests cover warning path.

### Phase 2 — Full Fix (Option B)

- Spawner applies orientation matrix to fence geometry post-neighbour-detection.
- `SubVoxel.bounds` and `SpatialGrid` use rotated AABB.
- Phase 1 warning removed; spec updated to document rotation support.
- New tests cover rotated collision bounds and geometry correctness.

---

## 5. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | Fix 1 (`map-format-multi-axis-rotation`) is complete and merged. `orientations: Vec<OrientationMatrix>` on `MapData` and `rotation: Option<usize>` on `VoxelData` are available. |
| 2 | `apply_orientation_matrix()` is available in `src/systems/game/map/format/rotation.rs` and works correctly on any `SubVoxelGeometry` bit array. |
| 3 | Fix 2 (`staircase-double-rotation`) is complete (commit `4874885`). The loader pipeline — `migrate_legacy_rotations() → normalise_staircase_variants() → validate_map()` — is stable. |
| 4 | Neighbour detection for fences queries axis-aligned world positions and does not depend on voxel orientation. This assumption holds for Phase 1; Phase 2 intentionally keeps axis-aligned neighbour detection (FR-2.4.2). |
| 5 | `SubVoxel.bounds` are pre-computed at spawn time from geometry. Any Phase 2 change to fence geometry due to rotation must recompute bounds using the rotated positions. |
| 6 | The `SpatialGrid` accepts AABB insertion; inserting a rotated AABB for a rotated fence does not require structural changes to `SpatialGrid`. |

---

## 6. Open Questions

*All questions resolved.*

| # | Question | Resolution |
|---|----------|------------|
| 1 | Should Phase 1 be Option A (warn+document) or jump straight to Option B (full rotation)? | **Option A for Phase 1.** Full rotation (Option B) requires resolving the neighbour detection axis question and recomputing collision bounds — this is scoped to Phase 2. |
| 2 | Should neighbour detection axes rotate with the fence geometry in Option B? | **No.** Neighbour detection remains world-axis-aligned (FR-2.4.2). Rotation affects the visual/collision geometry of the post, not which adjacent cells are checked for connectivity. |
| 3 | Should `rotation: None` on identity be enforced — i.e., should Phase 1 normalise `Fence` voxels that carry `rotation: Some(identity)` to `rotation: None`? | **Out of scope for Phase 1.** The editor strip (FR-2.3.1) handles new saves. Legacy files with fence+rotation simply receive a warning on load. |

---

## 7. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | Fix 1 — Orientation matrix system (`map-format-multi-axis-rotation`) | **Done** (commit `eda90e3`) | Team |
| 2 | Fix 2 — Staircase normalisation (`staircase-double-rotation`) | **Done** (commit `4874885`) | Team |
| 3 | Phase 2 decision: confirm neighbour-detection axis semantics before implementing Option B | **Resolved** (FR-2.4.2 — world-axis-aligned) | Implementer |

---

## 8. Reference: Example RON Snippets

| Type | Example | Notes |
|------|---------|-------|
| Fence, no rotation (valid) | `(pos:(3,0,5), voxel_type:Stone, pattern:Some(Fence), rotation:None)` | Loads fine; neighbour-connected geometry; no warning |
| Fence with rotation (Phase 1) | `(pos:(3,0,5), voxel_type:Stone, pattern:Some(Fence), rotation:Some(0))` | Loads; rotation ignored; `warn!()` emitted; visual result identical to `rotation:None` |
| Fence with rotation (Phase 2) | `(pos:(3,0,5), voxel_type:Stone, pattern:Some(Fence), rotation:Some(0))` | Loads; orientation matrix applied to fence geometry; rotated post and rails rendered |

---

*Created: 2026-03-26*  
*Source: `docs/investigations/2026-03-22-1427-map-format-analysis.md` — Finding 3*
