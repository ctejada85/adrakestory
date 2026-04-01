# Requirements — Pillar Geometry / Name Mismatch

**Source:** Map Format Analysis Investigation — 2026-03-22
**Bug:** `docs/bugs/pillar-geometry-name-mismatch/ticket.md`
**Status:** Final — all questions resolved 2026-03-31

---

## 1. Overview

`SubVoxelPattern::Pillar` occupies sub-voxels 3–4 on all three axes: a
2×2×2 cube (8 sub-voxels) at the dead centre of the voxel cell
(`src/systems/game/map/geometry/patterns.rs:48–62`). In world space this is a
**0.25 × 0.25 × 0.25** cube. It is not a column.

Three observable defects result:

- **Name mismatch** — `Pillar` universally implies a tall vertical structural
  element. The real shape is a floating centred cube.
- **Misleading editor preview** — the editor draws a full-height vertical column
  for `Pillar` (`src/editor/ui/properties/voxel_tools.rs:143–153`), which does
  not represent the actual geometry.
- **Silent collision gaps** — stacking `Pillar` voxels vertically (the expected
  author intent) leaves ~0.875 world-unit gaps between cells with no collision
  geometry. The player can walk through what appears to be a solid column.

The fix introduces `SubVoxelPattern::CenterCube` carrying the current 2×2×2
geometry and repurposes `SubVoxelPattern::Pillar` to a true 2×2×8
floor-to-ceiling column. Backward compatibility is maintained via a serde alias
so all existing map files continue to load unchanged.

---

## 2. Functional Requirements

### 2.1 New CenterCube Variant

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1.1 | Add `SubVoxelPattern::CenterCube` to the `SubVoxelPattern` enum in `src/systems/game/map/format/patterns.rs`. | Phase 1 |
| FR-2.1.2 | `CenterCube` must carry `#[serde(alias = "Pillar")]` so that map files written before this fix (containing `pattern: Some(Pillar)`) deserialise as `CenterCube` without error or geometry change. | Phase 1 |
| FR-2.1.3 | `CenterCube` geometry must be exactly the current `SubVoxelGeometry::pillar()` implementation: x, y, z ∈ {3, 4}, 8 sub-voxels total. | Phase 1 |

### 2.2 Repurposed Pillar Variant — Column Geometry

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.2.1 | `SubVoxelPattern::Pillar` geometry must span x ∈ {3, 4}, y ∈ {0..8}, z ∈ {3, 4}: a 2×2×8 floor-to-ceiling column with 32 sub-voxels. | Phase 1 |
| FR-2.2.2 | The new column geometry function (`SubVoxelGeometry::column_2x2()` or equivalent) must be added to `src/systems/game/map/geometry/patterns.rs`. | Phase 1 |
| FR-2.2.3 | `SubVoxelPattern::Pillar` in `geometry_with_rotation()` must dispatch to the new 2×2×8 geometry function. | Phase 1 |

### 2.3 Backward Compatibility

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.3.1 | Map files containing `pattern: Some(Pillar)` must load without error after the fix, deserialising as `CenterCube` (the 2×2×2 geometry) via the serde alias (FR-2.1.2). | Phase 1 |
| FR-2.3.2 | Map files containing no `pattern` field (defaulting to `Full`) are unaffected. | Phase 1 |
| FR-2.3.3 | The new `Pillar` token (column geometry) is written to file only when the author explicitly selects the new `Pillar` variant in the editor or writes it manually. No existing map file is silently upgraded. | Phase 1 |

### 2.4 Editor Updates

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.4.1 | The pattern dropdown in `src/editor/ui/properties/voxel_tools.rs` must include `CenterCube` as a selectable option. | Phase 1 |
| FR-2.4.2 | The editor preview for `Pillar` must be updated to show a full-height column visual (matching the new 2×2×8 geometry). | Phase 1 |
| FR-2.4.3 | The editor preview for `CenterCube` must show a small centred square (matching the 2×2×2 geometry). | Phase 1 |
| FR-2.4.4 | The pattern dropdown in `src/editor/ui/toolbar/tool_options.rs` must include `CenterCube`. | Phase 1 |
| FR-2.4.5 | The hotbar pattern cycling array in `src/editor/controller/hotbar.rs` must include `CenterCube`. | Phase 1 |

### 2.5 Documentation Update

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.5.1 | `docs/api/map-format-spec.md` must be updated to document both `Pillar` (2×2×8 column, 32 sub-voxels) and `CenterCube` (2×2×2 centred cube, 8 sub-voxels). | Phase 1 |

---

## 3. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-3.1 | `#[serde(alias = "Pillar")]` on `CenterCube` must be the sole mechanism for backward compat. No loader migration pass is required. | Phase 1 |
| NFR-3.2 | All existing derives on `SubVoxelPattern` (`Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default`) must be preserved on the modified enum. | Phase 1 |
| NFR-3.3 | `SubVoxelPattern::Full` remains the `#[default]` variant. | Phase 1 |
| NFR-3.4 | Both `adrakestory` and `map_editor` binaries must compile without new errors or warnings. | Phase 1 |
| NFR-3.5 | The existing geometry factory tests and format dispatch tests for `Pillar` must be updated to target `CenterCube`; new tests for the `Pillar` column geometry must be added. | Phase 1 |

---

## 4. Phase Scoping

### Phase 1 — MVP (this fix)

- `CenterCube` variant added with serde alias, 2×2×2 geometry.
- `Pillar` repurposed to 2×2×8 column geometry.
- Editor dropdown, preview, hotbar all updated.
- `map-format-spec.md` updated.
- Unit tests updated and added.
- Both binaries compile cleanly.

### Phase 2 — Future (out of scope)

- Consider adding further structural column variants (e.g. thicker `Pillar4x4`)
  if the palette needs expansion.
- Consider whether the Fence variant's full-height post should be renamed or
  refactored now that `Pillar` is a true column (they share the 2×2 XZ footprint
  but serve different visual purposes).

---

## 5. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | The serde alias approach is sufficient for backward compatibility. No existing map file needs a migration pass because `pattern: Some(Pillar)` will deserialise as `CenterCube` transparently. |
| 2 | Changing `Pillar` to a 2×2×8 geometry does not affect any currently existing map file: all existing files that had `Pillar` will load as `CenterCube` via the alias. Only new `Pillar` placements (post-fix) will use the column geometry. |
| 3 | The editor preview is a small schematic drawing, not a true voxel render. Updating it to show a tall narrow bar (for `Pillar`) and a small centred square (for `CenterCube`) is sufficient; pixel-perfect accuracy is not required. |
| 4 | The `geometry_with_rotation()` dispatch for `CenterCube` does not need to handle rotation — the 2×2×2 cube is symmetric, so rotation has no visual effect (same as the current `Pillar` behaviour). |
| 5 | The column geometry for the new `Pillar` is also symmetric around the Y axis (x and z are fixed at {3,4}), so rotation of the column around Y is a no-op visually. Rotation around X or Z would tilt the column, which is a legitimate author action via the orientation matrix system and requires no special handling. |

---

## 6. Open Questions

*All questions resolved.*

| # | Question | Resolution |
|---|----------|------------|
| 1 | Should the old geometry be preserved (option a: rename + new variant) or should `Pillar` simply change geometry (option b)? | **Option (a)**: preserve the old geometry as `CenterCube`. Maps authored with `Pillar` retain their 2×2×2 appearance via the serde alias. New `Pillar` placements get the column. This is non-breaking. |
| 2 | What serde alias mechanism to use? | `#[serde(alias = "Pillar")]` on `CenterCube`. The variant serialises as `"CenterCube"` when written; on read, `"Pillar"` in the file deserialises as `CenterCube`. This is the same mechanism used for `Platform → PlatformXZ` and `StaircaseX → Staircase`. |
| 3 | Does the editor cycling array (camera.rs) need updating? | Yes — `CenterCube` should be added to the cycle array to remain accessible from the keyboard shortcut. |
| 4 | Should the column be 2×2×8 or narrower/wider? | 2×2×8, matching the XZ footprint of the existing fence post (`FencePost` uses the same x ∈ {3,4}, z ∈ {3,4} footprint). This gives a visually slim but non-trivial column that looks proportionate next to fence posts. |

---

## 7. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | Fix 6 — VoxelType module relocation | **Done** (commit `aa7dbb8`) | Team |

No blockers. This is an independent geometry / format change.

---

## 8. Reference: Sub-Voxel Geometry

Sub-voxels use a 0-indexed 8×8×8 grid. Each sub-voxel is
`SUB_VOXEL_SIZE = 0.125` world units. The voxel cell spans [−0.5, +0.5] on
each axis relative to the voxel origin.

| Sub-voxel index | Centre world offset | Bounds |
|---|---|---|
| 0 | −0.4375 | [−0.5, −0.375] |
| 1 | −0.3125 | [−0.375, −0.25] |
| 2 | −0.1875 | [−0.25, −0.125] |
| 3 | −0.0625 | [−0.125, 0.0] |
| 4 | +0.0625 | [0.0, +0.125] |
| 5 | +0.1875 | [+0.125, +0.25] |
| 6 | +0.3125 | [+0.25, +0.375] |
| 7 | +0.4375 | [+0.375, +0.5] |

**CenterCube** (current `Pillar`): x, y, z ∈ {3, 4} — 8 sub-voxels in the
centre 25% of the cell on every axis. World extent: 0.25 × 0.25 × 0.25.

**New Pillar (column)**: x ∈ {3, 4}, y ∈ {0..8}, z ∈ {3, 4} — 32 sub-voxels
spanning full cell height. World extent: 0.25 wide × 1.0 tall × 0.25 deep.
Geometry at the top cell boundary (y=7 max = +0.5) and bottom boundary
(y=0 min = −0.5) is contiguous with the adjacent stacked Pillar cell: no gap.

---

## 9. Reference: Files Affected

| File | Change |
|------|--------|
| `src/systems/game/map/geometry/patterns.rs` | **Modified** — add `column_2x2()` factory function |
| `src/systems/game/map/format/patterns.rs` | **Modified** — add `CenterCube` variant with alias; change `Pillar` dispatch |
| `src/systems/game/map/geometry/tests.rs` | **Modified** — rename `test_pillar()` → `test_center_cube()`; add `test_pillar_column()` |
| `src/systems/game/map/format/patterns.rs` (tests) | **Modified** — rename `test_pillar_pattern_has_8_positions()` → `test_center_cube_pattern_has_8_positions()`; add column test |
| `src/editor/ui/properties/voxel_tools.rs` | **Modified** — add `CenterCube` to dropdown; fix `Pillar` preview; add `CenterCube` preview |
| `src/editor/ui/toolbar/tool_options.rs` | **Modified** — add `CenterCube` to pattern name list and dropdown |
| `src/editor/controller/hotbar.rs` | **Modified** — add `CenterCube` to hotbar palette and name |
| `src/editor/camera.rs` | **Modified** — add `CenterCube` to pattern cycle array |
| `docs/api/map-format-spec.md` | **Modified** — document `Pillar` (column) and `CenterCube` variants |

---

*Created: 2026-03-31*
*Source: `docs/investigations/2026-03-22-1427-map-format-analysis.md` — Finding 7*
