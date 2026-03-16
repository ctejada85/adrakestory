# Architecture — Fix Interior Detection HashSet Rebuild on Hot Reload

---

## 1. Current Architecture

### 1.1 Component Overview

| Component | File | Role |
|-----------|------|------|
| `InteriorState` | `src/systems/game/interior_detection.rs` | Resource holding current region, throttle counter, and occupied voxel cache |
| `detect_interior_system` | same | Bevy system: invalidates cache, rebuilds if needed, then runs ceiling detection |
| `build_occupied_voxel_set` | same | Iterates all `SpatialGrid` cells → extracts voxel positions → returns `HashSet<IVec3>` |

### 1.2 Current Data Flow (Per Frame)

```
detect_interior_system()
│
├─ throttle check → return early if not yet due
│
├─ added_sub_voxels.is_empty() || removed_sub_voxels.is_empty()?
│   └─ YES → occupied_voxels_cache = None          ← invalidate
│
├─ occupied_voxels_cache.is_none()?
│   └─ YES → build_occupied_voxel_set()             ← O(200k) rebuild
│            (iterates ALL SpatialGrid entities)
│
└─ ceiling detection + flood fill + region update
```

### 1.3 Problem

On hot reload, `handle_map_reload` despawns all entities and sets `GameInitialized(false)`. `spawn_map_system` runs the following frame and spawns all ~200,000 `SubVoxel` entities in a single tick. On that tick `Added<SubVoxel>` is non-empty, the cache is cleared, and `build_occupied_voxel_set` runs immediately — an O(200k) loop in one frame while the player is still live in-game.

The rebuild is not needed until spawning finishes. The fix defers it.

---

## 2. Target Architecture

### 2.1 Design Principles

1. **Minimal footprint.** The fix adds one `bool` field to `InteriorState` and rewrites one conditional block in `detect_interior_system`. No new types, collections, or systems.
2. **Same-frame resume.** On the settle frame (spawn done), the rebuild runs and detection resumes in the same execution — no extra one-frame lag for the player.
3. **Zero per-frame allocation.** The flag is a scalar; no heap allocations are introduced.

### 2.2 Modified Components

| Component | Change |
|-----------|--------|
| `InteriorState` | Add `pub rebuild_pending: bool` field with `Default = false` |
| `detect_interior_system` | Replace immediate-invalidate block with defer pattern (see §2.3) |

### 2.3 New Data Flow (Per Frame)

```
detect_interior_system()
│
├─ throttle check → return early if not yet due
│
├─ added_sub_voxels non-empty OR removed_sub_voxels non-empty?
│   └─ YES → rebuild_pending = true; return            ← defer, no rebuild yet
│
├─ rebuild_pending == true?  (spawn has just settled)
│   └─ YES → rebuild_pending = false
│            occupied_voxels_cache = None
│            build_occupied_voxel_set()                ← exactly one rebuild
│            (fall through to detection on same frame)
│
├─ occupied_voxels_cache.is_none()?   (cold start)
│   └─ YES → build_occupied_voxel_set()
│
└─ ceiling detection + flood fill + region update
```

### 2.4 Struct Change

```rust
// BEFORE
#[derive(Resource, Default)]
pub struct InteriorState {
    pub current_region: Option<InteriorRegion>,
    pub last_detection_pos: Vec3,
    pub frames_since_update: u32,
    pub occupied_voxels_cache: Option<HashSet<IVec3>>,
}

// AFTER
#[derive(Resource, Default)]
pub struct InteriorState {
    pub current_region: Option<InteriorRegion>,
    pub last_detection_pos: Vec3,
    pub frames_since_update: u32,
    pub occupied_voxels_cache: Option<HashSet<IVec3>>,
    /// True while a spawn wave is in progress; triggers one rebuild on settle.
    pub rebuild_pending: bool,
}
```

### 2.5 System Logic Change (Appendix A — Code Template)

```rust
// Replace the current map_changed block (lines ~118–128) with:

// --- Defer rebuild while spawn wave is in progress ---
let spawn_in_progress = !added_sub_voxels.is_empty() || !removed_sub_voxels.is_empty();
if spawn_in_progress {
    interior_state.rebuild_pending = true;
    return;
}

// --- Settle frame: spawn done, perform deferred rebuild ---
if interior_state.rebuild_pending {
    interior_state.rebuild_pending = false;
    interior_state.occupied_voxels_cache = None;
}

// --- Cold start or post-reload: rebuild if cache is absent ---
let occupied_voxels = if let Some(ref cache) = interior_state.occupied_voxels_cache {
    cache
} else {
    let new_cache = build_occupied_voxel_set(&spatial_grid, &sub_voxels);
    interior_state.occupied_voxels_cache = Some(new_cache);
    interior_state.occupied_voxels_cache.as_ref().unwrap()
};
// ... rest of detection unchanged
```

---

## 3. Sequence Diagrams

### 3.1 Hot Reload Frame Sequence

```
Frame N   : handle_map_reload — despawn all, set GameInitialized(false)
Frame N+1 : spawn_map_system  — spawn 200k SubVoxels
            detect_interior_system — Added<SubVoxel> non-empty
              → rebuild_pending = true; return            (no 200k rebuild)
Frame N+2 : Added<SubVoxel> empty, rebuild_pending = true
              → rebuild once; resume detection immediately
```

### 3.2 Normal Gameplay (No Change)

```
Each frame : detect_interior_system
  → spawn_in_progress = false, rebuild_pending = false
  → cache present → ceiling detection → region update
```

---

## 4. Test Plan

| Test | Verifies |
|------|---------|
| `rebuild_deferred_while_spawn_in_progress` | When `spawn_in_progress = true`, `rebuild_pending` is set to `true` and the function returns before detection | 
| `rebuild_fires_on_settle_frame` | When `rebuild_pending = true` and `spawn_in_progress = false`, the cache is rebuilt and `rebuild_pending` is cleared |
| `cold_start_builds_cache_immediately` | When `rebuild_pending = false`, `spawn_in_progress = false`, and cache is `None`, rebuild fires inline |
| `flag_clears_after_single_rebuild` | `rebuild_pending` is `false` on the frame after the settle rebuild |

---

## 5. Invariants

| Invariant | How Maintained |
|-----------|---------------|
| `build_occupied_voxel_set` is called at most once per spawn wave | `rebuild_pending` flag gates the call; cleared immediately after |
| Interior detection never runs with a stale cache | Cache is `None` on the settle frame before `build_occupied_voxel_set` runs |
| `rebuild_pending` never stays `true` across multiple settle frames | Cleared unconditionally on the first frame where `spawn_in_progress = false` |
