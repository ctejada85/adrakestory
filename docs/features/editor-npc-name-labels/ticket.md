# Editor NPC Name Labels

**Date:** 2026-04-08
**Component:** Map Editor / NPC

---

## Story

As a level designer, I want to see each NPC's name displayed above its sphere marker in the editor viewport so that I can identify and work with individual NPCs without having to select them one at a time.

---

## Description

The map editor currently represents NPC entities as anonymous blue spheres in the 3D viewport. While the properties panel allows editing an NPC's `"name"` property (`src/editor/ui/properties/entity_props.rs:102–168`) and the outliner lists NPC names (`src/editor/ui/outliner.rs:264–272`), the viewport itself gives no visual indication of which sphere corresponds to which character. This ticket adds a floating name label above each NPC sphere in the editor viewport, rendered as an egui overlay projected from world space to screen space. The label stays in sync when the name is edited in the properties panel. Out of scope: game-side `Text2d` labels (covered by `docs/bugs/npc-display-names/ticket.md`), custom fonts, and animated fade effects.

---

## Acceptance Criteria

1. When the editor viewport is open and the map contains at least one NPC with a non-default name, a text label showing that NPC's name is visible above its sphere marker.
2. NPCs whose `"name"` property is absent, empty, or the exact string `"NPC"` (case-sensitive) do not display a label.
3. When a designer edits an NPC's name in the properties panel and the change is committed (focus lost), the viewport label updates to reflect the new name on the next rendered frame.
4. Labels are positioned consistently above all NPC spheres regardless of the number of NPCs on screen.
5. Existing editor functionality — selection highlight, outliner display, history undo/redo for name edits, radius editing — is unaffected.

---

## Non-Functional Requirements

- Labels must use the existing `bevy_egui` pipeline already present in the editor; no additional crates may be added.
- The label rendering must project world-space NPC positions to screen space each frame; it must not store screen-space coordinates as persistent state.
- The feature is editor-only (`src/bin/map_editor.rs` / `src/editor/`); no changes to game binary systems, game components, or the RON map format are required.
- Adding labels for 20+ NPCs must not cause a measurable frame-time regression in the editor (labels are a simple egui text draw call per NPC, not mesh entities).

---

## Tasks

1. Add a helper function `world_to_screen(camera: &Camera, camera_transform: &GlobalTransform, world_pos: Vec3) -> Option<egui::Pos2>` in `src/editor/ui/viewport.rs` (or an appropriate shared util module) that projects a world position to egui screen coordinates.
2. In the editor's viewport rendering system (the egui pass that already draws the 3D scene), iterate `EditorEntityMarker` entities whose corresponding `EntityData` has `entity_type == EntityType::Npc`, retrieve their world `Transform`, and call `world_to_screen` to obtain label positions.
3. For each NPC with a non-empty, non-`"NPC"` name, draw an `egui::Label` (or `ui.label()`) at the projected screen position, offset upward by a fixed amount to clear the sphere marker.
4. Ensure the label draw calls are skipped when `world_to_screen` returns `None` (NPC behind camera or outside viewport).
5. Manually verify in the editor: open `assets/maps/default.ron`, confirm the "Village Elder" NPC at `(9.5, 1.0, 5.5)` shows a label, confirm the two anonymous NPCs at `(9.0, 1.0, 8.0)` and `(12.0, 1.0, 6.0)` show no label, and confirm editing the name in the properties panel updates the label immediately.
