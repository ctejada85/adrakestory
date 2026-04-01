# Fix: Pillar Geometry Does Not Match Its Name

**Date:** 2026-03-31
**Severity:** Low (p3)
**Component:** Sub-voxel geometry — `src/systems/game/map/geometry/patterns.rs`, format pattern — `src/systems/game/map/format/patterns.rs`, editor UI — `src/editor/ui/properties/voxel_tools.rs`

---

## Story

As a level designer, I want the `Pillar` pattern to behave as a floor-to-ceiling
column so that stacking Pillar voxels vertically produces a solid column with
continuous collision, matching what the name and the editor preview both imply.

---

## Description

`SubVoxelPattern::Pillar` occupies only sub-voxels 3–4 on all three axes — a
2×2×2 cube (8 sub-voxels) at the dead centre of the voxel cell
(`src/systems/game/map/geometry/patterns.rs:48–62`). It is not a column. In
world space, the occupied region is a **0.25 × 0.25 × 0.25** cube centred at
the voxel origin, with ~0.875 world units of empty, uncollided space above and
below it inside the voxel cell.

Three problems follow from this:

1. **Name mismatch.** The identifier `Pillar` universally implies a tall, narrow,
   vertical structural element. The actual shape is a floating centred cube.
2. **Misleading editor preview.** The editor draws a full-height vertical column
   for `Pillar` (`src/editor/ui/properties/voxel_tools.rs:143–153`), which does
   not match the real geometry.
3. **Silent collision gaps.** Stacking Pillar voxels vertically — the natural
   thing a designer would do to create a column — produces a visual appearance
   that looks solid but has ~0.875 world-unit collision gaps between each voxel
   cell. The player can pass through what looks like a solid column.

The fix introduces a new `CenterCube` variant carrying the current 2×2×2
geometry, renames the `Pillar` enum variant to a true floor-to-ceiling 2×2×8
column geometry, and preserves backward compatibility via
`#[serde(alias = "Pillar")]` on the old shape and migration of existing map
files. The editor preview for `Pillar` is corrected to match the new geometry,
and the preview for `CenterCube` is updated to show a small centred cube.

---

## Acceptance Criteria

1. `SubVoxelPattern::Pillar` geometry spans y = 0–7 (a 2×2×8 floor-to-ceiling
   column: 32 sub-voxels at x ∈ {3,4}, z ∈ {3,4}, y ∈ {0..8}).
2. A new `SubVoxelPattern::CenterCube` variant carries the former 2×2×2 geometry
   (8 sub-voxels at x, y, z ∈ {3,4}).
3. Maps saved with `pattern: Some(Pillar)` before this fix load correctly after
   the fix: the old `Pillar` token must deserialise as `CenterCube` (i.e. the
   2×2×2 geometry is preserved, not silently changed to the new column).
4. The editor pattern dropdown shows both `Pillar` and `CenterCube` as selectable
   options.
5. The editor preview for `Pillar` shows a full-height column; the editor preview
   for `CenterCube` shows a small centred square.
6. Stacking two `Pillar` voxels vertically in a test map produces no collision
   gap between them (the geometry of adjacent cells meets at the cell boundary).
7. `cargo build` succeeds for both `adrakestory` and `map_editor` with zero new
   errors or warnings.
8. `cargo test` passes with no new failures.
9. `cargo clippy` reports zero new errors.
10. `docs/api/map-format-spec.md` is updated to document both `Pillar` (column)
    and `CenterCube` variants.

---

## Non-Functional Requirements

- The serde alias `#[serde(alias = "Pillar")]` on `CenterCube` must be set so
  that all existing map files round-trip correctly without any data migration
  pass. Loading an old map must never silently change the visual geometry.
- All existing derives on `SubVoxelPattern` (`Serialize, Deserialize, Clone,
  Copy, Debug, PartialEq, Eq, Default`) must be preserved.
- `SubVoxelPattern::Full` remains the `#[default]`, not `Pillar` or `CenterCube`.
- Both binaries must compile without new warnings after the change.
- The 8-sub-voxel geometry test (`geometry/tests.rs:61–67`) must be updated to
  reflect `CenterCube`; a new test for the `Pillar` column geometry is required.

---

## Tasks

1. Add `SubVoxelGeometry::column_2x2()` (or equivalent) in
   `src/systems/game/map/geometry/patterns.rs` implementing the 2×2×8 geometry
   (x ∈ {3,4}, y ∈ {0..8}, z ∈ {3,4}).
2. Add `SubVoxelPattern::CenterCube` to the enum in
   `src/systems/game/map/format/patterns.rs` with
   `#[serde(alias = "Pillar")]` and dispatching to `SubVoxelGeometry::pillar()`
   (the existing 2×2×2 geometry).
3. Change `SubVoxelPattern::Pillar` dispatch in `geometry_with_rotation()` to
   call the new 2×2×8 geometry function.
4. Update the editor pattern dropdown in
   `src/editor/ui/properties/voxel_tools.rs` to include `CenterCube` and
   correct the `Pillar` preview to show a full-height column.
5. Update the pattern dropdown in `src/editor/ui/toolbar/tool_options.rs` to
   include `CenterCube`.
6. Update hotbar pattern cycling in `src/editor/controller/hotbar.rs` to
   include `CenterCube`.
7. Update `docs/api/map-format-spec.md` to document both `Pillar` (column) and
   `CenterCube` variants.
8. Update or add unit tests: rename the existing `test_pillar()` geometry test
   to `test_center_cube()`, add a `test_pillar_column()` that asserts 32
   sub-voxels spanning y = 0–7, and add a `test_center_cube_pattern_has_8_positions()`
   for the format dispatch.
9. Run `cargo build`, `cargo test`, `cargo clippy`; fix any failures or new
   warnings.
