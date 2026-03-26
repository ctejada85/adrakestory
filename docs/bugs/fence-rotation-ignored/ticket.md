# Fix: Fence Rotation Field Silently Ignored

**Date:** 2026-03-26  
**Severity:** Medium (p2)  
**Component:** Map format — `Fence` / `src/systems/game/map/spawner/chunks.rs`

---

## Story

As a level designer, I want a `Fence` voxel with a `rotation` value to either rotate visually or emit a clear warning at load time, so that the rotation field is never silently ignored and I am never misled into thinking my fence orientation has been applied.

---

## Description

The spawner branches on `pattern.is_fence()` before resolving the orientation matrix. The fence path calls `pattern.fence_geometry_with_neighbors(neighbors)` directly — which does not accept an orientation parameter — and never reads `voxel_data.rotation`. Any `rotation: Option<usize>` stored on a `Fence` voxel is silently discarded at spawn time. The field is parsed, validated (index bounds checked), and round-tripped to the file by the editor without effect.

The fence spawning path was written before the orientation matrix system existed. When Fix 1 (`map-format-multi-axis-rotation`) introduced `geometry_with_rotation()`, only the non-fence `else` branch was updated.

The fix is split into two phases:
- **Phase 1 (Option A):** Emit `warn!()` in `validate_map()` for fence+rotation; strip `rotation` from fences in the editor on save; update `map-format-spec.md`.
- **Phase 2 (Option B):** Apply the orientation matrix to fence geometry after neighbour detection; recompute collision bounds from rotated geometry; remove Phase 1 warning.

See `bug.md` for the full bug description, root cause analysis, and reproduction steps.

---

## Acceptance Criteria

1. Loading a map with `pattern: Some(Fence), rotation: Some(i)` emits a `warn!()` log message that identifies the voxel position and states that `rotation` has no effect on `Fence` geometry.
2. The warning does not prevent the map from loading. The fence renders as a normal neighbour-connected fence post.
3. Loading a map with `pattern: Some(Fence), rotation: None` produces no warning.
4. After the Phase 1 fix, saving a map from the editor produces a RON file where all `Fence` voxels have `rotation: None`, regardless of any rotation value that was in the original file.
5. The editor rotation tool does not apply rotation to `Fence` voxels (disabled or silent no-op).
6. `docs/api/map-format-spec.md` documents that `rotation` is ignored for `Fence` voxels in Phase 1, lists the load-time warning, and recommends leaving `rotation: None` on fence voxels.
7. `cargo test` passes with no failures; new unit tests cover: warning emitted for fence+rotation, no warning for fence with `rotation: None`, and no warning for non-fence pattern with rotation.
8. Both `adrakestory` and `map_editor` binaries compile without error or new warning.

---

## Non-Functional Requirements

- The warning path in `validate_map()` must not allocate per-voxel.
- `SubVoxelPattern::fence_geometry_with_neighbors()` must not be modified in Phase 1.
- `SubVoxelGeometry::fence_with_connections()` must not be modified in Phase 1.
- `apply_orientation_matrix()` must not be modified.
- No `f32` or floating-point arithmetic may be introduced.
- Changes must compile for both the `adrakestory` and `map_editor` binaries.

---

## Tasks

1. In `src/systems/game/map/validation.rs`, add a check inside `validate_map()` that iterates all voxels and calls `warn!()` for each voxel where `pattern == Some(SubVoxelPattern::Fence)` and `rotation == Some(_)`. The message must include the voxel position and state that `rotation` has no effect on `Fence` geometry.
2. In the map editor save path (`src/editor/`), add a pass that sets `voxel.rotation = None` for all voxels where `pattern == Some(SubVoxelPattern::Fence)` before the RON serialiser runs.
3. In the map editor rotation tool (`src/editor/`), guard the rotation operation so it is a no-op or disabled when the currently selected voxel has `pattern == Some(SubVoxelPattern::Fence)`.
4. Update `docs/api/map-format-spec.md` to document: (a) `rotation` is ignored for `Fence` voxels in Phase 1; (b) a `warn!()` is emitted at load time when `rotation: Some(_)` is present on a fence; (c) authors should leave `rotation: None` on fence voxels.
5. Write unit tests covering all Phase 1 acceptance criteria: warning emitted for fence+rotation, no warning for fence+`None`, no warning for non-fence+rotation, and (if applicable) editor save strips rotation from fences.
6. Run `cargo test` and `cargo clippy`; fix any failures or warnings.
