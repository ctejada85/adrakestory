# Requirements — Staircase Directional Variant Normalisation

**Source:** Map Format Analysis Investigation — 2026-03-22  
**Bug:** `docs/bugs/staircase-double-rotation/bug.md`  
**Status:** Final — all questions resolved 2026-03-26

---

## 1. Overview

`SubVoxelPattern` exposes four staircase variants: `StaircaseX`, `StaircaseNegX`, `StaircaseZ`, and `StaircaseNegZ`. The three directional variants are not independent geometries — they are implemented in `SubVoxelPattern::geometry()` (`patterns.rs:64–74`) as pre-baked Y-axis rotations of `StaircaseX`:

| Variant | Pre-bake in `geometry()` |
|---------|--------------------------|
| `StaircaseNegX` | `staircase_x().rotate(Y, 2)` |
| `StaircaseZ`    | `staircase_x().rotate(Y, 1)` |
| `StaircaseNegZ` | `staircase_x().rotate(Y, 3)` |

The orientation matrix system introduced by Fix 1 (`docs/bugs/map-format-multi-axis-rotation/`) applies an explicit matrix on top of this pre-baked geometry at spawn time via `geometry_with_rotation()`. The two rotation applications compound silently: a `StaircaseZ` voxel with `rotation: Some(i)` (Y+90°) produces Y+180° geometry, not Y+90° geometry. The file and the editor give no indication of the compounding.

The fix has two parts: (1) a loader normalisation pass that converts all directional staircase variants to `Staircase` (the single canonical variant, renamed from `StaircaseX`) and absorbs the implicit pre-bake into the voxel's explicit orientation matrix; and (2) a map editor UI change that removes the directional variants from the pattern picker, so they can never be authored going forward. After normalisation the file is the single source of truth for orientation; no geometric meaning is embedded in the pattern name.

---

## 2. Functional Requirements

### 2.1 Normalisation Pass

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1.1 | The loader must include a `normalise_staircase_variants()` pass that runs after `migrate_legacy_rotations()` and before `validate_map()`. | Phase 1 |
| FR-2.1.2 | For each voxel with `pattern: Some(StaircaseNegX)`, the pass must compose the Y+180° pre-bake matrix with the voxel's current orientation (identity if `rotation: None`, else `orientations[rotation]`) via `multiply_matrices(pre_bake, current)`, find or insert the composed matrix in `map.orientations`, set `voxel.rotation = Some(index)`, and set `voxel.pattern = Some(Staircase)`. | Phase 1 |
| FR-2.1.3 | For each voxel with `pattern: Some(StaircaseZ)`, the pass must apply the same logic with the Y+90° pre-bake matrix. | Phase 1 |
| FR-2.1.4 | For each voxel with `pattern: Some(StaircaseNegZ)`, the pass must apply the same logic with the Y+270° pre-bake matrix. | Phase 1 |
| FR-2.1.5 | A voxel with `pattern: Some(Staircase)` must be unchanged by the normalisation pass. | Phase 1 |
| FR-2.1.6 | A voxel with no staircase pattern must be unchanged by the normalisation pass. | Phase 1 |

### 2.2 Composition Order

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.2.1 | The composition must apply the pre-bake as the inner (first-applied) rotation: `M_composed = multiply_matrices(M_existing, M_prebake)`. The result must produce geometry identical to calling `staircase_x().rotate(pre_bake_axis, pre_bake_angle)` then applying the existing orientation matrix. | Phase 1 |
| FR-2.2.2 | Where the voxel had `rotation: None` (identity), the composed matrix is simply `M_prebake`. The normalisation must set `voxel.rotation = Some(index_of_prebake)` and `voxel.pattern = Some(Staircase)`. | Phase 1 |

### 2.3 Orientation List Deduplication

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.3.1 | The normalisation pass must use the existing `find_or_insert_orientation()` helper (or equivalent) to deduplicate composed matrices before appending to `map.orientations`. The same matrix must not appear twice in the orientations list. | Phase 1 |
| FR-2.3.2 | If the composed matrix is already present in `map.orientations`, only the index is reused — no new entry is appended. | Phase 1 |

### 2.4 Backward Compatibility

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.4.1 | `StaircaseNegX`, `StaircaseZ`, and `StaircaseNegZ` must continue to deserialise from RON without error. Existing map files that use these variants must load correctly. | Phase 1 |
| FR-2.4.2 | After normalisation, if the map is saved, the written RON must contain only `Staircase` for staircase voxels. The directional variants must never be written on save. | Phase 1 |
| FR-2.4.3 | A map containing only `Staircase` staircase voxels (i.e., already normalised) must load and render identically before and after this change. | Phase 1 |
| FR-2.4.4 | The old `StaircaseX` name must remain loadable as a `#[serde(alias)]` for maps written before this rename. | Phase 1 |

### 2.5 Geometry Correctness

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.5.1 | The geometry produced by loading a normalised voxel (`Staircase` + composed orientation) must be bit-for-bit identical to the geometry that was previously produced by the un-normalised variant + the same explicit orientation (i.e., the double-rotation bug is present before normalisation; after normalisation it is absent and the pre-bake is accounted for exactly once). | Phase 1 |
| FR-2.5.2 | The geometry produced by loading a `StaircaseZ` voxel with `rotation: None` must be identical to `staircase_x().rotate(Y, 1)` — the pre-bake is preserved, not dropped. | Phase 1 |

### 2.6 Documentation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.6.1 | `docs/api/map-format-spec.md` must be updated to state that `StaircaseNegX`, `StaircaseZ`, and `StaircaseNegZ` are backward-compat aliases that are normalised to `Staircase` + rotation on load, and that `StaircaseX` is a backward-compat alias for `Staircase`. | Phase 1 |
| FR-2.6.2 | The three directional variants in `patterns.rs` must carry a doc comment explaining they are load-time aliases and are not written on save. | Phase 1 |

### 2.7 Map Editor

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.7.1 | The map editor pattern picker must not expose `StaircaseNegX`, `StaircaseZ`, or `StaircaseNegZ` as selectable options. Only `Staircase` is available; the desired facing direction is achieved by applying a Y-axis rotation. | Phase 1 |
| FR-2.7.2 | If the editor currently holds an in-memory voxel with a directional staircase variant (e.g., loaded from a file before normalisation runs), the normalisation pass must have converted it before it reaches any editor code. No editor code path should encounter a directional staircase variant at runtime. | Phase 1 |

---

## 3. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-3.1 | All matrix arithmetic in the normalisation pass must use integer types only — no `f32` or floating-point. | Phase 1 |
| NFR-3.2 | The normalisation pass must be O(n) in the number of voxels, with no per-voxel heap allocations beyond orientation list deduplication. | Phase 1 |
| NFR-3.3 | `SubVoxelGeometry::rotate()` and `rotate_point()` must not be modified. | Phase 1 |
| NFR-3.4 | `SubVoxelPattern::geometry()`'s pre-bake code at `patterns.rs:64–74` must not be removed (it may be called by other paths). Only the interaction with `geometry_with_rotation()` is addressed. | Phase 1 |
| NFR-3.5 | Both binaries (`adrakestory` and `map_editor`) must compile without error or warning. | Phase 1 |
| NFR-3.6 | Existing unit tests in `rotation.rs`, `patterns.rs`, and `geometry/tests.rs` must continue to pass. New tests must cover all acceptance criteria from `ticket.md`. | Phase 1 |

---

## 4. Phase Scoping

### Phase 1 — MVP

- `normalise_staircase_variants()` pass added to loader pipeline.
- All three directional variants normalised to `Staircase` + composed orientation on load.
- `StaircaseX` enum variant renamed to `Staircase`; old names kept as `#[serde(alias)]` for backward compat.
- Backward compat: directional variants and `StaircaseX` continue to deserialise.
- On save, only `Staircase` written for staircase voxels.
- Map editor pattern picker exposes only `Staircase`; directional variants removed from UI.
- `map-format-spec.md` updated to document normalisation behaviour and rename.
- All existing tests pass; new tests cover geometry correctness for each variant.

### Phase 2 — Enhanced

- Editor properties panel displays the effective staircase facing direction derived from the orientation matrix.

### Future Phases

- Consider removing `StaircaseNegX`, `StaircaseZ`, `StaircaseNegZ` as enum variants entirely (breaking change) once all known maps have been migrated and saved.

---

## 5. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | Fix 1 (`map-format-multi-axis-rotation`) is complete and merged. The `orientations: Vec<OrientationMatrix>` field on `MapData` and `rotation: Option<usize>` on `VoxelData` are available. |
| 2 | `axis_angle_to_matrix()`, `multiply_matrices()`, and `find_or_insert_orientation()` are available in `src/systems/game/map/format/rotation.rs` (re-exported from `format/mod.rs`). |
| 3 | Only Y-axis pre-bakes are present in the current codebase. No X-axis or Z-axis pre-bakes exist for any other pattern variant. |
| 4 | The `Fence` pattern's separate issue with `geometry_with_rotation()` is out of scope. |
| 5 | The composition order is: pre-bake applied first (inner), existing orientation applied second (outer). This mirrors the order in which the geometry was previously computed. |
| 6 | The maximum number of distinct orientations in any map is 24; deduplication via linear scan of the orientations list is acceptable. |
| 7 | `Staircase` is the new canonical enum variant name. The old `StaircaseX` name is kept as `#[serde(alias = "StaircaseX")]` only. |

---

## 6. Open Questions

*All questions resolved.*

| # | Question | Resolution |
|---|----------|------------|
| 1 | Should `StaircaseNegX`, `StaircaseZ`, `StaircaseNegZ` be kept as full enum variants or collapse to a single variant? | **Single canonical variant**, renamed `Staircase` (from `StaircaseX`). The three directional variants remain as full enum variants (for the normalisation pass to inspect) but are never authored or written. The old `StaircaseX` name is kept as `#[serde(alias = "StaircaseX")]` on `Staircase`. |
| 2 | Should the map editor hide directional variants in Phase 1 or Phase 2? | **Phase 1.** The editor pattern picker exposes only `Staircase`. Normalisation runs on load so no in-memory state ever contains a directional variant. |

---

## 7. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | Fix 1 — Orientation matrix system (`map-format-multi-axis-rotation`) | **Done** (commit `eda90e3`) | Team |
| 2 | `find_or_insert_orientation()` helper available in `format/rotation.rs` | **Done** — added in Fix 1 (`rotation.rs:145`) | Implementer |

---

## 8. Reference: Example RON Snippets

| Type | Example | Notes |
|------|---------|-------|
| Legacy directional (pre-normalisation) | `(pos:(1,0,2), voxel_type:Stone, pattern:Some(StaircaseZ), rotation:None)` | Loads fine; normalisation converts to Staircase + Y+90° |
| Legacy directional with rotation | `(pos:(2,0,1), voxel_type:Stone, pattern:Some(StaircaseZ), rotation:Some(0))` where `orientations[0]` = Y+90° | Normalised to Staircase + composed(Y+90°, Y+90°) = Y+180° |
| Legacy StaircaseX name | `(pos:(0,0,0), voxel_type:Stone, pattern:Some(StaircaseX), rotation:None)` | Deserialises to Staircase via alias; unchanged by normalisation pass |
| Normalised (canonical form) | `(pos:(1,0,2), voxel_type:Stone, pattern:Some(Staircase), rotation:Some(1))` where `orientations[1]` = Y+90° | Written on save; no pre-bake in pattern name |

---

*Created: 2026-03-26*  
*Source: `docs/investigations/2026-03-22-1427-map-format-analysis.md` — Finding 2*
