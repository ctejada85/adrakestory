# Bug: Interior Detection Rebuilds Full SubVoxel HashSet on Every Map Load and Hot Reload

**Date:** 2026-03-16  
**Severity:** Medium  
**Status:** Open  
**Component:** Interior detection system  
**Platform:** Cross-platform  

---

## Description

`detect_interior_system()` maintains a cached `HashSet` of all occupied voxel positions for floor-fill and ceiling queries. The cache is invalidated whenever `Added<SubVoxel>` or `RemovedComponents<SubVoxel>` is detected — which occurs on every map load and every hot reload. On invalidation, the system iterates all `SubVoxel` entities in the spatial grid to rebuild the set from scratch (O(n) where n ≈ 200,000+ entities on a typical map). This produces a visible frame spike on map load and, more impactfully, during in-game hot reload where the game is expected to continue running smoothly.

---

## Actual Behavior

- On map load or hot reload, `Added<SubVoxel>` becomes non-empty
- `build_occupied_voxel_set()` runs and iterates all sub-voxel entities through the spatial grid
- On a typical map with ~200,000 `SubVoxel` entities this is a 200,000+ iteration loop in a single frame
- Frame time spikes by several milliseconds during the rebuild
- Spike is visible as a brief freeze on map entry and on every Ctrl+R hot reload

---

## Expected Behavior

- Map load freeze is tolerable since it occurs during a loading screen
- Hot reload (F5 / Ctrl+R during gameplay) should not cause a visible gameplay freeze
- The HashSet rebuild should not block the main thread for more than ~1 frame worth of budget

---

## Root Cause Analysis

**File:** `src/systems/game/interior_detection.rs`  
**Function:** `detect_interior_system()` / `build_occupied_voxel_set()`  
**Approximate lines:** 120, 126–129

```rust
// detect_interior_system() — interior_detection.rs ~line 120
let needs_rebuild = !added_sub_voxels.is_empty() || !removed_sub_voxels.is_empty();

if needs_rebuild {
    // Iterates ALL SubVoxel entities — O(200,000+) on a full map
    *occupied_voxels = build_occupied_voxel_set(&spatial_grid, &sub_voxel_query);
}
```

```rust
// build_occupied_voxel_set() — ~line 126
fn build_occupied_voxel_set(...) -> HashSet<IVec3> {
    let mut set = HashSet::new();
    for entity in spatial_grid.all_entities() {      // iterate every entity
        if let Ok(sv) = sub_voxel_query.get(entity) {
            set.insert(sv.voxel_position);           // extract position
        }
    }
    set
}
```

The full rebuild is triggered by even a single `Added<SubVoxel>` event, which occurs for every one of the ~200,000 entities spawned during map load. While only the first `is_empty()` check matters (it short-circuits on the first non-empty frame), that one frame processes all entities.

During hot reload, the same sequence repeats: all entities are despawned and respawned, triggering another full rebuild on the reload frame.

---

## Steps to Reproduce

1. Build and run: `cargo run --release -- --map assets/maps/default.ron`
2. Enable a frame-time display or attach a profiler
3. Press Ctrl+R or F5 to trigger hot reload
4. Observe the frame spike on the reload frame (usually 50–200 ms depending on map size)

---

## Suggested Fix

**Option A — Defer to Next Frame After Spawn Settles (recommended, low risk):**  
Hot reload despawns and respawns all entities over multiple frames. Instead of rebuilding on the first frame where `Added<SubVoxel>` is non-empty, wait until `Added<SubVoxel>` transitions back to empty (spawn is complete) before rebuilding. This amortizes the cost across the loading window.

```rust
// Track whether a rebuild is pending, rebuild only when additions stop
if !added_sub_voxels.is_empty() {
    *rebuild_pending = true;
    return; // wait for spawn to complete
}
if *rebuild_pending && added_sub_voxels.is_empty() {
    *rebuild_pending = false;
    *occupied_voxels = build_occupied_voxel_set(&spatial_grid, &sub_voxel_query);
}
```

**Option B — Incremental Update Instead of Full Rebuild:**  
Instead of discarding and rebuilding the full set, insert/remove individual voxel positions as entities are added or removed:

```rust
// On Added<SubVoxel>
for entity in added_sub_voxels.iter() {
    if let Ok(sv) = sub_voxel_query.get(entity) {
        occupied_voxels.insert(sv.voxel_position);
    }
}
// On RemovedComponents<SubVoxel> — positions are no longer queryable, needs cached copy
```

Tradeoff: Requires storing a reverse map from `Entity` → `IVec3` to handle removals (since the component is gone when `RemovedComponents` fires).

**Option C — Spawn-Time Pre-Population:**  
Populate the `HashSet` during map spawn (in `spawn_map_system`) as sub-voxels are created, rather than rebuilding it lazily in `detect_interior_system`. The system then only needs incremental updates.

---

## Related

- Investigation: `docs/investigations/2026-03-16-0809-macos-fps-drop-many-voxels.md` — Finding 5
- Related: `docs/bugs/2026-03-15-2141-p2-interior-detection-frame-spikes.md` — prior fix addressed the per-frame throttling; this is a remaining one-shot spike on load/reload
