# Requirements: Remove Orphaned `Voxel` Marker Entities

**Bug report:** `docs/bugs/2026-03-16-1310-p3-orphaned-voxel-marker-entities.md`
**Priority:** p3 Medium
**Investigation:** `docs/investigations/2026-03-16-1310-off-screen-voxel-performance.md`

## Problem Statement

One `Voxel` marker entity is spawned per voxel during map load. No system queries or uses these entities. They accumulate in the ECS world, consuming memory and change-detection storage proportional to map size, and are needlessly despawned and re-spawned on every hot reload.

## Functional Requirements

### FR-1 — Remove the orphaned spawn call
The `ctx.commands.spawn(Voxel)` call in `spawn_voxels_chunked()` must be removed.

### FR-2 — Remove the unused `Voxel` component
The `Voxel` component struct in `components.rs` must be removed because no code creates or queries it after FR-1.

### FR-3 — No orphaned imports
All `use` statements referencing `Voxel` must be cleaned up.

### FR-4 — No gameplay regression
No collision, rendering, LOD, occlusion, or hot-reload behaviour may be affected. The `Voxel` component was purely ornamental.

## Non-Functional Requirements

### NFR-1 — Reduced ECS entity count
After the fix, loading a map should produce fewer entities by exactly `map.world.voxels.len()`.

### NFR-2 — Faster hot-reload flush
Hot reload despawn/respawn command queue flush time decreases proportionally to map voxel count.

## Out of Scope

- Modifying `SubVoxel` entities
- Modifying `VoxelChunk` entities
- Any changes to the `Voxel` component type if it is found to be used elsewhere

## Acceptance Criteria

- [ ] `grep -r "spawn(Voxel"` finds no matches in `src/`
- [ ] `grep -r "struct Voxel"` finds no matches in `src/`
- [ ] `cargo clippy` reports zero new warnings
- [ ] `cargo test` passes
- [ ] `cargo build --release` succeeds
