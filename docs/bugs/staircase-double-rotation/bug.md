# Bug Report: Staircase Directional Variants Apply Double Rotation

**Date:** 2026-03-26
**Severity:** Medium (p2)
**Component:** Map format — `SubVoxelPattern` / `src/systems/game/map/format/patterns.rs`
**Status:** Open

---

## Description

`SubVoxelPattern` exposes four staircase directional variants: `StaircaseX`,
`StaircaseNegX`, `StaircaseZ`, and `StaircaseNegZ`. The directional variants are
not independent geometries — they are pre-baked rotations of `StaircaseX` applied
inside `SubVoxelPattern::geometry()`:

```rust
// patterns.rs:64–74
Self::StaircaseNegX => {
    SubVoxelGeometry::staircase_x().rotate(RotationAxis::Y, 2)
}
Self::StaircaseZ => {
    SubVoxelGeometry::staircase_x().rotate(RotationAxis::Y, 1)
}
Self::StaircaseNegZ => {
    SubVoxelGeometry::staircase_x().rotate(RotationAxis::Y, 3)
}
```

When `geometry_with_rotation()` is subsequently called with a non-`None`
orientation matrix, that matrix is applied **on top of** the already-rotated
base geometry. The result is a compounded rotation that the author did not
intend and cannot predict without knowing about the hidden pre-bake.

---

## Actual Behavior

A voxel written as:

```ron
(pos: (2, 0, 1), voxel_type: Stone, pattern: Some(StaircaseZ), rotation: Some(0))
// where orientations[0] = [[0,0,1],[0,1,0],[-1,0,0]]  (Y+90°)
```

produces geometry equivalent to `StaircaseX` rotated Y+90° (for `StaircaseZ`)
then rotated another Y+90° from the orientation matrix — i.e., `StaircaseNegX`
geometry. The author expected `StaircaseZ` oriented at Y+90°, which would be
`StaircaseNegX` geometry, but arrived there by accident and with no documented
reason.

More concretely, a level designer who selects `StaircaseZ` in the editor and
then rotates it +90° around Y gets different geometry than they would get by
selecting `Staircase` and rotating it +180° around Y — even though the
visual result should be identical and predictable from the rotation alone.

---

## Expected Behavior

The orientation matrix stored on the voxel should express the **total**
orientation of the pattern relative to the canonical `Staircase` geometry. The
pattern variant name should carry no hidden geometric meaning beyond identifying
which base shape to use.

Given `pattern: Some(StaircaseZ)` and `rotation: Some(i)` where `orientations[i]`
is Y+90°, the spawner should produce geometry identical to `Staircase` rotated
by the composition of `StaircaseZ`'s implicit Y+90° pre-bake and the explicit
Y+90° matrix — or alternatively the variant should be normalised and the
pre-bake absorbed into the explicit matrix.

---

## Root Cause

`SubVoxelPattern` conflates two orthogonal concerns:

1. **Shape identity** — which base geometry template to use (`staircase_x()`)
2. **Orientation** — how that shape is rotated in world space

The directional variants (`StaircaseNegX`, `StaircaseZ`, `StaircaseNegZ`) were
introduced as a shorthand that bakes orientation into the pattern name. This
was workable before the orientation matrix system existed, but the matrix
system now provides a proper, general orientation mechanism. The baked
rotations in `geometry()` interact with the external orientation matrix in an
undocumented and surprising way.

The fix is to normalise all directional staircase variants to `Staircase` (the
single canonical variant, renamed from `StaircaseX`) in the map file and absorb
the implicit pre-bake into the voxel's explicit orientation matrix. The three
directional variants become aliases (for backward-compatible loading) that are
immediately normalised to `(pattern: Staircase, rotation: <Y-rotation-index>)`
on load. The old `StaircaseX` name is kept as a `#[serde(alias)]` on `Staircase`.

---

## Steps to Reproduce

1. Open the map editor.
2. Place a voxel with `pattern: Some(StaircaseZ)`.
3. In the editor, apply a Y+90° rotation (orientation matrix `[[0,0,1],[0,1,0],[-1,0,0]]`).
4. Save the map and inspect the RON. The file will contain:
   `pattern: Some(StaircaseZ), rotation: Some(i)` where `orientations[i]` = Y+90°.
5. Load the map. The rendered geometry will be `StaircaseX` rotated Y+180° (≡ `StaircaseNegX`),
   not `StaircaseX` rotated Y+90° (≡ `StaircaseZ`) as the file literally states.

Alternatively, observe directly in code:

```rust
// These two calls produce the same geometry (double Y+90° = Y+180°):
SubVoxelPattern::StaircaseZ.geometry_with_rotation(
    Some(&axis_angle_to_matrix(RotationAxis::Y, 1))
)
// is identical to:
SubVoxelPattern::Staircase.geometry_with_rotation(
    Some(&axis_angle_to_matrix(RotationAxis::Y, 2))
)
```

---

## Impact

| Affected Party | Impact |
|----------------|--------|
| Level designers | Cannot predict staircase orientation from the pattern name + rotation fields; must know about hidden pre-bakes |
| Map authors (hand-editing RON) | `StaircaseZ` with a rotation produces silently wrong geometry |
| Editor | Rotation operations on `StaircaseNegX`/`StaircaseZ`/`StaircaseNegZ` voxels compound with the hidden pre-bake; undo/redo is affected |
| Map format spec | Current spec documents the directional variants but does not mention the pre-bake or its interaction with `rotation` |

---

## Suggested Fix

**Normalise directional staircase variants on load.**

1. In the map loader (after `migrate_legacy_rotations()`, before `validate_map()`),
   add a pass `normalise_staircase_variants()` that converts each staircase
   directional variant to `StaircaseX` and composes the implicit pre-bake into
   the voxel's orientation matrix:

   | Loaded pattern | Implicit pre-bake | New pattern | New orientation |
   |----------------|------------------|-------------|----------------|
   | `StaircaseNegX` | Y+180° | `Staircase` | compose(Y+180°, existing) |
   | `StaircaseZ`    | Y+90°  | `Staircase` | compose(Y+90°, existing) |
   | `StaircaseNegZ` | Y+270° | `Staircase` | compose(Y+270°, existing) |
   | `Staircase`     | —      | unchanged   | unchanged |

2. Mark `StaircaseNegX`, `StaircaseZ`, `StaircaseNegZ` as `#[serde(alias)]`
   entries that deserialise to `StaircaseX` (or keep them as variants during
   deserialisation and normalise immediately afterward).

3. On editor save, only `StaircaseX` is ever written. The directional aliases
   remain loadable for backward compatibility.

See `ticket.md` for the full acceptance criteria and task breakdown.

---

## Related

- `docs/investigations/2026-03-22-1427-map-format-analysis.md` — Finding 2
- `docs/bugs/map-format-multi-axis-rotation/` — prerequisite fix (orientation matrix system, completed)
- `src/systems/game/map/format/patterns.rs` — `SubVoxelPattern::geometry()`
- `src/systems/game/map/loader.rs` — migration shim location
- `docs/api/map-format-spec.md` — spec does not document pre-baked rotations
