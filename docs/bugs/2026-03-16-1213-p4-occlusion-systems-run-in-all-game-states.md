# Bug: Occlusion Systems Run in All Game States, Not Just InGame

**Date:** 2026-03-16  
**Severity:** Low  
**Status:** Open  
**Component:** Occlusion system / System registration  
**Platform:** Cross-platform  

---

## Description

`OcclusionPlugin` registers `detect_interior_system`, `update_occlusion_uniforms`, and `debug_draw_occlusion_zone` in the `Update` schedule without a `run_if(in_state(GameState::InGame))` guard. These systems run in every game state including `IntroAnimation`, `TitleScreen`, `LoadingMap`, and `Paused`. While the systems return early when their required resources are absent (e.g., no `OcclusionMaterialHandle` during loading), the query overhead and early-exit evaluation still occur every frame in non-gameplay states, adding unnecessary cost.

---

## Actual Behavior

- `detect_interior_system` runs every frame in all states, including `TitleScreen` and `LoadingMap`
- `update_occlusion_uniforms` runs every frame in all states; increments `frame_counter` even in `TitleScreen`
- `debug_draw_occlusion_zone` runs every frame in all states; reads `KeyboardInput` in all states
- All three systems perform query evaluation and early-exit checks even when no game entities exist

---

## Expected Behavior

- All three occlusion systems should only run while `GameState::InGame` (or `InGame | Paused` if paused-state occlusion is desired)
- System overhead in non-gameplay states should be zero

---

## Root Cause Analysis

**File:** `src/systems/game/occlusion/mod.rs`  
**Function:** `OcclusionPlugin::build()`  
**Approximate line:** 521

```rust
impl Plugin for OcclusionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<OcclusionMaterial>::default())
            .insert_resource(OcclusionConfig::default())
            .insert_resource(InteriorState::default())
            .add_systems(
                Update,
                (
                    detect_interior_system,
                    update_occlusion_uniforms,
                    debug_draw_occlusion_zone,
                )
                    .chain(),
                // ← missing: .run_if(in_state(GameState::InGame))
            );
    }
}
```

All other gameplay systems in `main.rs` are correctly gated with `.run_if(in_state(GameState::InGame))` or placed in state-specific system sets. The occlusion systems were registered via a plugin and missed the state guard.

---

## Steps to Reproduce

This is a code-quality issue with no visible symptom under normal conditions. To verify:

1. Add `trace!("[occlusion] running")` at the top of `detect_interior_system`
2. Run: `RUST_LOG=trace cargo run --release 2>&1 | grep occlusion`
3. Observe the log emitting during `TitleScreen` and `LoadingMap` states

---

## Suggested Fix

Add a `run_if` state condition to the system chain in `OcclusionPlugin::build()`:

```rust
use crate::states::GameState;

.add_systems(
    Update,
    (
        detect_interior_system,
        update_occlusion_uniforms,
        debug_draw_occlusion_zone,
    )
        .chain()
        .run_if(in_state(GameState::InGame)),
)
```

If paused-state occlusion is desired (e.g., to keep the visual correct while a pause menu is overlaid), use:

```rust
.run_if(in_state(GameState::InGame).or(in_state(GameState::Paused)))
```

---

## Related

- Investigation: `docs/investigations/2026-03-16-0809-macos-fps-drop-many-voxels.md` — Finding 6
