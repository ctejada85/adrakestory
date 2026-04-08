# Requirements — Outliner Inline Rename

**Source:** Feature discussion — 2026-04-08
**Status:** Draft

---

## 1. Overview

The Outliner panel currently renders every entity row as a read-only selectable label. A level designer who wants to set or change an entity's name must first select the entity in the Outliner, then find and edit the "Name:" field in the Properties panel on the right side of the screen. This two-panel round-trip is unnecessary for a common, lightweight operation.

Inline rename adds a direct rename affordance to the Outliner: double-clicking a non-PlayerSpawn entity row switches its label into a text input. The name commits on Enter or focus-lost and cancels on Escape, integrating cleanly into the existing undo/redo stack. The Properties panel's Name field is unaffected and continues to work independently; both widgets write to the same underlying storage (`EntityData.properties["name"]`), so they are always in sync.

---

## 2. Data Domains

Entity names are stored as an optional string property — not a dedicated struct field.

| Domain | Description | Where stored |
|--------|-------------|--------------|
| **Entity name** | A human-readable label shown in the Outliner and as a floating viewport label above the entity. Optional: absence means the entity has no label. | `EntityData.properties["name"]` (key `"name"` in the `HashMap<String, String>`) |
| **Entity type** | The structural type of the entity (`Npc`, `Enemy`, `Item`, `Trigger`, `LightSource`, `PlayerSpawn`). Read-only — cannot be changed after placement. | `EntityData.entity_type: EntityType` |

`PlayerSpawn` entities have no name concept and must be excluded from the rename feature.

---

## 3. Functional Requirements

### 3.1 Activation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1.1 | Double-clicking a non-PlayerSpawn entity row in the Outliner must enter inline rename mode for that entity. | Phase 1 |
| FR-3.1.2 | Double-clicking a `PlayerSpawn` entity row must not enter rename mode. | Phase 1 |
| FR-3.1.3 | Only one entity may be in rename mode at a time. If entity A is in rename mode and the user double-clicks entity B, entity A must be committed first (same as focus-lost) before entity B enters rename mode. | Phase 1 |
| FR-3.1.4 | A "Rename" option in the entity row context menu must also enter inline rename mode for that entity. | Phase 2 |
| FR-3.1.5 | Pressing F2 while an entity is selected in the Outliner must enter inline rename mode for that entity. | Phase 2 |

### 3.2 Rename UI

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.2.1 | When rename mode is entered, the entity row label must be replaced with a text input pre-filled with the entity's current name. If the entity has no name (`properties["name"]` absent), the text input must be pre-filled with an empty string, not the entity type name. | Phase 1 |
| FR-3.2.2 | The text input must receive keyboard focus automatically on the first frame it appears. | Phase 1 |
| FR-3.2.3 | While the text input is active, the row must not respond to single-click selection events. | Phase 1 |

### 3.3 Commit

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.3.1 | Pressing Enter while the text input has focus must commit the rename and exit rename mode. | Phase 1 |
| FR-3.3.2 | The text input losing focus (clicking elsewhere in the editor) must commit the rename and exit rename mode. | Phase 1 |
| FR-3.3.3 | After commit, if the name changed from its value at rename-mode entry, exactly one `EditorAction::ModifyEntity` entry must be pushed onto the undo stack. | Phase 1 |
| FR-3.3.4 | After commit, if the name did not change, no history entry must be pushed. | Phase 1 |
| FR-3.3.5 | A name set to an empty string is valid and must be stored as an absent key (remove `"name"` from `properties`) rather than storing an empty string. | Phase 2 |

### 3.4 Cancel

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.4.1 | Pressing Escape while the text input has focus must cancel the rename: restore the entity's name to the value it had when rename mode was entered, exit rename mode, and push no history entry. | Phase 1 |
| FR-3.4.2 | If the entity was deleted from the map while rename mode was open, rename mode must be exited silently (no panic, no history entry, no restoration attempt). | Phase 1 |

### 3.5 Real-time sync

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.5.1 | Every keystroke during rename must write through to `EntityData.properties["name"]` immediately (write-through, not buffer-and-commit). The viewport label above the entity and the Properties panel Name field must both reflect the in-progress name on the same frame. | Phase 1 |

### 3.6 Compatibility

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.6.1 | The Outliner's existing single-click selection, context menu ("Delete"), scroll-to-entity, hover tooltip, and filter behaviors must be unaffected by this feature. | Phase 1 |
| FR-3.6.2 | The Properties panel "Name:" field must continue to function correctly and remain in sync with names set via the Outliner. | Phase 1 |

---

## 4. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-4.1 | The rename state must be stored in `OutlinerState` as `renaming_index: Option<usize>`. It must not be added to `EditorState` or registered as a separate Bevy `Resource`. | Phase 1 |
| NFR-4.2 | Per-frame cost must not increase beyond a single `Option<usize>` comparison per rendered entity row when no rename is in progress. | Phase 1 |
| NFR-4.3 | The egui temp-storage snapshot key for the undo entry must be distinct from the Properties panel snapshot key (`"entity_name_snapshot"`) to prevent cross-panel collision when both panels are open and the same entity is visible in both. | Phase 1 |
| NFR-4.4 | The feature must apply to the `map_editor` binary only. The game binary (`adrakestory`) must not be modified. | Phase 1 |
| NFR-4.5 | The write-through + snapshot pattern (Coding Guardrail 12) must be followed. Rebuilding the name from stored state each frame without write-through is not permitted. | Phase 1 |

---

## 5. Phase Scoping

### Phase 1 — MVP

- Double-click activation on non-PlayerSpawn entity rows
- Auto-focused text input pre-filled with current name (or empty)
- Write-through on every keystroke; real-time viewport label sync
- Commit on Enter or focus-lost; one undo entry per session
- Cancel on Escape; name restored, no history entry
- Graceful exit if entity is deleted mid-rename
- PlayerSpawn excluded from rename affordance
- All existing Outliner behaviors unaffected

### Phase 2 — Enhanced

- "Rename" item in the entity row context menu
- F2 shortcut to enter rename mode for the selected entity
- Empty-string commit removes the `"name"` key from `properties` entirely (rather than storing `""`)

### Future Phases

- Inline rename for named voxel groups (if named groups are added to the map format)
- Bulk rename with pattern substitution

---

## 6. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | Entity names are stored as `EntityData.properties["name"]` — a string value in a flat `HashMap<String, String>`. There is no dedicated `name` field on the struct. |
| 2 | `PlayerSpawn` is the only entity type excluded from inline rename. All other five types (`Npc`, `Enemy`, `Item`, `Trigger`, `LightSource`) support it. |
| 3 | The egui version in use is 0.33.3 (via `bevy_egui` 0.39.1). The implementation relies on `response.double_clicked()`, `response.request_focus()`, `response.lost_focus()`, `ui.data_mut()` / `get_temp` / `remove`, and `ui.input(|i| i.key_pressed(...))` — all available in this version. |
| 4 | `OutlinerState` is a Bevy `Resource` passed by mutable reference to `render_outliner_panel`. Adding a field requires no registration change. |
| 5 | Rename mode is per-session (not persisted). Reloading the map clears `renaming_index`. |
| 6 | The undo entry uses `EditorAction::ModifyEntity { index, old_data, new_data }` — the full `EntityData` clone pattern already in use for the Properties panel Name field. No new `EditorAction` variant is needed. |

---

## 7. Open Questions

| # | Question | Owner |
|---|----------|-------|
| 1 | Should empty-name commit remove the `"name"` key from `properties` (cleaner map file) or store `""`? Phase 2 targets removal, but the Phase 1 behavior (store `""`) is simpler and avoids a special case in commit logic. | Team |
| 2 | Should the "Rename" context menu item (Phase 2) scroll the Outliner to the renaming row if it is currently scrolled out of view? | Team |

---

## 8. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | `render_entity_name_field()` in `entity_props.rs` — establishes the canonical write-through + snapshot pattern that this feature replicates | Done (commit `e6b99c8`) | — |
| 2 | Coding Guardrail 12 documented in `docs/developer-guide/coding-guardrails.md` | Done (commit `a850c2c`) | — |

---

*Created: 2026-04-08*
*Companion documents: [Ticket](./ticket.md) | [Architecture](./architecture.md)*
