# Architecture — Editor Entity Labels Phase 2

**Feature:** Editor Entity Labels Phase 2
**Date:** 2026-04-08
**Status:** Draft

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| v1 | 2026-04-08 | — | Initial draft — derived from Phase 1 implementation and codebase exploration |
| **v2** | **2026-04-08** | — | **Open questions resolved: suppression strings are editor-defined conventions (Enemy/Item spawning not yet implemented); scope extended to Trigger (cyan) and LightSource (orange)** |

---

## Table of Contents

1. [Current Architecture](#1-current-architecture)
2. [Target Architecture](#2-target-architecture)
3. [Appendices](#appendices)

---

## 1. Current Architecture

### 1.1 Relevant Files

| File | Purpose |
|------|---------|
| `src/editor/ui/viewport.rs` | Viewport overlay system; contains `render_npc_name_labels`, `should_show_npc_label`, `FIRA_MONO_FAMILY`, `LABEL_Y_OFFSET`, `LABEL_FONT_SIZE` |
| `src/editor/ui/mod.rs` | Re-exports `render_npc_name_labels`, `FIRA_MONO_FAMILY`, `render_viewport_overlays` |
| `src/editor/ui/toolbar/menus.rs` | View menu with `show_grid` and `snap_to_grid` checkboxes |
| `src/editor/ui/toolbar/controls.rs` | Toolbar view-toggles strip with grid and snap buttons |
| `src/editor/state.rs` | `EditorState` resource — no label visibility field |
| `src/bin/map_editor/main.rs` | System registration: `render_npc_name_labels.after(render_ui)` |
| `src/bin/map_editor/setup.rs` | `setup_egui_fonts` — loads FiraMono into egui |
| `src/systems/game/map/format/entities.rs` | `EntityType` enum (`Npc`, `Enemy`, `Item`, …), `EntityData` struct |
| `src/editor/renderer.rs` | `EditorEntityMarker { entity_index }` — spawned for each entity |

### 1.2 Phase 1 Label System — Current Flow

```mermaid
flowchart TD
    A["render_entity_name_labels system (Update, after render_ui)"]
    B["EguiContexts::ctx_mut() → Ok(ctx)"]
    C["Single<(&Camera, &GlobalTransform), With<Camera3d>>"]
    D["Query<(&EditorEntityMarker, &GlobalTransform)> — all markers"]
    E{entity_type == Npc?}
    F{should_show_npc_label(name)?}
    G["camera.world_to_viewport → Ok(screen_pos)"]
    H["egui::Area plain white ui.label()"]
    I[skip]

    A --> B --> C
    A --> D
    D --> E
    E -->|No| I
    E -->|Yes| F
    F -->|No| I
    F -->|Yes| G
    G -->|Err| I
    G -->|Ok| H
```

### 1.3 Current `EditorState` (relevant fields)

```rust
// src/editor/state.rs:33
pub struct EditorState {
    pub current_map: MapData,
    pub show_grid:   bool,   // default true
    pub grid_opacity: f32,   // default 0.3
    pub snap_to_grid: bool,  // default true
    // …no show_entity_labels field
}
```

### 1.4 Current View Menu (menus.rs:141–163)

```rust
fn render_view_menu(…, editor_state: &mut EditorState, …) {
    ui.checkbox(&mut editor_state.show_grid,     "▦ Show Grid");
    ui.checkbox(&mut editor_state.snap_to_grid,  "⊞ Snap to Grid");
    ui.separator();
    ui.label("Grid Opacity");
    ui.add(egui::Slider::new(&mut editor_state.grid_opacity, 0.0..=1.0));
}
```

### 1.5 Current Toolbar View Toggles (controls.rs:9–37)

```rust
fn render_view_toggles(…, editor_state: &mut EditorState) {
    // Grid toggle — ui.selectable_label(is_active, icon)
    // Snap toggle — ui.selectable_label(is_active, icon)
}
```

---

## 2. Target Architecture

### 2.1 Design Principles

1. **Minimal footprint** — add one field to `EditorState`, extend one system, and touch three UI files. No new resources, components, or crates.
2. **Consistent patterns** — follow the existing `ui.checkbox` pattern for the View menu and `ui.selectable_label` pattern for the toolbar, matching `show_grid` / `snap_to_grid`.
3. **Single system** — replace `render_npc_name_labels` with a generalised `render_entity_name_labels` rather than adding a second system per entity type.
4. **Per-type color table** — entity-type-to-color mapping is an inline `match` arm inside the system; no separate resource or config struct is needed at this scope.
5. **No breaking changes** — rename is the only API change; the new function signature is drop-in compatible with the existing `.after(render_ui)` registration.

### 2.2 New / Modified Components

| Component | File | Change |
|-----------|------|--------|
| `EditorState` | `src/editor/state.rs` | Add `pub show_entity_labels: bool` field, default `true` |
| `render_view_menu` | `src/editor/ui/toolbar/menus.rs` | Add `ui.checkbox(&mut editor_state.show_entity_labels, "🏷 Show Entity Labels")` below Snap to Grid |
| `render_view_toggles` | `src/editor/ui/toolbar/controls.rs` | Add `ui.selectable_label` button for `show_entity_labels` |
| `render_npc_name_labels` → `render_entity_name_labels` | `src/editor/ui/viewport.rs` | Rename; add toggle guard; extend to Enemy+Item; add pill frame; generalise suppression helper |
| `should_show_npc_label` → `should_show_label` | `src/editor/ui/viewport.rs` | Add `default: &str` parameter; keep existing NPC behavior as special case |
| `src/editor/ui/mod.rs` | mod.rs | Update re-export name |
| `src/bin/map_editor/main.rs` | main.rs | Update system registration name |

### 2.3 `EditorState` Change

```rust
// src/editor/state.rs — add to EditorState struct
pub show_entity_labels: bool,  // default: true
```

```rust
// src/editor/state.rs — add to EditorState::default()
show_entity_labels: true,
```

### 2.4 View Menu Change

```rust
// src/editor/ui/toolbar/menus.rs — inside render_view_menu, after snap_to_grid
ui.checkbox(&mut editor_state.snap_to_grid,       "⊞ Snap to Grid");
ui.checkbox(&mut editor_state.show_entity_labels, "🏷 Show Entity Labels");
ui.separator();
```

### 2.5 Toolbar Toggle Change

```rust
// src/editor/ui/toolbar/controls.rs — inside render_view_toggles
let labels_icon = if editor_state.show_entity_labels { "🏷" } else { "🏷" };
let labels_btn = egui::Button::new(labels_icon).min_size(egui::vec2(28.0, 24.0));
let labels_resp = ui.add(labels_btn.fill(if editor_state.show_entity_labels {
    egui::Color32::from_rgb(70, 100, 150)
} else {
    ui.visuals().widgets.inactive.bg_fill
}));
if labels_resp.clicked() {
    editor_state.show_entity_labels = !editor_state.show_entity_labels;
}
labels_resp.on_hover_text("Toggle entity labels");
```

### 2.6 Generalised Label System — Target Flow

```mermaid
flowchart TD
    A["render_entity_name_labels system"]
    B["ctx_mut() → Ok(ctx)"]
    C[Single camera query]
    D["Query all EditorEntityMarker + GlobalTransform"]
    G{show_entity_labels?}
    E{entity_type ∈ Npc | Enemy | Item?}
    F{should_show_label(name, default)?}
    H["world_to_viewport → Ok(screen_pos)"]
    I["egui::Area → egui::Frame pill → colored ui.label()"]
    J[return early]
    K[skip]

    A --> B --> G
    G -->|false| J
    G -->|true| C
    A --> D
    D --> E
    E -->|No| K
    E -->|Yes| F
    F -->|No| K
    F -->|Yes| H
    H -->|Err| K
    H -->|Ok| I
```

### 2.7 Label Color and Default Suppression Table

| `EntityType` | Label color | Default name suppressed |
|--------------|-------------|------------------------|
| `Npc` | `Color32::WHITE` | `"NPC"` |
| `Enemy` | `Color32::from_rgb(255, 100, 100)` | `"Enemy"` (editor-defined convention — no game-side default exists yet; see Appendix B) |
| `Item` | `Color32::from_rgb(255, 215, 0)` | `"Item"` (editor-defined convention — no game-side default exists yet; see Appendix B) |
| `Trigger` | `Color32::from_rgb(100, 220, 220)` | `"Trigger"` (editor-defined convention) |
| `LightSource` | `Color32::from_rgb(255, 180, 50)` | `"LightSource"` (editor-defined convention) |

### 2.8 `should_show_label` Helper

```rust
/// Returns `true` when `name` should be displayed as a floating viewport label.
///
/// Suppresses labels for names that are absent (empty string) or equal to the
/// entity type's `default` placeholder.
fn should_show_label(name: &str, default: &str) -> bool {
    !name.is_empty() && name != default
}
```

All five existing unit tests are updated to call `should_show_label(name, "NPC")` instead of `should_show_npc_label(name)`. Additional tests cover Enemy and Item suppression.

### 2.9 `render_entity_name_labels` — Sketch

```rust
pub fn render_entity_name_labels(
    mut contexts: EguiContexts,
    editor_state: Res<EditorState>,
    markers: Query<(&EditorEntityMarker, &GlobalTransform)>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return; };

    // Respect the visibility toggle.
    if !editor_state.show_entity_labels {
        return;
    }

    let (camera_comp, camera_transform) = camera.into_inner();

    for (marker, marker_transform) in &markers {
        let Some(entity_data) = editor_state.current_map.entities.get(marker.entity_index)
        else { continue; };

        // Per-type color and default suppression string.
        let (label_color, default_name) = match entity_data.entity_type {
            EntityType::Npc         => (egui::Color32::WHITE,                     "NPC"),
            EntityType::Enemy       => (egui::Color32::from_rgb(255, 100, 100),   "Enemy"),
            EntityType::Item        => (egui::Color32::from_rgb(255, 215, 0),     "Item"),
            EntityType::Trigger     => (egui::Color32::from_rgb(100, 220, 220),   "Trigger"),
            EntityType::LightSource => (egui::Color32::from_rgb(255, 180, 50),    "LightSource"),
            _                       => continue, // PlayerSpawn — not labelled
        };

        let name = entity_data.properties.get("name").map(String::as_str).unwrap_or("");
        if !should_show_label(name, default_name) { continue; }

        let world_pos = marker_transform.translation() + Vec3::Y * LABEL_Y_OFFSET;
        let Ok(screen_pos) = camera_comp.world_to_viewport(camera_transform, world_pos)
        else { continue; };

        egui::Area::new(egui::Id::new(("entity_label", marker.entity_index)))
            .fixed_pos(egui::pos2(screen_pos.x, screen_pos.y))
            .pivot(egui::Align2::CENTER_BOTTOM)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_rgba_unmultiplied(30, 30, 30, 180))
                    .inner_margin(egui::Margin::symmetric(6, 4))
                    .corner_radius(4.0)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(name)
                                .size(LABEL_FONT_SIZE)
                                .color(label_color)
                                .family(egui::FontFamily::Name(FIRA_MONO_FAMILY.into())),
                        );
                    });
            });
    }
}
```

### 2.10 System Registration Change

```rust
// src/bin/map_editor/main.rs — update existing line
// Before:
ui::render_npc_name_labels.after(ui_system::render_ui),
// After:
ui::render_entity_name_labels.after(ui_system::render_ui),
```

### 2.11 Phase Boundaries

| Scope | In Phase 2 | Not in Phase 2 |
|-------|-----------|----------------|
| Visibility toggle | ✅ View menu + toolbar | — |
| Enemy labels (red) | ✅ | — |
| Item labels (gold) | ✅ | — |
| Trigger labels (cyan) | ✅ | — |
| LightSource labels (orange) | ✅ | — |
| Background pill | ✅ | — |
| NPC labels (Phase 1) | ✅ unchanged | — |
| PlayerSpawn labels | ❌ | Not planned (spawn point, not a named character) |
| Hover tooltip | ❌ | Phase 3 |
| Click-through selection | ❌ | Phase 3 |
| Configurable font size | ❌ | Phase 3 |
| `Trigger` / `LightSource` labels | ❌ | Unplanned |

---

## Appendices

### A. Key File Locations

| Component | File | Lines |
|-----------|------|-------|
| `EditorState` struct | `src/editor/state.rs` | 33–78 |
| `render_view_menu` | `src/editor/ui/toolbar/menus.rs` | 141–163 |
| `render_view_toggles` | `src/editor/ui/toolbar/controls.rs` | 9–37 |
| `render_npc_name_labels` (Phase 1) | `src/editor/ui/viewport.rs` | 387–439 |
| `should_show_npc_label` (Phase 1) | `src/editor/ui/viewport.rs` | 374–376 |
| `FIRA_MONO_FAMILY`, `LABEL_Y_OFFSET`, `LABEL_FONT_SIZE` | `src/editor/ui/viewport.rs` | 13–25 |
| `ui::mod.rs` re-exports | `src/editor/ui/mod.rs` | 1–16 |
| System registration | `src/bin/map_editor/main.rs` | ~190 |
| `EntityType` enum | `src/systems/game/map/format/entities.rs` | 27–40 |
| `EditorEntityMarker` | `src/editor/renderer.rs` | 41–44 |

### B. Open Questions & Decisions

#### Resolved

| # | Question | Resolution |
|---|----------|------------|
| 1 | Should the toggle use a new `Resource` or an `EditorState` field? | `EditorState` field — consistent with `show_grid` / `snap_to_grid`, no new types needed. |
| 2 | Should a separate system be added per entity type? | No — a single generalised system with a `match` arm is simpler and easier to maintain. |
| 3 | Should the pill style be shared with other overlays via a helper? | Not required for Phase 2 scope; inline is sufficient. Extraction is a future refactor if a third pill variant is added. |
| 4 | What default name does `spawn_enemy()` / `spawn_item()` use at runtime? | Enemy and Item spawning is not yet implemented — both are TODO stubs (`src/systems/game/map/spawner/mod.rs:488–495`). No game-side default exists. Suppression strings `"Enemy"` / `"Item"` are **editor-defined conventions**; must be kept in sync when game-side spawning is eventually implemented. |
| 5 | Should `Trigger` and `LightSource` entities be labelled in Phase 2? | Yes — all entity types with a `"name"` property receive labels. Trigger (cyan `rgb(100,220,220)`) and LightSource (orange `rgb(255,180,50)`) added to the match arm. `PlayerSpawn` remains excluded (spawn point, not a named character). |

#### Open

No open questions remain.

---

*Created: 2026-04-08*
*Companion: `docs/features/editor-entity-labels-phase2/requirements.md`*
*Phase 1 reference: `docs/features/editor-npc-name-labels/`*
