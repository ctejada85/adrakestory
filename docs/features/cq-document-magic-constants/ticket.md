# Improvement: Replace Magic Numbers With Named Constants and Semantic Types

**Date:** 2026-04-09
**Severity:** Low
**Component:** Game — Loader / Validation / State Machine (`src/main.rs`, `src/systems/game/map/validation/mod.rs`)

---

## Story

As a developer reading the codebase, I want numeric thresholds replaced with named constants or semantic types so that their purpose is immediately clear without needing to trace the surrounding logic.

---

## Description

Two magic numbers exist in the game's core flow:

1. **`0.6` in `check_map_loaded`** (`src/main.rs:365`):
   ```rust
   if map_data.is_some() && (progress.is_complete() || progress.percentage() >= 0.6)
   ```
   The value `0.6` is the `LoadProgress::SpawningVoxels(0.0)` threshold (60 %; see `LoadProgress::percentage()` in `src/systems/game/map/loader/mod.rs:43`). There is no comment explaining why this particular percentage is sufficient to transition to `InGame`, nor why `is_complete()` is not the sole guard. A reader cannot easily tell whether this is intentional fallback logic or a leftover debugging shortcut.

2. **`* 2.0` in entity Y-bound validation** (`src/systems/game/map/validation/mod.rs:166`):
   ```rust
   let max_y = world.height as f32 * 2.0; // Allow some height above world
   ```
   The comment "Allow some height above world" partially explains the intent, but the factor `2.0` is arbitrary and undocumented. Future changes to world height semantics could silently break the intended headroom without anyone noticing.

---

## Acceptance Criteria

1. The `0.6` literal in `check_map_loaded` is replaced with a named constant (e.g. `MIN_PROGRESS_FOR_INGAME_TRANSITION: f32 = 0.6`) with a doc comment explaining why this threshold permits an early transition.
2. Alternatively (preferred), the early-transition path is replaced with `progress.is_complete()` as the sole guard if the fallback is no longer needed, or the `LoadProgress` enum gains a semantic variant (e.g. `ReadyForTransition`) that encodes the intent explicitly.
3. The `2.0` multiplier in validation is replaced with a named constant (e.g. `ENTITY_Y_HEADROOM_FACTOR: f32 = 2.0`) with a doc comment explaining the design intent.
4. `cargo clippy` and `cargo test` pass with no regressions.

---

## Non-Functional Requirements

- No behavior change; named constants must have the same values as the literals they replace.
- The constants must be defined in the same file as their usage or in a dedicated `constants` module if one exists.

---

## Tasks

1. In `src/main.rs`, extract the `0.6` literal into a named constant with a doc comment; OR simplify the guard if the fallback path is vestigial.
2. In `src/systems/game/map/validation/mod.rs`, extract the `2.0` multiplier into a named constant with a doc comment.
3. Run `cargo clippy` and `cargo test` to confirm no regressions.
