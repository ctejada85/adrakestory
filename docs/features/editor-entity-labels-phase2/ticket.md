# Editor Entity Labels — Phase 2

**Date:** 2026-04-08
**Component:** Map Editor / Viewport Overlays

---

## Story

As a level designer, I want entity name labels to be toggleable, visually distinct by entity type, and displayed for all entity types that carry a name property, so that I can quickly identify any named entity in a complex map without visual clutter when I don't need labels.

---

## Description

Phase 1 added floating NPC name labels to the editor viewport (`src/editor/ui/viewport.rs`). The labels are always visible, styled as plain white text, and limited to `EntityType::Npc`. Phase 2 enhances the feature in three ways: (1) adds a show/hide toggle in the View menu and toolbar so designers can disable labels on busy maps; (2) extends label rendering to all entity types that carry a `"name"` property — `Enemy`, `Item`, `Trigger`, and `LightSource` — each with a distinct color and editor-defined default-name suppression; and (3) wraps each label in a semi-transparent background pill for legibility against any 3D scene. Out of scope: hover tooltips, click-through selection, and configurable font size (planned for Phase 3). `PlayerSpawn` entities are not labelled (spawn point, not a named character).

---

## Acceptance Criteria

1. The View menu contains a "Show Entity Labels" checkbox that, when unchecked, hides all floating entity labels in the viewport; labels reappear when the checkbox is re-checked.
2. The toolbar's view-toggles row contains a "🏷" toggle button that mirrors the View menu checkbox state; toggling either control updates the other.
3. When `show_entity_labels` is `true`, named Enemy entities (non-empty, non-`"Enemy"` name) display a red label above their sphere marker.
4. When `show_entity_labels` is `true`, named Item entities (non-empty, non-`"Item"` name) display a gold label above their sphere marker.
5. When `show_entity_labels` is `true`, named Trigger entities (non-empty, non-`"Trigger"` name) display a cyan label above their marker.
6. When `show_entity_labels` is `true`, named LightSource entities (non-empty, non-`"LightSource"` name) display a warm orange label above their marker.
7. NPC labels remain white; all labels are wrapped in a semi-transparent dark rounded pill matching the style of other viewport overlays.
8. Toggling labels off and back on does not affect entity selection, outliner display, history undo/redo, or any other editor functionality.
9. Existing unit tests for `should_show_label` pass; new unit tests cover Enemy, Item, Trigger, and LightSource suppression logic.

---

## Non-Functional Requirements

- The label toggle state must be stored in `EditorState::show_entity_labels: bool` (default `true`); no new resources or ECS components may be introduced.
- No additional Rust crates may be added; the implementation must use only `bevy_egui` already present in the editor.
- The feature is editor-only; no changes to game binary systems, game components, or the RON map format are permitted.
- Rendering labels for 20+ entities across all labeled types must not cause a measurable frame-time regression.

---

## Tasks

1. Add `show_entity_labels: bool` field (default `true`) to `EditorState` in `src/editor/state.rs`.
2. Add "Show Entity Labels" checkbox to `render_view_menu` in `src/editor/ui/toolbar/menus.rs`, bound to `editor_state.show_entity_labels`.
3. Add a "🏷" toggle button to `render_view_toggles` in `src/editor/ui/toolbar/controls.rs`, bound to `editor_state.show_entity_labels`.
4. Rename `render_npc_name_labels` → `render_entity_name_labels` in `src/editor/ui/viewport.rs`; extend the `match` arm to handle `Enemy` (red / `"Enemy"`), `Item` (gold / `"Item"`), `Trigger` (cyan / `"Trigger"`), and `LightSource` (orange / `"LightSource"`).
5. Wrap each label in a styled `egui::Frame` (semi-transparent dark pill, `corner_radius(4.0)`, `inner_margin(4, 6)`) to match the other viewport overlays.
6. Replace `should_show_npc_label` with `should_show_label(name, default)` helper; update all unit tests and add cases for each new entity type.
7. Update `src/editor/ui/mod.rs` export from `render_npc_name_labels` → `render_entity_name_labels`.
8. Update `src/bin/map_editor/main.rs` system registration to use `render_entity_name_labels`.
9. Manually verify in the editor: toggle labels off — all labels disappear; toggle on — NPC (white), Enemy (red), Item (gold), Trigger (cyan), LightSource (orange) labels appear correctly; entities with default or absent names show no label.
