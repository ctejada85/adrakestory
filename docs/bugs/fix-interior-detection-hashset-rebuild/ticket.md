# Fix Interior Detection HashSet Rebuild on Hot Reload

**Date:** 2026-03-16  
**Severity:** Medium (P3)  
**Component:** Interior detection system  

---

## Story

As a player, I want hot reload (Ctrl+R) to not freeze the game so that I can iterate on maps without interrupting gameplay.

---

## Description

`detect_interior_system` rebuilds its full `HashSet<IVec3>` of occupied voxel positions on the first frame that `Added<SubVoxel>` is non-empty — which coincides with the frame that `spawn_map_system` spawns all ~200,000 sub-voxels. This causes a single-frame spike of 50–200 ms visible during in-game hot reload. The fix adds a `rebuild_pending: bool` flag to `InteriorState`: when a spawn wave is detected the system defers the rebuild and returns early; on the first settle frame (spawn done) it performs the rebuild exactly once and resumes detection immediately. Map-load behaviour is unchanged.

---

## Acceptance Criteria

1. After pressing Ctrl+R, no visible frame spike occurs during the respawn phase (rebuild is deferred until after spawn settles).
2. Interior detection resumes correctly on the settle frame immediately after the deferred rebuild.
3. On game startup (cold start, no spawn in progress), the cache is built on the first execution as before.
4. `rebuild_pending` is `false` on every frame except those where a spawn wave is active.
5. `build_occupied_voxel_set` is called at most once per spawn wave (not once per `Added<SubVoxel>` frame).
6. All pre-existing interior detection tests pass without modification.
7. New unit tests covering the `rebuild_pending` flag transitions all pass.

---

## Non-Functional Requirements

- The fix must not introduce any per-frame heap allocation.
- The change must be confined to `src/systems/game/interior_detection.rs`; no other files are modified.
- `InteriorState` must retain `#[derive(Resource, Default)]`; `rebuild_pending` defaults to `false`.

---

## Tasks

1. Add `pub rebuild_pending: bool` field to `InteriorState` in `interior_detection.rs`.
2. Replace the `map_changed → cache = None` block in `detect_interior_system` with the defer pattern: set `rebuild_pending = true` and return early when spawn is in progress; clear flag and rebuild on the settle frame (see `architecture.md` Appendix A).
3. Write unit tests for:
   - `rebuild_deferred_while_spawn_in_progress` — flag is set and function returns before detection runs.
   - `rebuild_fires_on_settle_frame` — cache is rebuilt and flag is cleared when `rebuild_pending = true` and spawn is no longer in progress.
   - `cold_start_builds_cache_immediately` — cache built inline when `rebuild_pending = false`, no spawn in progress, cache is `None`.
   - `flag_clears_after_single_rebuild` — `rebuild_pending` is `false` after the settle rebuild.
4. Run `cargo test --lib`, `cargo clippy --lib`, and `cargo build --release`; fix any failures.
5. Manually trigger Ctrl+R hot reload and confirm no visible freeze using the F3 FPS overlay.
