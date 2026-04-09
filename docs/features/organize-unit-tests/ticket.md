# Organize Unit Tests into Sibling Test Files

**Date:** 2026-04-08
**Component:** All systems — test organization refactor

---

## Story

As a developer, I want every module's unit tests in a sibling `tests.rs` file so that production code files are easy to navigate and the whole codebase follows a single, consistent convention with no exceptions.

---

## Description

The project has 485 unit tests spread across 41 files. 39 of those files still embed `#[cfg(test)] mod tests { ... }` inline at the bottom of the production code file. Two files already follow the correct pattern (`geometry/mod.rs` and `occlusion/mod.rs`), and `coding-style.md` §7 already documents the sibling `tests.rs` approach as the standard. This ticket migrates all 39 remaining inline test modules to that standard and corrects `AGENTS.md`, which currently contradicts `coding-style.md` by describing inline tests as the convention. No file is left as an exception regardless of size.

---

## Acceptance Criteria

1. Every source file that previously contained an inline `#[cfg(test)] mod tests { ... }` block now contains only the delegation line `#[cfg(test)] mod tests;` in its place.
2. Every migrated file's tests live in a sibling `tests.rs` file that opens with `use super::*;` and contains all previously inline tests — names, assertions, and helpers unmodified.
3. `cargo test` passes with the same 485 tests at every commit during migration — no tests are added, removed, or renamed.
4. `cargo clippy` and both `cargo build` / `cargo build --bin map_editor` pass with zero new warnings after all migrations are complete.
5. `AGENTS.md` no longer states that tests are inline; it references `coding-style.md` §7 as the authoritative test convention.
6. `coding-style.md` §7 is reviewed and confirmed accurate (no changes needed if already correct; update if any detail is stale).

---

## Non-Functional Requirements

- No production behavior may change. This is a file-layout refactor only — no logic, types, or public API may be modified.
- Each group of migrations must be independently buildable and testable (one commit per logical group, verified before proceeding).
- Private access must be preserved in all migrated tests. The `tests` module remains a child of its parent, so `super::*` provides identical access to the inline case.
- No new `#[allow(...)]` suppressions may be introduced.

---

## Background: Why Sibling `tests.rs`

`coding-style.md` §7 documents this pattern. Summary for implementers:

For a flat file `foo.rs`, convert to a directory module:

```
# Before
src/systems/game/foo.rs        ← production code + inline tests mixed

# After
src/systems/game/foo/
  mod.rs                       ← production code only
  tests.rs                     ← tests only
```

The delegation line replaces the entire inline block in `mod.rs`:

```rust
#[cfg(test)]
mod tests;
```

`tests.rs` opens with:

```rust
use super::*;
```

For files that are already `mod.rs` (e.g., `spawner/mod.rs`), no directory conversion is needed — add `tests.rs` as a sibling in the same directory and add the delegation line.

Private access is preserved identically to the inline case because `tests` is still a child module.

---

## Target Files

39 files require migration. 2 files (`geometry/mod.rs`, `occlusion/mod.rs`) are already complete and serve as the reference implementation.

### Game — `src/systems/game/`

| File | Lines | Tests | Action |
|------|-------|-------|--------|
| `map/format/rotation.rs` | 855 | 29 | Convert to directory module |
| `map/validation.rs` | 682 | 33 | Convert to directory module |
| `gamepad.rs` | 638 | 21 | Convert to directory module |
| `map/spawner/mod.rs` | 638 | 25 | Already a directory — add sibling `tests.rs` |
| `map/spawner/entities.rs` | 616 | 36 | Convert to directory module |
| `map/spawner/chunks.rs` | 603 | 16 | Convert to directory module |
| `interior_detection.rs` | 591 | 11 | Convert to directory module |
| `npc_labels.rs` | 506 | 13 | Convert to directory module |
| `collision.rs` | 405 | 6 | Convert to directory module |
| `input.rs` | 390 | 8 | Convert to directory module |
| `map/format/patterns.rs` | 328 | 18 | Convert to directory module |
| `physics.rs` | 339 | 2 | Convert to directory module |
| `player_movement.rs` | 324 | 1 | Convert to directory module |
| `map/loader.rs` | 261 | 3 | Convert to directory module |
| `hot_reload/state.rs` | 174 | 2 | Convert to directory module |
| `resources.rs` | 197 | 9 | Convert to directory module |
| `map/format/camera.rs` | 121 | 6 | Convert to directory module |
| `map/spawner/meshing/occupancy.rs` | 150 | 10 | Convert to directory module |
| `map/spawner/meshing/palette.rs` | 121 | 6 | Convert to directory module |
| `map/spawner/shadow_quality.rs` | 93 | 5 | Convert to directory module |

### Settings — `src/systems/settings/`

| File | Lines | Tests | Action |
|------|-------|-------|--------|
| `vsync.rs` | 442 | 22 | Convert to directory module |

### Editor — `src/editor/`

| File | Lines | Tests | Action |
|------|-------|-------|--------|
| `ui/outliner.rs` | 1,031 | 29 | Convert to directory module |
| `camera.rs` | 946 | 18 | Convert to directory module |
| `ui/viewport.rs` | 668 | 19 | Convert to directory module |
| `controller/input.rs` | 585 | 5 | Convert to directory module |
| `file_io.rs` | 566 | 15 | Convert to directory module |
| `state.rs` | 494 | 21 | Convert to directory module |
| `controller/camera.rs` | 394 | 17 | Convert to directory module |
| `controller/hotbar.rs` | 390 | 6 | Convert to directory module |
| `tools/input/helpers.rs` | 360 | 13 | Convert to directory module |
| `controller/cursor.rs` | 360 | 4 | Convert to directory module |
| `shortcuts.rs` | 320 | 3 | Convert to directory module |
| `controller/palette.rs` | 301 | 3 | Convert to directory module |
| `history.rs` | 314 | 3 | Convert to directory module |
| `grid/mesh.rs` | 132 | 1 | Convert to directory module |
| `grid/bounds.rs` | 132 | 1 | Convert to directory module |
| `grid/cursor_indicator.rs` | 144 | 1 | Convert to directory module |
| `grid/systems.rs` | 100 | 1 | Convert to directory module |
| `grid/cursor_indicator.rs` | 144 | 1 | Convert to directory module |

---

## Tasks

Tasks are grouped by module cluster. Run `cargo test` after each commit before proceeding.

1. Migrate `src/systems/game/map/spawner/mod.rs` — add sibling `tests.rs` (no directory conversion needed)
2. Migrate `src/systems/game/map/spawner/` flat files — `chunks.rs`, `entities.rs`, `shadow_quality.rs`, `meshing/palette.rs`, `meshing/occupancy.rs`
3. Migrate `src/systems/game/map/format/` — `rotation.rs`, `patterns.rs`, `camera.rs`
4. Migrate `src/systems/game/map/` top-level — `validation.rs`, `loader.rs`
5. Migrate `src/systems/game/` core systems — `gamepad.rs`, `physics.rs`, `input.rs`, `collision.rs`, `interior_detection.rs`, `npc_labels.rs`, `player_movement.rs`, `resources.rs`
6. Migrate `src/systems/game/hot_reload/state.rs` and `src/systems/settings/vsync.rs`
7. Migrate `src/editor/grid/` — `systems.rs`, `mesh.rs`, `bounds.rs`, `cursor_indicator.rs`
8. Migrate `src/editor/controller/` — `input.rs`, `camera.rs`, `cursor.rs`, `hotbar.rs`, `palette.rs`
9. Migrate `src/editor/tools/input/helpers.rs`
10. Migrate `src/editor/ui/` — `viewport.rs`, `outliner.rs`
11. Migrate `src/editor/` top-level — `camera.rs`, `state.rs`, `file_io.rs`, `history.rs`, `shortcuts.rs`
12. Update `AGENTS.md` — replace the inline-tests statement with a reference to `coding-style.md` §7
13. Review `coding-style.md` §7 — confirm the section is accurate and up to date; update if needed
14. Run full verification: `cargo test && cargo clippy && cargo build && cargo build --bin map_editor` — confirm 485 passing, zero new warnings
