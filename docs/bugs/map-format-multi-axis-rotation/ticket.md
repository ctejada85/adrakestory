# Fix: Multi-Axis Voxel Rotation Composition

**Date:** 2026-03-22  
**Severity:** High  
**Component:** Map format — `MapData` / `VoxelData` / `src/systems/game/map/format/`

---

## Story

As a level designer, I want voxels to retain their full orientation when I rotate them around multiple axes so that I can create diagonal staircases, wall-mounted platforms, and other non-cardinal arrangements without silent data loss.

---

## Description

`RotationState::compose()` discards the earlier rotation when two rotations use different axes (`rotation.rs:32–36`). Only the last axis is stored in the map file, so any multi-axis orientation is permanently lost on save.

The fix introduces a top-level `orientations` list in the map file. Each entry is a 3×3 integer rotation matrix. Voxels reference an entry by index (`rotation: Some(0)`). The editor computes new matrices via integer 3×3 matrix multiplication and deduplicates before appending to the list. Existing single-axis map files are transparently migrated on load via a backward-compat shim that converts the old `rotation_state: Some((axis, angle))` syntax into the equivalent matrix. The staircase double-rotation issue and the Fence rotation no-op are out of scope for this ticket.

---

## Acceptance Criteria

1. Applying a Y+90° rotation followed by an X+90° rotation to a voxel in the editor and saving produces a map file with an `orientations` entry whose matrix, when applied via `apply_orientation_matrix()`, yields sub-voxel geometry identical to calling `SubVoxelGeometry::rotate(Y,1)` then `SubVoxelGeometry::rotate(X,1)` directly.
2. A voxel with `rotation_state: Some((axis: Y, angle: 1))` in an existing map file loads and renders identically before and after this change.
3. A voxel with `rotation_state: Some((axis: X, angle: 2))` in an existing map file loads and renders identically before and after this change.
4. A voxel entry with `rotation: Some(5)` in a map whose `orientations` list has fewer than 6 entries is rejected at validation time with a descriptive error.
5. A matrix entry in the `orientations` list that is not a valid 90°-grid rotation (determinant ≠ 1, or entries outside {−1, 0, 1}) is rejected at validation time.
6. Rotating the same voxel twice to the same resulting orientation produces only one entry in the `orientations` list (deduplication by matrix value).
7. `cargo test` passes with no failures; new unit tests cover: matrix application parity with `rotate()`, multi-axis composition, index out-of-bounds rejection, invalid matrix rejection, deduplication, and legacy `(axis, angle)` migration for all 12 single-axis states.

---

## Non-Functional Requirements

- Must not introduce `f32` or floating-point arithmetic anywhere in the orientation pipeline.
- `SubVoxelGeometry::rotate()` and `rotate_point()` must not be modified.
- Changes must compile for both the `adrakestory` and `map_editor` binaries.
- Map load time must not increase measurably; orientation lookup is O(1) per voxel (index into a `Vec`).
- The `orientations` list must not duplicate equivalent matrices; the editor must deduplicate by value before appending.

---

## Tasks

1. Add `type OrientationMatrix = [[i32; 3]; 3]` to `src/systems/game/map/format/rotation.rs`; implement `axis_angle_to_matrix(axis: RotationAxis, angle: i32) -> OrientationMatrix` and `multiply_matrices(a: &OrientationMatrix, b: &OrientationMatrix) -> OrientationMatrix`.
2. Implement `apply_orientation_matrix(geometry: SubVoxelGeometry, matrix: &OrientationMatrix) -> SubVoxelGeometry` that decomposes the matrix into at most two `SubVoxelGeometry::rotate()` calls.
3. Add `#[serde(default)] pub orientations: Vec<OrientationMatrix>` to `MapData` in `src/systems/game/map/format/mod.rs`.
4. Replace `rotation_state: Option<RotationState>` on `VoxelData` with `#[serde(default)] rotation: Option<usize>` and add `#[serde(default)] rotation_state: Option<LegacyRotationState>` for the compat shim (`world.rs`).
5. Implement `migrate_legacy_rotations(map: &mut MapData)` in the loader — converts all `rotation_state` fields to matrix entries in `map.orientations` and sets the corresponding `rotation` index.
6. Update `SubVoxelPattern::geometry_with_rotation()` in `patterns.rs` to accept `Option<&OrientationMatrix>` and call `apply_orientation_matrix()`.
7. Update `spawner/chunks.rs` to look up `map_data.orientations.get(voxel.rotation?)` and pass it to `geometry_with_rotation()`.
8. Update editor rotation composition in `src/editor/tools/input/helpers.rs` to use `multiply_matrices()` and deduplicate into `map_data.orientations`.
9. Add validation rules to `validation.rs`: (a) each matrix is a valid 90°-grid rotation; (b) each voxel `rotation` index is within bounds of `map.orientations`.
10. Write unit tests covering all acceptance criteria (see §Acceptance Criteria).
11. Run `cargo test` and `cargo clippy`; fix any failures or warnings.
12. Manually verify in the editor: place a voxel, rotate it around Y then X, save the file, inspect the RON to confirm the `orientations` entry and `rotation` index, reload, confirm geometry is correct.
