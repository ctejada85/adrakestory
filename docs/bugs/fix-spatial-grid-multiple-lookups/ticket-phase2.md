# User Story: Share Spatial Grid Lookup Between Movement and Physics (Phase 2)

**Ticket ID:** fix-spatial-grid-multiple-lookups-phase2  
**Priority:** P2  
**Component:** Collision system / Physics / Player movement  
**Depends on:** `docs/bugs/fix-spatial-grid-multiple-lookups/ticket.md` (Phase 1 must be complete)  
**Bug:** `docs/bugs/2026-03-16-1213-p2-spatial-grid-multiple-lookups-per-frame.md`  
**Requirements:** `docs/bugs/fix-spatial-grid-multiple-lookups/requirements.md`  
**Architecture:** `docs/bugs/fix-spatial-grid-multiple-lookups/architecture.md`

---

## Story

**As a** player on a voxel-dense map,  
**I want** `apply_physics` to reuse the spatial grid entity slice already computed by
`move_player` rather than issuing its own independent query,  
**so that** total spatial grid queries per frame drop from 2 to 1 on frames where the
player is moving.

---

## Context

Phase 1 reduced `move_player` from up to 4 AABB lookups to 1. After Phase 1, the
per-frame count is:

| Frame condition | Queries |
|-----------------|---------|
| Player moving | 1 (`move_player`) + 1 (`apply_physics`) = **2** |
| Player standing still | 0 (`move_player` skips) + 1 (`apply_physics`) = **1** |

Phase 2 targets the "player moving" case, reducing it to **1 total query per frame** by
storing the pre-fetched entity slice in a `Local` resource (or a lightweight `ResMut`)
that `apply_physics` reads on the same frame.

---

## Approach

Because `move_player` runs in `GameSystemSet::Movement` and `apply_physics` runs in
`GameSystemSet::Physics` (Movement always before Physics in the same frame), it is safe
for `move_player` to write a cached slice into a resource and for `apply_physics` to read
it.

### New resource

```rust
/// Entities near the player pre-fetched by move_player for reuse by apply_physics.
///
/// Cleared at the start of each `move_player` run. `apply_physics` reads this if
/// non-empty, otherwise falls back to its own query.
#[derive(Resource, Default)]
pub struct PreFetchedCollisionEntities {
    pub entities: Vec<Entity>,
}
```

### `move_player` writes the cache

After computing `prefetched_entities` (the widened AABB result from Phase 1):

```rust
pre_fetched.entities.clone_from(&prefetched_entities);
// or move if prefetched_entities is not needed after this point:
pre_fetched.entities = prefetched_entities;
```

The resource must be cleared at the start of `move_player` so a stale slice from the
previous frame is never used when the player is not moving:

```rust
pre_fetched.entities.clear();
```

### `apply_physics` reads the cache

```rust
pub fn apply_physics(
    ...
    pre_fetched: Res<PreFetchedCollisionEntities>,
) {
    let relevant: &[Entity] = if !pre_fetched.entities.is_empty() {
        &pre_fetched.entities
    } else {
        // fallback: player not moving this frame (move_player cleared the cache)
        // issue own query as before
        ...
    };
}
```

The fallback path keeps the system correct when the player is standing still (gravity
and landing still need to work).

---

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC-1 | `PreFetchedCollisionEntities` resource is registered in the game plugin |
| AC-2 | `move_player` clears the resource at its start and writes the pre-fetched slice after computing it |
| AC-3 | `apply_physics` uses the cached slice when non-empty; falls back to its own query when empty |
| AC-4 | Total `get_entities_in_aabb` calls per frame when player is moving = 1 |
| AC-5 | Total `get_entities_in_aabb` calls per frame when player is standing still = 1 (apply_physics fallback) |
| AC-6 | Ground detection and ceiling detection in `apply_physics` produce identical results to Phase 1 |
| AC-7 | Unit tests cover: resource populated → `apply_physics` uses cache; resource empty → fallback query runs |
| AC-8 | `cargo test --lib` passes with zero new failures |
| AC-9 | `cargo clippy --lib` reports zero errors |

---

## Non-Functional Requirements

| # | Requirement |
|---|-------------|
| NFR-1 | `PreFetchedCollisionEntities` is a lightweight wrapper — no additional bookkeeping beyond a `Vec<Entity>` |
| NFR-2 | System ordering is not changed; `GameSystemSet` sequence (Input → Movement → Physics → Visual → Camera) is preserved |
| NFR-3 | The resource must not outlive a single frame — clear at start of `move_player` is sufficient |
| NFR-4 | No change to `SpatialGrid` or `check_sub_voxel_collision` signatures |

---

## Tasks

1. **Define and register `PreFetchedCollisionEntities` resource**
   - Add struct to `src/systems/game/resources.rs`
   - Register with `app.init_resource::<PreFetchedCollisionEntities>()` in the game plugin

2. **Update `move_player` to write the resource**
   - Accept `ResMut<PreFetchedCollisionEntities>` parameter
   - Clear at the start of the function (before the movement guard)
   - After computing `prefetched_entities`, assign to `pre_fetched.entities`

3. **Update `apply_physics` to read the resource**
   - Accept `Res<PreFetchedCollisionEntities>` parameter
   - Use cached entities when non-empty; fall back to `get_entities_in_aabb` otherwise

4. **Write unit tests**
   - `prefetched_entities_used_in_apply_physics_when_populated` — verify fallback is skipped
   - `apply_physics_falls_back_when_cache_empty` — verify own query runs when cache is empty

5. **Validate**
   - `cargo test --lib`
   - `cargo clippy --lib`
   - `cargo build --release`

6. **Commit**
   - Conventional commit: `perf(collision): share pre-fetched entities between move_player and apply_physics`
