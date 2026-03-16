# Requirements — Fix Interior Detection HashSet Rebuild on Hot Reload

**Source:** Bug report `docs/bugs/2026-03-16-1213-p3-interior-detection-hashset-rebuild-on-hot-reload.md`  
**Status:** Draft  

---

## 1. Overview

`detect_interior_system` maintains a `HashSet<IVec3>` of all occupied voxel positions used for ceiling ray-cast and flood-fill queries. The cache is invalidated whenever `Added<SubVoxel>` or `RemovedComponents<SubVoxel>` is detected — which occurs on every map load and every hot reload. On invalidation, the system immediately rebuilds the set by iterating all `SubVoxel` entities in the spatial grid (O(n), n ≈ 200,000+ on a typical map). This causes a single-frame spike of 50–200 ms.

On map load the spike occurs during the loading screen, where it is tolerable. On in-game hot reload (Ctrl+R / F5) the spike is visible as a brief gameplay freeze. This fix defers the rebuild until the spawn wave has settled — when `Added<SubVoxel>` transitions back to empty — so the heavy frame does not coincide with active gameplay.

---

## 2. Functional Requirements

### 2.1 Rebuild Deferral

| ID | Requirement | Phase |
|----|-------------|-------|
| FR-1 | When `Added<SubVoxel>` is non-empty, the system must set a `rebuild_pending` flag and return early without rebuilding the cache. | 1 |
| FR-2 | When `RemovedComponents<SubVoxel>` is non-empty, the system must set the `rebuild_pending` flag and return early without rebuilding the cache. | 1 |
| FR-3 | When `rebuild_pending` is `true` and both `Added<SubVoxel>` and `RemovedComponents<SubVoxel>` are empty, the system must perform exactly one full rebuild and clear `rebuild_pending`. | 1 |
| FR-4 | While `rebuild_pending` is `true`, the system must not attempt interior detection; it must return early (suspending detection) until the rebuild occurs. | 1 |

### 2.2 Cold-Start Behaviour

| ID | Requirement | Phase |
|----|-------------|-------|
| FR-5 | On the first execution after game startup (cache is `None`, no spawn activity in progress), if `rebuild_pending` is `false` and both change queries are empty, the system must build the cache immediately as it does today. | 1 |

### 2.3 Invariants Preserved

| ID | Requirement | Phase |
|----|-------------|-------|
| FR-6 | All existing ceiling detection, flood-fill, and throttle logic must remain unchanged. | 1 |
| FR-7 | The system must continue to fall back to a full rebuild whenever the cache is `None` and no spawn is in progress. | 1 |

---

## 3. Non-Functional Requirements

| ID | Requirement | Phase |
|----|-------------|-------|
| NFR-1 | No additional per-frame allocation. The flag is a scalar `bool` field; no new collections are introduced. | 1 |
| NFR-2 | Change is confined to `interior_detection.rs` and the `InteriorState` resource. No other systems are affected. | 1 |
| NFR-3 | All existing tests must continue to pass. New tests must cover the flag transitions. | 1 |

---

## 4. Out of Scope

- **Incremental updates (Option B):** Inserting/removing individual voxel positions via a `HashMap<Entity, IVec3>` reverse map is not in scope for this fix. It is a more thorough long-term solution but requires additional data structures and is higher risk.
- **Spawn-time pre-population (Option C):** Populating the cache inside `spawn_map_system` is not in scope.
- **Off-thread rebuild:** Running the rebuild on a background thread / async task is not in scope.

---

## 5. Open Questions

| ID | Question | Status |
|----|----------|--------|
| Q1 | Should interior detection resume on the same frame as the rebuild, or the frame after? (Current proposal: same frame — rebuild then detect on the settle frame.) | ✅ Same frame (rebuild + detect in the same execution) |
| Q2 | Is a brief suspension of interior detection during hot reload (while `rebuild_pending` is true) acceptable? | ✅ Yes — hot reload already disrupts visual state |
