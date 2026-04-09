# Implementation Plan — Organize Unit Tests into Sibling Test Files

**Date:** 2026-04-08
**Companion documents:** [Ticket](./ticket.md) | [Requirements](./requirements.md) | [Architecture](./architecture.md)

---

## Overview

This plan migrates all 39 inline `#[cfg(test)] mod tests { ... }` blocks to the sibling `tests.rs` pattern defined in `coding-style.md` §7 and demonstrated by the two reference implementations (`geometry/tests.rs`, `occlusion/tests.rs`). Work is organized into 11 phases. Each phase ends with a `cargo test` verification and a single commit before the next phase begins.

**Total files to migrate:** 39  
**Total tests (must remain constant):** 485  
**Estimated commits:** 11 (one per phase) + 1 final documentation commit

---

## Quick Reference: The Transformation

For every file in this plan, the same two-step mechanical change is applied:

**Step 1 — In `mod.rs` (or the renamed flat file), replace the entire inline block:**

```rust
// REMOVE this entire block (however many lines it spans):
#[cfg(test)]
mod tests {
    use super::*;
    // ... all test functions and helpers ...
}

// REPLACE with this single line:
#[cfg(test)]
mod tests;
```

**Step 2 — Create `tests.rs` as a sibling:**

```rust
// tests.rs — everything that was inside the braces above, verbatim:
use super::*;

#[test]
fn example() { ... }
```

For **flat files** (`foo.rs`): first rename to `foo/mod.rs` by creating a `foo/` directory and moving the file. The `mod foo;` declaration in the parent file does not change.

For **existing `mod.rs` files**: no rename needed. Create `tests.rs` in the same directory.

---

## Phase 1 — `map/spawner/mod.rs` (Type B — no rename needed)

**Why first:** `spawner/mod.rs` is already a directory module. This is the simplest possible migration — no file rename, just extract the test block to a sibling `tests.rs`. It's a low-risk warm-up that validates the workflow.

### Files

| File | Lines | Tests |
|------|-------|-------|
| `src/systems/game/map/spawner/mod.rs` | 638 | 25 |

### Steps

1. Open `src/systems/game/map/spawner/mod.rs`.
2. Locate the `#[cfg(test)] mod tests { ... }` block (begins around line 145).
3. Cut everything inside the braces (the `use super::*;` line and all test functions).
4. Replace the entire block with `#[cfg(test)] mod tests;`.
5. Create `src/systems/game/map/spawner/tests.rs` with the cut content (opened with `use super::*;`).
6. Run `cargo test` — must report **485 passing**.
7. Commit: `refactor(tests): extract spawner/mod.rs tests to tests.rs`

---

## Phase 2 — `map/spawner/` flat files (5 files)

**Why grouped:** All five files live in the same directory cluster and are independent of each other. Converting them together in one commit keeps the changeset reviewable while reducing the number of commits.

### Files

| File | Lines | Tests | New path after rename |
|------|-------|-------|-----------------------|
| `map/spawner/chunks.rs` | 603 | 16 | `spawner/chunks/mod.rs` |
| `map/spawner/entities.rs` | 616 | 36 | `spawner/entities/mod.rs` |
| `map/spawner/shadow_quality.rs` | 93 | 5 | `spawner/shadow_quality/mod.rs` |
| `map/spawner/meshing/palette.rs` | 121 | 6 | `spawner/meshing/palette/mod.rs` |
| `map/spawner/meshing/occupancy.rs` | 150 | 10 | `spawner/meshing/occupancy/mod.rs` |

### Steps (repeat for each file)

1. Create the new directory: `mkdir src/systems/game/map/spawner/chunks/`
2. Move the file: `mv src/systems/game/map/spawner/chunks.rs src/systems/game/map/spawner/chunks/mod.rs`
3. In `mod.rs`: replace the inline test block with `#[cfg(test)] mod tests;`.
4. Create `src/systems/game/map/spawner/chunks/tests.rs` with the extracted content.
5. Repeat for the other 4 files.
6. Run `cargo test` — must report **485 passing**.
7. Commit: `refactor(tests): extract spawner flat-file tests to tests.rs`

> **Note on `meshing/` subdirectory:** `palette.rs` and `occupancy.rs` live under `spawner/meshing/`. Their new paths become `spawner/meshing/palette/mod.rs` and `spawner/meshing/occupancy/mod.rs`. The `mod palette;` and `mod occupancy;` declarations in `spawner/meshing/mod.rs` (or equivalent) do not change.

---

## Phase 3 — `map/format/` (3 files)

**Why grouped:** All three files belong to the same `map/format/` module. `rotation.rs` is the largest file in this group (855 lines, 29 tests) and the highest-value extraction in the entire codebase by line count saved.

### Files

| File | Lines | Tests | New path after rename |
|------|-------|-------|-----------------------|
| `map/format/rotation.rs` | 855 | 29 | `map/format/rotation/mod.rs` |
| `map/format/patterns.rs` | 328 | 18 | `map/format/patterns/mod.rs` |
| `map/format/camera.rs` | 121 | 6 | `map/format/camera/mod.rs` |

### Steps

1. For each file: create directory, move file to `mod.rs`, extract test block to `tests.rs`.
2. `rotation.rs` has a large test block starting around line 307 — take care to include all 29 test functions and any helper functions defined within the test module.
3. Run `cargo test` — must report **485 passing**.
4. Commit: `refactor(tests): extract map/format tests to tests.rs`

---

## Phase 4 — `map/` top-level (2 files)

### Files

| File | Lines | Tests | New path after rename |
|------|-------|-------|-----------------------|
| `map/validation.rs` | 682 | 33 | `map/validation/mod.rs` |
| `map/loader.rs` | 261 | 3 | `map/loader/mod.rs` |

### Steps

1. For each file: create directory, move file to `mod.rs`, extract test block to `tests.rs`.
2. `validation.rs` has 33 tests beginning around line 367 — the largest test count of any single file in the codebase. Verify all 33 are present in the new `tests.rs`.
3. Run `cargo test` — must report **485 passing**.
4. Commit: `refactor(tests): extract map/validation and map/loader tests to tests.rs`

---

## Phase 5 — `systems/game/` core (8 files)

**Why grouped:** These 8 files are the core gameplay systems. They are independent of each other and span a range of sizes (197–638 lines). Grouping them in one commit is efficient; the resulting diff is large but mechanically uniform.

### Files

| File | Lines | Tests | New path after rename |
|------|-------|-------|-----------------------|
| `gamepad.rs` | 638 | 21 | `gamepad/mod.rs` |
| `interior_detection.rs` | 591 | 11 | `interior_detection/mod.rs` |
| `npc_labels.rs` | 506 | 13 | `npc_labels/mod.rs` |
| `collision.rs` | 405 | 6 | `collision/mod.rs` |
| `input.rs` | 390 | 8 | `input/mod.rs` |
| `physics.rs` | 339 | 2 | `physics/mod.rs` |
| `player_movement.rs` | 324 | 1 | `player_movement/mod.rs` |
| `resources.rs` | 197 | 9 | `resources/mod.rs` |

### Steps

1. For each file: create directory, move file to `mod.rs`, extract test block to `tests.rs`.
2. Process in order of decreasing test count to surface any issues early: `gamepad` → `npc_labels` → `interior_detection` → `input` → `collision` → `resources` → `physics` → `player_movement`.
3. Run `cargo test` — must report **485 passing**.
4. Commit: `refactor(tests): extract systems/game core tests to tests.rs`

---

## Phase 6 — `hot_reload/state.rs` and `settings/vsync.rs` (2 files)

**Why grouped:** These two files are in different subsystems but are both small and isolated. Grouping them avoids a commit with only 1 file.

### Files

| File | Lines | Tests | New path after rename |
|------|-------|-------|-----------------------|
| `systems/game/hot_reload/state.rs` | 174 | 2 | `hot_reload/state/mod.rs` |
| `systems/settings/vsync.rs` | 442 | 22 | `settings/vsync/mod.rs` |

### Steps

1. For each file: create directory, move to `mod.rs`, extract test block to `tests.rs`.
2. `vsync.rs` has 22 tests beginning around line 256 — confirm all are captured.
3. Run `cargo test` — must report **485 passing**.
4. Commit: `refactor(tests): extract hot_reload/state and settings/vsync tests to tests.rs`

---

## Phase 7 — `editor/grid/` (4 files)

**Why grouped:** Four small files in the same directory, each with exactly 1 test. All conversions are trivial. Process together for efficiency.

### Files

| File | Lines | Tests | New path after rename |
|------|-------|-------|-----------------------|
| `editor/grid/systems.rs` | 100 | 1 | `grid/systems/mod.rs` |
| `editor/grid/mesh.rs` | 132 | 1 | `grid/mesh/mod.rs` |
| `editor/grid/bounds.rs` | 132 | 1 | `grid/bounds/mod.rs` |
| `editor/grid/cursor_indicator.rs` | 144 | 1 | `grid/cursor_indicator/mod.rs` |

### Steps

1. For each file: create directory, move to `mod.rs`, extract test block to `tests.rs`.
2. Each `tests.rs` will be very short (5–15 lines). This is expected and fine.
3. Run `cargo test` — must report **485 passing**.
4. Commit: `refactor(tests): extract editor/grid tests to tests.rs`

---

## Phase 8 — `editor/controller/` (5 files)

### Files

| File | Lines | Tests | New path after rename |
|------|-------|-------|-----------------------|
| `controller/input.rs` | 585 | 5 | `controller/input/mod.rs` |
| `controller/camera.rs` | 394 | 17 | `controller/camera/mod.rs` |
| `controller/hotbar.rs` | 390 | 6 | `controller/hotbar/mod.rs` |
| `controller/cursor.rs` | 360 | 4 | `controller/cursor/mod.rs` |
| `controller/palette.rs` | 301 | 3 | `controller/palette/mod.rs` |

### Steps

1. For each file: create directory, move to `mod.rs`, extract test block to `tests.rs`.
2. Run `cargo test` — must report **485 passing**.
3. Commit: `refactor(tests): extract editor/controller tests to tests.rs`

---

## Phase 9 — `editor/tools/` (1 file)

### Files

| File | Lines | Tests | New path after rename |
|------|-------|-------|-----------------------|
| `tools/input/helpers.rs` | 360 | 13 | `tools/input/helpers/mod.rs` |

### Steps

1. Create directory `src/editor/tools/input/helpers/`.
2. Move file to `helpers/mod.rs`.
3. Extract test block (13 tests starting around line 219) to `helpers/tests.rs`.
4. Run `cargo test` — must report **485 passing**.
5. Commit: `refactor(tests): extract editor/tools/input/helpers tests to tests.rs`

> **Note on nesting depth:** `helpers.rs` lives three levels deep inside `editor/tools/input/`. After conversion the path becomes `editor/tools/input/helpers/mod.rs`. This is deep but correct; Rust handles arbitrary nesting without issue.

---

## Phase 10 — `editor/ui/` (2 files)

**Why last among code files:** These are the two largest files in the entire codebase. `outliner.rs` at 1,031 lines is the longest file overall. Placing them near the end means if any earlier phase surfaces a workflow issue, it can be corrected before tackling the most complex files.

### Files

| File | Lines | Tests | New path after rename |
|------|-------|-------|-----------------------|
| `ui/viewport.rs` | 668 | 19 | `ui/viewport/mod.rs` |
| `ui/outliner.rs` | 1,031 | 29 | `ui/outliner/mod.rs` |

### Steps

1. **`viewport.rs`:**
   - Create `src/editor/ui/viewport/`.
   - Move to `viewport/mod.rs`.
   - Locate test block (starts around line 524, 19 tests).
   - Extract to `viewport/tests.rs`.
   - Run `cargo test`.

2. **`outliner.rs`:**
   - Create `src/editor/ui/outliner/`.
   - Move to `outliner/mod.rs`.
   - Locate test block (starts around line 587, 29 tests).
   - Extract to `outliner/tests.rs`.
   - Run `cargo test`.

3. Must report **485 passing** after each file.
4. Commit: `refactor(tests): extract editor/ui tests to tests.rs`

---

## Phase 11 — `editor/` top-level (5 files)

### Files

| File | Lines | Tests | New path after rename |
|------|-------|-------|-----------------------|
| `editor/camera.rs` | 946 | 18 | `editor/camera/mod.rs` |
| `editor/state.rs` | 494 | 21 | `editor/state/mod.rs` |
| `editor/file_io.rs` | 566 | 15 | `editor/file_io/mod.rs` |
| `editor/history.rs` | 314 | 3 | `editor/history/mod.rs` |
| `editor/shortcuts.rs` | 320 | 3 | `editor/shortcuts/mod.rs` |

### Steps

1. For each file: create directory, move to `mod.rs`, extract test block to `tests.rs`.
2. `camera.rs` (946 lines, 18 tests) is the second-largest file overall — test block starts around line 676.
3. `state.rs` (494 lines, 21 tests) — test block starts around line 255.
4. Process in order: `camera` → `state` → `file_io` → `history` → `shortcuts`.
5. Run `cargo test` after each file — must report **485 passing** each time.
6. Commit: `refactor(tests): extract editor top-level tests to tests.rs`

---

## Phase 12 — Developer Guidelines

**Prerequisite:** All 11 code migration phases complete. `cargo test` reports 485 passing with zero failures.

### Files to update

| File | Change |
|------|--------|
| `AGENTS.md` | Line 76: replace "Tests are inline with `#[cfg(test)]` modules at bottom of files" with a reference to `coding-style.md` §7 |
| `docs/developer-guide/coding-style.md` | Review §7 — confirm the syntax, example, and rules are accurate; update if any detail is stale |

### Steps

1. Open `AGENTS.md`.
2. Find line 76: `- Tests are inline with \`#[cfg(test)]\` modules at bottom of files`
3. Replace with: `- Tests live in sibling \`tests.rs\` files — see \`docs/developer-guide/coding-style.md\` §7`
4. Open `docs/developer-guide/coding-style.md` §7.
5. Read the section in full. Verify:
   - The delegation syntax (`#[cfg(test)] mod tests;`) is shown correctly.
   - The `tests.rs` opening (`use super::*;`) is shown correctly.
   - The example test names and bodies are realistic.
   - No references to inline test modules remain.
6. Make corrections if needed. If no changes are required, note "§7 confirmed accurate — no changes needed."
7. Run `cargo test && cargo clippy && cargo build && cargo build --bin map_editor` — final full verification.
8. Commit: `docs: update AGENTS.md and coding-style.md to reflect tests.rs convention`

---

## Verification Checklist

Run this checklist after Phase 12 before closing the ticket.

```bash
# 1. All tests pass
cargo test
# Expected: "test result: ok. 485 passed; 0 failed"

# 2. No new clippy warnings
cargo clippy
# Expected: zero warnings beyond pre-existing baseline

# 3. Game binary builds
cargo build
# Expected: no errors

# 4. Editor binary builds
cargo build --bin map_editor
# Expected: no errors

# 5. No inline test blocks remain
rg '#\[cfg\(test\)\]\s*mod tests \{' src/
# Expected: zero matches

# 6. All tests.rs files have correct opening line
rg '^use super::\*;' src/ --include='tests.rs'
# Expected: one match per migrated file (39 total)
```

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Test block extraction misses a helper function or `use` statement | Medium | One or more tests fail to compile | Run `cargo test` after every phase; compiler errors are immediate |
| A `mod foo;` declaration in a parent file needs updating | Low | Compile error | This cannot happen — Rust resolves `foo.rs` and `foo/mod.rs` identically. Confirmed by Rust Reference. |
| `outliner.rs` or `camera.rs` test block has complex internal structure (sub-modules, conditional compilation) | Low | Extraction requires more care | Read the full test block before extracting; move the entire `mod tests { ... }` contents as-is |
| Phase ordering introduces a merge conflict if other branches are active | Low | Minor rework | Each phase is a single atomic commit; rebase is straightforward |
| `cargo clippy` introduces new warnings due to visibility changes | Low | Warnings to resolve | Visibility of items in production code does not change; `pub` status is unchanged |

---

## Summary Table

| Phase | Scope | Files | Tests | Commit message |
|-------|-------|-------|-------|----------------|
| 1 | `spawner/mod.rs` | 1 | 25 | `refactor(tests): extract spawner/mod.rs tests to tests.rs` |
| 2 | `spawner/` flat files | 5 | 73 | `refactor(tests): extract spawner flat-file tests to tests.rs` |
| 3 | `map/format/` | 3 | 53 | `refactor(tests): extract map/format tests to tests.rs` |
| 4 | `map/` top-level | 2 | 36 | `refactor(tests): extract map/validation and map/loader tests to tests.rs` |
| 5 | `systems/game/` core | 8 | 71 | `refactor(tests): extract systems/game core tests to tests.rs` |
| 6 | `hot_reload/` + `settings/` | 2 | 24 | `refactor(tests): extract hot_reload/state and settings/vsync tests to tests.rs` |
| 7 | `editor/grid/` | 4 | 4 | `refactor(tests): extract editor/grid tests to tests.rs` |
| 8 | `editor/controller/` | 5 | 35 | `refactor(tests): extract editor/controller tests to tests.rs` |
| 9 | `editor/tools/` | 1 | 13 | `refactor(tests): extract editor/tools/input/helpers tests to tests.rs` |
| 10 | `editor/ui/` | 2 | 48 | `refactor(tests): extract editor/ui tests to tests.rs` |
| 11 | `editor/` top-level | 5 | 60 | `refactor(tests): extract editor top-level tests to tests.rs` |
| 12 | Developer guidelines | 2 docs | — | `docs: update AGENTS.md and coding-style.md to reflect tests.rs convention` |
| **Total** | | **39 files + 2 docs** | **485** | |
