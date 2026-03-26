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

The fix normalises all directional staircase variants to `StaircaseX` in a loader pass (`normalise_staircase_variants()`) that runs after `migrate_legacy_rotations()` and before `validate_map()`. The implicit pre-bake is absorbed into the voxel's explicit orientation matrix via `multiply_matrices()` + `find_or_insert_orientation()`. The three directional variants remain deserialisation-compatible for backward compat but are never written on save.

See `bug.md` for the full bug description, reproduction steps, and impact analysis.

---

## Acceptance Criteria

1. Loading a map with `pattern: Some(StaircaseZ), rotation: None` produces geometry identical to `SubVoxelGeometry::staircase_x().rotate(Y, 1)` — the pre-bake is preserved in the orientation matrix.
2. Loading a map with `pattern: Some(StaircaseZ), rotation: Some(i)` where `orientations[i]` = Y+90° produces geometry identical to `SubVoxelGeometry::staircase_x().rotate(Y, 1).rotate(Y, 1)` (Y+180°). The double rotation is correctly composed, not double-applied post-normalisation.
3. Loading a map with `pattern: Some(StaircaseNegX), rotation: None` produces geometry identical to `SubVoxelGeometry::staircase_x().rotate(Y, 2)`.
4. Loading a map with `pattern: Some(StaircaseNegZ), rotation: None` produces geometry identical to `SubVoxelGeometry::staircase_x().rotate(Y, 3)`.
5. After normalisation, all three directional variants are converted to `pattern: Some(StaircaseX)` in memory; if the map is saved, the written file contains only `StaircaseX`.
6. A map with `pattern: Some(StaircaseX), rotation: None` is unchanged by the normalisation pass.
7. `StaircaseNegX`, `StaircaseZ`, and `StaircaseNegZ` written in a RON file continue to deserialise without error (backward compatibility).
8. `cargo test` passes with no failures; new unit tests cover: normalisation of each directional variant with `rotation: None`, normalisation with an existing non-None rotation, `StaircaseX` unchanged by pass, and round-trip test (save after normalisation loads to same geometry).

---

## Non-Functional Requirements

- Must not introduce `f32` or floating-point arithmetic.
- `SubVoxelGeometry::rotate()` and `rotate_point()` must not be modified.
- `SubVoxelPattern::geometry()` pre-bake code in `patterns.rs:64–74` must not be removed — it is still used for voxels loaded from legacy maps before normalisation runs. The normalisation pass removes the need to call it with a compounded matrix at spawn time.
- Changes must compile for both the `adrakestory` and `map_editor` binaries.
- The normalisation pass must be O(n) in the number of voxels with no additional per-voxel allocations beyond orientation deduplication.

---

## Tasks

1. Add a `normalise_staircase_variants(map: &mut MapData)` function in `src/systems/game/map/format/rotation.rs` (or `loader.rs`) that iterates `map.world.voxels` and for each voxel whose `pattern` is `StaircaseNegX`, `StaircaseZ`, or `StaircaseNegZ`:
   - Computes the implicit pre-bake matrix using `axis_angle_to_matrix(Y, angle)` (angle = 2, 1, 3 respectively).
   - Retrieves the voxel's current orientation matrix (identity if `rotation: None`, else `map.orientations[rotation]`).
   - Composes the two matrices via `multiply_matrices(pre_bake, current)`.
   - Calls `find_or_insert_orientation(&mut map.orientations, composed)` to obtain the index.
   - Sets `voxel.pattern = Some(SubVoxelPattern::StaircaseX)` and `voxel.rotation = Some(index)`.
2. Call `normalise_staircase_variants(map)` in `src/systems/game/map/loader.rs` after the `migrate_legacy_rotations(map)` call and before `validate_map(map)`.
3. Verify the composition order: the pre-bake is applied first (inner), then the existing orientation (outer). Confirm with a unit test that the final geometry matches calling `staircase_x().rotate(pre_bake_axis, pre_bake_angle)` then applying the existing orientation.
4. Ensure `StaircaseNegX`, `StaircaseZ`, `StaircaseNegZ` remain as valid `SubVoxelPattern` enum variants with `#[serde(rename)]` or `#[serde(alias)]` as appropriate so existing map files deserialise without error.
5. Update `src/systems/game/map/format/patterns.rs` to add a doc comment to the three directional variants noting they are backward-compat aliases normalised to `StaircaseX` on load and never written on save.
6. Update `docs/api/map-format-spec.md` to document that `StaircaseNegX`, `StaircaseZ`, and `StaircaseNegZ` are normalised on load; the canonical form is `StaircaseX` + rotation.
7. Write unit tests covering all acceptance criteria (see §Acceptance Criteria).
8. Run `cargo test` and `cargo clippy`; fix any failures or warnings.
