# Bug Report: Fence Silently Ignores `rotation` at Spawn Time

**Date:** 2026-03-26
**Severity:** Medium (p2)
**Component:** Map format — `Fence` / `src/systems/game/map/spawner/chunks.rs`
**Status:** Open

---

## Description

`Fence` voxels use a neighbour-aware spawning path that bypasses
`geometry_with_rotation()` entirely. Any `rotation: Option<usize>` stored on a
`Fence` voxel is parsed, held in memory, and written back by the editor — but has
zero effect on the rendered geometry or collision bounds. A map author who sets a
rotation on a fence to orient its post or rails will see the fence remain
identical to an unrotated fence. No warning is emitted; the silently-ignored field
is round-tripped to the file as though it were meaningful.

---

## Actual Behavior

- A `Fence` voxel with `rotation: Some(i)` renders as a plain neighbour-connected
  fence post, ignoring the orientation matrix entirely.
- The `rotation` field is read during deserialisation, validated by
  `validate_orientations()`, and written back on save — giving authors the
  impression it has an effect.
- No `warn!()` or error is produced during loading or spawning.
- The spec does not document this exception.

---

## Expected Behavior

- Either the `rotation` field on a `Fence` voxel should be applied to the
  generated geometry (so authors can orient fence posts), **or** a `warn!()` should
  be emitted at load or spawn time when a non-`None` `rotation` is present on a
  `Fence` voxel, making the no-op behaviour explicit.
- `docs/api/map-format-spec.md` should document the exception so authors know
  not to rely on rotation for fences.
- The editor should ideally strip `rotation` from `Fence` voxels on save to
  prevent round-tripping a meaningless field.

---

## Root Cause Analysis

**File:** `src/systems/game/map/spawner/chunks.rs`
**Function:** `spawn_voxels_chunked()`
**Approximate line:** 104–115

The spawner branches on `pattern.is_fence()` before resolving the orientation:

```rust
// chunks.rs:104–115
let geometry = if pattern.is_fence() {
    let neighbors = (
        fence_positions.contains(&(x - 1, y, z)),
        fence_positions.contains(&(x + 1, y, z)),
        fence_positions.contains(&(x, y, z - 1)),
        fence_positions.contains(&(x, y, z + 1)),
    );
    pattern.fence_geometry_with_neighbors(neighbors)   // ← rotation never consulted
} else {
    let orientation = voxel_data.rotation.and_then(|i| map.orientations.get(i));
    pattern.geometry_with_rotation(orientation)         // ← only non-fence path
};
```

`fence_geometry_with_neighbors()` calls `SubVoxelGeometry::fence_with_connections()`
directly and does not accept an orientation parameter. The `voxel_data.rotation`
field is never read for fence voxels.

The fence spawning path was written before the orientation matrix system existed.
When the matrix system was introduced (Fix 1 — `map-format-multi-axis-rotation`),
the fence branch was not updated to apply the orientation.

---

## Steps to Reproduce

1. Open the map editor.
2. Place a `Fence` voxel.
3. Apply any rotation (e.g. Y+90° via the rotation tool), saving the map.
4. The RON file will contain `pattern: Some(Fence), rotation: Some(i)`.
5. Reload the map. The fence post renders identically to an unrotated fence —
   the rotation has no visible effect.

Alternatively, observe in code:

```rust
// These two calls produce identical geometry:
SubVoxelPattern::Fence.fence_geometry_with_neighbors((true, false, false, false))
// is identical regardless of what rotation index the voxel carries.
```

---

## Impact

| Affected Party | Impact |
|----------------|--------|
| Level designers | Cannot orient fences; any rotation applied in the editor is silently discarded at spawn time |
| Map authors (hand-editing RON) | Setting `rotation` on a `Fence` voxel produces no effect, but the field is preserved and re-validated on each load — leading to confusion |
| Editor | Rotation tool operates on fence voxels without error, creating the false impression that orientation is applied |
| Map format spec | The exception is not documented; authors have no indication that `Fence` ignores `rotation` |

---

## Suggested Fix

Update the spawner branch in `chunks.rs:104–115` to apply the orientation matrix
to the neighbour-generated geometry:

```rust
let geometry = if pattern.is_fence() {
    let neighbors = (/* ... */);
    let fence_geo = pattern.fence_geometry_with_neighbors(neighbors);
    // Apply orientation after neighbour-aware generation
    let orientation = voxel_data.rotation.and_then(|i| map.orientations.get(i));
    if let Some(matrix) = orientation {
        apply_orientation_matrix(fence_geo, matrix)
    } else {
        fence_geo
    }
} else {
    let orientation = voxel_data.rotation.and_then(|i| map.orientations.get(i));
    pattern.geometry_with_rotation(orientation)
};
```

Neighbour detection remains world-axis-aligned — the four adjacent positions
queried do not rotate with the fence geometry. Collision bounds (`SubVoxel.bounds`)
and the `SpatialGrid` insertion must use the rotated AABB derived from the rotated
geometry.

See `ticket.md` for the full acceptance criteria and task breakdown.

---

## Related

- `docs/investigations/2026-03-22-1427-map-format-analysis.md` — Finding 3
- `docs/bugs/map-format-multi-axis-rotation/` — prerequisite fix (orientation matrix system, completed)
- `docs/bugs/staircase-double-rotation/` — similar rotation-bypass issue, fixed
- `src/systems/game/map/spawner/chunks.rs` — `spawn_voxels_chunked()` fence branch
- `src/systems/game/map/format/patterns.rs` — `fence_geometry_with_neighbors()`
- `src/systems/game/map/validation.rs` — validation does not warn on fence+rotation
- `docs/api/map-format-spec.md` — spec does not document the exception
