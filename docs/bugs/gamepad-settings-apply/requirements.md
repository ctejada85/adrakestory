# Requirements — Gamepad Settings Apply

**Source:** `docs/bugs/gamepad-settings-apply/ticket.md` — 2026-04-01
**Status:** Draft

---

## 1. Overview

`GamepadSettings` (`src/systems/game/gamepad.rs`) declares three fields — `trigger_deadzone`, `invert_camera_y`, and `camera_sensitivity` — that carry `#[allow(dead_code)]` annotations because nothing reads them. The `gather_gamepad_input` system only consumes `stick_deadzone` and `movement_sensitivity`.

This feature wires two of those unused fields — `trigger_deadzone` and `invert_camera_y` — into `gather_gamepad_input`. `camera_sensitivity` is intentionally left unwired. No new ECS systems, resources, or crates are required. The change is entirely local to `gather_gamepad_input` in `src/systems/game/gamepad.rs`.

---

## 3. Functional Requirements

### 3.1 Camera Y-Axis Inversion

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1.1 | When `GamepadSettings::invert_camera_y` is `true`, the Y component of the right-stick look vector must be negated before storing in `PlayerInput::look_direction`. | Phase 1 |
| FR-3.1.2 | When `invert_camera_y` is `false` (the default), vertical camera input must be unchanged. | Phase 1 |
| FR-3.1.3 | Inversion is applied after deadzone. | Phase 1 |

### 3.2 Trigger Deadzone

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.2.1 | When polling `GamepadButton::LeftTrigger2` and `GamepadButton::RightTrigger2`, raw axis values below `GamepadSettings::trigger_deadzone` must produce no action. | Phase 1 |
| FR-3.2.2 | The default value `trigger_deadzone: 0.1` must preserve existing trigger behaviour exactly. | Phase 1 |
| FR-3.2.3 | Changing `trigger_deadzone` at runtime must take effect within the same frame. | Phase 1 |

### 3.3 Testing

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.3.1 | A unit test must verify that `invert_camera_y: true` negates the Y component of `look_direction` relative to `invert_camera_y: false`. | Phase 1 |

---

## 4. Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-4.1 | No new ECS system may be introduced. All changes must live inside the existing `gather_gamepad_input` function. | Phase 1 |
| NFR-4.2 | Keyboard and mouse input paths (`gather_keyboard_input`, camera mouse-delta) must not be affected. | Phase 1 |
| NFR-4.3 | The editor binary must not be affected. `GamepadSettings` is a game-only resource. | Phase 1 |
| NFR-4.4 | `#[allow(dead_code)]` annotations on `trigger_deadzone` and `invert_camera_y` must be removed once they are read. | Phase 1 |

---

## 5. Phase Scoping

### Phase 1 — MVP

- Apply `invert_camera_y` negation to right-stick look vector
- Apply `trigger_deadzone` when polling trigger buttons
- Remove `#[allow(dead_code)]` from `trigger_deadzone` and `invert_camera_y`
- Unit tests for Y-axis inversion

### Future Phases

- Settings UI (separate ticket) that exposes these values to the player
- `camera_sensitivity` wiring (intentionally deferred)

---

## 6. Assumptions & Constraints

| # | Assumption / Constraint |
|---|------------------------|
| 1 | The right stick controls `PlayerInput::look_direction`, not `PlayerInput::camera_delta`. The comment at `gamepad.rs:182` ("Camera delta is not used from right stick anymore") confirms this. Inversion therefore targets `look_direction`. |
| 2 | Triggers (`LeftTrigger2`, `RightTrigger2`) are not yet polled in `gather_gamepad_input`. The deadzone check requires adding that polling. |
| 3 | `GamepadSettings` is already inserted as a `Resource` with correct defaults and will not need structural changes. |
| 4 | Bevy's `Gamepad::get(GamepadAxis::*)` returns raw `f32` values in `[-1.0, 1.0]`. Trigger axes must be identified and polled separately from stick axes. |

---

## 7. Resolved Questions

| # | Question | Resolution |
|---|----------|------------|
| 1 | Which `GamepadAxis` variant maps to `LeftTrigger2` / `RightTrigger2` in Bevy 0.15? | `GamepadAxis::LeftZ` (left trigger) and `GamepadAxis::RightZ` (right trigger) — confirmed in `src/editor/controller/input.rs`. |
| 2 | Should trigger deadzone affect `just_pressed` detection (digital threshold) or raw axis value? | Raw axis value. Read `GamepadAxis::LeftZ`/`RightZ` directly and gate on `value >= trigger_deadzone`. `GamepadButton::LeftTrigger2`/`RightTrigger2` use Bevy's internal threshold and cannot honour a custom deadzone. |

---

## 8. Dependencies & Blockers

| # | Dependency | Status | Owner |
|---|-----------|--------|-------|
| 1 | `GamepadSettings` resource already inserted in `main.rs` | Done | — |
| 2 | `gather_gamepad_input` already takes `Res<GamepadSettings>` as a parameter | Done | — |

---

*Created: 2026-04-08*
*Source: docs/bugs/gamepad-settings-apply/ticket.md*
