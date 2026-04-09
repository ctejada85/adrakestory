# Bug: NPC Overlap Fallback Always Pushes Player in the +X Direction

**Date:** 2026-04-09
**Severity:** Medium
**Component:** Game — Physics (`src/systems/game/physics/mod.rs`)

---

## Story

As a player, I want the game to push me away from an NPC in the correct direction when I stand exactly on top of one so that I am not teleported sideways without cause.

---

## Description

`apply_npc_collision` (lines 190–231 of `src/systems/game/physics/mod.rs`) separates the player from overlapping NPCs by computing the horizontal push direction from `(dx, dz)`. When `horizontal_distance <= 0.001` (i.e. the player and NPC centres share nearly identical XZ coordinates), the else-branch on line 227 fires:

```rust
player_transform.translation.x += min_distance;
```

This unconditionally pushes the player in the `+X` world direction by `min_distance` (up to `0.7` units), regardless of map geometry, the player's facing direction, or any other context. A player standing on an NPC will be snapped sideways in the same fixed direction every frame until separation is achieved, which can push them into a wall or off a ledge.

The fix is to push in the player's current facing direction, or along the camera-forward vector projected onto XZ, or simply along a deterministic but less game-breaking direction (e.g. `+Z` or a random unit vector seeded once per encounter).

---

## Acceptance Criteria

1. When the player is within `horizontal_distance <= 0.001` of an NPC, the fallback push direction is not always `+X`.
2. The fallback push still separates the player from the NPC within one or two frames.
3. The normal case (`horizontal_distance > 0.001`) is unchanged.
4. A unit test exercises the fallback branch and asserts the push distance equals `min_distance` in the chosen direction.

---

## Non-Functional Requirements

- The fix must not introduce any per-frame heap allocation in the collision system.
- The fallback case is rare; the chosen direction does not need to be perfectly optimal, only non-degenerate.

---

## Tasks

1. In the else-branch of `apply_npc_collision`, replace the hard-coded `+X` push with the player's current facing direction projected onto XZ (or another non-degenerate alternative such as `Vec3::Z`).
2. Add a comment explaining the fallback strategy.
3. Add a unit test in `src/systems/game/physics/tests.rs` for the `horizontal_distance ≈ 0` case.
4. Manual verification: place an NPC, walk directly on top of it, observe the player is pushed in a sensible direction.
