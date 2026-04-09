# Test: Improve EditorHistory Unit Test Coverage

**Date:** 2026-04-09
**Severity:** Medium
**Component:** Editor — History (`src/editor/history/mod.rs`, `src/editor/history/tests.rs`)

---

## Story

As a developer, I want comprehensive unit tests for `EditorHistory` so that edge cases and invariants are verified automatically and regressions are caught immediately.

---

## Description

The existing tests in `src/editor/history/tests.rs` cover the happy path for `push`, `undo`, `redo`, and redo-stack clear-on-new-action. The following cases are untested:

- **`max_history` eviction**: pushing more than `max_history` actions should evict the oldest.
- **`with_max_size(0)` edge case**: a history with capacity zero should silently discard all pushes; `undo()` and `redo()` should return `None`.
- **redo cap enforcement** (related to BUG in `bug-redo-ignores-history-cap`): after redo, `undo_stack.len()` must not exceed `max_history`.
- **`Batch` action inverse order**: undoing a `Batch` must apply inner actions in reverse order.
- **`ModifyEntity` restore**: undoing a `ModifyEntity` action must restore `old_data` exactly.
- **`undo_description` / `redo_description`**: these should return the description of the top item or `None` when the stack is empty.

---

## Acceptance Criteria

1. A test verifies that pushing `max_history + 10` actions results in `undo_count() == max_history`.
2. A test verifies that `with_max_size(0)` followed by `push`, `undo`, `redo` never panics and returns expected values.
3. A test verifies that calling `redo()` when `undo_stack.len() == max_history` does not grow the stack beyond `max_history` (pending `bug-redo-ignores-history-cap` fix; the test should initially fail and then pass after the fix).
4. A test verifies that undoing a `Batch { actions: [A, B, C] }` calls inverses in the order C, B, A.
5. A test verifies that undoing a `ModifyEntity` action restores `old_data` field-for-field.
6. A test verifies `undo_description()` returns `None` on an empty stack and `Some(...)` with the correct description after a push.
7. All new tests live in `src/editor/history/tests.rs` and pass with `cargo test`.

---

## Non-Functional Requirements

- Tests must not depend on Bevy's ECS runtime; `EditorHistory` is a plain Rust struct and can be tested in isolation.
- Test helper functions for constructing `VoxelData` and `EntityData` should be shared within the test module to avoid repetition.

---

## Tasks

1. Add test for `max_history` eviction.
2. Add test for `with_max_size(0)`.
3. Add test for redo cap (marked `#[ignore]` until `bug-redo-ignores-history-cap` is fixed, or structured so it drives that fix).
4. Add test for `Batch` inverse ordering.
5. Add test for `ModifyEntity` undo restoring `old_data`.
6. Add tests for `undo_description()` and `redo_description()`.
7. Run `cargo test editor::history` and confirm all pass.
