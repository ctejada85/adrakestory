# Requirements — Editor Entity Labels Phase 2

**Source:** `docs/features/editor-entity-labels-phase2/ticket.md` — 2026-04-08
**Status:** Draft

---

## 1. Overview

Phase 1 of the Editor NPC Name Labels feature (`docs/features/editor-npc-name-labels/`) added floating white text labels above NPC sphere markers in the editor viewport. Labels are always visible, plain text with no background, and limited to `EntityType::Npc`. On maps with many NPCs the labels create permanent visual noise with no way to hide them, and designers cannot identify any named entity other than NPCs from the viewport alone.

Phase 2 addresses these gaps in three areas:
1. **Visibility toggle** — a checkbox in the View menu and a button in the toolbar let the designer show or hide all entity labels in one action.
2. **All-entity label support** — labels are extended to all entity types that carry a `"name"` property (`Enemy`, `Item`, `Trigger`, `LightSource`) using the same world-to-screen projection and default-name suppression pattern established for NPCs. Each entity type receives a distinct label color.
3. **Styled pill** — each label is wrapped in a semi-transparent rounded frame, matching the visual style of the keyboard-mode indicator and selection-tooltip overlays already present in `viewport.rs`.

No changes to the RON map format, the game binary, or existing ECS components are required.

---

## 2. Functional Requirements

### 2.1 Label Visibility Toggle

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1.1 | The View menu (`src/editor/ui/toolbar/menus.rs`) must include a "Show Entity Labels" checkbox below the existing "Snap to Grid" entry. When unchecked, all floating entity labels are suppressed for the current frame and every subsequent frame until re-checked. | Phase 2 |
| FR-2.1.2 | The toolbar's view-toggles strip (`src/editor/ui/toolbar/controls.rs`) must include a "🏷" toggle button with tooltip `"Toggle entity labels"`. Its active state must mirror `EditorState::show_entity_labels`. | Phase 2 |
| FR-2.1.3 | Both the View menu checkbox and the toolbar toggle button must be bound to the same `EditorState::show_entity_labels: bool` field; changing one must be reflected by the other on the next frame. | Phase 2 |
| FR-2.1.4 | The default value of `show_entity_labels` must be `true` so that labels appear without the designer needing to opt in. | Phase 2 |
| FR-2.1.5 | The toggle state is session-only; it must not be persisted to disk or stored in the RON map file. | Phase 2 |

### 2.2 Enemy Entity Labels

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.2.1 | When `show_entity_labels` is `true`, the label system must render a floating label above each `EntityType::Enemy` entity whose `"name"` property is non-empty and not equal to the string `"Enemy"` (case-sensitive). | Phase 2 |
| FR-2.2.2 | No label is rendered for an Enemy whose `"name"` property is absent. | Phase 2 |
| FR-2.2.3 | No label is rendered for an Enemy whose `"name"` property is the empty string `""`. | Phase 2 |
| FR-2.2.4 | No label is rendered for an Enemy whose `"name"` property is the exact string `"Enemy"` (case-sensitive editor-defined placeholder; see Assumption 2). | Phase 2 |
| FR-2.2.5 | Enemy labels must be rendered in red — `egui::Color32::from_rgb(255, 100, 100)` — to distinguish them visually from other entity type labels. | Phase 2 |

### 2.3 Item Entity Labels

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.3.1 | When `show_entity_labels` is `true`, the label system must render a floating label above each `EntityType::Item` entity whose `"name"` property is non-empty and not equal to `"Item"` (case-sensitive). | Phase 2 |
| FR-2.3.2 | No label is rendered for an Item whose `"name"` property is absent. | Phase 2 |
| FR-2.3.3 | No label is rendered for an Item whose `"name"` property is the empty string `""`. | Phase 2 |
| FR-2.3.4 | No label is rendered for an Item whose `"name"` property is the exact string `"Item"` (case-sensitive editor-defined placeholder; see Assumption 2). | Phase 2 |
| FR-2.3.5 | Item labels must be rendered in gold — `egui::Color32::from_rgb(255, 215, 0)` — to distinguish them from other entity type labels. | Phase 2 |

### 2.4 Trigger Entity Labels

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.4.1 | When `show_entity_labels` is `true`, the label system must render a floating label above each `EntityType::Trigger` entity whose `"name"` property is non-empty and not equal to `"Trigger"` (case-sensitive). | Phase 2 |
| FR-2.4.2 | No label is rendered for a Trigger whose `"name"` property is absent, empty, or the exact string `"Trigger"`. | Phase 2 |
| FR-2.4.3 | Trigger labels must be rendered in cyan — `egui::Color32::from_rgb(100, 220, 220)` — to distinguish them from other entity type labels. | Phase 2 |

### 2.5 LightSource Entity Labels

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.5.1 | When `show_entity_labels` is `true`, the label system must render a floating label above each `EntityType::LightSource` entity whose `"name"` property is non-empty and not equal to `"LightSource"` (case-sensitive). | Phase 2 |
| FR-2.5.2 | No label is rendered for a LightSource whose `"name"` property is absent, empty, or the exact string `"LightSource"`. | Phase 2 |
| FR-2.5.3 | LightSource labels must be rendered in warm orange — `egui::Color32::from_rgb(255, 180, 50)` — to visually suggest a light source. | Phase 2 |

### 2.6 Background Pill Styling

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.6.1 | Every entity label must be wrapped in a semi-transparent rounded `egui::Frame` background pill. | Phase 2 |
| FR-2.6.2 | The pill must use a dark, semi-transparent fill (`egui::Color32::from_rgba_unmultiplied(30, 30, 30, 180)`) so the label is legible against both light and dark areas of the 3D scene. | Phase 2 |
| FR-2.6.3 | The pill corner radius must be `4.0` logical pixels, consistent with existing overlays in `viewport.rs`. | Phase 2 |
| FR-2.6.4 | The pill inner margin must be `4` vertical and `6` horizontal logical pixels. | Phase 2 |

### 2.7 NPC Label Continuity

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.7.1 | NPC labels must continue to display in white (`egui::Color32::WHITE`) with the same FiraMono font and 24px size as Phase 1. | Phase 2 |
| FR-2.7.2 | NPC default-name suppression (`"NPC"`, empty, absent) must be preserved without change. | Phase 2 |
| FR-2.7.3 | The `should_show_label(name, default)` helper must replace the Phase 1 `should_show_npc_label(name)` function; the original behavior must be covered by the new helper when called with `default = "NPC"`. | Phase 2 |

### 2.8 System Integration

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.8.1 | The Phase 1 `render_npc_name_labels` system must be renamed to `render_entity_name_labels` in `src/editor/ui/viewport.rs`, `src/editor/ui/mod.rs`, and `src/bin/map_editor/main.rs`. | Phase 2 |
| FR-2.8.2 | The system must continue to run after `render_ui` in the `Update` schedule, as required by the egui draw ordering constraint. | Phase 2 |
| FR-2.8.3 | Existing editor functionality — entity sphere markers, outliner, properties panel name editing, undo/redo — must not be affected by any changes in this phase. | Phase 2 |

---

## 3. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-3.1 | The toggle state must be stored in `EditorState::show_entity_labels: bool`; no new Bevy `Resource` types or ECS components may be introduced. | Phase 2 |
| NFR-3.2 | No additional Rust crates may be added; the implementation uses only `bevy_egui` already present in the editor. | Phase 2 |
| NFR-3.3 | The feature is editor-only; no changes to game binary systems, game components, or the RON map format are permitted. | Phase 2 |
| NFR-3.4 | Rendering labels for 20+ entities across all labeled types must not cause a measurable frame-time regression. Each label remains a single `egui::Area` with one `egui::Frame`-wrapped `ui.label()` call. | Phase 2 |
| NFR-3.5 | World-to-screen positions must continue to be projected each frame; screen-space coordinates must not be cached as persistent ECS state. | Phase 2 |

---

## 4. Phase Scoping

### Phase 1 — Completed

- Floating white NPC name labels in the editor viewport
- World-to-screen projection via `Camera::world_to_viewport`
- Default-name suppression for NPC (`"NPC"`, empty, absent)
- System `render_npc_name_labels` registered after `render_ui`
- FiraMono font loaded via `setup_egui_fonts`

### Phase 2 — This document

- `show_entity_labels` toggle in View menu and toolbar
- Enemy labels (red) with `"Enemy"` default suppression
- Item labels (gold) with `"Item"` default suppression
- Trigger labels (cyan) with `"Trigger"` default suppression
- LightSource labels (orange) with `"LightSource"` default suppression
- Background pill on all entity labels
- Rename system to `render_entity_name_labels`

### Phase 3 — Future

- Mouse hover tooltip showing full entity property summary (name, type, position, index)
- Click-through from viewport label to select the entity in the outliner and properties panel
- Configurable label font size via editor preferences

---

## 5. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | All entity types (`Enemy`, `Item`, `Trigger`, `LightSource`) are defined in `src/systems/game/map/format/entities.rs` and store metadata in the freeform `properties: HashMap<String, String>` field of `EntityData`. |
| 2 | Enemy and Item spawning is not yet implemented in the game binary (`src/systems/game/map/spawner/mod.rs:488–495` are TODO stubs). The suppression strings `"Enemy"` and `"Item"` are editor-defined conventions — not derived from game-side defaults — and must be kept in sync with game-side defaults when enemy/item spawning is eventually implemented. `Trigger` and `LightSource` follow the same convention using their variant names as placeholders. |
| 3 | `EditorEntityMarker::entity_index` continues to be a valid index into `EditorState::current_map.entities` at all times. |
| 4 | The View menu in `src/editor/ui/toolbar/menus.rs` already contains `show_grid` and `snap_to_grid` checkboxes; `show_entity_labels` follows the same `ui.checkbox(&mut editor_state.field, label)` pattern (lines 144–151). |
| 5 | The toolbar view-toggles strip in `src/editor/ui/toolbar/controls.rs` uses `ui.selectable_label(is_active, text)` for the grid and snap buttons; the labels toggle follows the same pattern. |
| 6 | `LABEL_Y_OFFSET`, `LABEL_FONT_SIZE`, and `FIRA_MONO_FAMILY` constants defined in Phase 1 are reused unchanged for all entity types. |

---

## 6. Open Questions

All questions resolved — no open items.

| # | Question | Resolution |
|---|----------|------------|
| 1 | What is the actual default `"name"` value written by the game's `spawn_enemy()` / `spawn_item()` functions? | Enemy and Item spawning is not yet implemented (TODO stubs in `src/systems/game/map/spawner/mod.rs:488–495`). No game-side default exists. Suppression strings `"Enemy"` / `"Item"` are editor-defined conventions. See Assumption 2. |
| 2 | Should `Trigger` and `LightSource` entities also receive labels in this phase? | Yes — all entity types with a `"name"` property receive labels in Phase 2. Trigger (cyan) and LightSource (orange) added. See §2.4 and §2.5. |

---

## 7. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | Phase 1 `render_npc_name_labels` system in `src/editor/ui/viewport.rs` | Done — committed `b773afd` | — |
| 2 | `setup_egui_fonts` with FiraMono loaded | Done — committed `96cbba1` | — |
| 3 | `EditorEntityMarker` component and `render_entities_system` | Done — `src/editor/renderer.rs:41–44` | — |
| 4 | `EntityType` variants (all six) | Done — `src/systems/game/map/format/entities.rs:27–40` | — |
| 5 | View menu pattern (`ui.checkbox`) and toolbar toggle pattern (`ui.selectable_label`) | Done — `src/editor/ui/toolbar/menus.rs:144–151`, `controls.rs:9–37` | — |

---

*Created: 2026-04-08*
*Updated: 2026-04-08 — Open questions resolved; scope extended to Trigger and LightSource*
*Source: `docs/features/editor-entity-labels-phase2/ticket.md`*
*Companion: `docs/features/editor-npc-name-labels/requirements.md` (Phase 1)*
