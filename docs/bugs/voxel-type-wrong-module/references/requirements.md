# Requirements — VoxelType Module Relocation

**Source:** Map Format Analysis Investigation — 2026-03-22
**Bug:** `docs/bugs/voxel-type-wrong-module/ticket.md`
**Status:** Final — all questions resolved 2026-03-31

---

## 1. Overview

`VoxelType` is the enum that names the four material variants used in the RON
map format (`Air`, `Grass`, `Dirt`, `Stone`). It carries
`#[derive(Serialize, Deserialize)]` because it appears as a field on `VoxelData`,
which is the central persistent voxel record. Despite this, `VoxelType` is
currently defined in `src/systems/game/components.rs` — the ECS component module —
rather than in the format module where all other format types live.

`VoxelType` is never used as a Bevy `Component`. It does not appear in any
`Query`, `With<>`, or ECS system parameter. Its presence in `components.rs` is
historical; nothing about its behaviour or semantics requires it to be there.

The fix moves `VoxelType` into `src/systems/game/map/format/voxel_type.rs` and
adds re-exports from both `format/mod.rs` and `components.rs`. All existing
import paths remain valid through the re-export — no consumer file requires a
change to its `use` statements.

---

## 2. Functional Requirements

### 2.1 New Home for VoxelType

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1.1 | `VoxelType` must be defined in `src/systems/game/map/format/voxel_type.rs`. | Phase 1 |
| FR-2.1.2 | All existing derives (`Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize`) must be preserved exactly on the moved definition. | Phase 1 |
| FR-2.1.3 | All four variants (`Air`, `Grass`, `Dirt`, `Stone`) must be preserved exactly. | Phase 1 |

### 2.2 Re-exports for Backward Compatibility

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.2.1 | `src/systems/game/map/format/mod.rs` must re-export `VoxelType` so that `use crate::systems::game::map::format::VoxelType` resolves correctly. | Phase 1 |
| FR-2.2.2 | `src/systems/game/components.rs` must re-export `VoxelType` from the format module so that all existing `use crate::systems::game::components::VoxelType` import paths continue to compile without modification. | Phase 1 |

### 2.3 Internal Format Module Import Fix

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.3.1 | `src/systems/game/map/format/world.rs` must import `VoxelType` from within the format module (e.g. `use super::voxel_type::VoxelType` or `use super::VoxelType`). The current cross-import `use crate::systems::game::components::VoxelType` must be removed. | Phase 1 |

### 2.4 Documentation Update

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.4.1 | `docs/api/map-format-spec.md` must be updated to note that `VoxelType` is defined in `src/systems/game/map/format/voxel_type.rs`. | Phase 1 |

### 2.5 No Consumer Changes Required

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.5.1 | No file outside `src/systems/game/map/format/` and `src/systems/game/components.rs` is required to change its import path. The re-exports must make the move transparent to all consumers. | Phase 1 |

---

## 3. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-3.1 | Both `adrakestory` and `map_editor` binaries must compile without new errors or warnings after the move. | Phase 1 |
| NFR-3.2 | `cargo test` must pass with no new failures. Existing tests that reference `VoxelType` variants in literals must compile unchanged via the re-export. | Phase 1 |
| NFR-3.3 | `cargo clippy` must report zero new errors. | Phase 1 |
| NFR-3.4 | The serialised RON format is unchanged — `VoxelType` variant names (`Air`, `Grass`, `Dirt`, `Stone`) are identical before and after the move, so all existing map files remain loadable. | Phase 1 |

---

## 4. Phase Scoping

### Phase 1 — MVP (this fix)

- Create `src/systems/game/map/format/voxel_type.rs` with the moved definition.
- Re-export from `format/mod.rs` and `components.rs`.
- Fix the `world.rs` cross-import.
- Update `docs/api/map-format-spec.md`.
- Both binaries compile cleanly with zero new clippy errors or warnings.

### Phase 2 — Future (out of scope)

- Expand the variant set (e.g. `Sand`, `Wood`, `Metal`) once `VoxelType` lives
  in the correct module.
- Consider whether the renderer should read `voxel_type` to drive colour
  selection (currently the renderer ignores it and uses spatial hash colouring).

---

## 5. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | The eleven consumer files that import `VoxelType` via `crate::systems::game::components::VoxelType` do not need to change because `components.rs` will re-export the type. |
| 2 | `rotation.rs` uses `VoxelType` only in test literal fixtures (`#[cfg(test)]`); no production code in that file will be affected. |
| 3 | The renderer (`spawner/chunks.rs`) ignores `voxel_type` at runtime; this fix does not change runtime colouring behaviour. |
| 4 | Serde round-tripping is unaffected: the variant names and their string representations in RON files are identical before and after the move. |

---

## 6. Open Questions

*All questions resolved.*

| # | Question | Resolution |
|---|----------|------------|
| 1 | Must any consumer import path change? | **No.** The re-export from `components.rs` preserves all existing paths (FR-2.2.2). |
| 2 | Does the RON format change? | **No.** Variant names are identical; all existing map files remain loadable (NFR-3.4). |
| 3 | Should the renderer be updated to use `voxel_type` for colouring in this fix? | **No.** Renderer colouring is a separate concern and out of scope for Phase 1. |

---

## 7. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | Fix 1 — Orientation matrix system | **Done** (commit `eda90e3`) | Team |
| 2 | Fix 4 — Duplicate voxel detection | **Done** (commit `9f960d1`) | Team |
| 3 | Fix 5 — Entity property validation | **Done** (commit `a978d23`) | Team |

No blockers. This is an independent structural refactor.

---

## 8. Reference: Files Affected

| File | Change |
|------|--------|
| `src/systems/game/map/format/voxel_type.rs` | **New** — contains `VoxelType` definition |
| `src/systems/game/map/format/mod.rs` | **Modified** — add `mod voxel_type;` and `pub use voxel_type::VoxelType;` |
| `src/systems/game/components.rs` | **Modified** — replace definition with `pub use crate::systems::game::map::format::VoxelType;` |
| `src/systems/game/map/format/world.rs` | **Modified** — fix cross-import to use `super::VoxelType` |
| `docs/api/map-format-spec.md` | **Modified** — update VoxelType location note |

**Not changed** (import paths remain valid via re-export):

- `src/systems/game/map/format/defaults.rs`
- `src/systems/game/map/format/rotation.rs` (tests only)
- `src/systems/game/map/validation.rs` (tests only)
- `src/editor/state.rs`
- `src/editor/controller/hotbar.rs`
- `src/editor/controller/input.rs`
- `src/editor/ui/properties/voxel_tools.rs`
- `src/editor/ui/toolbar/tool_options.rs`
- `src/editor/ui/outliner.rs`
- `src/editor/tools/voxel_tool/mod.rs`
- `src/editor/history.rs` (tests only)
- `src/editor/shortcuts.rs` (tests only)
- `src/editor/file_io.rs` (tests only)

---

*Created: 2026-03-31*
*Source: `docs/investigations/2026-03-22-1427-map-format-analysis.md` — Finding 6*
