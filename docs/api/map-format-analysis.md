# Map Format Analysis

Analysis of the RON map format used by A Drake's Story (Version 1.0.0).  
Covers format structure, pros, cons, and concrete improvement proposals.

> **Resolution status**: Findings 1–8 have been fully implemented. Finding 9 (custom-properties namespace) has been documented (ticket, requirements, architecture) and is tracked in [`docs/bugs/custom-properties-namespace/`](../bugs/custom-properties-namespace/ticket.md). For a complete resolution log see [`docs/investigations/2026-03-22-1427-map-format-analysis.md`](../investigations/2026-03-22-1427-map-format-analysis.md).
>
> **Note on Cons sections below**: The cons describe the state of the format *at the time of the investigation*. Most issues cited have since been resolved by Findings 1–8. The cons are preserved as historical context; the investigation log lists the resolution for each one.

---

## 1. Format Overview

Maps are stored as `.ron` (Rusty Object Notation) files under `assets/maps/`. A map file is a single root tuple with six required top-level fields:

```
metadata | world | entities | lighting | camera | custom_properties
```

The **world** section holds a sparse voxel list — only non-Air voxels are stored. Each voxel entry carries a grid position, a material type (`VoxelType`), an optional sub-voxel pattern, and an optional rotation state. Sub-voxel geometry is fully **derived at load time** from the pattern + rotation combination; it is never stored in the file. Chunks are a purely runtime concept.

Key source locations:

| Concern | File |
|---|---|
| Root struct | `src/systems/game/map/format/mod.rs:25` |
| World / VoxelData | `src/systems/game/map/format/world.rs` |
| Patterns | `src/systems/game/map/format/patterns.rs` |
| Rotation | `src/systems/game/map/format/rotation.rs` |
| Entities | `src/systems/game/map/format/entities.rs` |
| VoxelType | `src/systems/game/map/format/voxel_type.rs` |
| Validation | `src/systems/game/map/validation.rs` |
| Spec | `docs/api/map-format-spec.md` |

---

## 2. Pros

### 2.1 Human-readable and diff-friendly
RON is text-based and structurally close to Rust. Fields are named, so a diff of two map files is immediately meaningful. Authoring and debugging maps by hand is practical.

### 2.2 Sparse voxel list keeps files small
Only occupied (non-Air) voxels are written. A 64×8×64 village with ~400 placed voxels is 419 lines, not thousands. This scales well for maps that are mostly open space.

### 2.3 Sub-voxel geometry is derived, not stored
Storing only the pattern enum + one rotation value instead of 512 individual sub-voxel booleans reduces file size by a factor of ~100 for solid voxels and makes patterns easy to change without touching geometry data.

### 2.4 Backward-compatible aliases
Deprecated identifiers (`Platform`, `Staircase`, `FenceX`, `FenceZ`, `FenceCorner`) are retained as `#[serde(alias)]` entries in `patterns.rs:17–42`. Old maps continue to load without migration.

### 2.5 `#[serde(default)]` for optional fields
`pattern` and `rotation_state` on `VoxelData` (`world.rs:29–33`) and `properties` on `EntityData` (`entities.rs:14`) use `#[serde(default)]`, which means they can be omitted entirely. This keeps common-case voxels concise.

### 2.6 Fail-fast validation
`validate_map()` (`validation.rs:7`) checks dimensions, voxel bounds, version string, player spawn presence, entity positions, and lighting values before spawning begins. Bad maps are rejected with typed errors rather than silently producing broken scenes.

### 2.7 Rotation math is lossless
The integer doubling trick (`geometry/rotation.rs`) avoids floating-point accumulation errors when rotating sub-voxel bit arrays. All rotations are exact.

### 2.8 Neighbour-aware Fence pattern
The `Fence` pattern automatically connects to adjacent fence voxels at spawn time (`spawner/chunks.rs:81–115`). Map authors do not need to specify connectivity — placing fences next to each other "just works".

---

## 3. Cons

### 3.1 Rotation model is fundamentally incomplete
`RotationState` stores a single axis + angle pair. `compose()` (`rotation.rs:27–37`) silently discards the first rotation when the two axes differ:

```rust
// rotation.rs:34
// "For simplicity, we'll store the most recent rotation"
Self::new(axis, angle)
```

This means multi-axis rotations cannot be faithfully round-tripped through the file format. A map author who rotates a voxel first around X then around Y will only see the Y rotation in the saved file. Diagonal orientations (e.g., a staircase at 45°, or any tilted surface) are impossible to represent.

### 3.2 Staircase and rotation interact unexpectedly
`StaircaseNegX`, `StaircaseZ`, `StaircaseNegZ` are pre-baked rotations of `StaircaseX` inside `SubVoxelPattern::geometry()` (`patterns.rs:64–75`). An additional `rotation_state` on those variants stacks on top of the already-rotated geometry. Writing `pattern: Some(StaircaseZ), rotation_state: Some((axis: Y, angle: 1))` produces double rotation — effectively `StaircaseNegX` — with no warning. The spec does not document this interaction.

### 3.3 Fence silently ignores `rotation_state`
The fence spawning path bypasses `geometry_with_rotation` entirely (`spawner/chunks.rs:104–115`). Any `rotation_state` on a `Fence` voxel is parsed, stored in memory, and written back on save — but has zero effect at runtime. The spec does not mention this exception, which will confuse authors trying to orient fences.

### 3.4 Only four material types
`VoxelType` (`src/systems/game/map/format/voxel_type.rs`) has four variants: `Air`, `Grass`, `Dirt`, `Stone`. Expanding the visual palette requires changing the format enum, recompiling, and migrating all existing maps if a new variant is inserted anywhere other than the end.

### 3.5 Entity properties are untyped strings
All entity configuration is `HashMap<String, String>`. A `LightSource` entity's `intensity`, `range`, `shadows`, and `color` are free-text strings parsed at spawn time with silent fallbacks (`entities.rs:184–242`). Invalid values (wrong types, out-of-range numbers, malformed color strings) fail silently and apply default values. There is no schema, no editor validation, and no error message.

### 3.6 No duplicate-voxel detection
Two `VoxelData` entries with the same `pos` are both processed. Their sub-voxels are unioned in the occupancy grid and both contribute to the chunk mesh. The resulting voxel has the geometry of both entries superimposed. No error or warning is produced. In large hand-edited maps this leads to invisible corruption.

### 3.7 `VoxelType` is defined in ECS components, not the format module
`VoxelType` lives in `src/systems/game/components.rs` and is imported by the format module. This creates a coupling between the serialisation layer and the runtime component model. Adding a new material type requires touching `components.rs`, which triggers rebuilds of all systems using the component.

**Status**: Resolved by Finding 5. `VoxelType` now lives in `src/systems/game/map/format/voxel_type.rs` and is re-exported into `components.rs`.

### 3.8 Pillar geometry does not match its name
`Pillar` is a 2×2×2 floating cube at the centre of the voxel cell (sub-voxels 3–4 on all axes), not a floor-to-ceiling column. Stacking Pillar voxels creates a visual column with a gap between each segment. The name and shape are mismatched, which leads to unexpected collision behaviour (the gaps have no collision).

### 3.9 Camera is stored as a static snapshot
`CameraData` holds a fixed `position`, `look_at`, and `rotation_offset`. The camera immediately snaps to this position when the map loads. There is no way to declare a starting distance, field of view, follow offset, or any other dynamic camera property. Adjusting the camera experience requires either changing the map file or hard-coding overrides in the spawner.

### 3.10 `custom_properties` has no schema or reserved namespace
Both `MapData.custom_properties` and `EntityData.properties` accept arbitrary string maps. There is no convention for key names, no reserved namespace for engine features, and no documented extension points. Two independent systems that both want to store properties in the same map will silently collide on key names.

### 3.11 Lighting model is sparse
Only one directional light and a flat ambient value are supported at the map level. Point lights exist as `LightSource` entities but are configured through the untyped string properties system (con 3.5). There is no support for area lights, emissive voxels, or baked lightmaps.

### 3.12 World dimensions are declared but not enforced for entities
The `world.width / height / depth` fields define the voxel grid, but entity positions use world-space floats and are only loosely checked (±1 unit outside bounds in X/Z, up to `height×2` in Y). There is nothing preventing a `PlayerSpawn` from being placed inside a solid voxel.

---

## 4. Improvement Proposals

### P1 — Fix multi-axis rotation (High priority)

**Problem:** `RotationState::compose()` silently discards the first rotation when axes differ.

**Proposal:** Replace the single-axis `RotationState` with a compact quaternion or a 3×3 integer rotation matrix stored in the file. For the 90° grid, only 24 distinct orientations exist and all can be encoded as an index 0–23.

```ron
// Option A: orientation index (0-23 covers all 90° grid orientations)
rotation: 7

// Option B: keep axis/angle but store a sequence
rotations: [(axis: Y, angle: 1), (axis: X, angle: 1)]
```

Option A is the most compact and unambiguous. Option B maintains backward compatibility with the current field shape.

**Impact:** Enables diagonal staircases, wall corners, and other orientations that are currently impossible.

---

### P2 — Validate and warn on Fence + rotation_state (Medium priority)

**Problem:** `rotation_state` on `Fence` voxels is silently ignored at runtime.

**Proposal:** Either:
- (a) Remove `rotation_state` from `Fence` entries during map save in the editor, and emit a `warn!()` during validation if the field is present.
- (b) Define what rotation *should* mean for a Fence (e.g., rotate the rail heights) and implement it.

At minimum, document the exception clearly in `map-format-spec.md` and add a validation warning.

---

### P3 — Introduce a typed entity properties schema (Medium priority)

**Problem:** Entity configuration is `HashMap<String, String>` with silent parse failures.

**Proposal:** Replace the string map with a typed per-variant configuration struct using Serde's enum adjacency:

```ron
// Current
(entity_type: LightSource, position: (5.0, 2.0, 3.0), properties: {"intensity": "8000", "range": "12.5"})

// Proposed
(entity_type: LightSource(intensity: 8000.0, range: 12.5, shadows: false, color: (1.0, 0.9, 0.7)), position: (5.0, 2.0, 3.0))
```

Benefits: compile-time schema, parse errors surface at map load time, editor auto-complete becomes possible, no silent defaults.

Migration: keep `properties` as a deprecated fallback with a one-version transition window.

---

### P4 — Add duplicate-voxel detection to validation (Low cost, high value)

**Problem:** Duplicate `pos` entries silently corrupt the mesh.

**Status**: Implemented by Finding 4. The check below is now active in `validation.rs`.

```rust
// validation.rs — part of validate_voxel_positions()
let mut seen = HashSet::new();
for voxel in &world.voxels {
    if !seen.insert(voxel.pos) {
        return Err(MapLoadError::ValidationError(
            format!("Duplicate voxel position {:?}", voxel.pos)
        ));
    }
}
```

---

### P5 — Decouple VoxelType from components.rs (Medium priority)

**Problem:** The serialisable material enum lives in the ECS component file, coupling format and runtime.

**Proposal:** Move `VoxelType` (and its `#[derive(Serialize, Deserialize)]`) into `src/systems/game/map/format/` and re-export it into `components.rs`. The format module should own its own types; the ECS component can use or wrap them.

---

### P6 — Expand the material palette via a registry (Low priority for now, design decision)

**Problem:** Only four materials exist; adding more requires recompiling ECS systems.

**Proposal (two options):**

- **A — Extend the enum:** Add new variants (`Sand`, `Wood`, `Brick`, …) to `VoxelType`. Straightforward, fast, type-safe. Requires recompile and a migration step. Best if the material list stays small.
- **B — Data-driven material IDs:** Replace `VoxelType` with a `u16` material ID and load material definitions (texture, color, physics properties) from a separate asset file. The map format stores only the ID; materials are resolved at spawn time. Best if the game will have dozens of materials or user-defined content.

For the current scope, Option A is simpler.

---

### P7 — Rename Pillar or fix its geometry (Low priority)

**Problem:** `Pillar` is a 2×2×2 centre-floating cube, not a pillar.

**Proposal:** Either:
- (a) Rename to `CenterCube` or `SmallCube` and introduce a new `Pillar` variant that is a 2×2×8 floor-to-ceiling column.
- (b) Change the existing geometry to occupy Y=0 to Y=7 (a true floor-to-ceiling column) in a 2×2×8 shape.

Option (a) is non-breaking (old maps using `Pillar` still load). Option (b) changes the visual appearance of any existing `Pillar` voxels.

---

### P8 — Add collision-aware PlayerSpawn validation (Low priority)

**Problem:** `PlayerSpawn` can be placed inside a solid voxel with no warning.

**Proposal:** After the voxel list is parsed and before spawning, check whether the spawn position overlaps any solid sub-voxel bounds and emit a `warn!()` log if so. This is purely advisory (maps should still load) but catches a common authoring mistake.

---

### P9 — Add a `Fence` note and reserved property namespace to the spec (Documentation)

**Problem:** `rotation_state` on Fence is undocumented as a no-op; `custom_properties` keys have no convention.

**Proposal:**
- Document the Fence rotation exception in `map-format-spec.md`.
- Reserve a key prefix (e.g., `engine:*`) for engine-defined properties and document currently used keys.

---

## 5. Summary Table

| # | Issue | Severity | Effort |
|---|---|---|---|
| P1 | Multi-axis rotation silently broken | High | Medium |
| P2 | Fence ignores rotation_state | Medium | Low |
| P3 | Entity properties untyped, silent failures | Medium | Medium |
| P4 | Duplicate voxel positions not detected | Medium | Low |
| P5 | VoxelType coupled to ECS components | Low | Low |
| P6 | Only four material types | Low | Low–High |
| P7 | Pillar geometry/name mismatch | Low | Low |
| P8 | PlayerSpawn inside solid voxel not warned | Low | Low |
| P9 | Spec missing Fence exception and key namespace | Low | Trivial |

The two highest-leverage changes are **P1** (rotation model) and **P3** (typed entity properties), as both affect what kinds of maps can be authored and how reliably they load. **P4** is low cost with immediate reliability benefit and should be tackled first.
