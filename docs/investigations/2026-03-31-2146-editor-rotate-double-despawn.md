# Investigation: Editor Rotate — Double-Despawn Warning
**Date:** 2026-03-31 21:46
**Status:** Complete
**Component:** `src/editor/tools/selection_tool/preview.rs`

## Summary

When rotating voxels in the map editor, Bevy emits warnings of the form:

```
WARN bevy_ecs::error::handler: Entity despawned: The entity with ID 551v0 is invalid; its index now has generation 1.
```

Two independent preview systems — `render_transform_preview` and `render_rotation_preview` — both query `With<TransformPreview>` and both call `commands.entity(entity).despawn()` on every result in the same frame, during `TransformMode::Rotate`. Because Bevy defers `Commands`, both systems enqueue a despawn for the same entity IDs before any flush occurs. The second despawn command runs against an entity slot whose generation has already incremented, producing the warning.

## Environment

- Platform: Win32
- Editor binary: `map_editor`
- Trigger: Press `R` to enter rotate mode → rotate with arrow keys → each state change fires the warning

## Investigation Method

1. Searched editor source for all systems that query `With<TransformPreview>` and call `despawn()`.
2. Inspected system registration in `src/bin/map_editor/main.rs` to verify ordering constraints.
3. Traced `active_transform.is_changed()` condition under `TransformMode::Rotate`.
4. Confirmed deferred-command behaviour explains the generation mismatch.

## Findings

### Finding 1 — Shared `TransformPreview` cleanup between two unordered systems (p1 Critical)

**Location:** `src/editor/tools/selection_tool/preview.rs:32–34` and `preview.rs:107–110`

Both `render_transform_preview` (line 32) and `render_rotation_preview` (line 107) contain an identical cleanup loop:

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

Both systems are registered in `src/bin/map_editor/main.rs` with **no ordering constraint between them**:

```rust
.add_systems(Update, tools::render_transform_preview)   // line 170
.add_systems(Update, tools::render_rotation_preview)    // line 171
```

**During `TransformMode::Rotate`:**

- `render_transform_preview` passes its mode guard (`mode != None`) and its change guard (`is_changed()`), runs the cleanup loop, then spawns coarse cube previews.
- `render_rotation_preview` passes its mode guard (`mode == Rotate`) and its change guard, runs the **same** cleanup loop on the same entity set, then spawns sub-voxel previews.

Because Bevy defers `Commands`, neither system sees the other's despawns at query time. Both systems iterate the same live entity IDs and both enqueue a `despawn()` for each. After the frame's command flush, the first despawn is applied (generation increments from 0 → 1); the second despawn runs against the now-invalid slot, producing the warning.

The number of warnings per frame equals the number of `TransformPreview` entities alive at the start of the frame (matching the three entity IDs 551v0, 552v0, 553v0 in the sample output — one per selected voxel).

### Finding 2 — `render_transform_preview` also spawns previews that are immediately despawned (p2 High)

Because both systems run in the same frame, `render_transform_preview` spawns new coarse cube previews at the end of its execution (lines 74–85), and `render_rotation_preview` then immediately queues despawns for those newly-spawned entities (line 108). The newly-spawned entity IDs will not exist yet at query time (deferred spawn), so this specific subset does **not** produce a generation-mismatch warning — but the coarse cube previews are wasted work: they are spawned and despawned within the same command-flush cycle and never appear on screen.

## Root Cause Summary

| # | Root Cause | Location | Priority | Severity | Notes |
|---|-----------|----------|----------|----------|-------|
| 1 | Two systems share `With<TransformPreview>` cleanup with no ordering | `preview.rs:32–34` and `preview.rs:107–110` | p1 | Critical | Produces despawn-generation warnings every frame during rotate |
| 2 | `render_transform_preview` spawns coarse previews that `render_rotation_preview` immediately clears | `preview.rs:74–85` (spawn) and `preview.rs:107–110` (despawn) | p2 | High | Wasted GPU/CPU work; previews never visible |

## Recommended Fixes

**Option A — Remove the cleanup loop from `render_rotation_preview` and add ordering (minimal change)**

In `preview.rs`, delete lines 107–110 from `render_rotation_preview`. Then in `main.rs`, enforce ordering:

```rust
.add_systems(Update,
    (tools::render_transform_preview, tools::render_rotation_preview)
        .chain()
)
```

With ordering, `render_rotation_preview` runs after `render_transform_preview` has already cleared and re-spawned coarse previews. Since `render_rotation_preview` also spawns sub-voxel previews, the two sets will coexist until the **next** frame's `render_transform_preview` clears them — which is another problem. This option alone does not fully solve Finding 2.

**Option B — Give each system exclusive ownership of its preview type (preferred)**

Introduce two marker components: `MovePreview` and `RotationPreview` (both replacing `TransformPreview`). Each system queries only its own marker. Cleanup for each mode is self-contained.

**Option C — Merge into a single system (cleanest)**

Merge `render_transform_preview` and `render_rotation_preview` into one system that switches on `active_transform.mode`. A single cleanup loop runs once, followed by a single spawn path. No shared-query race is possible.

## Related Bugs

- `docs/bugs/editor-rotate-double-despawn/bug.md` (to be created)
