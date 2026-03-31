# Bug: Two preview systems share TransformPreview cleanup with no ordering

**Date:** 2026-03-31
**Priority:** p2
**Severity:** High
**Status:** Open
**Component:** `src/editor/tools/selection_tool/preview.rs`

---

## Description

When rotating voxels in the map editor, Bevy emits "entity already despawned" warnings
every frame that the rotation preview updates. Two independent systems —
`render_transform_preview` and `render_rotation_preview` — both iterate all
`With<TransformPreview>` entities and call `despawn()` in the same frame with no ordering
constraint between them. Because `Commands` are deferred, both systems enqueue despawns
for the same entity IDs before any flush, causing the second despawn to target an entity
slot whose generation has already incremented.

---

## Actual Behavior

- Opening a rotate operation (`R`) and pressing any rotation key causes multiple `WARN` log
  lines per frame:
  ```
  WARN bevy_ecs::error::handler: Entity despawned: The entity with ID 551v0 is invalid; its index now has generation 1.
  ```
- One warning is emitted per `TransformPreview` entity alive at the start of the frame
  (one per selected voxel).
- The coarse cube previews spawned by `render_transform_preview` are immediately despawned
  by `render_rotation_preview` in the same command flush and never appear on screen.

---

## Expected Behavior

- No "entity already despawned" warnings during rotate operations.
- Each `TransformPreview` entity is despawned exactly once per frame.
- Only the sub-voxel rotation previews (blue translucent geometry) are visible during
  `TransformMode::Rotate`; the coarse cube previews are not spawned at all in that mode.

---

## Root Cause Analysis

**File:** `src/editor/tools/selection_tool/preview.rs`
**Functions:** `render_transform_preview()` (line 9), `render_rotation_preview()` (line 89)
**Lines:** 32–34 and 107–110

Both systems contain identical cleanup loops querying the same component:

```rust
// render_transform_preview — line 32
for entity in existing_previews.iter() {
    commands.entity(entity).despawn();
}

// render_rotation_preview — line 107
for entity in existing_previews.iter() {
    commands.entity(entity).despawn();
}
```

Both systems pass their guards (`mode != None` / `mode == Rotate`, `is_changed()`) during
an active rotation. Bevy defers `Commands`; neither system sees the other's pending
despawns at query time. Both enqueue `despawn()` for the same entity IDs. After the flush,
the first despawn increments the entity slot's generation; the second despawn targets the
old generation and triggers the warning.

The systems are registered with no ordering constraint in `src/bin/map_editor/main.rs`:

```rust
.add_systems(Update, tools::render_transform_preview)   // ~line 170
.add_systems(Update, tools::render_rotation_preview)    // ~line 171
```

---

## Steps to Reproduce

1. `cargo run --bin map_editor --release`
2. Select one or more voxels in the editor.
3. Press `R` to enter rotate mode.
4. Press any rotation key (arrow key or equivalent).
5. Observe `WARN bevy_ecs::error::handler: Entity despawned: ...` in the terminal output,
   once per selected voxel per state change.

---

## Suggested Fix

**Option A — Remove the cleanup loop from `render_rotation_preview` and enforce ordering (minimal change)**

Delete lines 107–110 from `render_rotation_preview`. Add `.chain()` ordering in
`main.rs` so `render_rotation_preview` always runs after `render_transform_preview`:

```rust
.add_systems(Update,
    (tools::render_transform_preview, tools::render_rotation_preview)
        .chain()
)
```

`render_transform_preview` owns all cleanup; `render_rotation_preview` only appends its
sub-voxel spawns. However, in `Rotate` mode `render_transform_preview` still spawns coarse
cube previews that coexist with the sub-voxel previews until the next frame — a visual
artefact.

**Option B — Gate `render_transform_preview` to non-rotate modes only (preferred minimal fix)**

Add an early-return to `render_transform_preview` for `TransformMode::Rotate`:

```rust
if active_transform.mode == TransformMode::None {
    for entity in existing_previews.iter() { commands.entity(entity).despawn(); }
    return;
}
if active_transform.mode == TransformMode::Rotate {
    return; // rotation preview is handled exclusively by render_rotation_preview
}
```

Then in `render_rotation_preview`, the existing cleanup loop at lines 107–110 is the sole
owner of `TransformPreview` lifetime during rotate. No ordering change needed. Clean
separation: move previews owned by `render_transform_preview`, rotate previews owned by
`render_rotation_preview`.

**Option C — Merge into one system (cleanest)**

Merge both functions into a single `render_transform_preview` that switches on
`active_transform.mode`. A single cleanup loop runs once per frame. No shared-query race
is possible.

---

## Related

- Investigation: `docs/investigations/2026-03-31-2146-editor-rotate-double-despawn.md`
