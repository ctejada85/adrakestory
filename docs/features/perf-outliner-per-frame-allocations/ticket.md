# Performance: Eliminate Per-Frame Allocations in the Outliner Panel

**Date:** 2026-04-09
**Severity:** Medium
**Component:** Editor — Outliner / Camera Input (`src/editor/ui/outliner/mod.rs`, `src/editor/camera/mod.rs`, `src/editor/controller/input/mod.rs`)

---

## Story

As a developer, I want the outliner and input systems to avoid heap allocations on every frame so that the editor maintains steady frame pacing on large maps.

---

## Description

Two allocation hot-paths exist in the editor's per-frame update loop:

### 1. Outliner `BTreeMap` rebuilt every frame (PERF-01)

`render_voxels_section` (`src/editor/ui/outliner/mod.rs:149`) constructs a `BTreeMap<VoxelType, Vec<(i32, i32, i32)>>` from scratch on every frame, even when the map has not changed:

```rust
let mut voxels_by_type: BTreeMap<VoxelType, Vec<(i32, i32, i32)>> = BTreeMap::new();
for voxel in &editor_state.current_map.world.voxels {
    voxels_by_type.entry(voxel.voxel_type).or_default().push(voxel.pos);
}
```

On a map with tens of thousands of voxels this is O(n log n) work at 60 fps. The map changes rarely; the grouping only needs rebuilding when `is_modified` is set.

### 2. `iter().any()` linear scans for voxel presence (PERF-02)

`handle_gamepad_voxel_actions` (`src/editor/camera/mod.rs:461` and `src/editor/camera/mod.rs:500`) checks voxel existence with `iter().any(|v| v.pos == grid_pos)`, which is O(n) per input event. With 50 000 voxels and a 60 fps trigger rate this is 3 M comparisons/second for a trivial presence check.

The `EditorState` already has `current_map.world.voxels` as a `Vec`. A `HashSet<(i32, i32, i32)>` or `HashMap` of positions maintained alongside the `Vec` would reduce presence checks to O(1).

---

## Acceptance Criteria

1. `render_voxels_section` rebuilds the `BTreeMap` at most once per map change, not once per frame. On frames where the map is unchanged, the cached grouping is used directly.
2. Voxel presence checks in `handle_gamepad_voxel_actions` (and any other input handler) are O(1) on average.
3. The outliner correctly reflects the current voxel set after any add, remove, or undo/redo operation.
4. No correctness regression: all existing voxel grouping, filtering, and selection behaviors are preserved.
5. A benchmark or comment documents the expected complexity improvement.

---

## Non-Functional Requirements

- The cached grouping must be invalidated whenever `EditorState::mark_modified()` is called or a `RenderMapEvent` is processed.
- The `HashSet` for presence checks must be kept in sync with `world.voxels`; any code path that adds or removes a voxel must update both.
- The solution must not increase peak memory usage by more than 2× the existing `Vec<VoxelData>` size.

---

## Tasks

1. Add a `voxels_by_type: Option<BTreeMap<VoxelType, Vec<(i32, i32, i32)>>>` cache to `OutlinerState` (or `EditorState`), initialized to `None`.
2. Invalidate the cache whenever `mark_modified()` is called or a render event clears stale state.
3. In `render_voxels_section`, populate the cache on first use (when `None`) and reuse it otherwise.
4. Add a `voxel_positions: HashSet<(i32, i32, i32)>` field to `EditorState::current_map.world` (or as a parallel `EditorState` field).
5. Update all add/remove voxel paths to maintain the `HashSet`.
6. Replace `iter().any(|v| v.pos == grid_pos)` calls in the camera/input systems with `HashSet::contains`.
7. Verify the outliner is correct by adding/removing voxels and confirming the panel updates.
