# Architecture — Fix Occlusion Systems State Guard

---

## 1. Current Architecture

### 1.1 System Registration (Current)

```rust
// src/systems/game/occlusion/mod.rs — OcclusionPlugin::build()
app.add_systems(
    Update,
    (
        detect_interior_system,
        update_occlusion_uniforms,
        debug_draw_occlusion_zone,
    )
        .chain(),
    // ← no run_if guard
);
```

### 1.2 Problem

The Bevy scheduler dispatches all `Update` systems every frame regardless of state, then evaluates `run_if` conditions. Without a condition, the three occlusion systems are always dispatched. Each system performs query evaluation and resource lookups on every frame in every state, even when no game entities exist.

All other gameplay systems in `main.rs` use one of:
```rust
.run_if(in_state(GameState::InGame))
.run_if(in_state(GameState::Paused))
```

The occlusion plugin was registered independently and missed this pattern.

---

## 2. Target Architecture

### 2.1 System Registration (After Fix)

```rust
use bevy::prelude::in_state;
use crate::states::GameState;

app.add_systems(
    Update,
    (
        detect_interior_system,
        update_occlusion_uniforms,
        debug_draw_occlusion_zone,
    )
        .chain()
        .run_if(in_state(GameState::InGame).or(in_state(GameState::Paused))),
);
```

### 2.2 State Dispatch Table

| GameState | Before | After |
|-----------|--------|-------|
| `IntroAnimation` | Dispatched (early-exit) | **Skipped** |
| `TitleScreen` | Dispatched (early-exit) | **Skipped** |
| `LoadingMap` | Dispatched (early-exit) | **Skipped** |
| `InGame` | Dispatched (full run) | Dispatched (full run) — unchanged |
| `Paused` | Dispatched (full run) | Dispatched (full run) — unchanged |
| `Settings` | Dispatched (early-exit) | **Skipped** |

### 2.3 Why Include `Paused`

`GameState::Paused` renders the pause menu as a 2D overlay on top of the live game world. Voxel chunks remain visible in the background. If occlusion systems stopped running on pause, the occlusion transparency effect would freeze in its last state for the duration of the pause — acceptable but slightly incorrect if the camera or player position were last updated before pausing.

Including `Paused` costs nothing (the player is stationary, so `update_occlusion_uniforms` will find no changed transforms and skip the GPU upload via the two-level cache).

---

## 3. Invariants

| Invariant | How Maintained |
|-----------|---------------|
| System chain order preserved | `.chain()` is placed before `.run_if()` — both are applied to the same `SystemConfigs` |
| No resource changes | `Option<Res<...>>` guards in each system are preserved and remain valid safety nets |
| No import changes needed | `in_state` and `GameState` are already in scope in `occlusion/mod.rs` |
