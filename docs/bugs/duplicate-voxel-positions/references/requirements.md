# Requirements — Duplicate Voxel Position Detection

**Source:** Map Format Analysis Investigation — 2026-03-22  
**Bug:** `docs/bugs/duplicate-voxel-positions/ticket.md`  
**Status:** Final — all questions resolved 2026-03-31

---

## 1. Overview

`validate_voxel_positions()` in `src/systems/game/map/validation.rs:45–57` iterates
`world.voxels` and checks that each entry's `pos` falls within world bounds. It does
not check for duplicate `pos` values. When two `VoxelData` entries share the same
`(i32, i32, i32)` position, both pass validation without error and are processed
independently by the spawner: their sub-voxels are unioned in the occupancy grid and
both contribute geometry to the chunk mesh. The result is superimposed geometry with
no warning. In large hand-edited RON files this can produce invisible mesh corruption
that is difficult to diagnose.

The fix extends `validate_voxel_positions()` with a single-pass `HashSet` check. If
any `pos` is seen more than once, the function returns
`Err(MapLoadError::ValidationError(...))` before any spawning occurs. No changes to
the spawner, chunk meshing, or editor are required.

---

## 2. Functional Requirements

### 2.1 Duplicate Detection

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1.1 | The validator must detect when two or more `VoxelData` entries in `world.voxels` share the same `pos` value and return `Err(MapLoadError::ValidationError(...))` on the first duplicate found. | Phase 1 |
| FR-2.1.2 | The error message must include the duplicate position coordinates so the author can locate the offending entry in the RON file. | Phase 1 |
| FR-2.1.3 | Detection must occur inside `validate_voxel_positions()`, so it runs as part of `validate_map()` before any spawning begins. | Phase 1 |

### 2.2 Existing Validation Preserved

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.2.1 | The bounds check introduced before this fix — that each `pos` is within `[0, width) × [0, height) × [0, depth)` — must continue to work unchanged. | Phase 1 |
| FR-2.2.2 | A map with no duplicate positions must pass `validate_voxel_positions()` without error (existing valid maps must not regress). | Phase 1 |

### 2.3 Error Ordering

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.3.1 | If a voxel is both out-of-bounds and a duplicate of another voxel, the out-of-bounds error is returned first (bounds check executes before duplicate check within the same iteration). | Phase 1 |

---

## 3. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-3.1 | The duplicate check must complete in a single pass over `world.voxels` using a `HashSet<(i32, i32, i32)>` — no additional sorting or second pass. | Phase 1 |
| NFR-3.2 | No new `MapLoadError` variants are required; `ValidationError(String)` is the correct error type. | Phase 1 |
| NFR-3.3 | The spawner (`chunks.rs`), chunk meshing, and editor code must not be modified. | Phase 1 |
| NFR-3.4 | Both binaries (`adrakestory` and `map_editor`) must compile without new errors or warnings. | Phase 1 |
| NFR-3.5 | All existing tests in `validation.rs` must continue to pass. | Phase 1 |

---

## 4. Phase Scoping

### Phase 1 — MVP

- `validate_voxel_positions()` detects and rejects maps with duplicate `pos` entries.
- Error message includes the duplicate position.
- Bounds check is preserved and executes before the duplicate check.
- Unit test `test_duplicate_voxel_position` added to `validation.rs`.
- All existing tests continue to pass; both binaries compile cleanly.

---

## 5. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | The orientation matrix system (Fix 1, commit `eda90e3`) and staircase normalisation (Fix 2, commit `4874885`) are complete and merged. The loader pipeline `migrate_legacy_rotations() → normalise_staircase_variants() → validate_map()` is stable. |
| 2 | `VoxelData::pos` is `(i32, i32, i32)`, which implements `Hash` and `Eq`, making it directly usable as a `HashSet` key. |
| 3 | Deduplication (merging two entries at the same position into one) is explicitly out of scope. The author must correct the map file. |
| 4 | The fix is reject-on-first-duplicate (not collect-all-duplicates). A single-pass `HashSet::insert` returning `false` is sufficient. |

---

## 6. Open Questions

*All questions resolved.*

| # | Question | Resolution |
|---|----------|------------|
| 1 | Should the validator collect all duplicates before returning, or fail-fast on the first? | **Fail-fast.** Consistent with all other `validate_voxel_positions()` checks, which return on the first error found. |
| 2 | Does the bounds check or duplicate check run first? | **Bounds check first** (FR-2.3.1). The existing per-voxel `if x < 0 || ...` guard runs at the top of each iteration; `HashSet::insert` runs after it in the same loop. |

---

## 7. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | Fix 1 — Orientation matrix system (`map-format-multi-axis-rotation`) | **Done** (commit `eda90e3`) | Team |
| 2 | Fix 2 — Staircase normalisation (`staircase-double-rotation`) | **Done** (commit `4874885`) | Team |

---

## 8. Reference: Example Scenarios

| Type | RON snippet | Expected result |
|------|-------------|-----------------|
| Valid — no duplicates | Two voxels at `(0,0,0)` and `(1,0,0)` | `validate_map()` returns `Ok(())` |
| Duplicate — same position | Two voxels at `(3,1,5)` | `Err(ValidationError("Duplicate voxel position (3, 1, 5)"))` |
| Out-of-bounds AND duplicate | Two voxels at `(999,0,0)` in a 64-wide map | `Err(InvalidVoxelPosition(999, 0, 0))` — bounds error takes precedence |

---

*Created: 2026-03-31*  
*Source: `docs/investigations/2026-03-22-1427-map-format-analysis.md` — Finding 4*
