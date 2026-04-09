# Requirements — Organize Unit Tests into Sibling Test Files

**Source:** `docs/features/organize-unit-tests/ticket.md` — 2026-04-08
**Status:** Draft

---

## 1. Overview

The project has 485 unit tests across 41 source files. 39 of those files embed their tests inline using `#[cfg(test)] mod tests { ... }` at the bottom of the production code file. This causes several files to exceed 600–1,000 lines, mixing test code with production code and making both harder to navigate.

Two files already follow the correct pattern (`src/systems/game/map/geometry/mod.rs` and `src/systems/game/occlusion/mod.rs`): they declare `#[cfg(test)] mod tests;` in the production file and keep all test code in a sibling `tests.rs` file. `coding-style.md` §7 already documents this as the project standard. This feature migrates all 39 remaining inline test modules to match, and corrects `AGENTS.md`, which currently contradicts the style guide by describing inline tests as the convention. No production behavior, public API, or logic changes in any way.

---

## 2. Functional Requirements

### 2.1 File Layout

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1.1 | Every source file that contains an inline `#[cfg(test)] mod tests { ... }` block must be migrated to use a `#[cfg(test)] mod tests;` delegation line instead. | Phase 1 |
| FR-2.1.2 | Every migrated module must have a sibling `tests.rs` file co-located with its production code file. For flat files converted to directory modules, `tests.rs` must live inside the new directory alongside `mod.rs`. For existing `mod.rs` files, `tests.rs` must live in the same directory as `mod.rs`. | Phase 1 |
| FR-2.1.3 | Every `tests.rs` file must open with `use super::*;` to preserve access to all items in the parent module, identical to the inline case. | Phase 1 |
| FR-2.1.4 | No source file in the repository may contain an inline `#[cfg(test)] mod tests { ... }` block after the migration is complete. | Phase 1 |

### 2.2 Module Structure Conversion

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.2.1 | Any flat source file `foo.rs` whose inline test block is being extracted must be converted to a directory module: `foo.rs` becomes `foo/mod.rs`, and the test file becomes `foo/tests.rs`. | Phase 1 |
| FR-2.2.2 | Any source file that is already a `mod.rs` within a directory must have `tests.rs` added as a sibling file in the same directory. No structural conversion is required for these files. | Phase 1 |
| FR-2.2.3 | Parent module declarations (`mod foo;` in parent files) must not change as a result of the conversion. Rust resolves both `foo.rs` and `foo/mod.rs` to the same `mod foo;` declaration. | Phase 1 |

### 2.3 Test Content Preservation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.3.1 | Every test function must be preserved exactly: name, body, assertions, and helper functions must be byte-for-byte identical to their inline originals. | Phase 1 |
| FR-2.3.2 | The total test count must remain 485 after each individual migration commit. No tests may be added, removed, or renamed during this refactor. | Phase 1 |
| FR-2.3.3 | Test helper functions, structs, and constants that were defined within the inline `mod tests` block must be moved to `tests.rs` unchanged. | Phase 1 |
| FR-2.3.4 | Any `use` statements that were inside the inline test module must be moved to `tests.rs` unchanged. | Phase 1 |

### 2.4 Build and Test Integrity

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.4.1 | `cargo test` must pass with 485 tests after every individual migration commit before the next migration begins. | Phase 1 |
| FR-2.4.2 | `cargo build` (game binary) must pass after all migrations are complete. | Phase 1 |
| FR-2.4.3 | `cargo build --bin map_editor` (editor binary) must pass after all migrations are complete. | Phase 1 |
| FR-2.4.4 | `cargo clippy` must report zero new warnings after all migrations are complete. | Phase 1 |

### 2.5 Developer Guidelines

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.5.1 | `AGENTS.md` must be updated to remove the statement that describes tests as inline with `#[cfg(test)]` modules at the bottom of files. It must instead reference `coding-style.md` §7 as the authoritative test organization convention. | Phase 1 |
| FR-2.5.2 | `coding-style.md` §7 must be reviewed and confirmed accurate. If any detail (syntax, conventions, or examples) is stale or inconsistent with the migration outcome, it must be corrected. | Phase 1 |

---

## 3. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-3.1 | No production behavior may change. This is a file-layout refactor only. No logic, public API, ECS components, systems, or Bevy resources may be modified. | Phase 1 |
| NFR-3.2 | Private access must be preserved identically. The `tests` module remains a child of its parent after conversion (via `super::*`), providing the same access as the inline form. | Phase 1 |
| NFR-3.3 | No new `#[allow(...)]` suppressions may be introduced. | Phase 1 |
| NFR-3.4 | Each migration group must be delivered as a separate commit that builds and passes tests independently, enabling bisection if a regression is introduced. | Phase 1 |
| NFR-3.5 | The migration must not alter test execution order, test isolation, or any test output observable via `cargo test`. | Phase 1 |
| NFR-3.6 | No new Rust crates or dependencies may be added. | Phase 1 |

---

## 4. Phase Scoping

This is a single-phase refactoring migration. All work is Phase 1. There are no Phase 2 or future capabilities.

### Phase 1 — Full Migration

- Migrate all 39 files with inline test modules to sibling `tests.rs` pattern
- Update `AGENTS.md` to reflect the correct convention
- Confirm `coding-style.md` §7 is accurate

---

## 5. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | Rust module resolution treats `foo.rs` and `foo/mod.rs` as equivalent for `mod foo;` declarations. This is guaranteed by the Rust compiler; no parent files need to be updated when `foo.rs` is converted to `foo/mod.rs`. |
| 2 | The two reference implementations (`geometry/mod.rs` → `geometry/tests.rs` and `occlusion/mod.rs` → `occlusion/tests.rs`) are correct and complete. They serve as the template for all migrations. |
| 3 | All 485 tests are currently passing. Confirmed via `cargo test` on the post-`gamepad-settings-apply` commit (HEAD `896c240`). |
| 4 | `coding-style.md` §7 already documents the target convention correctly. The main documentation gap is `AGENTS.md` line 76, which contradicts it. |
| 5 | No test in the codebase references another test module by path. All tests are self-contained within their own `mod tests` block. |
| 6 | The `src/systems/game/map/spawner/mod.rs` file is already a directory module and requires only a `tests.rs` sibling — no structural conversion needed. |
| 7 | Files under `src/editor/grid/` (4 files, 1 test each) are among the smallest in the codebase. While the per-file overhead of conversion is the same regardless of size, the resulting `tests.rs` files will be very short (5–20 lines each). This is acceptable — consistency is the goal. |

---

## 6. Open Questions

All questions resolved — no open items.

| # | Question | Resolution |
|---|----------|------------|
| 1 | Should small files (e.g., < 100 lines, 1–2 tests) also be migrated? | Yes. The goal is consistency across the entire codebase with no exceptions, regardless of file size. |
| 2 | Should a top-level `tests/` directory be created for integration tests? | Out of scope. No integration tests exist, and creating integration test infrastructure is a separate concern. |
| 3 | Does converting `foo.rs` to `foo/mod.rs` require any change to how the module is declared in its parent? | No. Rust resolves both `foo.rs` and `foo/mod.rs` from `mod foo;`. No parent files change. |
| 4 | Will the change cause any import path differences visible to callers? | No. Module paths (e.g., `crate::systems::game::gamepad::GamepadSettings`) are unchanged by the directory module conversion. |

---

## 7. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | All 485 tests must be passing before migration begins | Done — HEAD `896c240` | — |
| 2 | `coding-style.md` §7 must be the established standard | Done — already documented correctly | — |

---

*Created: 2026-04-08*
*Source: `docs/features/organize-unit-tests/ticket.md`*
