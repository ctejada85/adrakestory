# Investigation: Map Format Pros, Cons, and Improvement Areas

**Date:** 2026-03-22 14:27
**Status:** Complete
**Component:** Map format / Serialisation / Validation

## Resolution Log

| Finding | Ticket | Status | Date |
|---------|--------|--------|------|
| 1 — Multi-axis rotation silently broken | `docs/bugs/map-format-multi-axis-rotation/` | **Fixed** — `RotationState` replaced with `orientations: Vec<OrientationMatrix>` + `rotation: Option<usize>` per voxel. Commit `eda90e3`. | 2026-03-26 |
| 2 — Staircase double-rotation | `docs/bugs/staircase-double-rotation/` | **Fixed** — `normalise_staircase_variants()` loader pass added; `StaircaseX` renamed to `Staircase` with serde alias; directional variants removed from editor picker. Commit `4874885`. | 2026-03-26 |
| 3 — Fence silently ignores rotation | `docs/bugs/fence-rotation-ignored/` | **Fixed** — Spawner applies orientation matrix to fence geometry after world-axis neighbour detection; `world_dir_to_local()` maps neighbours into local frame. `docs/api/map-format-spec.md` updated. Commits `56ca5fa`, `fd80558`, `54b057f`. | 2026-03-26 |
| 4 — Duplicate voxel positions not detected | `docs/bugs/duplicate-voxel-positions/` | **Fixed** — `validate_voxel_positions()` extended with `HashSet` duplicate check. Commit `9f960d1`. | 2026-03-31 |
| 5 — Entity properties silent parse failures | `docs/bugs/entity-properties-silent-parse-failure/` | **Fixed** — `validate_entity_properties()` added to `validate_entities()`; validates LightSource and Npc property strings before spawning. Commit pending. | 2026-03-31 |
| 6–9 | — | Not yet tracked. | — |

## Summary

Review of the RON map format (version 1.0.0) used by A Drake's Story. The
format is human-readable, sparse, and well-structured for the current scope.
However, several design issues limit authoring expressiveness and reliability:
the rotation model silently loses information for multi-axis orientations;
entity configuration is untyped with silent parse failures; and duplicate
voxel positions are not detected, silently corrupting meshes.

## Environment

- Format version: 1.0.0
- Serialisation: `ron` crate, `#[derive(Serialize, Deserialize)]`
- Root struct: `src/systems/game/map/format/mod.rs:25`
- Spec: `docs/api/map-format-spec.md`
- Example maps reviewed: `assets/maps/simple_test.ron`, `assets/maps/default.ron`, `assets/maps/village_64x64.ron`

---

## Findings

### Finding 1 — Multi-axis rotation silently broken (p1 High) ✅ Fixed

**File:** `src/systems/game/map/format/rotation.rs` (pre-fix)

> **Resolved 2026-03-26 — commit `eda90e3`.** `RotationState` has been replaced with a
> top-level `orientations: Vec<OrientationMatrix>` list on `MapData`. Each voxel now
> stores a `rotation: Option<usize>` index. `multiply_matrices()` composes rotations
> correctly. Legacy `rotation_state` fields are auto-migrated on load. See
> `docs/bugs/map-format-multi-axis-rotation/` for the full ticket and architecture.

`RotationState::compose()` only added angles when both rotations shared the same
axis. For different axes it silently discarded the earlier rotation and stored
only the new one:

```rust
// rotation.rs:34 (pre-fix)
// "For simplicity, we'll store the most recent rotation"
Self::new(axis, angle)
```

A map author who rotated a voxel first around X then around Y would only see
the Y rotation persisted to the file. Multi-axis orientations — diagonal
staircases, tilted platforms, any non-cardinal angle — could not be represented or
round-tripped. There was no error, no warning, and no documentation of the
limitation.

---

### Finding 2 — Staircase variants and `rotation` stack unexpectedly (p2 Medium) ✅ Fixed

**File:** `src/systems/game/map/format/patterns.rs:64–75`

> **Resolved 2026-03-26 — commit `4874885`.** `normalise_staircase_variants()` loader
> pass added after `migrate_legacy_rotations()`. `StaircaseX` renamed to `Staircase` with
> `#[serde(alias = "StaircaseX")]` for backward compat. Directional variants
> (`StaircaseNegX`, `StaircaseZ`, `StaircaseNegZ`) normalised on load: their implicit
> Y-axis pre-bake is absorbed into the voxel's explicit orientation matrix before any
> spawning occurs. Editor pattern picker now exposes only `Staircase`. See
> `docs/bugs/staircase-double-rotation/` for the full ticket and architecture.

`StaircaseNegX`, `StaircaseZ`, `StaircaseNegZ` are pre-baked rotations of
`StaircaseX` computed inside `SubVoxelPattern::geometry()`. If a `rotation`
orientation matrix is also present on the voxel, `geometry_with_rotation()`
applies it on top of the already-rotated geometry. Writing:

```ron
pattern: Some(StaircaseZ), rotation: Some(1)   // orientation index 1 = Y+90°
```

produces the geometry of `StaircaseNegX` (two Y rotations composed) with no
warning. The spec documents neither the pre-baked rotations nor their interaction
with the orientation matrix.

---

### Finding 3 — Fence silently ignores `rotation_state` (p2 Medium)

**File:** `src/systems/game/map/spawner/chunks.rs:104–115`

The fence spawning path is neighbour-aware and bypasses
`geometry_with_rotation()` entirely. Any `rotation_state` on a `Fence` voxel
is parsed, held in memory, and written back by the editor — but has zero
runtime effect. The spec does not document this exception. Authors who try to
orient fences via `rotation_state` will see no result.

---

### Finding 4 — Duplicate voxel positions not detected (p2 Medium)

**File:** `src/systems/game/map/validation.rs:42–53`

`validate_voxel_positions()` checks bounds but does not check for duplicate
`pos` entries. Two `VoxelData` structs with the same position are both
processed: their sub-voxels are unioned in the occupancy grid and both
contribute independently to the chunk mesh. The result is a voxel with
superimposed geometry from two entries. No error or warning is produced. In
large hand-edited maps this leads to invisible mesh corruption.

---

### Finding 5 — Entity properties are untyped strings with silent parse failures (p2 Medium)

**File:** `src/systems/game/map/format/entities.rs:14–15`, spawner `entities.rs:184–242`

All entity configuration is `HashMap<String, String>`. A `LightSource`
entity's `intensity`, `range`, `shadows`, and `color` are free-text strings
parsed at spawn time. Invalid values — non-numeric strings, out-of-range
numbers, malformed color syntax — fall back to hardcoded defaults with no log
entry and no error. There is no schema, no editor-side validation, and no
documented key names for entities other than `LightSource` and `Npc`.

---

### Finding 6 — Only four material types (p3 Low)

**File:** `src/systems/game/components.rs:44–50`

`VoxelType` has four variants: `Air`, `Grass`, `Dirt`, `Stone`. Expanding the
visual palette requires modifying an ECS component and recompiling all systems
that use it. The enum is defined in `components.rs` rather than the format
module, coupling the serialisation layer to the runtime component model.

---

### Finding 7 — Pillar geometry does not match its name (p3 Low)

**File:** `src/systems/game/map/geometry/patterns.rs` (pillar implementation)

`Pillar` occupies sub-voxels 3–4 on all three axes: a 2×2×2 floating cube at
the exact centre of the voxel cell. It is not a floor-to-ceiling column.
Stacking `Pillar` voxels produces a visual column with an unoccupied gap inside
each cell — meaning no collision in that gap — which is non-obvious.

---

### Finding 8 — Camera stored as a static snapshot with no dynamic properties (p3 Low)

**File:** `src/systems/game/map/format/camera.rs`

`CameraData` holds a fixed `position`, `look_at`, and `rotation_offset`. There
is no way to declare starting distance, field of view, follow offset, or
camera constraints. The camera snaps to the stored position on map load with no
interpolation. Adjusting camera behaviour requires either changing the map file
or overriding in code.

---

### Finding 9 — `custom_properties` has no namespace convention (p3 Low)

**File:** `src/systems/game/map/format/mod.rs`

Both `MapData.custom_properties` and `EntityData.properties` accept arbitrary
`HashMap<String, String>`. There is no reserved key prefix for engine-defined
values and no documented convention. Two independent systems storing data in
the same map will silently collide on key names.

---

## Pros

| # | Strength |
|---|----------|
| 1 | Human-readable RON: text-based, diff-friendly, practical to hand-author |
| 2 | Sparse voxel list: only non-Air voxels stored; large open maps stay small |
| 3 | Geometry derived from pattern + rotation: does not store 512 sub-voxel booleans per voxel |
| 4 | Backward-compatible aliases: `Platform`, `Staircase`, `FenceX`, `FenceZ`, `FenceCorner` kept via `#[serde(alias)]` |
| 5 | `#[serde(default)]` on optional fields: common-case voxels omit `pattern` and `rotation_state` without error |
| 6 | Fail-fast validation: bad maps rejected with typed errors before any spawning begins |
| 7 | Lossless integer rotation math: no floating-point accumulation errors in sub-voxel bit-array rotations |
| 8 | Automatic fence connectivity: placing fences adjacent to each other is sufficient; no explicit connection data needed |

---

## Root Cause Summary

| # | Finding | Location | Priority | Impact | Status |
|---|---------|----------|----------|--------|--------|
| 1 | Multi-axis rotation silently discards first rotation | `rotation.rs` (pre-fix) | p1 | Cannot represent non-cardinal voxel orientations | **Fixed** `eda90e3` |
| 2 | Staircase variant + rotation produces double rotation | `patterns.rs:64–75` | p2 | Unexpected geometry, no warning | **Fixed** `4874885` |
| 3 | Fence ignores rotation at runtime | `spawner/chunks.rs:104–115` | p2 | Author intent silently lost | **Fixed** — commits `56ca5fa`, `fd80558`, `54b057f` |
| 4 | Duplicate voxel positions not detected | `validation.rs:42–53` | p2 | Silent mesh corruption | **Fixed** `9f960d1` |
| 5 | Entity properties untyped, parse failures silent | `entities.rs:14–15` | p2 | Invalid config produces wrong runtime state | **Fixed** — see `docs/bugs/entity-properties-silent-parse-failure/` |
| 6 | Only 4 material types; VoxelType in components.rs | `components.rs:44–50` | p3 | Limited palette; format/ECS coupling | Open |
| 7 | Pillar geometry is floating cube, not a column | geometry patterns | p3 | Misleading name, unexpected collision gaps | Open |
| 8 | Camera stored as static snapshot | `camera.rs` | p3 | No dynamic camera properties expressible in format | Open |
| 9 | custom_properties has no namespace | `format/mod.rs` | p3 | Key collisions possible across systems | Open |

---

## Recommended Fixes

### Fix 1 — Replace RotationState with a 24-orientation index (Finding 1) ✅ Done

All 90° grid orientations form a group of exactly 24 distinct rotations.
Replace the single-axis `{axis, angle}` struct with a compact orientation index
(0–23). This covers every valid 90°-grid orientation without ambiguity and is
more compact to store.

**Implemented** (commit `eda90e3`): a top-level `orientations: Vec<OrientationMatrix>`
list is stored in `MapData`; voxels reference entries by `rotation: Option<usize>` index.
`multiply_matrices()` handles correct multi-axis composition in the editor. Legacy
`rotation_state` fields are auto-migrated on load.

---

### Fix 2 — Canonicalise staircase direction via orientation matrix only (Finding 2) ✅ Done

The four staircase directional variants (`StaircaseX`, `StaircaseNegX`,
`StaircaseZ`, `StaircaseNegZ`) all map to the same base geometry. The direction
is now expressed entirely via the voxel's `rotation` orientation matrix.

**Implemented** (commit `4874885`): `normalise_staircase_variants()` loader pass
absorbs the implicit Y-axis pre-bakes of the three directional variants into the
voxel's explicit orientation matrix. `StaircaseX` renamed to `Staircase`; old
names kept as `#[serde(alias)]`. Editor picker exposes only `Staircase`.
See `docs/bugs/staircase-double-rotation/ticket.md` for the full fix plan.

---

### Fix 3 — Detect and reject duplicate voxel positions in validation (Finding 4)

Low cost, immediate reliability benefit:

```rust
// validation.rs — add to validate_voxel_positions()
let mut seen = std::collections::HashSet::new();
for voxel in &world.voxels {
    if !seen.insert(voxel.pos) {
        return Err(MapLoadError::ValidationError(
            format!("Duplicate voxel position {:?}", voxel.pos)
        ));
    }
}
```

---

### Fix 4 — Typed entity properties via Serde enum adjacency (Finding 5)

Replace `HashMap<String, String>` with a typed per-variant configuration
struct:

```ron
// Proposed
(
    entity_type: LightSource(
        intensity: 8000.0,
        range: 12.5,
        shadows: false,
        color: (1.0, 0.9, 0.7),
    ),
    position: (5.0, 2.0, 3.0),
)
```

Keep `properties: HashMap<String, String>` as a deprecated fallback for one
version, then remove it.

---

### Fix 5 — Document and validate Fence + rotation (Finding 3)

Minimum viable fix: emit `warn!()` during validation when `rotation` is
`Some(...)` on a `Fence` voxel. Update `map-format-spec.md` to document the
exception. Optionally strip the field during editor save to prevent confusion.

---

### Fix 6 — Move VoxelType into the format module (Finding 6)

Move `VoxelType` (with its `Serialize`/`Deserialize` derives) from
`src/systems/game/components.rs` into `src/systems/game/map/format/` and
re-export it into `components.rs`. The format module should own its own types;
the ECS component can use or wrap them.

---

## Related

- `docs/api/map-format-spec.md` — normative format specification
- `docs/api/map-format-analysis.md` — extended analysis with full pros/cons and improvement proposals
- `docs/bugs/map-format-multi-axis-rotation/` — Finding 1 ticket (fixed)
- `docs/bugs/staircase-double-rotation/` — Finding 2 ticket
- `src/systems/game/map/format/` — all format type definitions
- `src/systems/game/map/validation.rs` — validation implementation
- `src/systems/game/map/spawner/chunks.rs` — chunk/fence spawning logic
