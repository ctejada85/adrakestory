# Bug: EditorHistory::redo() Does Not Enforce the Maximum History Cap

**Date:** 2026-04-09
**Severity:** Low
**Component:** Editor — History (`src/editor/history/mod.rs`)

---

## Story

As a developer maintaining the editor, I want every code path that pushes onto the undo stack to enforce the `max_history` cap so that memory usage stays bounded regardless of how undo and redo are interleaved.

---

## Description

`EditorHistory::push()` (line 48 of `src/editor/history/mod.rs`) correctly evicts the oldest entry when `undo_stack.len() > max_history`. However, `redo()` (line 72) pushes onto `undo_stack` with no cap check:

```rust
pub fn redo(&mut self) -> Option<EditorAction> {
    if let Some(action) = self.redo_stack.pop() {
        self.undo_stack.push(action.clone()); // no cap enforcement
        Some(action)
    } else {
        None
    }
}
```

In practice, the redo stack can hold at most `max_history` items (since it is populated only by undoing items from the bounded undo stack), so an overflow requires an adversarial sequence. Nevertheless, the invariant that `undo_stack.len() <= max_history` is documented in `MAX_HISTORY_SIZE` and should be maintained by every push site. The inconsistency also makes `with_max_size(0)` behave incorrectly when redo is called.

Additionally, `undo_stack.remove(0)` in `push()` (line 57) is O(n) because `Vec::remove` shifts all subsequent elements. Replacing `undo_stack` and `redo_stack` with `VecDeque` would make both the eviction (`pop_front`) and the normal push/pop O(1).

---

## Acceptance Criteria

1. After calling `redo()`, `undo_stack.len()` never exceeds `max_history`.
2. Calling `redo()` when `undo_stack.len() == max_history` evicts the oldest entry before inserting the redone action (same policy as `push()`).
3. `with_max_size(0)` — calling `redo()` immediately after `undo()` — does not panic and leaves both stacks empty.
4. Existing tests for `push()` and `undo()` continue to pass.
5. New unit tests in `src/editor/history/tests.rs` cover: redo cap enforcement, `with_max_size(0)` round-trip, and undo-then-redo preserving the original action data.

---

## Non-Functional Requirements

- `undo_stack` and `redo_stack` should be changed from `Vec` to `VecDeque` to eliminate the O(n) `remove(0)` eviction cost in `push()`.
- The public API (`push`, `undo`, `redo`, `can_undo`, `can_redo`, `undo_count`, `redo_count`, `clear`) must remain unchanged.
- No heap allocation may be introduced per `push`/`undo`/`redo` call beyond what already exists.

---

## Tasks

1. Change `undo_stack: Vec<EditorAction>` and `redo_stack: Vec<EditorAction>` to `VecDeque<EditorAction>` in `src/editor/history/mod.rs`.
2. Update `push()` to use `push_back` and `pop_front` (eviction) instead of `push` and `remove(0)`.
3. Update `undo()` to use `pop_back` / `push_back` on the appropriate stacks.
4. Add the cap check to `redo()`: after pushing onto `undo_stack`, evict front if `len > max_history`.
5. Update `undo_count()` and `redo_count()` if they rely on `Vec`-specific methods (`.len()` is identical on `VecDeque`).
6. Add unit tests as described in the acceptance criteria.
7. Manual verification: perform 110 actions (default cap is 100), undo 10, redo 10; confirm `undo_count()` stays ≤ 100.
