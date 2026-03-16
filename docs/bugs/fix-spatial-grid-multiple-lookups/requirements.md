# Requirements: Fix Spatial Grid Multiple Lookups Per Frame

**Bug Reference:** `docs/bugs/2026-03-16-1213-p2-spatial-grid-multiple-lookups-per-frame.md`  
**Priority:** P2  
**Component:** Collision system / Physics / Player movement  
**Platform:** Cross-platform

---

## 1. Problem Statement

Every frame the player is moving, the collision and physics systems collectively issue 3–4
calls to `spatial_grid.get_entities_in_aabb()`. Each call allocates a new `Vec<Entity>`,
iterates triple-nested loops over matching grid cells, and clones entity IDs. On maps with
high voxel density the combined cost adds ~0.1–0.3 ms per frame and scales linearly with
map size. This wastes CPU budget on all platforms and is a major contributor to the FPS
drops reported on macOS.

### Call breakdown (worst-case diagonal movement + step-up)

| Call site | File | Approx. line | Condition |
|-----------|------|--------------|-----------|
| Diagonal collision initial check | `collision.rs` | 127 | Always when moving diagonally |
| Step-up re-check | `collision.rs` | 203 | When a step-up candidate is found |
| X-axis fallback | `collision.rs` | 127 (via `apply_axis_movement`) | When diagonal blocked |
| Z-axis fallback | `collision.rs` | 127 (via `apply_axis_movement`) | When diagonal blocked |
| Physics vertical check | `physics.rs` | 85 | Always, every frame |

---

## 2. Goals

1. Reduce spatial grid AABB query calls from 3–4 per frame to **at most 2 per frame**:
   one lookup shared across all horizontal collision checks in `move_player`, and one
   separate lookup for the vertical physics check in `apply_physics`.
2. Eliminate redundant `Vec<Entity>` allocations by passing the pre-fetched entity slice
   down to inner functions rather than re-querying.
3. Preserve all existing collision behaviour: wall sliding, step-up, diagonal fast-path.
4. Keep the `check_sub_voxel_collision` signature change backward-compatible by accepting
   an optional pre-fetched slice (`Option<&[Entity]>`).

---

## 3. Non-Goals

- Changes to the spatial grid data structure itself (hash map, cell size, etc.)
- Merging `move_player` and `apply_physics` into a single system
- Changing the physics simulation model (gravity, step-up height, etc.)
- GPU-side or render-pipeline optimisations

---

## 4. Functional Requirements

### FR-1 — Single lookup in `move_player`

`move_player` must issue **at most one** `spatial_grid.get_entities_in_aabb()` call per
frame. The AABB used for this lookup must be wide enough to cover:

- The player's current bounding cylinder
- The furthest possible position after applying movement this frame

The same entity slice must be passed to:
- The diagonal collision check
- Both axis-fallback checks (X and Z)
- The step-up re-check inside `check_sub_voxel_collision`

### FR-2 — `check_sub_voxel_collision` accepts a pre-fetched slice

The function signature must be extended to accept an `Option<&[Entity]>` parameter:

```rust
pub fn check_sub_voxel_collision(
    spatial_grid: &SpatialGrid,
    sub_voxel_query: &Query<&SubVoxel, Without<Player>>,
    params: CollisionParams,
    prefetched: Option<&[Entity]>,
) -> CollisionResult
```

When `prefetched` is `Some(entities)`, the function uses that slice directly (both for
the initial check and the step-up re-check). When `None`, it falls back to issuing its
own `get_entities_in_aabb()` call (backward-compatible path).

### FR-3 — `apply_physics` keeps its own lookup

`apply_physics` queries a vertically-adjusted AABB (based on `new_y` after gravity) that
differs from the horizontal movement AABB. It must **not** share the same lookup as
`move_player` because the two systems run in different `GameSystemSet` phases. It may
continue to call `get_entities_in_aabb` once per frame independently.

### FR-4 — Widened query AABB in `move_player`

The pre-fetch AABB in `move_player` must expand the player's cylinder bounds by the full
movement delta this frame in the XZ plane, and by the maximum possible step-up height
upward in Y so the same slice covers the step-up re-check without a second grid query:

```
min.x = player_x - radius - abs(move_delta.x)
max.x = player_x + radius + abs(move_delta.x)
min.z = player_z - radius - abs(move_delta.z)
max.z = player_z + radius + abs(move_delta.z)
min.y = player_y - half_height            (no downward expansion needed)
max.y = player_y + half_height + SUB_VOXEL_SIZE + STEP_UP_TOLERANCE
```

The upward Y expansion ensures the entity slice captures any sub-voxel that could
be involved in a step-up collision verification at the stepped-up height.

### FR-5 — No behaviour regression

Collision resolution must produce bit-identical results to the current implementation
in all movement scenarios:
- Cardinal movement (X only, Z only)
- Diagonal movement (no collision)
- Diagonal movement (wall slide)
- Step-up on staircase voxels
- Step-up blocked by ceiling

---

## 5. Acceptance Criteria

| # | Criterion | Verification |
|---|-----------|--------------|
| AC-1 | `move_player` calls `get_entities_in_aabb` at most once per frame | Code review |
| AC-2 | `check_sub_voxel_collision` accepts `Option<&[Entity]>` and uses it when `Some` | Unit test |
| AC-3 | Step-up re-check reuses the pre-fetched slice (no second grid query when `prefetched` is set) | Unit test |
| AC-4 | `apply_physics` still issues exactly one lookup (unchanged) | Code review |
| AC-5 | All existing collision behaviour is preserved (wall slide, step-up, diagonal fast-path) | Existing tests pass |
| AC-6 | New unit tests cover the prefetch path and the fallback (`None`) path | Test run |
| AC-7 | `cargo test --lib` passes with zero new failures | CI |
| AC-8 | `cargo clippy --lib` reports zero errors | CI |

---

## 6. Constraints

- Must work within existing Bevy 0.15.3 ECS patterns
- `check_sub_voxel_collision` must remain a free function (not a method) to match the
  current codebase style
- `apply_axis_movement` is an `#[inline]` private helper — its signature may be updated
  to accept a pre-fetched slice without breaking public API

---

## 7. Open Questions

| # | Question | Status |
|---|----------|--------|
| Q-1 | Should the widened AABB also expand Y to cover step-up height, avoiding a second lookup even for step-up? | **Decided: Yes** — expand Y by `SUB_VOXEL_SIZE + STEP_UP_TOLERANCE` upward so the same slice covers both the initial check and the step-up re-check |
| Q-2 | Should `apply_physics` also be refactored to share a lookup with `move_player` in a future phase? | **Decided: Yes, Phase 2** — captured in `docs/bugs/fix-spatial-grid-multiple-lookups/ticket-phase2.md` |
