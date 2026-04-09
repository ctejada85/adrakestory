# Bug: Outliner Rename Mode Targets the Wrong Entity After Lower-Index Deletion

**Date:** 2026-04-09
**Severity:** Medium
**Component:** Editor — Outliner (`src/editor/ui/outliner/mod.rs`)

---

## Story

As a level designer, I want renaming an entity to be unaffected by deletions of other entities so that I never accidentally rename the wrong entity.

---

## Description

`OutlinerState::renaming_index` stores a raw `Vec` index into `editor_state.current_map.entities`. When an entity at a lower index is deleted (line 536 of `src/editor/ui/outliner/mod.rs`), the `Vec` shifts all subsequent entries down by one. The deletion guard (lines 525–534) only exits rename mode when the deleted index matches `renaming_index` exactly. If `renaming_index` is `Some(5)` and the entity at index `3` is deleted, the guard does not fire; on the next frame `renaming_index` is still `Some(5)` but now points to the entity that was previously at index `6`. The rename text input then modifies the wrong entity without any warning.

The fix is to update (or invalidate) `renaming_index` when any entity with a smaller index is deleted: if `deleted_index < renaming_index`, decrement `renaming_index` by one; if `deleted_index == renaming_index`, clear rename mode (already handled).

---

## Acceptance Criteria

1. Deleting entity at index `i` when `renaming_index = Some(j)` and `i < j` decrements `renaming_index` to `Some(j - 1)`, keeping the correct entity in rename mode.
2. Deleting entity at index `i` when `renaming_index = Some(i)` clears rename mode (existing behavior, must be preserved).
3. Deleting entity at index `i` when `renaming_index = Some(j)` and `i > j` leaves `renaming_index` unchanged.
4. After adjustment, completing the rename pushes a history entry for the entity that was originally being renamed.
5. A unit test exercises all three cases (i < j, i == j, i > j) and verifies the resulting `renaming_index` value.

---

## Non-Functional Requirements

- The fix is confined to the deletion handler in `render_entities_section` (`src/editor/ui/outliner/mod.rs`).
- No change to `OutlinerState`'s public API is required.

---

## Tasks

1. In the deletion handler (after `entity_to_delete` is resolved), add index-adjustment logic for `renaming_index` as described above.
2. Add unit tests in `src/editor/ui/outliner/tests.rs` covering all three index relationship cases.
3. Manual verification: open the editor, add three entities (A at 0, B at 1, C at 2), begin renaming C (index 2), then delete B (index 1), confirm the rename text field still targets C (now at index 1) and not a different entity.
