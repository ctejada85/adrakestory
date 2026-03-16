# Fix: Interior Detection BFS Frame Spikes

**Date:** 2026-03-15  
**Severity:** High  
**Component:** Occlusion / Interior Detection  

---

## Story

As a player, I want the game to run smoothly at all times so that I am not distracted by periodic stutters while exploring interiors.

---

## Description

`detect_interior_system` causes visible frame spikes every ~167 ms by running a BFS flood-fill (up to 1,000 voxels, 5–50 ms) six times per second and rebuilding a full voxel occupancy `HashSet` via an entity-count check on every detection cycle. The fix raises the update interval from 10 to 60 frames and replaces the fragile entity-count cache-invalidation with Bevy's `Added<SubVoxel>` / `RemovedComponents<SubVoxel>` change-detection. The default `OcclusionMode` remains `Hybrid`. The BFS logic, flood-fill algorithm, and hysteresis behaviour are unchanged and out of scope.

---

## Acceptance Criteria

1. The default value of `OcclusionConfig.region_update_interval` is `60` frames.
2. The default value of `OcclusionConfig.mode` remains `OcclusionMode::Hybrid`.
3. When `mode` is `ShaderBased`, `detect_interior_system` returns early without executing any BFS or cache work.
4. `InteriorState` no longer contains a `cache_entity_count` field.
5. During steady-state gameplay (no `SubVoxel` entities added or removed), `build_occupied_voxel_set` is not called — the existing cache is reused across detection cycles.
6. When `SubVoxel` entities are added (e.g., after hot reload), the occupancy cache is cleared and rebuilt on the next detection cycle.
7. When `SubVoxel` entities are removed, the occupancy cache is cleared and rebuilt on the next detection cycle.
8. `InteriorRegion`, `flood_fill_ceiling_region_voxel`, `find_ceiling_voxel_above`, and the player movement gate (< 0.3 units) are functionally unchanged.
9. `OcclusionMode::Hybrid` and `OcclusionMode::RegionBased` remain functional when configured explicitly.
10. All existing unit tests in `interior_detection.rs` pass without modification.
11. Unit tests cover: steady-state cache reuse, cache invalidation on add, cache invalidation on remove, and the early-return when mode is `ShaderBased`.

---

## Non-Functional Requirements

- The occupancy `HashSet` must not be rebuilt on any frame where no `SubVoxel` entities were added or removed.
- The change-detection check must introduce no per-frame heap allocations.
- `InteriorState` must remain a `Resource` with `Default` derivation; no public API is added to the module.
- The fix is scoped to the game binary only; the map editor does not use `detect_interior_system` and must not be affected.
- The GPU `OcclusionUniforms` struct, its shader binding, and the WGSL shader must not be modified.
- `architecture.md` in `docs/developer-guide/` must be updated to reflect the default mode and cache-invalidation changes.

---

## Tasks

1. Change `OcclusionConfig` default: `region_update_interval` `10` → `60` (mode default stays `Hybrid`)
2. Remove `cache_entity_count: usize` field from `InteriorState`
3. Add `added_sub_voxels: Query<(), Added<SubVoxel>>` and `mut removed_sub_voxels: RemovedComponents<SubVoxel>` parameters to `detect_interior_system`
4. Replace entity-count scan with event-based invalidation: if either parameter is non-empty, set `occupied_voxels_cache = None`
5. Remove the `current_entity_count` local variable and its `cache_entity_count` write
6. Write unit tests: steady-state reuse, add-triggers-rebuild, remove-triggers-rebuild, ShaderBased early-return
7. Run `cargo test --lib`, `cargo clippy --lib`, `cargo build --release` — fix all failures
8. Update `docs/developer-guide/architecture.md` to reflect default mode and cache strategy changes
9. Manually run the game with F3, walk through an interior, and confirm no visible periodic stutter
