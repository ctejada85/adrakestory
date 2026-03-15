# Bug: Camera Follow Lerp Is Frame-Rate-Dependent and Causes Visible Lag

**Date:** 2026-03-15  
**Severity:** Medium  
**Status:** Open  
**Component:** Camera  

---

## Description

The `follow_player_camera` system interpolates the camera toward the player each frame using `lerp(target, follow_speed * delta_secs)`. With `follow_speed = 5.0` at 60 fps, the lerp factor is approximately `0.083`, meaning the camera closes only ~8% of the remaining gap per frame. This creates a camera that perpetually trails the player with a perceived lag of 400–500ms before reaching 99% convergence. Additionally, the interpolation factor is frame-rate-dependent, causing the camera to behave differently at different frame rates.

---

## Actual Behavior

- Camera continuously lags behind the player during movement
- At 60 fps: ~25 frames (~417ms) to reach 99% of target position
- At 30 fps: ~13 frames (~433ms) — similar time, but larger per-frame jumps feel jerkier
- At 120 fps: ~50 frames (~417ms) — more micro-steps but same total lag
- The same issue affects `rotate_camera()` via `slerp(target, rotation_speed * delta)`
- Movement feels "floaty" and disconnected from input

---

## Expected Behavior

- Camera should follow the player responsively with minimal perceptible delay
- Camera behavior (time to reach target) should be identical regardless of frame rate
- The lerp formula should use frame-rate-independent exponential decay

---

## Root Cause Analysis

**File:** `src/systems/game/camera.rs`  
**Function:** `follow_player_camera()`  
**Approximate line:** 43

```rust
camera_transform.translation = camera_transform.translation.lerp(
    target_position,
    game_camera.follow_speed * time.delta_secs(),  // follow_speed = 5.0
);
```

`lerp(a, b, t)` where `t = speed * delta` is a common approximation of exponential decay, but it is **not frame-rate-independent**. The correct formulation uses the natural exponential:

```
alpha = 1 - e^(-speed * delta)
```

With this formula, the convergence time is the same regardless of whether the game runs at 30, 60, or 120 fps.

Additionally, `follow_speed = 5.0` is too low for a responsive third-person camera. A value of 15–20 is more typical for a game-feel-appropriate follow speed that minimises perceptible lag.

**File:** `src/systems/game/map/spawner/mod.rs` (~line 484)
```rust
follow_speed: 5.0,   // Set at camera spawn time
```

---

## Steps to Reproduce

1. Run: `cargo run --release`
2. Move the player character in any direction
3. Observe that the camera visibly lags behind the player rather than staying locked to it
4. Compare to the map editor camera, which responds immediately to input

---

## Suggested Fix

**Fix lerp formula** in `follow_player_camera()`:
```rust
// Frame-rate-independent exponential decay
let alpha = 1.0 - (-game_camera.follow_speed * time.delta_secs()).exp();
camera_transform.translation = camera_transform.translation.lerp(target_position, alpha);
```

Apply the same fix to the `slerp` in `rotate_camera()`.

**Increase follow speed** at spawn site (`src/systems/game/map/spawner/mod.rs`):
```rust
follow_speed: 15.0,  // Was 5.0 — much more responsive
```

---

## Related

- Investigation: [2026-03-15-2141-game-binary-graphics-lag.md](../investigations/2026-03-15-2141-game-binary-graphics-lag.md)
