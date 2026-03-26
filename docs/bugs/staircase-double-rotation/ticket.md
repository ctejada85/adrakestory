# Fix: Staircase Directional Variant Double Rotation

**Date:** 2026-03-26  
**Severity:** Medium (p2)  
**Component:** Map format — `SubVoxelPattern` / `src/systems/game/map/format/`

---

## Story

As a level designer, I want staircase orientation to be fully described by the voxel's `rotation` field so that rotating a staircase in the editor produces the geometry I see and no hidden pre-bake is silently compounded with my explicit rotation.

---

## Description

Three staircase pattern variants (`StaircaseNegX`, `StaircaseZ`, `StaircaseNegZ`) bake a Y-axis rotation into `SubVoxelPattern::geometry()` at `patterns.rs:64–74`. When a non-`None` orientation matrix is also present on the voxel, it is applied on top of the hidden pre-bake, producing compounded geometry that neither the file nor the editor communicates to the author.

The fix has two parts: (1) a loader normalisation pass (`normalise_staircase_variants()`) that runs after `migrate_legacy_rotations()` and before `validate_map()`, converting all directional staircase variants to `Staircase` (the single canonical variant, renamed from `StaircaseX`) and absorbing the implicit pre-bake into the voxel's explicit orientation matrix; and (2) a map editor UI change that removes the directional variants from the pattern picker. The three directional variants remain deserialisation-compatible for backward compat (kept as enum variants but never written on save). The old `StaircaseX` name is retained as a `#[serde(alias)]` on `Staircase`.

See `bug.md` for the full bug description, reproduction steps, and impact analysis.

---

## Acceptance Criteria

1. Loading a map with `pattern: Some(StaircaseZ), rotation: None` produces geometry identical to `SubVoxelGeometry::staircase_x().rotate(Y, 1)` — the pre-bake is preserved in the orientation matrix.
2. Loading a map with `pattern: Some(StaircaseZ), rotation: Some(i)` where `orientations[i]` = Y+90° produces geometry identical to `SubVoxelGeometry::staircase_x().rotate(Y, 1).rotate(Y, 1)` (Y+180°). The double rotation is correctly composed, not double-applied post-normalisation.
3. Loading a map with `pattern: Some(StaircaseNegX), rotation: None` produces geometry identical to `SubVoxelGeometry::staircase_x().rotate(Y, 2)`.
4. Loading a map with `pattern: Some(StaircaseNegZ), rotation: None` produces geometry identical to `SubVoxelGeometry::staircase_x().rotate(Y, 3)`.
5. After normalisation, all three directional variants are converted to `pattern: Some(Staircase)` in memory; if the map is saved, the written file contains only `Staircase`.
6. A map with `pattern: Some(Staircase), rotation: None` is unchanged by the normalisation pass.
7. `StaircaseNegX`, `StaircaseZ`, and `StaircaseNegZ` written in a RON file continue to deserialise without error (backward compatibility). `StaircaseX` also continues to deserialise as `Staircase` via a `#[serde(alias)]`.
8. The map editor pattern picker does not expose `StaircaseNegX`, `StaircaseZ`, or `StaircaseNegZ`; only `Staircase` is selectable.
9. `cargo test` passes with no failures; new unit tests cover: normalisation of each directional variant with `rotation: None`, normalisation with an existing non-None rotation, identity edge case (`StaircaseNegZ + M_y90 = None`), `Staircase` unchanged by pass, and round-trip test (save after normalisation loads to same geometry).

---

## Non-Functional Requirements

- Must not introduce `f32` or floating-point arithmetic.
- `SubVoxelGeometry::rotate()` and `rotate_point()` must not be modified.
- `SubVoxelPattern::geometry()` pre-bake code in `patterns.rs:64–74` must not be removed — it is still used for voxels loaded from legacy maps before normalisation runs. The normalisation pass removes the need to call it with a compounded matrix at spawn time.
- Changes must compile for both the `adrakestory` and `map_editor` binaries.
- The normalisation pass must be O(n) in the number of voxels with no additional per-voxel allocations beyond orientation deduplication.
- The `Staircase` variant (renamed from `StaircaseX`) must serialise as `"Staircase"` in new maps; `"StaircaseX"` must remain loadable via `#[serde(alias)]`.

---

## Tasks

1. Rename `SubVoxelPattern::StaircaseX` to `Staircase` in `src/systems/game/map/format/patterns.rs`. Add `#[serde(alias = "StaircaseX")]` to preserve backward compat for files written with the old name. Update all internal references (`geometry()` match arm, tests, etc.).
2. Add a `normalise_staircase_variants(orientations: &mut Vec<OrientationMatrix>, voxels: &mut [VoxelData])` function in `src/systems/game/map/format/rotation.rs` that iterates voxels and for each voxel whose `pattern` is `StaircaseNegX`, `StaircaseZ`, or `StaircaseNegZ`:
   - Computes the implicit pre-bake matrix using `axis_angle_to_matrix(Y, angle)` (angle = 2, 1, 3 respectively).
   - Retrieves the voxel's current orientation matrix (identity if `rotation: None`, else `map.orientations[rotation]`).
   - Composes the two matrices via `multiply_matrices(existing, pre_bake)`.
   - If composed equals `IDENTITY`, sets `voxel.rotation = None`; otherwise calls `find_or_insert_orientation` and sets `voxel.rotation = Some(index)`.
   - Sets `voxel.pattern = Some(SubVoxelPattern::Staircase)`.
3. Call `normalise_staircase_variants(&mut map.orientations, &mut map.world.voxels)` in `src/systems/game/map/loader.rs` after the `migrate_legacy_rotations()` call and before `validate_map()` in both `load_from_file()` (line 144) and `load_simple()` (line 162).
4. Add doc comments to `StaircaseNegX`, `StaircaseZ`, and `StaircaseNegZ` in `patterns.rs` noting they are backward-compat aliases normalised to `Staircase` on load and never written on save.
5. Remove `StaircaseNegX`, `StaircaseZ`, and `StaircaseNegZ` from the map editor pattern picker. Only `Staircase` should be selectable; the desired facing direction is set via the rotation tool.
6. Update `docs/api/map-format-spec.md` to document: `Staircase` as the canonical variant (renamed from `StaircaseX`); `StaircaseX` as a load-only alias; and `StaircaseNegX`/`StaircaseZ`/`StaircaseNegZ` as backward-compat aliases normalised on load.
7. Write unit tests covering all acceptance criteria (see §Acceptance Criteria), including the identity edge case (`StaircaseNegZ + M_y90 → rotation: None`).
8. Run `cargo test` and `cargo clippy`; fix any failures or warnings.
