# Outliner Inline Rename — Phase 2 Enhancements

**Date:** 2026-04-08
**Component:** Map Editor — Outliner Panel (`src/editor/ui/outliner.rs`)

---

## Story

As a level designer, I want to trigger entity rename from the context menu or with F2 and have empty names cleanly remove the name property so that I can rename entities quickly from the keyboard and avoid leaving stale empty-string values in saved maps.

---

## Description

Phase 1 added double-click inline rename to the Outliner. Three gaps remain: there is no keyboard shortcut to enter rename mode for a selected entity, the context menu has no "Rename" item, and committing an empty name stores `""` in `properties["name"]` rather than removing the key. This ticket closes all three gaps. The implementation is confined to `src/editor/ui/outliner.rs` and reuses the Phase 1 rename entry path unchanged — Phase 2 adds only new activation routes and one post-commit cleanup step. No new Bevy Resources, no new `EditorAction` variants, and no changes to the game binary are needed.

---

## Acceptance Criteria

1. A "Rename" option appears in the context menu for every non-PlayerSpawn entity row. Clicking it enters inline rename mode for that entity; the renaming row is scrolled into view on the same frame via `response.scroll_to_me(None)` on the text input's response.
2. "Rename" does not appear in the context menu for `PlayerSpawn` entity rows.
3. Pressing F2 while exactly one non-PlayerSpawn entity is selected in the Outliner enters inline rename mode for that entity. If zero entities are selected, more than one is selected, or the selected entity is a `PlayerSpawn`, F2 has no effect.
4. F2 does not enter rename mode if a rename is already in progress.
5. Committing an empty name (pressing Enter or focus-lost with an empty text input) removes the `"name"` key from `properties` entirely; no empty string is stored in the saved map.
6. Committing a non-empty name continues to insert the name into `properties["name"]` exactly as Phase 1 does; the empty-name cleanup does not affect non-empty commits.
7. The undo entry for an empty-name commit (where old name was non-empty) reflects the removed key: after Ctrl+Z, `properties["name"]` is restored to its previous non-empty value.
8. All Phase 1 acceptance criteria remain satisfied: double-click activation, Enter/Escape/focus-lost commit and cancel, write-through, undo stack, auto-focus, PlayerSpawn exclusion, row visual continuity.
9. Unit tests cover: context menu Rename sets `renaming_index`; F2 enters rename for a single selected non-PlayerSpawn entity; F2 is a no-op when no entity is selected; F2 is a no-op when rename is already active; empty-name commit removes the `"name"` key; non-empty commit still stores the name.

---

## Non-Functional Requirements

- Must not add per-frame cost beyond a single `Option<usize>` check plus one `ui.input` call per panel frame when no rename is in progress.
- The scroll-to-row call (`response.scroll_to_me(None)`) must be made only on the first frame the text input is rendered after context-menu or F2 activation, not on every frame of an active rename.
- The empty-name cleanup must execute after the existing commit block (history push) so that the `ModifyEntity` snapshot already captures the final state before key removal.
- Applies to the `map_editor` binary only; no changes to the game binary (`adrakestory`).
- The F2 input check must use `ui.input(|i| i.key_pressed(egui::Key::F2))`, consistent with the existing Escape handling pattern in the same function.

---

## Tasks

1. Add a one-shot scroll flag to the rename entry path: introduce a mechanism (e.g., a second field `scroll_to_rename: bool` on `OutlinerState`, or reuse a temp-storage bool) that fires `response.scroll_to_me(None)` on the first frame the rename text input is rendered, then clears itself.
2. Add "Rename" to the entity row context menu in `render_entities_section`: inside `response.context_menu`, add a "Rename" button (non-PlayerSpawn guard) that saves the cancel snapshot and sets `outliner_state.renaming_index = Some(index)` and the scroll flag from Task 1.
3. Add F2 shortcut handling in `render_entities_section`: before (or after) the entity loop, read `ui.input(|i| i.key_pressed(egui::Key::F2))`; if `renaming_index` is `None` and exactly one non-PlayerSpawn entity is selected, save the cancel snapshot and set `renaming_index` to that entity's index.
4. Wire the scroll flag from Task 1 into the rename-mode row branch: when the flag is set, call `response.scroll_to_me(None)` on the text input response and clear the flag.
5. Add empty-name cleanup after the existing commit block in the `lost_focus` handler: if `editor_state.current_map.entities[index].properties.get("name").map(String::as_str).unwrap_or("").is_empty()`, call `properties.remove("name")` and `editor_state.mark_modified()`.
6. Write unit tests covering all new AC-9 scenarios.
7. Manual verification: (a) right-click → Rename scrolls the row into view and allows rename; (b) F2 on a selected entity enters rename; (c) committing an empty name removes the key from the saved map (inspect via map reload or Properties panel showing blank name field); (d) Ctrl+Z after an empty-name commit restores the previous name.

---

## Related

- Phase 1 ticket: `docs/features/outliner-inline-rename/ticket.md`
- Requirements: `docs/features/outliner-inline-rename/requirements.md` (§3.1 FR-3.1.4, FR-3.1.5; §3.3 FR-3.3.5; §5 Phase 2 scope)
- Architecture: `docs/features/outliner-inline-rename/architecture.md` (§2.9 Phase Boundaries; Appendix B Q5, Q6)
- Pattern reference: `src/editor/ui/properties/entity_props.rs` — `render_entity_name_field()` (write-through + snapshot)
- Coding Guardrail 12: `docs/developer-guide/coding-guardrails.md`
