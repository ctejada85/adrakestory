# Bug: Save-As Clears Dirty Flag for Edits Made While Dialog Was Open

**Date:** 2026-04-09
**Severity:** Low
**Component:** Editor — File I/O (`src/editor/file_io/mod.rs`)

---

## Story

As a level designer, I want the "unsaved changes" indicator to stay accurate after using Save As so that I am never misled into thinking my map is saved when it has pending edits.

---

## Description

The Save-As flow opens a native file dialog in a background thread (`handle_save_map_as`, line 64 of `src/editor/file_io/mod.rs`). Each frame, `check_save_dialog_result` polls the channel for a chosen path. When a path arrives, it snapshots `editor_state.current_map` and writes it to disk, then emits `FileSavedEvent`. On the same or following frame, `handle_file_saved` calls `editor_state.clear_modified()`.

Because `check_save_dialog_result` takes `editor_state: Res<EditorState>` (immutable) and `handle_file_saved` runs later, any mutation that occurs between these two systems in the same frame will be included on disk but then have its dirty flag cleared. More practically, any edit made while the dialog was open and committed to `current_map` will be saved, but if the dialog delivery and another mutation land in the same Bevy schedule slot, the cleared flag will not reflect those edits. The result is `is_modified = false` while the in-memory state differs from the file on disk.

The fix is to record a content-hash or deep-equality snapshot of the map at the moment the file is written and compare it to `current_map` before clearing the flag, or to ensure `clear_modified` is only called when the frame-stable map matches what was written.

---

## Acceptance Criteria

1. After Save As completes, `EditorState::is_modified` is `false` only if the saved snapshot matches the current in-memory map.
2. If the user makes an edit immediately after Save As (within the same frame as the `FileSavedEvent`), `is_modified` remains `true`.
3. No regression in the normal (no concurrent edit) Save As path: saving a clean map still clears the flag.

---

## Non-Functional Requirements

- The snapshot comparison must not allocate on every frame; it should only run on the frame when `FileSavedEvent` arrives.
- The change must not introduce a second full serialization pass on save.
- Scope is limited to `src/editor/file_io/mod.rs` and `src/editor/state.rs`.

---

## Tasks

1. Extend `FileSavedEvent` to carry a content hash or a version counter that was current at write time.
2. In `handle_file_saved`, compare the carried hash/counter against the current map state before calling `clear_modified()`.
3. Add a test that simulates a same-frame mutation and verifies `is_modified` is not incorrectly cleared.
4. Manual verification: open a map, trigger Save As, save to a new file, immediately type a character in the map name field, confirm the title bar still shows "modified".
