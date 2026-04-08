# Investigation: Entity Name Field Missing from Properties Panel

**Date:** 2026-04-08 16:05
**Status:** Complete
**Component:** Map Editor — Properties Panel (`src/editor/ui/properties/`)

---

## Summary

The Properties panel does not show a "Name:" text field when a non-NPC entity is selected. Enemy, Item, Trigger, and LightSource entities all support named viewport labels (added in Phase 2, commit `46ff5e3`), but the Properties panel has no mechanism to set that name for those types. Only `EntityType::Npc` exposes the name-editing UI.

---

## Environment

- Binary: `map_editor`
- Source: `src/editor/ui/properties/entity_props.rs`
- Present since: initial Properties panel commit `530d189` (Dec 15, 2025), which created the Name field as NPC-only from day one. Phase 2 commit `46ff5e3` made the gap more visible by extending viewport labels to the affected types without updating the Properties panel.

---

## Investigation Method

1. Traced the call chain from the Properties panel entry point down to the name-editing widget.
2. Read `entity_props.rs`, `selection.rs`, and `mod.rs` in full.
3. Cross-referenced against `viewport.rs` to confirm which entity types are now label-capable.
4. Inspected git log to identify when the divergence was introduced.

---

## Findings

### Finding 1 — Name field gated to NPC only (p2 High)

**File:** `src/editor/ui/properties/entity_props.rs`
**Function:** `render_single_entity_properties()`
**Lines:** 79–85

```rust
if entity_type == EntityType::Npc {
    ui.add_space(8.0);
    render_npc_properties(ui, editor_state, history, index);   // ← Name field is here
} else if entity_type == EntityType::LightSource {
    ui.add_space(8.0);
    render_light_source_properties(ui, editor_state, index);   // ← no Name field
}
// Enemy, Item, Trigger: no properties section at all
```

The `render_npc_properties` function renders both the "Name:" text edit and the "Radius:" slider inside a single `ui.group`. For every other entity type, this function is never called, so the Name field is never shown.

### Finding 2 — LightSource properties section also missing Name (p2 High)

**File:** `src/editor/ui/properties/entity_props.rs`
**Function:** `render_light_source_properties()`
**Lines:** 170–295

`LightSource` has a dedicated properties section (Intensity, Range, Shadows, Color) but it was written independently of `render_npc_properties` and omits the Name field entirely. A LightSource entity can display a named viewport label (e.g., "Torch") but there is no way to set that name in the editor UI.

### Finding 3 — Enemy, Item, Trigger have no properties section (p3 Medium)

For these three types the only content rendered in the Properties panel is the type header and the Position group. There is no properties section, so no Name field and no type-specific controls. This is acceptable as long as the Name field is added via Finding 1's fix — none of these types have meaningful type-specific properties yet.

### Finding 4 — Bug present since initial creation; Phase 2 made it visible (p2 High)

The Name field was written as NPC-only from the very first commit that created `entity_props.rs` — commit `530d189` (Dec 15, 2025, "feat: Add entity and voxel property editing panels"). At that point, LightSource had Intensity/Range/Shadows/Color but no Name field, and Enemy/Item/Trigger had no properties section at all. The omission was present from day one, not introduced later.

Commit `46ff5e3` (Phase 2) extended viewport label support to Enemy, Item, Trigger, and LightSource by adding branches in `render_entity_name_labels` (`viewport.rs:409–417`). The name is read from `entity_data.properties.get("name")` at render time. This commit made the gap more visible — four entity types can now display named labels but have no way to set the name in the Properties panel — but it did not introduce the underlying omission.

---

## Root Cause Summary

| # | Root Cause | Location | Priority | Severity |
|---|-----------|----------|----------|----------|
| 1 | Name field rendered only inside `render_npc_properties`; not called for non-NPC types | `entity_props.rs:79-85` | p2 | High |
| 2 | `render_light_source_properties` omits Name field entirely | `entity_props.rs:170` | p2 | High |
| 3 | Enemy, Item, Trigger have no properties section (and thus no Name field) | `entity_props.rs:79-85` | p3 | Medium |
| 4 | Name field omitted for non-NPC types since initial creation (`530d189`, Dec 15 2025); Phase 2 made it visible | commit `530d189` / `46ff5e3` | p2 | High |

---

## Recommended Fix

Extract the Name field logic from `render_npc_properties` into a standalone `render_entity_name_field(ui, editor_state, history, index)` helper. Call this helper in `render_single_entity_properties` for all entity types **except** `PlayerSpawn` — before the per-type properties block. This ensures every label-capable type gets the Name field and future entity types automatically inherit it.

See: `docs/bugs/entity-name-field-missing/architecture.md`

---

## Related Bugs

- `docs/bugs/entity-name-field-missing/ticket.md` — formal bug report
- Bug present since: initial Properties panel creation commit `530d189` (Dec 15, 2025); made more visible by Phase 2 work in `docs/features/editor-entity-labels-phase2/`
