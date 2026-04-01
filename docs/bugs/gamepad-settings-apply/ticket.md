# Gamepad Settings Apply

**Date:** 2026-04-01
**Component:** Gamepad / Settings

---

## Story

As a player using a gamepad, I want my sensitivity and camera inversion preferences to be applied during gameplay so that the controller feels comfortable to use.

---

## Description

`GamepadSettings` (`src/systems/game/gamepad.rs`) has three fields — `trigger_deadzone`, `invert_camera_y`, and `camera_sensitivity` — that are never read by any system. The input gathering code only reads `stick_deadzone` and `movement_sensitivity`. Players who change these settings (once a settings UI exists) would see no effect. This ticket wires the three unused fields into the relevant input-reading systems: `trigger_deadzone` into trigger polling, `invert_camera_y` into the camera delta calculation, and `camera_sensitivity` into the camera delta calculation.

---

## Acceptance Criteria

1. When `GamepadSettings::camera_sensitivity` is changed, the camera rotation speed on the right stick changes proportionally.
2. When `GamepadSettings::invert_camera_y` is `true`, vertical camera input from the right stick is negated.
3. When `GamepadSettings::trigger_deadzone` is increased, small trigger presses below the threshold produce no action.
4. Changing any of these settings at runtime (mid-game) takes effect within one frame.
5. Default values (`camera_sensitivity: 3.0`, `invert_camera_y: false`, `trigger_deadzone: 0.1`) preserve existing gameplay behaviour exactly.
6. Unit tests verify that `invert_camera_y` flips the Y delta, and that `camera_sensitivity` scales the delta.

---

## Non-Functional Requirements

- Must not introduce a new ECS system; the fix belongs in the existing `gather_gamepad_input` system (`src/systems/game/gamepad.rs`).
- Must not affect keyboard/mouse input paths.
- Game binary only — the editor does not use `GamepadSettings`.

---

## Tasks

1. Read `GamepadSettings` resource in `gather_gamepad_input`.
2. Apply `trigger_deadzone` when polling `GamepadButton::LeftTrigger2` / `RightTrigger2`.
3. Apply `camera_sensitivity` as a multiplier to the right-stick camera delta.
4. Apply `invert_camera_y` by negating `camera_delta.y` when the flag is set.
5. Write unit tests for `invert_camera_y` and `camera_sensitivity` scaling.
6. Manually verify all three settings produce the correct in-game effect with a connected gamepad.
