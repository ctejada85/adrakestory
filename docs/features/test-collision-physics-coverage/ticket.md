# Test: Expand Collision and Physics Unit Test Coverage

**Date:** 2026-04-09
**Severity:** Medium
**Component:** Game — Collision / Physics (`src/systems/game/collision/`, `src/systems/game/physics/`)

---

## Story

As a developer, I want unit tests for the player collision and physics logic so that regressions in step-up, wall collision, gravity, and NPC push behavior are caught before they reach the game.

---

## Description

`src/systems/game/collision/tests.rs` currently has six tests covering `CollisionResult` constructors, `CollisionParams` creation, and two prefetch-slice scenarios. `src/systems/game/physics/tests.rs` has two tests covering the prefetch-cache reuse logic. The core gameplay behaviors — step-up eligibility, wall blocking, NPC push direction, gravity clamping, and cylinder-vs-AABB overlap — have no direct unit tests.

Missing coverage:

- **`check_sub_voxel_collision` step-up path**: a sub-voxel at exactly one sub-voxel height above `current_floor_y` should produce `CollisionResult::step_up`; a sub-voxel two sub-voxels tall should produce `CollisionResult::blocking`.
- **Wall collision**: a sub-voxel fully within the player's Y range should block horizontal movement.
- **Cylinder-vs-AABB XZ miss**: a sub-voxel positioned just outside the cylinder radius in XZ should produce no collision.
- **NPC push fallback direction** (related to `bug-npc-push-arbitrary-direction`): when `horizontal_distance ≈ 0`, the push must not always be `+X`.
- **`apply_gravity` delta clamp**: `Player::velocity.y` must not change by more than `GRAVITY * 0.1` in a single frame regardless of delta input.

---

## Acceptance Criteria

1. A test in `src/systems/game/collision/tests.rs` verifies that a sub-voxel at `obstacle_height == SUB_VOXEL_SIZE` produces `can_step_up == true`.
2. A test verifies that a sub-voxel at `obstacle_height == SUB_VOXEL_SIZE * 2.0` produces `has_collision == true` and `can_step_up == false`.
3. A test verifies that a sub-voxel positioned `radius + epsilon` away in XZ produces `has_collision == false`.
4. A test verifies that a sub-voxel fully overlapping the player's Y range and within cylinder radius produces `has_collision == true`.
5. A test in `src/systems/game/physics/tests.rs` verifies that the NPC push fallback branch does not translate the player exactly `+X` (after `bug-npc-push-arbitrary-direction` is fixed; may be written before and marked `#[ignore]`).
6. A test verifies that `apply_gravity` clamps a very large delta (e.g. `dt = 10.0`) so `velocity.y` changes by at most `GRAVITY * 0.1`.
7. All new tests pass with `cargo test`.

---

## Non-Functional Requirements

- Tests for `check_sub_voxel_collision` must construct minimal `SubVoxel` and `SpatialGrid` instances without a running Bevy app; the existing test helpers in `collision/tests.rs` should be reused or extended.
- Tests for `apply_gravity` and `apply_npc_collision` may need to extract the pure arithmetic from the Bevy system parameter signature into testable free functions if Bevy ECS is not practical to instantiate in unit tests.

---

## Tasks

1. Add step-up eligibility tests (height = SUB_VOXEL_SIZE and height = 2×) to `src/systems/game/collision/tests.rs`.
2. Add XZ-miss and full-body-overlap tests to `src/systems/game/collision/tests.rs`.
3. Extract the gravity delta calculation from `apply_gravity` into a pure function (or test via `apply_physics` with a mock grid) and add a clamping test.
4. Add an NPC push fallback direction test to `src/systems/game/physics/tests.rs` (mark `#[ignore]` until `bug-npc-push-arbitrary-direction` is resolved if needed).
5. Run `cargo test game::collision` and `cargo test game::physics` and confirm all pass.
