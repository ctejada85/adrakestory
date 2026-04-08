# Inline Rename in the Outliner

**Date:** 2026-04-08
**Component:** Map Editor — Outliner Panel (`src/editor/ui/outliner.rs`)

---

## Story

As a level designer, I want to rename entities directly in the Outliner by double-clicking their row so that I can name entities without having to open the Properties panel.

---

## Description

Entity rows in the Outliner are currently read-only `selectable_label` widgets — clicking selects the entity, but there is no way to set or change the entity's name from the Outliner itself. The only rename path today is selecting the entity and editing the "Name:" field in the Properties panel. This ticket adds inline rename: double-clicking a non-PlayerSpawn entity row switches it to a text field, and the name is committed on Enter or focus-lost, or cancelled on Escape. PlayerSpawn entities are excluded because they have no name concept.

---

## Acceptance Criteria

1. Double-clicking a non-PlayerSpawn entity row in the Outliner replaces the row label with a focused text input pre-filled with the entity's current name (or empty string if no name is set).
2. While the text input is active, typing updates the entity's `properties["name"]` in real time; the viewport label above the entity reflects the new name in the same frame.
3. Pressing Enter or clicking outside the text input commits the rename. The committed name can be undone with Ctrl+Z in one step (one `ModifyEntity` history entry per rename session, not one per keystroke).
4. Pressing Escape cancels the rename; the entity's name reverts to the value it had when the double-click was initiated, and no history entry is pushed.
5. Double-clicking a `PlayerSpawn` entity row does not enter rename mode.
6. If the entity has no name (`properties["name"]` is absent), the text input is pre-filled with an empty string, not the entity type name.
7. Only one entity can be in rename mode at a time. Double-clicking a second entity while another is being renamed commits the first rename (equivalent to focus-lost) before entering rename mode for the second.
8. The Outliner's existing selection, scroll-to, context menu, and filter behaviors are unaffected.
9. The Properties panel "Name:" field remains fully functional and stays in sync: a name edited in the Outliner is immediately visible in the Properties panel on the same frame, and vice versa.

---

## Non-Functional Requirements

- Must not add per-frame cost beyond a single `Option<usize>` check per entity row.
- The inline rename state (`renaming_index`) must live in `OutlinerState` — not in `EditorState` or a new Bevy `Resource`.
- The undo snapshot must use egui temp storage keyed by entity index (Coding Guardrail 12 pattern). The key must be distinct from the Properties panel snapshot key to prevent cross-panel collisions.
- Applies to `map_editor` binary only; no changes to the game binary.

---

## Tasks

1. Add `renaming_index: Option<usize>` field to `OutlinerState` in `src/editor/ui/outliner.rs`.
2. In `render_entities_section`, detect `response.double_clicked()` on the `selectable_label` for non-PlayerSpawn entities; set `outliner_state.renaming_index = Some(index)` and save the `EntityData` snapshot to egui temp storage.
3. When `renaming_index == Some(index)`, replace the `selectable_label` with a `text_edit_singleline`; request focus on the first frame using `response.request_focus()`.
4. Implement write-through on `response.changed()`: insert updated name into `entity_data.properties["name"]` and call `editor_state.mark_modified()`.
5. Implement commit on `response.lost_focus()`: retrieve and remove the snapshot; if name changed, push `EditorAction::ModifyEntity`; clear `renaming_index`.
6. Implement cancel on Escape (`ui.input(|i| i.key_pressed(egui::Key::Escape))`): restore the snapshotted name, clear snapshot from temp storage, clear `renaming_index`; do not push a history entry.
7. Clear `renaming_index` when the renaming entity's index is no longer valid (entity deleted while rename is open).
8. Manual verification: double-click rename, commit with Enter, undo with Ctrl+Z, verify name reverts; repeat with Escape cancel.

---

## Related

- Requirements: `docs/features/outliner-inline-rename/requirements.md`
- Architecture: `docs/features/outliner-inline-rename/architecture.md`
- Pattern reference: `src/editor/ui/properties/entity_props.rs` — `render_entity_name_field()` (write-through + snapshot)
- Coding Guardrail 12: `docs/developer-guide/coding-guardrails.md`
