# Fix: Duplicate Voxel Positions Not Detected

**Date:** 2026-03-31  
**Severity:** Medium (p2)  
**Component:** Map validation — `src/systems/game/map/validation.rs`

---

## Story

As a level designer, I want the map loader to reject a map file that contains two voxels at the same position so that hand-editing mistakes are caught at load time instead of silently producing corrupted chunk meshes.

---

## Description

`validate_voxel_positions()` in `validation.rs:45–57` checks that each voxel is within world bounds but does not check for duplicate `pos` values. When two `VoxelData` entries share the same `(x, y, z)`, both are processed independently: their sub-voxels are unioned in the occupancy grid and both contribute geometry to the chunk mesh. The result is superimposed geometry from two entries with no error or warning. In large hand-edited maps this can produce invisible mesh corruption that is difficult to diagnose.

The fix adds a `HashSet` duplicate-position check to `validate_voxel_positions()` and emits a `MapLoadError::ValidationError` on the first duplicate found. No changes to the spawner, chunk meshing, or editor are required.

Out of scope: deduplication or merging of duplicate entries — the map file must be corrected by the author.

---

## Acceptance Criteria

1. Loading a map where two `VoxelData` entries share the same `pos` returns `Err(MapLoadError::ValidationError(...))` containing the duplicate position before any spawning occurs.
2. Loading a valid map with no duplicate positions succeeds without error (existing maps are not broken).
3. The error message includes the duplicate position coordinates so the author can locate the offending entry.
4. `validate_voxel_positions()` still rejects out-of-bounds positions as before; the new check does not interfere with the bounds check.
5. A unit test `test_duplicate_voxel_position` is added to `validation.rs` and asserts that `validate_map()` returns `Err` for a map with two voxels at the same position.
6. `cargo test` passes with no failures and `cargo clippy` reports no new errors.

---

## Non-Functional Requirements

- The duplicate check must use a `HashSet<(i32, i32, i32)>` allocated once per call — no additional passes over the voxel list.
- No changes to `MapLoadError` variants are required; `ValidationError(String)` is sufficient.
- The spawner (`chunks.rs`), chunk meshing, and editor code must not be modified.
- Both `adrakestory` and `map_editor` binaries must compile without new errors or warnings.

---

## Tasks

1. In `src/systems/game/map/validation.rs`, extend `validate_voxel_positions()` to build a `HashSet<(i32, i32, i32)>` as it iterates voxels and return `Err(MapLoadError::ValidationError(format!("Duplicate voxel position {:?}", pos)))` on the first `HashSet::insert` that returns `false`.
2. Add unit test `test_duplicate_voxel_position` in the `#[cfg(test)]` block at the bottom of `validation.rs`: create a `MapData::default_map()`, push a second `VoxelData` with the same `pos` as an existing voxel, and assert `validate_map()` returns `Err`.
3. Run `cargo test` and `cargo clippy`; fix any failures or new warnings.
4. Manually load `assets/maps/default.ron` and a deliberately broken map with a duplicate position to verify the happy path succeeds and the error path logs the correct message.
