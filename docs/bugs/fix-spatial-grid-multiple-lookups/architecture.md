# Architecture: Fix Spatial Grid Multiple Lookups Per Frame

**Bug Reference:** `docs/bugs/2026-03-16-1213-p2-spatial-grid-multiple-lookups-per-frame.md`  
**Requirements:** `docs/bugs/fix-spatial-grid-multiple-lookups/requirements.md`  
**Priority:** P2

---

## 1. Current Architecture

### 1.1 Call Graph (per frame, worst-case diagonal + step-up)

```
move_player()
  └── check_sub_voxel_collision()          ← Query 1: diagonal check
        └── get_entities_in_aabb()
        └── get_entities_in_aabb()         ← Query 2: step-up re-check

  └── apply_axis_movement()  (fallback)
        ├── check_sub_voxel_collision()    ← Query 3: X-axis check
        │     └── get_entities_in_aabb()
        └── check_sub_voxel_collision()    ← Query 4: Z-axis check
              └── get_entities_in_aabb()

apply_physics()
  └── get_entities_in_aabb()              ← Query 5: vertical/gravity check
```

**Total: up to 4 `get_entities_in_aabb` calls from `move_player` + 1 from `apply_physics` = 5 per frame.**

Each call allocates a `Vec<Entity>`, iterates triple-nested grid cell loops, and clones
entity IDs. Cost is O(k) where k is the number of SubVoxel entities near the player.

### 1.2 `check_sub_voxel_collision` current signature

```rust
pub fn check_sub_voxel_collision(
    spatial_grid: &SpatialGrid,
    sub_voxel_query: &Query<&SubVoxel, Without<Player>>,
    params: CollisionParams,
) -> CollisionResult
```

The function owns its own `get_entities_in_aabb` call internally. It re-queries for the
step-up verification pass even though the entities are already known from the first pass.

### 1.3 `apply_axis_movement` current signature

```rust
fn apply_axis_movement(
    spatial_grid: &SpatialGrid,
    sub_voxel_query: &Query<&SubVoxel, Without<Player>>,
    transform: &mut Transform,
    player: &mut Player,
    current_pos: Vec3,
    new_x: f32,
    new_z: f32,
    current_floor_y: &mut f32,
)
```

Calls `check_sub_voxel_collision` twice (X then Z), each issuing its own grid query.

---

## 2. Target Architecture

### 2.1 Call Graph (per frame, worst-case)

```
move_player()
  └── get_entities_in_aabb()              ← Query 1: one widened pre-fetch

  └── check_sub_voxel_collision(prefetched=Some(&entities))  ← reuses slice
        (no get_entities_in_aabb — uses prefetched for both checks)

  └── apply_axis_movement(prefetched=&entities)  (fallback)
        ├── check_sub_voxel_collision(prefetched=Some(&entities))  ← reuses slice
        └── check_sub_voxel_collision(prefetched=Some(&entities))  ← reuses slice

apply_physics()
  └── get_entities_in_aabb()              ← Query 2: unchanged
```

**Total: 2 `get_entities_in_aabb` calls per frame (down from up to 5).**

### 2.2 Pre-fetch AABB Widening Strategy

The single pre-fetch in `move_player` must cover all positions the player could occupy
this frame, including:

1. Current position (for the initial collision check)
2. New position after movement (`current_pos + move_delta`)
3. Stepped-up position (current Y + `SUB_VOXEL_SIZE + STEP_UP_TOLERANCE` upward)

```
prefetch_min = Vec3::new(
    player_x - radius - abs(move_delta.x),
    player_y - half_height,               // no downward expansion needed
    player_z - radius - abs(move_delta.z),
)
prefetch_max = Vec3::new(
    player_x + radius + abs(move_delta.x),
    player_y + half_height + SUB_VOXEL_SIZE + STEP_UP_TOLERANCE,  // covers step-up height
    player_z + radius + abs(move_delta.z),
)
```

This ensures the same slice is valid for the step-up re-check without a second query.

### 2.3 Updated `check_sub_voxel_collision` Signature

```rust
pub fn check_sub_voxel_collision(
    spatial_grid: &SpatialGrid,
    sub_voxel_query: &Query<&SubVoxel, Without<Player>>,
    params: CollisionParams,
    prefetched: Option<&[Entity]>,   // new parameter
) -> CollisionResult
```

Internal logic:

```rust
let owned: Vec<Entity>;
let relevant: &[Entity] = if let Some(pre) = prefetched {
    pre
} else {
    owned = spatial_grid.get_entities_in_aabb(player_min, player_max);
    &owned
};
// ... use `relevant` for the initial check ...

// Step-up re-check: also reuse `relevant` instead of re-querying
// (pre-fetch AABB is widened to cover stepped-up Y bounds)
for entity in relevant {
    // ... check at new_y ...
}
```

### 2.4 Updated `apply_axis_movement` Signature

```rust
fn apply_axis_movement(
    spatial_grid: &SpatialGrid,
    sub_voxel_query: &Query<&SubVoxel, Without<Player>>,
    transform: &mut Transform,
    player: &mut Player,
    current_pos: Vec3,
    new_x: f32,
    new_z: f32,
    current_floor_y: &mut f32,
    prefetched: &[Entity],           // new parameter (always Some in call sites)
)
```

Both `check_sub_voxel_collision` calls inside pass `Some(prefetched)`.

---

## 3. Data Flow Diagram

```
move_player() system
│
├── Compute move_delta (from PlayerInput + delta time)
│
├── get_entities_in_aabb(widened AABB)   ──► prefetched: Vec<Entity>
│                                                         │
├── [diagonal path]                                       │
│   └── check_sub_voxel_collision(prefetched=Some) ◄──── ┤
│         ├── initial check    (uses prefetched)          │
│         └── step-up re-check (uses prefetched)          │
│                                                         │
└── [axis-fallback path]                                  │
    └── apply_axis_movement(prefetched=&entities) ◄────── ┤
          ├── check_sub_voxel_collision(Some) ◄─────────  │
          └── check_sub_voxel_collision(Some) ◄─────────  ┘

apply_physics() system (separate GameSystemSet phase)
└── get_entities_in_aabb(vertical AABB)  ← independent, unchanged
```

---

## 4. Affected Files

| File | Change |
|------|--------|
| `src/systems/game/collision.rs` | Add `prefetched: Option<&[Entity]>` to `check_sub_voxel_collision`; reuse slice for step-up re-check |
| `src/systems/game/player_movement.rs` | Pre-fetch with widened AABB; pass slice to `check_sub_voxel_collision` and `apply_axis_movement`; add `prefetched` param to `apply_axis_movement` |
| `src/systems/game/physics.rs` | No change |

---

## 5. Invariants Preserved

| Invariant | How preserved |
|-----------|---------------|
| Wall sliding | `apply_axis_movement` still performs X and Z checks independently with same entity set |
| Step-up | Step-up re-check uses same widened slice; Y upper bound covers stepped-up height |
| Diagonal fast-path | Unchanged — diagonal check still tries combined XZ first |
| Physics correctness | `apply_physics` retains its own independent query — Phase 2 (`ticket-phase2.md`) will share it via a `PreFetchedEntities` resource |
| `SpatialGrid` not modified | Pure consumer-side refactor; no data structure changes |

---

## 6. Testing Strategy

### Pure unit tests (no `bevy::App`)

These cover the new `prefetched` parameter path in `check_sub_voxel_collision`:

| Test | What it verifies |
|------|-----------------|
| `collision_uses_prefetched_slice_when_provided` | When `prefetched=Some(&[])`, returns `no_collision` without calling `get_entities_in_aabb` (verified via empty slice) |
| `collision_falls_back_to_grid_when_prefetched_none` | When `prefetched=None` with a populated grid, entities are found |
| `step_up_recheck_uses_prefetched_slice` | When a step-up candidate exists and `prefetched` covers stepped-up height, no second grid query is issued |
| `widened_aabb_covers_move_delta` | Assert that the computed prefetch AABB min/max correctly includes `abs(move_delta)` expansion |

### Existing tests

All existing collision and movement tests must continue to pass unchanged. Their call
sites pass `None` (backward-compatible path), so no updates needed.

---

## Appendix A — Code Templates

### A.1 Updated `check_sub_voxel_collision` skeleton

```rust
pub fn check_sub_voxel_collision(
    spatial_grid: &SpatialGrid,
    sub_voxel_query: &Query<&SubVoxel, Without<Player>>,
    params: CollisionParams,
    prefetched: Option<&[Entity]>,
) -> CollisionResult {
    let collision_radius = params.radius;
    let player_min = Vec3::new(
        params.x - collision_radius,
        params.y - params.half_height,
        params.z - collision_radius,
    );
    let player_max = Vec3::new(
        params.x + collision_radius,
        params.y + params.half_height,
        params.z + collision_radius,
    );

    let owned: Vec<Entity>;
    let relevant: &[Entity] = if let Some(pre) = prefetched {
        pre
    } else {
        owned = spatial_grid.get_entities_in_aabb(player_min, player_max);
        &owned
    };

    // ... existing collision loop over `relevant` ...

    // Step-up re-check: reuse `relevant` instead of re-querying
    if let Some(height) = step_up_candidate {
        let new_y = params.current_floor_y + height + params.half_height;
        for entity in relevant {
            // ... check at new_y ...
        }
        return CollisionResult::step_up(height, new_y);
    }

    CollisionResult::no_collision()
}
```

### A.2 Pre-fetch in `move_player`

```rust
// Widen AABB to cover all positions reachable this frame
let prefetch_min = Vec3::new(
    current_pos.x - player.radius - move_delta.x.abs(),
    current_pos.y - player.half_height,
    current_pos.z - player.radius - move_delta.z.abs(),
);
let prefetch_max = Vec3::new(
    current_pos.x + player.radius + move_delta.x.abs(),
    current_pos.y + player.half_height + SUB_VOXEL_SIZE + STEP_UP_TOLERANCE,
    current_pos.z + player.radius + move_delta.z.abs(),
);
let prefetched_entities = spatial_grid.get_entities_in_aabb(prefetch_min, prefetch_max);
```

> Note: `SUB_VOXEL_SIZE` and `STEP_UP_TOLERANCE` are defined in `collision.rs` and must
> be made `pub(super)` or re-exported for use in `player_movement.rs`.
