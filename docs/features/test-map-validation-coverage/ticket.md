# Test: Expand Map Validation Unit Test Coverage

**Date:** 2026-04-09
**Severity:** Low
**Component:** Game — Map Validation (`src/systems/game/map/validation/mod.rs`, `src/systems/game/map/validation/tests.rs`)

---

## Story

As a developer, I want the map validation suite to cover entity bound, Y-headroom, and missing-spawn edge cases so that malformed maps are rejected reliably before they reach the spawner.

---

## Description

The existing tests in `src/systems/game/map/validation/tests.rs` cover the default map, zero-dimension worlds, out-of-bounds voxel positions, missing `PlayerSpawn`, duplicate voxel positions, and several `LightSource` property validations. The following cases are untested:

1. **Y-headroom multiplier (`* 2.0`)**: An entity at `y = world.height as f32 * 2.0 + 0.1` (just above the allowed ceiling) should fail validation. An entity at `y = world.height as f32 * 2.0` (exactly at the ceiling) should pass.

2. **Out-of-bounds entity position in X and Z**: An entity at `x = -1.1` (just below the `-1.0` lower bound) should fail. An entity at `x = -0.9` (just inside the lower bound) should pass.

3. **Missing `PlayerSpawn` edge case**: A map with zero entities (no `PlayerSpawn`) should fail validation with a `MissingPlayerSpawn` error (or equivalent).

4. **Multiple `PlayerSpawn` entities**: If the spawner supports only one player spawn, a map with two `PlayerSpawn` entries should either fail validation or the test should document the current behavior explicitly.

---

## Acceptance Criteria

1. A test verifies that an entity at `y = world.height as f32 * 2.0 + 0.1` fails validation.
2. A test verifies that an entity at `y = world.height as f32 * 2.0` passes validation (boundary is inclusive).
3. A test verifies that an entity at `x = -1.1` fails validation.
4. A test verifies that an entity at `x = -0.9` passes validation.
5. A test verifies that a map with no entities fails with a missing-spawn error (if the validator enforces this; if not, the test documents the current behavior).
6. A test documents the behavior for two `PlayerSpawn` entities (pass or fail, with explanation).
7. All new tests pass with `cargo test map::validation`.

---

## Non-Functional Requirements

- Tests must use the same `MapData` builder helpers used in the existing test file.
- No new public API is introduced; tests exercise `validate_map` directly.

---

## Tasks

1. Add Y-headroom boundary tests (just above and exactly at `height * 2.0`) to `src/systems/game/map/validation/tests.rs`.
2. Add X/Z lower-bound boundary tests (`-1.1` fails, `-0.9` passes).
3. Add a test for a map with zero entities (clarify expected outcome from reading the validator source).
4. Add a test documenting the two-`PlayerSpawn` behavior.
5. Run `cargo test` and confirm all pass.
