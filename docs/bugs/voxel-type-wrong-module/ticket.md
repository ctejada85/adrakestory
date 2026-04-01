# Fix: VoxelType Defined in Wrong Module

**Date:** 2026-03-31
**Severity:** Low (p3)
**Component:** Map format — `src/systems/game/map/format/`, ECS components — `src/systems/game/components.rs`

---

## Story

As a contributor adding a new material type, I want `VoxelType` to be defined
in the map format module so that I only need to edit one cohesive module to add
a new material, rather than navigating to an ECS component file that has no
other connection to the serialisation format.

---

## Description

`VoxelType` (`src/systems/game/components.rs:44–50`) is the enum that names the
four material types (`Air`, `Grass`, `Dirt`, `Stone`). It carries
`#[derive(Serialize, Deserialize)]` because it is a field on `VoxelData`, which
is the core persistent voxel record in the RON map format. However, it is
defined in `components.rs` — the file that holds Bevy ECS `Component` structs
used by the runtime.

`VoxelType` is never used as a Bevy component itself. It does not appear in any
`Query`, `With<>`, or system parameter. The only reason it lives in
`components.rs` is historical accident. Its serde derives make it a format type,
not an ECS type.

The format module (`src/systems/game/map/format/`) owns all other format types
(`VoxelData`, `WorldData`, `SubVoxelPattern`, `OrientationMatrix`, `EntityData`,
…). `VoxelType` is the sole exception. `world.rs` inside the format module is
already forced to cross-import it from `components.rs`:

```rust
// world.rs:5 — cross-import from ECS module into format module
use crate::systems::game::components::VoxelType;
```

This reverse dependency (format importing from ECS) is the wrong direction and
makes it harder to understand module ownership at a glance.

The fix moves `VoxelType` into a new file
`src/systems/game/map/format/voxel_type.rs` and re-exports it from both
`format/mod.rs` and `components.rs`. All eleven consumer import paths
(`use crate::systems::game::components::VoxelType`) remain valid via the
re-export — no consumer file changes are required.

---

## Acceptance Criteria

1. `VoxelType` is defined in `src/systems/game/map/format/voxel_type.rs`.
2. `src/systems/game/map/format/mod.rs` re-exports `VoxelType` so that
   `use crate::systems::game::map::format::VoxelType` works.
3. `src/systems/game/components.rs` re-exports `VoxelType` from the format
   module so that all existing `use crate::systems::game::components::VoxelType`
   import paths continue to compile without change.
4. `src/systems/game/map/format/world.rs` imports `VoxelType` from the local
   `format` module (e.g. `use super::voxel_type::VoxelType`) rather than from
   `crate::systems::game::components`.
5. `docs/api/map-format-spec.md` updated to note that `VoxelType` is part of
   the format module.
6. `cargo build` succeeds for both `adrakestory` and `map_editor` binaries with
   zero new errors or warnings.
7. `cargo test` passes with no new failures.
8. `cargo clippy` reports zero new errors.

---

## Non-Functional Requirements

- No consumer files (editor, game systems, tests) need to change their import
  paths — the re-export from `components.rs` preserves backward compatibility.
- All existing derives on `VoxelType` (`Clone, Copy, Debug, PartialEq, Eq,
  Hash, PartialOrd, Ord, Serialize, Deserialize`) must be preserved exactly.
- Both binaries must compile without new warnings after the move.

---

## Tasks

1. Create `src/systems/game/map/format/voxel_type.rs` containing the `VoxelType`
   definition with all existing derives.
2. Add `mod voxel_type;` and `pub use voxel_type::VoxelType;` to
   `src/systems/game/map/format/mod.rs`.
3. Replace the `VoxelType` definition in `src/systems/game/components.rs` with
   `pub use crate::systems::game::map::format::VoxelType;`.
4. Update the `use` statement in `src/systems/game/map/format/world.rs` to
   import from `super::voxel_type::VoxelType` (or `super::VoxelType` via
   `mod.rs`) instead of `crate::systems::game::components::VoxelType`.
5. Update `docs/api/map-format-spec.md` to note `VoxelType` is defined in
   `src/systems/game/map/format/voxel_type.rs`.
6. Run `cargo build`, `cargo test`, `cargo clippy`; fix any failures or new
   warnings.
