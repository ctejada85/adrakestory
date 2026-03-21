# VSync Configuration

**Date:** 2026-03-21
**Type:** Epic
**Component:** Display Settings / Window

---

## Overview

The game currently renders with `PresentMode::AutoNoVsync`, running as fast as possible with no tearing prevention and no frame rate cap. This epic adds a VSync toggle and a refresh-rate multiplier to the display settings, letting players choose tear-free rendering at the monitor's native refresh rate or a lower fraction of it (e.g., 30 fps on a 60 Hz display for battery saving). Settings persist in `settings.ron` and can be changed at runtime via the in-game settings screen.

---

## Epic Story

As a player, I want to enable VSync and choose a frame rate target relative to my monitor's refresh rate so that I can balance visual smoothness, power consumption, and input latency according to my hardware and preferences.

---

## Child Stories

| # | Title | File | Status |
|---|-------|------|--------|
| 1 | VSync toggle and runtime `PresentMode` mutation | [./stories/story-1-vsync-toggle.md] | Backlog |
| 2 | Refresh rate multiplier and software frame cap | [./stories/story-2-multiplier.md] | Backlog |
| 3 | Settings UI and persistence | [./stories/story-3-ui-persistence.md] | Backlog |

---

## Story 1 — VSync Toggle

As a player, I want to enable or disable VSync in the settings so that frames are synchronized to my monitor's refresh rate.

### Description

Currently `PresentMode::AutoNoVsync` is hardcoded in `main.rs` and never changed. This story introduces `VsyncConfig` (a new serializable resource), `apply_vsync_system`, and `detect_monitor_refresh_system`. When `vsync_enabled` is toggled, the system mutates `Window.present_mode` to `Fifo` or `AutoNoVsync` in the same frame without requiring a restart.

Out of scope for this story: the multiplier UI, the software frame cap, and `settings.ron` persistence (covered in Stories 2 and 3).

### Acceptance Criteria

1. `VsyncConfig` resource is inserted at app startup with `vsync_enabled = false` (preserves current behavior).
2. Changing `vsync_enabled` to `true` updates `Window.present_mode` to `PresentMode::Fifo` within one frame.
3. Changing `vsync_enabled` to `false` reverts `Window.present_mode` to `PresentMode::AutoNoVsync` within one frame.
4. `detect_monitor_refresh_system` runs at startup and populates `MonitorInfo.refresh_hz`; falls back to 60.0 and logs a warning if detection fails.
5. No existing behavior is changed when `vsync_enabled = false`.

### Non-Functional Requirements

- `apply_vsync_system` MUST be gated by `VsyncConfig.dirty` to avoid per-frame `Window` mutations.
- The system MUST run after `GameSystemSet::Input` and before `GameSystemSet::Camera`.
- No regression in frame timing when VSync is disabled.

### Tasks

1. Create `src/systems/settings/vsync.rs` with `VsyncConfig`, `MonitorInfo`, `apply_vsync_system`, `detect_monitor_refresh_system`.
2. Register `detect_monitor_refresh_system` as a `Startup` system in `SettingsPlugin`.
3. Register `apply_vsync_system` in the `Update` schedule in `SettingsPlugin`.
4. Insert `VsyncConfig::default()` and `MonitorInfo` resources in `SettingsPlugin`.
5. Write unit tests for `VsyncConfig::default()` and the dirty-flag gate.
6. Manual verification: toggle VSync at runtime via a debug key or Rust test; confirm `present_mode` changes.

---

## Story 2 — Refresh Rate Multiplier and Software Frame Cap

As a player, I want to set a frame rate target that is a fraction of my monitor's refresh rate so that I can reduce power consumption (e.g., 30 fps on a 60 Hz display).

### Description

This story adds `vsync_multiplier: f32` to `VsyncConfig` and integrates `bevy_framepace` as a frame rate limiter. When VSync is enabled and the multiplier is less than 1.0, `apply_vsync_system` computes `target_fps = refresh_hz × multiplier` and sets the `bevy_framepace` limiter accordingly. When multiplier = 1.0 or VSync is disabled, the limiter is set to `Limiter::Auto`.

### Acceptance Criteria

1. `vsync_multiplier` defaults to `1.0`.
2. Setting `vsync_multiplier = 0.5` with `vsync_enabled = true` caps frame rate to `refresh_hz × 0.5` fps (e.g., 30 fps on 60 Hz).
3. Setting `vsync_multiplier = 1.0` with `vsync_enabled = true` runs at native refresh rate with no software cap.
4. Values outside `[0.25, 4.0]` are clamped and a warning is logged.
5. Frame limiter is disabled (`Limiter::Auto`) when `vsync_enabled = false`.

### Non-Functional Requirements

- Frame cap jitter MUST be ≤ 1 ms over 60 seconds (measured via `bevy_framepace` built-in diagnostics).
- `bevy_framepace` MUST be added as a `Cargo.toml` dependency; version must be compatible with Bevy 0.18.

### Tasks

1. Add `bevy_framepace` to `Cargo.toml`; verify compatibility with Bevy 0.18.
2. Extend `VsyncConfig` with `vsync_multiplier: f32` and `#[serde(default = "default_vsync_multiplier")]`.
3. Extend `apply_vsync_system` to compute and apply the `bevy_framepace` limiter target.
4. Add multiplier clamping (`[0.25, 4.0]`) with `warn!()` logging for out-of-range values.
5. Write unit tests for multiplier clamping and target FPS calculation.
6. Manual verification: set multiplier = 0.5 on a 60 Hz display; confirm FPS counter reads ~30.

---

## Story 3 — Settings UI and Persistence

As a player, I want to change VSync settings in the in-game settings screen and have my choices saved so that they apply automatically on the next launch.

### Description

This story extends `SettingsPlugin` to load and save `VsyncConfig` alongside `OcclusionConfig` using the same `settings.ron` file. It also adds a "Display" section to the in-game settings UI with a VSync toggle and a multiplier dropdown (visible only when VSync is enabled). Existing `settings.ron` files without VSync fields load correctly via `#[serde(default)]`.

### Acceptance Criteria

1. `settings.ron` is extended with `vsync_enabled` and `vsync_multiplier`; fields are written on exit and read on startup.
2. Loading a `settings.ron` without VSync fields produces `VsyncConfig::default()` (no panic, no data loss).
3. The settings UI displays a "VSync" toggle under a "Display" heading.
4. The multiplier dropdown (options: 0.25×, 0.5×, 1.0×) is visible only when the VSync toggle is enabled.
5. Changing a setting in the UI immediately applies it (dirty flag → `apply_vsync_system`).

### Non-Functional Requirements

- Serialization round-trip: `VsyncConfig` written to RON and re-read MUST produce identical values.
- Settings UI MUST remain responsive; no frame stutter when opening the panel.

### Tasks

1. Extend `load_settings` in `src/systems/settings/mod.rs` to deserialize `VsyncConfig` from `settings.ron`.
2. Extend `save_settings` to serialize `VsyncConfig` back to `settings.ron`.
3. Add a "Display" section to the settings egui panel with the VSync toggle and multiplier dropdown.
4. Wire UI controls to `VsyncConfig` fields and set `dirty = true` on change.
5. Write a round-trip serialization test for `VsyncConfig`.
6. Manual verification: change VSync in UI → exit → relaunch → confirm settings restored.

---

## Epic Acceptance Criteria

1. All three child stories are complete and individually verified.
2. VSync can be toggled at runtime with no restart required.
3. Frame rate cap is active and accurate (±1 ms jitter) when multiplier < 1.0.
4. Settings persist correctly across restarts; existing `settings.ron` files load without error.
5. No regression in rendering, collision, or occlusion behavior.

---

## Epic Non-Functional Requirements

- All changes MUST compile without warnings under `cargo clippy`.
- VSync config changes MUST NOT trigger `PipelineCache` invalidation or shader recompilation.
- Feature MUST work on macOS, Windows, and Linux.

---

## Dependencies & Risks

| # | Item | Type | Status | Notes |
|---|------|------|--------|-------|
| 1 | Bevy 0.18 `Window` component mutation at runtime | Dependency | Resolved | Available in Bevy 0.18 |
| 2 | `bevy_framepace` compatibility with Bevy 0.18 | Dependency | Pending | Verify version before starting Story 2 |
| 3 | Monitor refresh rate API availability on all platforms | Risk | Open | Fallback to 60 Hz mitigates; test on each OS |
