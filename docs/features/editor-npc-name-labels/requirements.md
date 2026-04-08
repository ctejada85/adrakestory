# Requirements — Editor NPC Name Labels

**Source:** `docs/features/editor-npc-name-labels/ticket.md` — 2026-04-08
**Status:** Draft

---

## 1. Overview

The map editor represents NPC entities as anonymous blue spheres in the 3D viewport (`src/editor/renderer.rs:371`). While a designer can edit an NPC's `"name"` property in the properties panel (`src/editor/ui/properties/entity_props.rs:102–168`) and see names listed in the outliner (`src/editor/ui/outliner.rs:264–272`), the viewport itself gives no indication of which sphere belongs to which character.

This feature adds floating name labels above each NPC sphere marker in the editor viewport. Labels are rendered as egui overlays projected from world space to screen space and are updated automatically when the designer edits a name in the properties panel. No changes to the RON map format, the game binary, or existing editor data structures are required.

---

## 2. Functional Requirements

### 2.1 Label Rendering

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1.1 | Every frame, the editor viewport must display a text label above each NPC sphere marker whose `"name"` property is set to a non-empty, non-default value. | Phase 1 |
| FR-2.1.2 | Labels must be rendered as egui overlay elements (not as 3D mesh entities), consistent with the existing viewport overlay system in `src/editor/ui/viewport.rs`. | Phase 1 |
| FR-2.1.3 | Each label must be positioned by projecting the NPC's world-space position (plus a fixed Y offset above the sphere) to screen space using Bevy's `Camera::world_to_viewport` API. | Phase 1 |
| FR-2.1.4 | If `world_to_viewport` returns `None` for a given NPC (e.g., the NPC is behind the camera or outside the viewport), no label is drawn for that NPC that frame. | Phase 1 |

### 2.2 Label Content and Filtering

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.2.1 | The label text must be the exact value of the NPC's `"name"` property as stored in `EntityData::properties`. | Phase 1 |
| FR-2.2.2 | No label is rendered for an NPC whose `"name"` property is absent from `properties`. | Phase 1 |
| FR-2.2.3 | No label is rendered for an NPC whose `"name"` property is the empty string `""`. | Phase 1 |
| FR-2.2.4 | No label is rendered for an NPC whose `"name"` property is the exact string `"NPC"` (case-sensitive). This matches the default value applied by `spawn_npc()` at runtime and the filter used by the game-side `spawn_npc_label` system. | Phase 1 |

### 2.3 Label Synchronisation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.3.1 | When a designer commits a name change in the properties panel (focus lost on the text field), the viewport label for that NPC must reflect the new name on the next rendered frame. | Phase 1 |
| FR-2.3.2 | The label system must read NPC names directly from `EditorState::current_map.entities` every frame; no separate label state or cache may be introduced. | Phase 1 |

### 2.4 System Integration

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.4.1 | The label rendering must be implemented as a Bevy system registered in `src/bin/map_editor/main.rs`, consistent with how all other editor systems are registered. | Phase 1 |
| FR-2.4.2 | The new system must run after `render_ui` in the same `Update` schedule so that egui draw calls occur within the same frame as the rest of the UI. | Phase 1 |
| FR-2.4.3 | Existing editor functionality — sphere marker spawning (`render_entities_system`), outliner display, properties panel name editing, history undo/redo, radius editing — must not be affected. | Phase 1 |
| FR-2.4.4 | The feature is editor-only. No changes to game binary systems, game components (`Npc`, `NpcLabel`), or the RON map format are required or permitted by this ticket. | Phase 1 |

---

## 3. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-3.1 | The implementation must use the `bevy_egui` pipeline already present in the editor. No additional Rust crates may be added. | Phase 1 |
| NFR-3.2 | NPC world positions must be projected to screen space each frame using `Camera::world_to_viewport`. Screen-space coordinates must not be cached as persistent ECS state or resources. | Phase 1 |
| NFR-3.3 | The label rendering system must iterate only `EditorEntityMarker` entities tagged as NPC (resolved via index into `EditorState::current_map.entities`), not all ECS entities. | Phase 1 |
| NFR-3.4 | Drawing labels for 20 or more NPCs must not introduce a noticeable frame-time regression in the editor. Each label is a single `egui::Area` with one `ui.label()` call; no per-label mesh creation or asset allocation is performed. | Phase 1 |

---

## 4. Phase Scoping

### Phase 1 — MVP

- New `render_npc_name_labels` system in `src/editor/ui/viewport.rs`
- Projects NPC sphere world positions to screen space via `Camera::world_to_viewport`
- Renders egui label above each NPC with a non-default name
- Skips NPCs with absent, empty, or `"NPC"` names
- System registered in `src/bin/map_editor/main.rs` after `render_ui`

### Phase 2 — Enhanced

- Toggle in the editor toolbar or View menu to show/hide all NPC name labels
- Label style improvements: background pill, configurable font size, colour by entity type
- Show labels for other entity types (Enemy, Item) using their respective name properties

### Future Phases

- Mouse hover tooltip showing full NPC property summary (name, radius, index)
- Click-through from viewport label to select the NPC in the outliner and properties panel

---

## 5. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | `EditorEntityMarker::entity_index` always refers to a valid index in `EditorState::current_map.entities`. The label system may assume this invariant without bounds-checking; the existing renderer already relies on it. |
| 2 | Bevy 0.18 `Camera::world_to_viewport(&GlobalTransform, Vec3) -> Option<Vec2>` is available and returns `None` for positions behind the camera or outside the viewport frustum. |
| 3 | Exactly one `Camera3d` entity is active in the editor at any time (set up in `src/bin/map_editor/setup.rs`). The label system queries it with `Query<(&Camera, &GlobalTransform), With<Camera3d>>`. |
| 4 | The egui context passed to the label system is the same primary context used by the rest of `render_ui`. It is obtained from `EguiContexts::ctx_mut()`. |
| 5 | The label Y offset above the sphere top is `+0.8` world units (NPC sphere radius is `0.35`; offset target is the top of the sphere plus a small clearance). This value will be tuned during manual verification. |
| 6 | Name filtering is case-sensitive: only suppress the exact string `"NPC"`, consistent with `spawn_npc_label` in the game binary. |

---

## 6. Open Questions

All questions resolved — no open items.

---

## 7. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | `EditorEntityMarker` component and sphere spawning in `render_entities_system` | Done — `src/editor/renderer.rs:41–44, 362–416` | — |
| 2 | `render_viewport_overlays` egui overlay infrastructure | Done — `src/editor/ui/viewport.rs:35–91` | — |
| 3 | NPC name editing in properties panel (committed on focus-lost, stored in `EntityData::properties["name"]`) | Done — `src/editor/ui/properties/entity_props.rs:102–168` | — |
| 4 | Bevy 0.18 `Camera::world_to_viewport` API | Done — Bevy 0.18 in `Cargo.toml` | — |

---

*Created: 2026-04-08*
*Source: `docs/features/editor-npc-name-labels/ticket.md`*
*Companion: `docs/bugs/npc-display-names/requirements.md` (game-side NPC label requirements)*
