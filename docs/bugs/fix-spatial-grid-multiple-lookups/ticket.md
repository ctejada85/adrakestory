# User Story: Fix Spatial Grid Multiple Lookups Per Frame

**Ticket ID:** fix-spatial-grid-multiple-lookups  
**Priority:** P2  
**Component:** Collision system / Player movement  
**Bug:** `docs/bugs/2026-03-16-1213-p2-spatial-grid-multiple-lookups-per-frame.md`  
**Requirements:** `docs/bugs/fix-spatial-grid-multiple-lookups/requirements.md`  
**Architecture:** `docs/bugs/fix-spatial-grid-multiple-lookups/architecture.md`

---

## Story

**As a** player on a voxel-dense map,  
**I want** the collision system to issue at most one spatial grid AABB lookup per
movement frame (instead of 3–4),  
**so that** frame time is lower and frame rate is more stable.

---

## Description

Every frame the player moves, `move_player` issues up to four
`spatial_grid.get_entities_in_aabb()` calls: one per diagonal check, one per step-up
re-check, and one each for the X and Z axis-fallback collision checks. Each call
allocates a `Vec<Entity>` and iterates triple-nested grid cell loops.

The fix pre-fetches a single widened AABB in `move_player` and passes the resulting
entity slice down to all inner collision checks. `check_sub_voxel_collision` gains an
optional `prefetched: Option<&[Entity]>` parameter and uses the supplied slice for both
the initial check and the step-up re-check. `apply_physics` is unchanged.

---

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC-1 | `move_player` calls `get_entities_in_aabb` at most once per frame |
| AC-2 | `check_sub_voxel_collision` accepts `Option<&[Entity]>` and uses it when `Some` |
| AC-3 | Step-up re-check reuses the pre-fetched slice (no second grid query when `prefetched` is set) |
| AC-4 | `apply_physics` still calls `get_entities_in_aabb` exactly once (unchanged) |
| AC-5 | All existing collision behaviours are preserved: wall slide, step-up, diagonal fast-path |
| AC-6 | New unit tests cover the `prefetched=Some` path and the `prefetched=None` fallback |
| AC-7 | `cargo test --lib` passes with zero new failures |
| AC-8 | `cargo clippy --lib` reports zero errors |

---

## Non-Functional Requirements

| # | Requirement |
|---|-------------|
| NFR-1 | The spatial grid data structure is not modified |
| NFR-2 | `check_sub_voxel_collision` remains a free function (not a method) |
| NFR-3 | `apply_axis_movement` remains `#[inline]` |
| NFR-4 | No new public API surface is introduced; all new parameters are in existing internal functions |

---

## Tasks

1. **Make `SUB_VOXEL_SIZE` and `STEP_UP_TOLERANCE` accessible from `player_movement.rs`**
   - Change from `const` to `pub(super) const` in `collision.rs`
   - Required for the widened AABB calculation in `move_player`

2. **Add `prefetched: Option<&[Entity]>` to `check_sub_voxel_collision`**
   - Insert parameter after `params: CollisionParams`
   - When `Some`, bind the slice; when `None`, issue the existing `get_entities_in_aabb` call
   - Update all existing call sites in `player_movement.rs` to pass `None` (backward-compat)
   - Use the same slice for the step-up re-check (remove the second `get_entities_in_aabb` call)

3. **Add `prefetched: &[Entity]` to `apply_axis_movement`**
   - Insert parameter after `current_floor_y: &mut f32`
   - Pass `Some(prefetched)` to both `check_sub_voxel_collision` calls inside

4. **Pre-fetch in `move_player` with widened AABB**
   - Compute widened AABB using `abs(move_delta)` expansion in XZ and `SUB_VOXEL_SIZE + STEP_UP_TOLERANCE` expansion upward in Y
   - Call `spatial_grid.get_entities_in_aabb` once and store in `prefetched_entities`
   - Pass `Some(&prefetched_entities)` to the diagonal `check_sub_voxel_collision` call
   - Pass `&prefetched_entities` to `apply_axis_movement`

5. **Write unit tests in `collision.rs`**
   - `collision_uses_prefetched_slice_when_provided` — verify empty slice returns `no_collision`
   - `collision_falls_back_to_grid_when_prefetched_none` — verify grid is queried when `None`
   - `step_up_recheck_uses_prefetched_slice` — verify step-up path reuses the same slice

6. **Write unit tests in `player_movement.rs`**
   - `widened_aabb_covers_move_delta` — assert min/max of computed prefetch AABB matches expected expansion

7. **Validate**
   - `cargo test --lib`
   - `cargo clippy --lib`
   - `cargo build --release`

8. **Commit**
   - Conventional commit: `perf(collision): pre-fetch spatial grid query in move_player to eliminate redundant lookups`
