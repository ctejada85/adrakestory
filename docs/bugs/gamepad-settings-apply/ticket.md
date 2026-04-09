# Gamepad Settings Apply

**Date:** 2026-04-01
**Component:** Gamepad / Settings

---

## Story

As a player using a gamepad, I want my sensitivity and camera inversion preferences to be applied during gameplay so that the controller feels comfortable to use.

---

## Description

`GamepadSettings` (`src/systems/game/gamepad.rs`) has three fields — `trigger_deadzone`, `invert_camera_y`, and `camera_sensitivity` — that are never read by any system. The input gathering code only reads `stick_deadzone` and `movement_sensitivity`. Players who change these settings (once a settings UI exists) would see no effect. This ticket wires two of those unused fields into the relevant input-reading systems: `trigger_deadzone` into trigger polling, and `invert_camera_y` into the look direction calculation. `camera_sensitivity` is intentionally left unwired.

---

## Acceptance Criteria

1. When `GamepadSettings::invert_camera_y` is `true`, vertical camera input from the right stick is negated.
2. When `GamepadSettings::trigger_deadzone` is increased, small trigger presses below the threshold produce no action.
3. Changing any of these settings at runtime (mid-game) takes effect within one frame.
4. Default values (`invert_camera_y: false`, `trigger_deadzone: 0.1`) preserve existing gameplay behaviour exactly.
5. Unit tests verify that `invert_camera_y` flips the Y delta.

---

## Non-Functional Requirements

- Must not introduce a new ECS system; the fix belongs in the existing `gather_gamepad_input` system (`src/systems/game/gamepad.rs`).
- Must not affect keyboard/mouse input paths.
- Game binary only — the editor does not use `GamepadSettings`.

---

## Tasks

1. Read `GamepadSettings` resource in `gather_gamepad_input` (it is already a parameter — remove `#[allow(dead_code)]` from `trigger_deadzone` and `invert_camera_y`).
2. Apply `trigger_deadzone` by reading `GamepadAxis::LeftZ` / `RightZ` directly; gate on `value >= trigger_deadzone`. Store results in new `PlayerInput::left_trigger` / `right_trigger` fields.
3. Apply `invert_camera_y` by negating `look_direction.y` when the flag is set, after deadzone.
4. Write unit tests for `invert_camera_y` flipping the Y component.
5. Manually verify both settings produce the correct in-game effect with a connected gamepad.

---

## Documents

- [Requirements](./requirements.md)
- [Architecture](./architecture.md)
