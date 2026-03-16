# Requirements — Interior Detection Frame Spike Fix

**Source:** Bug report `2026-03-15-2141-p2-interior-detection-frame-spikes.md` — 2026-03-15  
**Status:** Draft

---

## 1. Overview

The `detect_interior_system` in `src/systems/game/interior_detection.rs` causes periodic visible frame stutter during normal gameplay. Two compounding issues are responsible:

1. The BFS flood-fill (up to 1,000 voxels) runs every 10 frames — approximately 6 times per second at 60 fps — producing 5–50 ms spikes every ~167 ms.
2. The occupancy `HashSet` cache is validated by counting all entities in the `SpatialGrid` every detection cycle. Any change to the entity count (e.g., hot reload) triggers a full rebuild that iterates every cell in the grid.

Additionally, the default `OcclusionMode` is `Hybrid`, which unconditionally enables this path for all players. Switching the default to `ShaderBased` eliminates the BFS path entirely for the common case.

This document covers a Phase 1 fix that addresses all three issues in a single change set.

---

## 2. Functional Requirements

### 2.1 BFS Rate Reduction

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1.1 | The default value of `OcclusionConfig.region_update_interval` must be raised from `10` to `60` frames. | Phase 1 |
| FR-2.1.2 | Existing users who explicitly configure `region_update_interval` must not be affected by the default change. | Phase 1 |

### 2.2 Default Mode Change

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.2.1 | The default value of `OcclusionConfig.mode` must change from `OcclusionMode::Hybrid` to `OcclusionMode::ShaderBased`. | Phase 1 |
| FR-2.2.2 | `OcclusionMode::Hybrid` and `OcclusionMode::RegionBased` must remain fully functional for users who configure them explicitly. | Phase 1 |
| FR-2.2.3 | When the mode is `ShaderBased`, `detect_interior_system` must return early without executing any BFS or cache work (existing early-return path, confirmed unchanged). | Phase 1 |

### 2.3 Event-Based Cache Invalidation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.3.1 | The occupancy cache (`InteriorState.occupied_voxels_cache`) must be invalidated when `SubVoxel` entities are added to the world. | Phase 1 |
| FR-2.3.2 | The occupancy cache must be invalidated when `SubVoxel` entities are removed from the world. | Phase 1 |
| FR-2.3.3 | Cache invalidation must use Bevy's built-in change-detection (`Added<SubVoxel>`, `RemovedComponents<SubVoxel>`) instead of comparing entity counts. | Phase 1 |
| FR-2.3.4 | The `InteriorState.cache_entity_count` field must be removed; it is replaced by Bevy change-detection. | Phase 1 |
| FR-2.3.5 | During steady-state gameplay (no map changes, player not teleporting), the occupancy cache must be rebuilt at most once per detection cycle, and never on frames where no geometry changes occurred. | Phase 1 |

### 2.4 Existing Behavior Preservation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.4.1 | `InteriorRegion`, `flood_fill_ceiling_region_voxel`, and `find_ceiling_voxel_above` must remain functionally unchanged. | Phase 1 |
| FR-2.4.2 | The player movement distance gate (skip if moved < 0.3 world units since last detection) must remain unchanged. | Phase 1 |
| FR-2.4.3 | Hysteresis logic (keep current region when player is near edge / no ceiling found directly above) must remain unchanged. | Phase 1 |
| FR-2.4.4 | The `frames_since_update` throttle counter must continue to reset to `0` after each detection cycle. | Phase 1 |

---

## 3. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-3.1 | During steady-state gameplay (no map changes), BFS must not execute more than once per second at 60 fps when `region_update_interval = 60`. | Phase 1 |
| NFR-3.2 | The occupancy cache must never be rebuilt on frames where no `SubVoxel` entities were added or removed. | Phase 1 |
| NFR-3.3 | The fix must introduce no per-frame heap allocations beyond what already exists in the BFS path. | Phase 1 |
| NFR-3.4 | The default player experience must exhibit no perceptible periodic stutter attributable to interior detection. | Phase 1 |
| NFR-3.5 | All existing unit tests in `interior_detection.rs` must continue to pass without modification. | Phase 1 |
| NFR-3.6 | `InteriorState` must remain a `Resource` with `Default` derivation; no new public API is added to the module. | Phase 1 |

---

## 4. Phase Scoping

### Phase 1 (all delivered together)

- Raise `region_update_interval` default from `10` → `60`
- Change default `OcclusionMode` from `Hybrid` → `ShaderBased`
- Remove `InteriorState.cache_entity_count`
- Replace entity-count cache invalidation with `Added<SubVoxel>` + `RemovedComponents<SubVoxel>` event detection

### Future Phases

- Replace BFS flood-fill with an incremental/spatial data structure to reduce per-run cost for large maps
- Add a dedicated `InteriorDetectionConfig` resource to separate detection tuning from occlusion rendering config
- Expose `region_update_interval` as a hot-reloadable field

---

## 5. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | Bevy 0.15 `RemovedComponents<T>` provides an `is_empty()` method (or equivalent) usable without draining the iterator. |
| 2 | `Added<SubVoxel>` as a query filter correctly captures entities spawned during hot reload. |
| 3 | Only one system reads `RemovedComponents<SubVoxel>` — `detect_interior_system`. If another system requires this, a shared resource flag approach must be used instead. |
| 4 | The voxel occupancy cache is specific to interior detection; no other system depends on `InteriorState.cache_entity_count`. |
| 5 | `ShaderBased` is the correct default for the common case (outdoor/open-world maps); maps requiring region-based occlusion must opt in via config. |

---

## 6. Open Questions

| # | Question | Owner |
|---|----------|-------|
| 1 | Should the `interior_height_threshold` default (currently `8.0`) be reviewed as part of this fix, or deferred? | — |

---

## 7. Dependencies & Blockers

| # | Dependency | Status |
|---|-----------|--------|
| 1 | Occlusion GPU re-upload fix (P1) — already shipped; `OcclusionConfig` is the same resource being modified here. | Done |

---

*Created: 2026-03-15*  
*Source: Bug report `docs/bugs/2026-03-15-2141-p2-interior-detection-frame-spikes.md`*  
*Companion document: [Architecture](./architecture.md)*
