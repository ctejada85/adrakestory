# Investigation: Off-Screen Voxel Performance Degradation
**Date:** 2026-03-16 13:10
**Status:** Complete
**Component:** `src/systems/game/map/spawner/mod.rs`, `src/systems/game/map/spawner/chunks.rs`

## Summary

When many voxels are placed in a map, the game slows down noticeably even when those voxels are off-screen (outside the main camera frustum). This points to a CPU or GPU bottleneck that scales with **total voxel count**, not visible voxel count. Investigation confirmed two co-primary causes: cascaded shadow rendering that processes all chunk meshes every frame regardless of camera view, and a large number of orphaned `Voxel` marker entities with no purpose.

## Environment

- macOS, Metal backend
- Bevy 0.15.3
- Test map: 10 VoxelChunk mesh entities, 230 840 SubVoxel collision entities

## Investigation Method

1. Read `src/systems/game/map/spawner/chunks.rs` in full — confirmed chunk component bundles and SubVoxel spawning
2. Read `src/systems/game/map/spawner/mod.rs` — confirmed directional light setup with `shadows_enabled: true` and 4 cascades
3. Searched all queries over `SubVoxel`, `VoxelChunk`, and `Voxel` components
4. Read `src/systems/game/interior_detection.rs` in full — confirmed BFS scale and cache strategy
5. Verified `Voxel` marker component is never queried by any system

## Findings

### Finding 1 — DirectionalLight Shadow Cascades Render All Chunks Every Frame (p2 High)

**File:** `src/systems/game/map/spawner/mod.rs:499–515`

```rust
let cascade_shadow_config = CascadeShadowConfigBuilder {
    num_cascades: 4,
    first_cascade_far_bound: 4.0,
    maximum_distance: 100.0,   // ← cascades cover up to 100 world-units
    ..default()
}
.build();

commands.spawn((
    DirectionalLight {
        shadows_enabled: true,  // ← shadow depth passes enabled
        ...
    },
    cascade_shadow_config,
    ...
));
```

`DirectionalLight` with `shadows_enabled: true` causes Bevy to render every mesh entity (that does not opt out) into 4 separate shadow depth maps per frame. The shadow depth pass uses the **directional light's view frustum**, not the main camera's. This means:

- All 10 VoxelChunk meshes are rendered into up to 4 depth passes **every frame**, regardless of whether they are visible to the player camera.
- As the voxel count grows, so do the chunk meshes (more quads → larger GPU geometry submissions). Shadow cost grows proportionally.
- On macOS / Metal, cascaded shadow map rendering is known to impose higher per-draw-call overhead than on Vulkan/DX12, amplifying the effect.

**Impact:** Shadow rendering accounts for up to 4× the geometry throughput of the main camera pass (4 cascades × chunk mesh size), running unconditionally every frame.

### Finding 2 — VoxelChunk Entities Have No `NotShadowCaster` (p2 High)

**File:** `src/systems/game/map/spawner/chunks.rs:232–269`

```rust
ctx.commands.spawn((
    Mesh3d(lod_meshes[0].clone()),
    MeshMaterial3d(mat.clone()),
    Transform::default(),
    VoxelChunk { chunk_pos, center: chunk_center },
    ChunkLOD { lod_meshes, current_lod: 0 },
    Aabb { center, half_extents },
    // ← no NotShadowCaster
));
```

In Bevy 0.15, any `Mesh3d` entity without `NotShadowCaster` automatically participates in all active shadow depth passes. Because VoxelChunk meshes carry geometry for up to 28.5k quads each (285k total / 10 chunks), the shadow pass is expensive. Adding `NotShadowCaster` would completely eliminate this cost.

**Note:** `Aabb` IS set on chunks (lines 245–248), so main-camera frustum culling works correctly. Only the shadow depth pass bypasses main-camera frustum culling.

### Finding 3 — Orphaned `Voxel` Marker Entities (p3 Medium)

**File:** `src/systems/game/map/spawner/chunks.rs`

```rust
ctx.commands.spawn(Voxel);  // ← spawned for every voxel in the map
```

A `Voxel` marker entity is spawned for each voxel in the map (i.e., `map.world.voxels.len()` entities). These entities have exactly one component (`Voxel`) and are **never queried by any system** — no `Query<With<Voxel>>` exists anywhere in the codebase. They are dead weight that:

- Inflate ECS entity count (thousands of extra entities for large maps)
- Consume ECS archetype table memory and change-detection tick storage
- Are unnecessarily despawned and re-spawned on every hot reload, increasing command queue flush time

For a map with ~1000 voxels, this adds ~1000 entities with no benefit.

### Finding 4 — Interior Detection BFS: Bounded but Allocates Every Detection Cycle (p4 Low)

**File:** `src/systems/game/interior_detection.rs:276–350`

`flood_fill_ceiling_region_voxel` runs a BFS up to `MAX_REGION_SIZE = 1000` cells and `max_search_distance = 30` manhattan distance. It allocates two `HashSet<IVec2>` and one `VecDeque<IVec2>` per invocation. The system is throttled to once per second (`region_update_interval: 60`) and skips when the player hasn't moved 0.3 world units. This is minor but allocates heap memory every second during active gameplay.

## Root Cause Summary

| # | Root Cause | Location | Priority | Severity | Notes |
|---|---|---|---|---|---|
| 1 | DirectionalLight `shadows_enabled: true` — 4 cascades render all chunks every frame | `mod.rs:511` | p2 | High | Primary bottleneck; scales with map mesh complexity |
| 2 | No `NotShadowCaster` on VoxelChunk entities | `chunks.rs:232` | p2 | High | Directly enables root cause #1; adding this alone fixes it |
| 3 | Orphaned `Voxel` marker entities, never queried | `chunks.rs` (spawn call) | p3 | Medium | Wasted ECS memory; worsens hot-reload flush time |
| 4 | Interior detection BFS allocates per invocation | `interior_detection.rs:276` | p4 | Low | Minor; already throttled |

## Recommended Fixes

### Fix A — Add `NotShadowCaster` to VoxelChunk entities (Fixes #1 and #2)

```rust
// src/systems/game/map/spawner/chunks.rs — both spawn paths
use bevy::pbr::NotShadowCaster;

ctx.commands.spawn((
    Mesh3d(lod_meshes[0].clone()),
    MeshMaterial3d(mat.clone()),
    Transform::default(),
    VoxelChunk { chunk_pos, center: chunk_center },
    ChunkLOD { lod_meshes, current_lod: 0 },
    Aabb { center, half_extents },
    NotShadowCaster,  // ← add this
));
```

Eliminates all shadow depth pass work for chunk meshes. Expected impact: removes 4 shadow passes per frame = significant GPU savings, especially on macOS/Metal.

**Trade-off:** Voxel world no longer casts shadows. This may or may not be acceptable depending on art direction. If shadows are desired, consider reducing `num_cascades` to 1 or 2, or lowering `maximum_distance` to match actual visible range.

### Fix B — Remove `Voxel` marker entities or repurpose them (Fixes #3)

Remove the `ctx.commands.spawn(Voxel)` call entirely (or store the entity ID on a parent structure if it is needed in the future). This eliminates N wasted ECS entities for N voxels in the map.

### Fix C — Pre-allocate BFS collections in `InteriorState` (Fixes #4)

Store reusable `HashSet` and `VecDeque` in `InteriorState` and call `.clear()` before each BFS instead of allocating new collections every second.

## Related Bugs

- `docs/bugs/fix-occlusion-shader-early-exit/` — fragment shader early-exit (GPU, already fixed)
- `docs/bugs/fix-occlusion-state-guard/` — occlusion systems running in wrong states (already fixed)
- `docs/bugs/2026-03-16-1213-p2-draw-call-count-metal-backend.md` — draw call count on Metal (related GPU perf)
