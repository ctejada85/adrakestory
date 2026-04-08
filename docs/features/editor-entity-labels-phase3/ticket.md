# Editor Entity Labels — Phase 3

**Date:** 2026-04-08
**Component:** Map Editor / Viewport Overlays

---

## Story

As a level designer, I want to hover over an entity label to see its full property summary and click it to select that entity in the outliner and properties panel, so that I can inspect and navigate to any named entity in the viewport without switching tools.

---

## Description

Phase 2 added floating name labels above all non-PlayerSpawn entity types with per-type colours and a show/hide toggle. The labels are purely decorative — hovering over them shows nothing and clicking has no effect. Phase 3 makes labels interactive in two ways: (1) a hover tooltip reveals the entity's type, world position, index, and type-specific properties (radius for NPC; intensity, range, colour, and shadows for LightSource); and (2) a click selects the entity — clearing other selections, inserting the index into `EditorState::selected_entities`, and scrolling the outliner to bring that entity's row into view on the next frame. Out of scope: configurable font size, multi-select via Shift+click, drag-to-move from label, and any game-binary changes.

---

## Acceptance Criteria

1. Hovering over any visible entity label for the standard egui tooltip delay shows a tooltip containing the entity's name, type, world position (formatted to two decimal places), and entity index.
2. For Npc entities, the tooltip additionally shows the `"radius"` property value if present in `EntityData::properties`.
3. For LightSource entities, the tooltip additionally shows `"intensity"`, `"range"`, `"color"`, and `"shadows"` property values, each only if present in `EntityData::properties`.
4. Clicking any entity label clears `EditorState::selected_entities` and inserts that entity's index; the active tool is not changed.
5. After clicking a label, the properties panel updates to show that entity's properties (this is automatic because the properties panel reads `selected_entities` every frame).
6. After clicking a label, the outliner scrolls to and highlights that entity's row within one frame.
7. When `show_entity_labels` is `false`, no labels are rendered and no tooltip or click interaction is possible for that frame.
8. Clicking a label does not modify `selected_voxels`, the undo/redo history, or any other editor state.
9. Unit tests cover: `entity_tooltip_lines` for an Npc entity with and without `"radius"`, for a LightSource with and without type-specific properties, and for a generic entity (Enemy); tests confirm lines contain expected substrings.

---

## Non-Functional Requirements

- The `outliner_scroll_to: Option<usize>` field must be added to `EditorState`; no new Bevy `Resource` types may be introduced.
- The scroll-to-selection mechanism must use egui's `Response::scroll_to_me(None)` on the outliner row response; no custom scroll-offset tracking is permitted.
- `render_entity_name_labels` must change its `Res<EditorState>` parameter to `ResMut<EditorState>`; the system's `.after(render_ui)` ordering constraint must be preserved.
- No additional Rust crates may be added.
- The feature is editor-only; no changes to game binary systems, game components, or the RON map format are permitted.
- Tooltip content generation must be extracted into a pure `entity_tooltip_lines` helper function to enable unit testing without Bevy world setup.

---

## Tasks

1. Add `pub outliner_scroll_to: Option<usize>` field (default `None`) to `EditorState` in `src/editor/state.rs`.
2. In `render_entity_name_labels` (`src/editor/ui/viewport.rs`): change `Res<EditorState>` → `ResMut<EditorState>`; after the `egui::Frame::show()` call inside each `egui::Area`, use `ui.interact(frame_response.response.rect, id, Sense::click())` to obtain a clickable/hoverable response.
3. On click of that response: `editor_state.selected_entities.clear(); editor_state.selected_entities.insert(index); editor_state.outliner_scroll_to = Some(index);`
4. On hover of that response: call `response.on_hover_ui(|ui| { for line in entity_tooltip_lines(entity_data, index) { ui.label(line); } })`.
5. Extract `fn entity_tooltip_lines(entity_data: &EntityData, entity_index: usize) -> Vec<String>` pure helper that returns the tooltip lines in order: name, type, position, index, then type-specific properties.
6. In `src/editor/ui/outliner.rs` `render_entities_section`: for each entity row that is selected, if `editor_state.outliner_scroll_to == Some(index)`, call `response.scroll_to_me(None)` on the `selectable_label` response and then set `editor_state.outliner_scroll_to = None`.
7. Write unit tests for `entity_tooltip_lines` (Npc with radius, Npc without radius, LightSource with properties, LightSource without, Enemy generic).
8. Manually verify in the editor: hover shows tooltip with correct content per entity type; click selects + scrolls outliner to the entity row; properties panel reflects the selection; toggling labels off hides labels and suppresses all interaction.
