# Requirements — Editor Entity Labels Phase 3

**Source:** `docs/features/editor-entity-labels-phase3/ticket.md` — 2026-04-08
**Status:** Draft

---

## 1. Overview

Phase 2 of the Editor Entity Labels feature (`docs/features/editor-entity-labels-phase2/`) added floating, colour-coded name labels above all non-`PlayerSpawn` entity types with a show/hide toggle and a styled pill background. The labels are purely decorative: hovering or clicking them has no effect. Designers must still navigate to the outliner or use the Select tool's 3D ray-cast to identify and select a specific entity.

Phase 3 makes labels interactive. A hover tooltip reveals the entity's full property summary without requiring any tool switch, and a single click on a label selects the entity — updating the properties panel immediately and scrolling the outliner to the entity's row within one frame.

No changes to the RON map format, the game binary, or existing ECS components are required.

---

## 2. Functional Requirements

### 2.1 Hover Tooltip — Common Fields

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1.1 | When the pointer rests over any visible entity label for the standard egui tooltip delay, the editor must display a tooltip containing: the entity's `"name"` property, the `EntityType` variant name, the world position formatted as `(x.xx, y.xx, z.xx)`, and the entity index. | Phase 3 |
| FR-2.1.2 | The tooltip must be rendered using egui's built-in `Response::on_hover_ui` mechanism so that positioning, delay, and dismissal are handled by egui without custom Area management. | Phase 3 |
| FR-2.1.3 | The tooltip must appear above labels of all five labelled entity types: `Npc`, `Enemy`, `Item`, `Trigger`, and `LightSource`. | Phase 3 |
| FR-2.1.4 | The tooltip must not appear when `EditorState::show_entity_labels` is `false` (labels are hidden, so no hover surface exists). | Phase 3 |

### 2.2 Hover Tooltip — Type-Specific Properties

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.2.1 | For `Npc` entities, the tooltip must additionally show the `"radius"` property value if and only if `EntityData::properties` contains the key `"radius"`. | Phase 3 |
| FR-2.2.2 | For `LightSource` entities, the tooltip must additionally show each of `"intensity"`, `"range"`, `"color"`, and `"shadows"` if and only if the key is present in `EntityData::properties`. Absent keys are silently omitted. | Phase 3 |
| FR-2.2.3 | For `Enemy`, `Item`, and `Trigger` entities, no type-specific properties are shown beyond the common fields in FR-2.1.1. | Phase 3 |

### 2.3 Click-Through Entity Selection

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.3.1 | A left-click on any visible entity label must clear `EditorState::selected_entities` and insert that entity's index as the sole selection. | Phase 3 |
| FR-2.3.2 | The active tool must not be changed as a result of clicking a label. | Phase 3 |
| FR-2.3.3 | `EditorState::selected_voxels` must not be modified when clicking a label. | Phase 3 |
| FR-2.3.4 | The undo/redo history (`EditorHistory`) must not be affected by clicking a label; selection changes are not undoable operations. | Phase 3 |
| FR-2.3.5 | The properties panel, which reads `selected_entities` every frame, must reflect the new selection on the same frame as the click without any additional integration work. | Phase 3 |

### 2.4 Outliner Scroll-to-Selection

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.4.1 | After clicking a label, the outliner must scroll to bring that entity's row into view within one rendered frame. | Phase 3 |
| FR-2.4.2 | The scroll-to behavior must use egui's `Response::scroll_to_me(None)` called on the entity row's `selectable_label` response inside the outliner's `ScrollArea`. | Phase 3 |
| FR-2.4.3 | The scroll-to request must be communicated from the label system to the outliner via `EditorState::outliner_scroll_to: Option<usize>`. The label system sets this field to `Some(index)` on click; the outliner consumes it (setting it back to `None`) on the following frame. | Phase 3 |
| FR-2.4.4 | The outliner must only consume `outliner_scroll_to` when the stored index matches an entity row it is currently rendering as selected. If the index is out of range or no longer selected, the field must still be cleared to prevent stale scroll requests. | Phase 3 |
| FR-2.4.5 | Clicking an entity from within the outliner itself must not trigger a scroll-to-self loop; `outliner_scroll_to` is only written by the label click path, not by the outliner's own click handler. | Phase 3 |

### 2.5 Interaction Guard

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.5.1 | When `show_entity_labels` is `false`, no `egui::Area` is rendered for entity labels and no `ui.interact()` region is created; consequently no hover or click interaction is possible. | Phase 3 |

---

## 3. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-3.1 | `outliner_scroll_to: Option<usize>` must be added as a field on `EditorState`; no new Bevy `Resource` types or ECS components may be introduced to implement this communication channel. | Phase 3 |
| NFR-3.2 | Tooltip content generation must be extracted into a pure `entity_tooltip_lines(entity_data: &EntityData, entity_index: usize) -> Vec<String>` helper function so the logic can be covered by unit tests without a full Bevy `World`. | Phase 3 |
| NFR-3.3 | The `render_entity_name_labels` system must change its `Res<EditorState>` parameter to `ResMut<EditorState>` to enable selection and scroll-to mutation; the `.after(render_ui)` ordering must be preserved. | Phase 3 |
| NFR-3.4 | No additional Rust crates may be added; the implementation must use only `bevy_egui` already present in the editor. | Phase 3 |
| NFR-3.5 | The feature is editor-only; no changes to game binary systems, game components, or the RON map format are permitted. | Phase 3 |
| NFR-3.6 | The scroll-to mechanism introduces a one-frame delay between a label click (which sets `outliner_scroll_to`) and the outliner consuming it. This delay is acceptable and must not be worked around with same-frame hacks (e.g., calling outliner render logic from within the label system). | Phase 3 |

---

## 4. Phase Scoping

### Phase 1 — Completed

- Floating white NPC name labels in the editor viewport
- World-to-screen projection via `Camera::world_to_viewport`
- Default-name suppression for NPC (`"NPC"`, empty, absent)
- System `render_npc_name_labels` registered after `render_ui`
- FiraMono font loaded via `setup_egui_fonts`

### Phase 2 — Completed

- `show_entity_labels` toggle in View menu and toolbar
- Enemy (red), Item (gold), Trigger (cyan), LightSource (orange) labels
- Background pill on all entity labels
- Renamed system `render_entity_name_labels`
- Generalised `should_show_label(name, default)` helper

### Phase 3 — This document

- Hover tooltip (name, type, position, index, type-specific properties)
- Click-through label → select entity in `selected_entities`
- `outliner_scroll_to` communication field on `EditorState`
- Outliner scroll-to-selection via `Response::scroll_to_me`
- `entity_tooltip_lines` pure helper + unit tests

### Future Phases

- Multi-select via Shift+click on labels
- Drag label to move entity (translation via label drag)
- Keyboard shortcut to cycle through entities by name search

---

## 5. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | `EditorEntityMarker::entity_index` continues to be a valid index into `EditorState::current_map.entities` at all times. |
| 2 | The outliner's `render_entities_section` already receives `editor_state: &mut EditorState` as a parameter (confirmed at `src/editor/ui/outliner.rs:289`), so adding `outliner_scroll_to` to `EditorState` does not require a signature change to the outliner function. |
| 3 | The outliner uses a single `egui::ScrollArea::vertical()` wrapping all content (`src/editor/ui/outliner.rs:92`). There is currently no scroll-to-selection mechanism; this phase introduces it for the first time, scoped to label-click-driven selections only. |
| 4 | `render_entity_name_labels` runs after `render_ui` in the same `Update` schedule. Because the outliner renders inside `render_ui`, a label click in frame N sets `outliner_scroll_to`; the outliner consumes it in frame N+1. This one-frame delay is imperceptible to users. |
| 5 | The properties panel reads `EditorState::selected_entities` every frame via `.iter().next()` (`src/editor/ui/properties/entity_props.rs:15`). No additional work is needed to make it reflect label-click selections. |
| 6 | `egui::Response::on_hover_ui` uses egui's built-in tooltip delay and positioning. The exact delay is determined by egui's style (`style.interaction.tooltip_delay`) and is not configurable in this phase. |
| 7 | `Sense::click()` in egui implies hover sensing; a response created with `ui.interact(rect, id, Sense::click())` correctly reports both `hovered()` and `clicked()`. |

---

## 6. Open Questions

All questions resolved — no open items.

| # | Question | Resolution |
|---|----------|------------|
| 1 | Should Shift+click on a label add to the selection rather than replace it? | Out of scope for Phase 3. Single-click replaces all selections (consistent with the Select tool and outliner single-click behavior). Multi-select via labels is a future phase item. |
| 2 | Should clicking a label switch the active tool to Select? | No. The label is a UI overlay, not a 3D viewport click; switching tools would be disruptive. Selection via label click is tool-agnostic. |
| 3 | Should the outliner scroll immediately (same frame) or on the next frame? | Next frame, via `outliner_scroll_to` on `EditorState`. Same-frame scroll would require calling outliner render logic from inside the label system, coupling two independent systems. One-frame delay is imperceptible. |
| 4 | Does the outliner currently have a scroll-to-selection mechanism? | No — confirmed by codebase search. Phase 3 introduces it for the first time, scoped to label-click-driven selections. |

---

## 7. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | Phase 2 `render_entity_name_labels` system in `src/editor/ui/viewport.rs` | Done — committed `46ff5e3` | — |
| 2 | `EditorEntityMarker` component and `render_entities_system` | Done — `src/editor/renderer.rs:41–44` | — |
| 3 | `EntityData` struct with `entity_type`, `position`, and `properties` fields | Done — `src/systems/game/map/format/entities.rs:7–23` | — |
| 4 | Outliner `render_entities_section` receiving `&mut EditorState` | Done — `src/editor/ui/outliner.rs:289` | — |
| 5 | Properties panel reading `selected_entities` every frame | Done — `src/editor/ui/properties/entity_props.rs:15` | — |

---

*Created: 2026-04-08*
*Source: `docs/features/editor-entity-labels-phase3/ticket.md`*
*Companion: `docs/features/editor-entity-labels-phase2/requirements.md` (Phase 2)*
