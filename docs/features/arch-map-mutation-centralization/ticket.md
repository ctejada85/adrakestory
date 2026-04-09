# Architecture: Centralize All Map Mutations Through a Single Edit Path

**Date:** 2026-04-09
**Severity:** Low
**Component:** Editor — State / Shortcuts / Renderer (`src/editor/`)

---

## Story

As a developer adding a new editing feature, I want all map mutations to flow through a single, well-known code path so that I never accidentally forget to update history, fire a render event, or set the dirty flag.

---

## Description

Map mutations in the editor currently occur through at least three independent code paths, each responsible for manually remembering to call `mark_modified()`, push a history entry, and write a `RenderMapEvent`:

| Code path | File |
|---|---|
| Keyboard shortcuts (undo/redo) | `src/editor/shortcuts/mod.rs` |
| Gamepad voxel actions | `src/editor/camera/mod.rs` (via `handle_gamepad_voxel_actions`) |
| File I/O (hot reload) | `src/editor/file_io/mod.rs` |
| Outliner (delete, rename) | `src/editor/ui/outliner/mod.rs` |
| Tool systems (mouse place/remove) | `src/editor/tools/` |

Each path handles the three concerns (history, render event, dirty flag) ad-hoc. The consequence is a class of bugs where one concern is omitted — BUG-01 (entity deletion skips history) is a direct example. A second, related issue is that `detect_map_changes` in the renderer tries to compensate for missed render events by polling counts, which is unreliable (see `bug-map-render-misses-property-edits`).

The fix is to introduce a `MapMutationWriter` abstraction (a Bevy system parameter or a method on `EditorState`) that atomically applies a mutation, records it in history, marks the state as modified, and queues a `RenderMapEvent`. All current mutation sites should delegate to this abstraction.

---

## Acceptance Criteria

1. A `MapMutationWriter` (or equivalent) is introduced that accepts an `EditorAction` and performs: `apply_action`, `history.push`, `mark_modified`, and `render_events.write(RenderMapEvent)` in one call.
2. All existing mutation call sites are updated to use `MapMutationWriter`. No mutation site directly touches `current_map` without going through the abstraction.
3. Undo and redo (`apply_action_inverse`/`apply_action`) remain exempt from history re-push (they already manage the stack explicitly) but still call `mark_modified` and `render_events.write`.
4. `detect_map_changes` in `src/editor/renderer.rs` may be simplified or removed once all mutations reliably fire render events.
5. `cargo clippy` and `cargo test` pass with no regressions.

---

## Non-Functional Requirements

- The abstraction must not introduce a Bevy `Command` or deferred system unless strictly necessary; eager application is required for correct same-frame collision/render state.
- The abstraction should be usable from egui UI code (which is not a normal Bevy system) as well as from Bevy system functions.
- No new ECS components or resources may be introduced for this change alone; the abstraction may be a plain Rust struct or a `SystemParam`.

---

## Tasks

1. Design and implement `MapMutationWriter` (or a method `EditorState::apply_mutation`) encapsulating apply + history push + mark_modified.
2. Update `handle_gamepad_voxel_actions` to use the new abstraction.
3. Update all tool systems in `src/editor/tools/` to use the new abstraction.
4. Update the outliner's delete handler (fixing BUG-01 is a prerequisite or can be done in the same PR).
5. Verify that `detect_map_changes` can be simplified or removed.
6. Run `cargo test` and perform a full editor smoke-test (place/remove voxel, undo, redo, hot-reload).
