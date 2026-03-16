# Bug: Orphaned `Voxel` Marker Entities Spawned Per Voxel but Never Queried

**Date:** 2026-03-16
**Priority:** p3
**Severity:** Medium
**Status:** Open
**Component:** `src/systems/game/map/spawner/chunks.rs`

---

## Description

A `Voxel` marker entity is spawned for every voxel in the map during `spawn_voxels_chunked()`. These entities have a single component (`Voxel`) and are never referenced or queried by any system in the codebase. They are dead weight that inflate ECS entity count proportionally to map size, increase memory usage, and slow down hot-reload despawn/respawn cycles without providing any benefit.

---

## Actual Behavior

- One `Voxel`-component entity is spawned per voxel in the map (e.g. ~1000 entities for a typical map)
- No system anywhere performs `Query<With<Voxel>>` or accesses these entities
- On hot reload, all `Voxel` entities are despawned and re-spawned, contributing to the command-queue flush spike
- ECS archetype table for the `Voxel` component is maintained for zero gameplay benefit

---

## Expected Behavior

- Only entities that are actively used by gameplay, rendering, or collision systems should be spawned
- Orphaned entities should not exist in the ECS world

---

## Root Cause Analysis

**File:** `src/systems/game/map/spawner/chunks.rs`
**Function:** `spawn_voxels_chunked()`

Inside the per-voxel spawning loop, a marker entity is created:

```rust
ctx.commands.spawn(Voxel);
```

The entity ID is not stored or returned. No system in `src/` contains a `Query<..., With<Voxel>>` or `Query<Entity, With<Voxel>>`. The `Voxel` component type is defined but appears to be a vestige of an earlier design that was not cleaned up.

For a map with N voxels, this spawns N unused entities. For the test map (230k sub-voxel collision entities), there are a corresponding number of voxels, potentially adding hundreds to thousands of `Voxel` entities that sit in the ECS consuming memory and change-detection tick storage each frame.

---

## Steps to Reproduce

1. Load any map
2. Search the running ECS world for entities with only the `Voxel` component — they will exist in the thousands
3. Search the codebase for `Query<..., With<Voxel>>` — no results

---

## Suggested Fix

Remove the orphaned spawn call:

```rust
// DELETE this line from spawn_voxels_chunked():
ctx.commands.spawn(Voxel);
```

If the `Voxel` component is intended for future use (e.g., to mark voxel-level entities for editor selection), keep the component definition but do not spawn the entities until they serve a purpose. Also remove the `Voxel` component struct if it has no planned use:

```rust
// src/systems/game/components.rs — remove if unused
#[derive(Component)]
pub struct Voxel;
```

---

## Related

- Investigation: `docs/investigations/2026-03-16-1310-off-screen-voxel-performance.md`
