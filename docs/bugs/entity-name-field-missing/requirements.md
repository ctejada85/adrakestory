# Requirements: Entity Name Field — Properties Panel Fix

**Date:** 2026-04-08
**Status:** Draft
**Component:** Map Editor — Properties Panel
**Related bug:** `docs/bugs/entity-name-field-missing/ticket.md`
**Related investigation:** `docs/investigations/2026-04-08-1605-entity-name-field-missing-properties-panel.md`

---

## Overview

The Properties panel currently exposes a "Name:" text field only for `Npc` entities. Phase 2 of the editor entity labels feature (commit `46ff5e3`) extended viewport floating labels to five entity types — Npc, Enemy, Item, Trigger, and LightSource — but did not update the Properties panel to expose name editing for the four newly label-capable types.

This fix makes the name field available for all entity types that can display viewport labels. It is a single-file change confined to `src/editor/ui/properties/entity_props.rs`, with no new data structures or systems required.

---

## Functional Requirements

### FR-1 — Name field visibility

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.1 | The Properties panel MUST render a "Name:" text field when any of the following entity types is selected: `Npc`, `Enemy`, `Item`, `Trigger`, `LightSource`. | Phase 1 |
| FR-1.2 | The Properties panel MUST NOT render a "Name:" text field for `PlayerSpawn` entities. | Phase 1 |
| FR-1.3 | The Name field MUST appear after the Position group and before any entity-type-specific properties. | Phase 1 |

### FR-2 — Name field behaviour

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1 | The Name field MUST be pre-populated with the current value of `entity_data.properties["name"]`, or empty if the key is absent. | Phase 1 |
| FR-2.2 | Changes MUST be committed to `entity_data.properties["name"]` only when the text field loses focus or the user presses Enter — not on every keystroke. | Phase 1 |
| FR-2.3 | A committed name change MUST push an `EditorAction::ModifyEntity` entry to `EditorHistory` so that the rename is undoable and redoable. | Phase 1 |
| FR-2.4 | If the user edits the field but restores the original value before committing, NO history entry MUST be created. (Guard: `new_name != current_name`.) | Phase 1 |
| FR-2.5 | A committed name change MUST call `editor_state.mark_modified()` to flag the map as having unsaved changes. | Phase 1 |

### FR-3 — NPC-specific properties

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1 | After extracting the Name field into a shared helper, `render_npc_properties` MUST continue to render the Radius slider unchanged. | Phase 1 |
| FR-3.2 | The NPC properties group label MUST remain "NPC Properties". | Phase 1 |

### FR-4 — LightSource-specific properties

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-4.1 | `render_light_source_properties` MUST remain unchanged. It renders Intensity, Range, Shadows, and Color — none of these are affected by this fix. | Phase 1 |

---

## Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-1 | The fix MUST NOT introduce any new `cargo clippy` warnings. | Phase 1 |
| NFR-2 | The fix MUST NOT change the Properties panel layout for `Npc` entities — the name field MUST continue to appear in the same visual position. | Phase 1 |
| NFR-3 | The fix MUST be confined to `src/editor/ui/properties/entity_props.rs`. No other source files require changes. | Phase 1 |
| NFR-4 | The shared name-field helper MUST be a private function (no `pub`) — it is an internal rendering detail. | Phase 1 |
| NFR-5 | Unit tests MUST cover the shared helper's commit-on-focus-lost logic and the no-op guard. These are logic tests on the helper's return conditions, not egui widget tests. | Phase 1 |

---

## Phase Scoping

### Phase 1 — This fix

- Extract `render_entity_name_field` helper from `render_npc_properties`.
- Call it in `render_single_entity_properties` for all non-PlayerSpawn entity types.
- Remove Name field rendering from `render_npc_properties`; leave Radius unchanged.
- Write unit tests for the helper's commit logic.

### Future

- Type-specific properties panels for Enemy (patrol radius, aggro range), Item (pickup type, value), and Trigger (trigger condition, action) — out of scope for this fix.
- Inline rename in the Outliner — out of scope for this fix.

---

## Assumptions and Constraints

1. `EntityData.properties` is a `HashMap<String, String>`. The key for the name is always the literal string `"name"`. This is consistent across `entity_props.rs`, `viewport.rs`, and `outliner.rs`.
2. The NPC default name placeholder `"NPC"` is meaningful to the label suppression logic in `viewport.rs:should_show_label`. The Name field must not change this default — it should pre-populate with the stored value or empty string, not a hardcoded placeholder.
3. `EditorAction::ModifyEntity` already exists in `src/editor/history.rs` and is the correct mechanism for entity property changes. No new action variant is needed.
4. `PlayerSpawn` entities intentionally have no viewport label and no name concept — confirmed in `viewport.rs:416` (`EntityType::PlayerSpawn => continue`).

---

## Dependencies

| # | Dependency | Status |
|---|-----------|--------|
| 1 | Phase 2 entity labels (viewport label support for Enemy/Item/Trigger/LightSource) | Done — commit `46ff5e3` |
| 2 | `EditorAction::ModifyEntity` in history | Done — `src/editor/history.rs` |

---

## Open Questions

None — all design decisions are resolved by the investigation and the existing NPC name field implementation.
