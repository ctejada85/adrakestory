# Bug: Name field locked to NPC — missing from Enemy, Item, Trigger, LightSource properties

**Date:** 2026-04-08
**Priority:** p2
**Severity:** High
**Status:** Open
**Component:** Map Editor — Properties Panel

---

## Description

The Properties panel renders a "Name:" text field only when an `Npc` entity is selected. Enemy, Item, Trigger, and LightSource entities can all display named floating labels in the 3D viewport (introduced in Phase 2), but there is no way to set or edit that name through the editor UI. The name field is unreachable for four of the five label-capable entity types.

---

## Actual Behavior

- Select an `Enemy`, `Item`, `Trigger`, or `LightSource` entity with the Select tool.
- The Properties panel shows only the entity type header and the Position group.
- No "Name:" text field is present.
- A `LightSource` entity shows Intensity/Range/Shadows/Color controls, but still no Name field.
- The `properties["name"]` key cannot be written through the editor for these types.

---

## Expected Behavior

- Every entity type that can display a named viewport label — Npc, Enemy, Item, Trigger, LightSource — must expose a "Name:" text field in the Properties panel.
- The name edit is undoable (creates an `EditorAction::ModifyEntity` history entry on focus-lost).
- PlayerSpawn entities are excluded (they have no viewport label and no name concept).
- Editing the name immediately affects the floating label shown above the entity in the viewport.

---

## Root Cause Analysis

**File:** `src/editor/ui/properties/entity_props.rs`
**Function:** `render_single_entity_properties()`
**Lines:** 79–85

```rust
if entity_type == EntityType::Npc {
    ui.add_space(8.0);
    render_npc_properties(ui, editor_state, history, index);  // Name field lives here
} else if entity_type == EntityType::LightSource {
    ui.add_space(8.0);
    render_light_source_properties(ui, editor_state, index);  // No Name field
}
// Enemy, Item, Trigger: no branch — no properties section rendered at all
```

The Name field was written as part of `render_npc_properties` alongside the Radius slider when `entity_props.rs` was first created (commit `530d189`, Dec 15, 2025). The omission of a Name field for other entity types was present from day one. Phase 2 (commit `46ff5e3`) added viewport label support for Enemy, Item, Trigger, and LightSource by updating `viewport.rs` — making the gap more visible — but the Properties panel was not updated at that time either.

---

## Steps to Reproduce

1. `cargo run --bin map_editor --release`
2. Place an Enemy, Item, Trigger, or LightSource entity on the map.
3. Switch to the Select tool.
4. Click the entity to select it.
5. Observe the Properties panel — no "Name:" field is visible.

For LightSource: the properties group shows Intensity, Range, Shadows, and Color, but no Name field.

---

## Suggested Fix

Extract the Name field into a standalone helper `render_entity_name_field` and call it in `render_single_entity_properties` for all non-PlayerSpawn entity types, before the per-type block:

```rust
// In render_single_entity_properties, after the Position group:
if entity_type != EntityType::PlayerSpawn {
    ui.add_space(8.0);
    render_entity_name_field(ui, editor_state, history, index);
}

// Per-type blocks remain, but render_npc_properties no longer renders the Name field:
if entity_type == EntityType::Npc {
    ui.add_space(8.0);
    render_npc_specific_properties(ui, editor_state, history, index);  // Radius only
} else if entity_type == EntityType::LightSource {
    ui.add_space(8.0);
    render_light_source_properties(ui, editor_state, index);  // unchanged
}
```

The new helper mirrors the existing Name field logic in `render_npc_properties` (lines 112–138): read current name, clone into a local `mut name`, call `ui.text_edit_singleline`, and commit via `EditorAction::ModifyEntity` on `response.lost_focus()`.

---

## Related

- Investigation: `docs/investigations/2026-04-08-1605-entity-name-field-missing-properties-panel.md`
- Requirements: `docs/bugs/entity-name-field-missing/requirements.md`
- Architecture: `docs/bugs/entity-name-field-missing/architecture.md`
- Bug present since: initial Properties panel creation — `docs/features/editor-entity-labels-phase2/` (Phase 2) made it more visible
