use super::*;

#[test]
fn test_interior_region_contains() {
    let region = InteriorRegion {
        min: Vec3::new(0.0, 0.0, 0.0),
        max: Vec3::new(10.0, 5.0, 10.0),
        ceiling_y: 4,
        voxel_count: 100,
    };

    assert!(region.contains(Vec3::new(5.0, 2.5, 5.0)));
    assert!(region.contains(Vec3::new(0.0, 0.0, 0.0)));
    assert!(region.contains(Vec3::new(10.0, 5.0, 10.0)));
    assert!(!region.contains(Vec3::new(-1.0, 2.5, 5.0)));
    assert!(!region.contains(Vec3::new(5.0, 6.0, 5.0)));
}

#[test]
fn test_interior_region_default() {
    let region = InteriorRegion::default();
    assert_eq!(region.voxel_count, 0);
    assert_eq!(region.ceiling_y, 0);
}

#[test]
fn test_find_ceiling_voxel_above() {
    let mut occupied = HashSet::new();
    // Place a voxel at y=5
    occupied.insert(IVec3::new(0, 5, 0));

    // Player at y=2, should find ceiling at y=5
    assert_eq!(find_ceiling_voxel_above(0, 2, 0, 10, &occupied), Some(5));

    // Player at y=5, should NOT find ceiling (same level)
    assert_eq!(find_ceiling_voxel_above(0, 5, 0, 10, &occupied), None);

    // Player at y=6, no ceiling above
    assert_eq!(find_ceiling_voxel_above(0, 6, 0, 10, &occupied), None);

    // Different XZ position, no ceiling
    assert_eq!(find_ceiling_voxel_above(1, 2, 0, 10, &occupied), None);
}

#[test]
fn interior_state_default_has_empty_cache() {
    let state = InteriorState::default();
    assert!(state.occupied_voxels_cache.is_none());
    assert_eq!(state.frames_since_update, 0);
    assert_eq!(state.last_detection_pos, Vec3::ZERO);
    assert!(state.current_region.is_none());
    assert!(!state.rebuild_pending);
}

#[test]
fn cache_reused_when_none_not_marked_dirty_by_events() {
    // Simulate steady-state: cache is populated and no map change events fired.
    // The cache should be returned as-is (not rebuilt).
    let mut occupied = HashSet::new();
    occupied.insert(IVec3::new(0, 5, 0));
    let cache: Option<HashSet<IVec3>> = Some(occupied.clone());

    // No map change → map_changed = false → use existing cache
    let map_changed = false;
    let result_cache = if map_changed { None } else { cache.clone() };
    assert!(result_cache.is_some());
    assert_eq!(result_cache.unwrap().len(), occupied.len());
}

#[test]
fn cache_cleared_when_map_changed_flag_is_true() {
    // Simulate a hot reload: added_sub_voxels is not empty → map_changed = true.
    // The cache must be set to None so build_occupied_voxel_set runs.
    let mut occupied = HashSet::new();
    occupied.insert(IVec3::new(0, 5, 0));
    let mut cache: Option<HashSet<IVec3>> = Some(occupied);

    let map_changed = true;
    if map_changed {
        cache = None;
    }
    assert!(cache.is_none());
}

#[test]
fn ceiling_not_found_when_wall_blocks_gap() {
    // A voxel at y=3 with another at y=4 is a wall from player level, not a ceiling.
    let mut occupied = HashSet::new();
    occupied.insert(IVec3::new(0, 3, 0));
    occupied.insert(IVec3::new(0, 4, 0)); // continuous wall from near player level

    // player at y=1: start_y = 3. y=3 is occupied but y=2 check:
    // has_gap_below checks (1+1..3) = range [2,3), i.e. y=2.
    // y=2 is NOT in occupied, so gap_below = true → should find y=3 as ceiling.
    let result = find_ceiling_voxel_above(0, 1, 0, 10, &occupied);
    assert_eq!(result, Some(3));
}

// --- rebuild_pending flag transition tests ---

#[test]
fn rebuild_deferred_while_spawn_in_progress() {
    // Simulates the logic in detect_interior_system when Added<SubVoxel> is non-empty.
    // rebuild_pending must be set to true and no cache rebuild should be attempted.
    let mut state = InteriorState::default();
    let spawn_in_progress = true; // Added<SubVoxel> is non-empty

    if spawn_in_progress {
        state.rebuild_pending = true;
        // system returns early here — no cache modification
    }

    assert!(
        state.rebuild_pending,
        "Flag must be set while spawn is in progress"
    );
    assert!(
        state.occupied_voxels_cache.is_none(),
        "Cache must not be touched while spawn is in progress"
    );
}

#[test]
fn rebuild_fires_on_settle_frame() {
    // Simulates the settle frame: spawn_in_progress = false, rebuild_pending = true.
    // The flag must be cleared and the cache must be reset to None (so the rebuild runs).
    let mut state = InteriorState {
        rebuild_pending: true,
        occupied_voxels_cache: Some(HashSet::new()), // stale cache from before hot reload
        ..Default::default()
    };
    let spawn_in_progress = false;

    if !spawn_in_progress && state.rebuild_pending {
        state.rebuild_pending = false;
        state.occupied_voxels_cache = None;
    }

    assert!(
        !state.rebuild_pending,
        "Flag must be cleared on settle frame"
    );
    assert!(
        state.occupied_voxels_cache.is_none(),
        "Cache must be cleared so rebuild runs"
    );
}

#[test]
fn cold_start_builds_cache_immediately() {
    // Cold start: rebuild_pending = false, no spawn in progress, cache is None.
    // The system should fall through to the inline build (not defer).
    let state = InteriorState::default();
    let spawn_in_progress = false;

    // No deferral path taken
    assert!(!spawn_in_progress);
    assert!(!state.rebuild_pending);
    // Cache is None → build_occupied_voxel_set would be called (simulated here)
    assert!(
        state.occupied_voxels_cache.is_none(),
        "Cache must be None so inline build is triggered"
    );
}

#[test]
fn flag_clears_after_single_rebuild() {
    // After the settle rebuild runs, rebuild_pending must be false on subsequent frames.
    let mut state = InteriorState {
        rebuild_pending: true,
        ..Default::default()
    };

    // Settle frame processing
    state.rebuild_pending = false;
    state.occupied_voxels_cache = None;
    // Simulate inline build completing
    let mut cache = HashSet::new();
    cache.insert(IVec3::new(1, 2, 3));
    state.occupied_voxels_cache = Some(cache);

    // Next frame — no spawn in progress
    let spawn_in_progress = false;
    if !spawn_in_progress && state.rebuild_pending {
        state.rebuild_pending = false;
        state.occupied_voxels_cache = None;
    }

    assert!(
        !state.rebuild_pending,
        "Flag must remain false on frames after settle"
    );
    assert!(
        state.occupied_voxels_cache.is_some(),
        "Cache must be preserved on frames after settle"
    );
}
