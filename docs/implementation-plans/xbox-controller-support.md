# Xbox Controller Support Implementation Plan

## Status: ✅ IMPLEMENTED

Xbox controller support has been fully implemented for the main game.

## Overview

This document outlines the implementation plan for adding Xbox controller support to A Drake's Story for the main game.

## Goals

1. ✅ **Full game playability** with Xbox controller
2. ✅ **Seamless input switching** between keyboard/mouse and controller
3. ✅ **Intuitive control mapping** that feels natural for the game's mechanics

---

## Research & Dependencies

### Bevy Gamepad Support

Bevy has built-in gamepad support via the `bevy::input::gamepad` module:

```rust
use bevy::input::gamepad::{
    Gamepad, GamepadAxis, GamepadAxisType, GamepadButton, GamepadButtonType,
    GamepadConnection, GamepadConnectionEvent, GamepadEvent,
};
```

**Key Resources:**
- `Res<ButtonInput<GamepadButton>>` - Button press state
- `Res<Axis<GamepadAxis>>` - Analog stick/trigger values
- `EventReader<GamepadConnectionEvent>` - Controller connect/disconnect events

### No Additional Dependencies Required

Bevy's gamepad support works out of the box on Windows with XInput (Xbox controllers).

---

## Control Mapping

### Game Controls

| Action | Xbox Controller | Current Keyboard/Mouse |
|--------|-----------------|------------------------|
| **Movement** | Left Stick | WASD |
| **Camera Orbit** | Right Stick | Mouse movement |
| **Jump** | A Button | Space |
| **Interact** | X Button | E |
| **Pause Menu** | Start Button | Escape |
| **Camera Reset** | Right Stick Click (R3) | Home |

### Menu/UI Navigation

| Action | Xbox Controller | Current Keyboard/Mouse |
|--------|-----------------|------------------------|
| **Navigate** | D-Pad / Left Stick | Arrow Keys / Mouse |
| **Select/Confirm** | A Button | Enter / Left-click |
| **Back/Cancel** | B Button | Escape |
| **Tab Left** | LB (Left Bumper) | - |
| **Tab Right** | RB (Right Bumper) | - |

---

## Implementation Phases

### Phase 1: Core Infrastructure (4-6 hours)

**Goal:** Set up gamepad detection and basic input reading

#### Tasks

1. **Create Gamepad Module** (`src/systems/game/gamepad.rs`)
   ```rust
   // Resources
   pub struct ActiveGamepad(pub Option<Gamepad>);
   pub struct GamepadSettings {
       pub stick_deadzone: f32,
       pub trigger_deadzone: f32,
       pub invert_y_axis: bool,
       pub sensitivity: f32,
   }
   
   // Systems
   pub fn detect_gamepad_connection(...)
   pub fn update_active_gamepad(...)
   ```

2. **Gamepad Connection Handling**
   - Detect when controller connects/disconnects
   - Store active gamepad ID in resource
   - Show on-screen notification for connection status

3. **Input Abstraction Layer**
   ```rust
   // Unified input resource that combines keyboard/mouse and gamepad
   pub struct PlayerInput {
       pub movement: Vec2,        // Normalized movement direction
       pub camera_delta: Vec2,    // Camera rotation delta
       pub jump_pressed: bool,
       pub interact_pressed: bool,
       pub pause_pressed: bool,
       pub input_source: InputSource,
   }
   
   pub enum InputSource {
       KeyboardMouse,
       Gamepad,
   }
   ```

4. **Deadzone Handling**
   - Implement circular deadzone for sticks
   - Configurable deadzone values

#### Files to Create/Modify
- `src/systems/game/gamepad.rs` (NEW)
- `src/systems/game/mod.rs` (add module)
- `src/systems/game/resources.rs` (add input resources)

---

### Phase 2: Game Movement & Camera (4-6 hours)

**Goal:** Full controller support for gameplay

#### Tasks

1. **Modify Player Movement System**
   - Read from unified `PlayerInput` resource
   - Support analog movement (not just 8-directional)
   - Smooth acceleration based on stick magnitude

2. **Modify Camera System**
   - Support right stick for camera orbit
   - Implement sensitivity settings
   - Add Y-axis invert option

3. **Update Input System**
   - Create system to populate `PlayerInput` from either source
   - Implement input source auto-switching
   - Last-used input source determines UI hints

#### Files to Modify
- `src/systems/game/player_movement.rs`
- `src/systems/game/camera.rs`
- `src/systems/game/input.rs`

---

### Phase 3: Menu Navigation (3-4 hours)

**Goal:** Navigate all menus with controller

#### Tasks

1. **Title Screen**
   - D-pad/stick navigation between menu items
   - A to select, B to go back
   - Visual focus indicator

2. **Pause Menu**
   - Same navigation pattern
   - Start button toggles pause

3. **bevy_egui Integration**
   - Research egui gamepad navigation support
   - May need custom focus management
   - Consider using `egui::Response::request_focus()`

#### Files to Modify
- `src/systems/title_screen/systems.rs`
- `src/systems/pause_menu/systems.rs`
- Possibly UI components

---

### Phase 4: Polish & Settings (2-3 hours)

**Goal:** User customization and quality of life

#### Tasks

1. **Settings Menu**
   - Sensitivity slider
   - Deadzone adjustment
   - Invert Y-axis toggle
   - Vibration on/off (if supported)

2. **Button Prompt System**
   - Show correct prompts based on input source
   - Xbox button icons for controller mode
   - Keyboard/mouse icons otherwise

3. **Persist Settings**
   - Save controller settings to config file
   - Load on startup

#### Files to Create/Modify
- `src/systems/game/settings.rs` (NEW or modify existing)
- UI files for settings menu

---

## Technical Considerations

### Input Priority

When both controller and keyboard/mouse are used:
1. Last-used input source takes priority
2. Small grace period before switching (prevents accidental switches)
3. UI prompts update to match active input

### Analog vs Digital

- **Movement**: Full analog support (stick magnitude = speed)
- **Camera**: Analog with adjustable sensitivity
- **Menus**: Digital only (D-pad or stick with threshold)

### Deadzone Implementation

```rust
fn apply_deadzone(value: Vec2, deadzone: f32) -> Vec2 {
    let magnitude = value.length();
    if magnitude < deadzone {
        Vec2::ZERO
    } else {
        // Rescale to 0-1 range after deadzone
        let normalized = value / magnitude;
        let rescaled_magnitude = (magnitude - deadzone) / (1.0 - deadzone);
        normalized * rescaled_magnitude
    }
}
```

### Multiple Controllers

For simplicity, support only one active controller:
- First connected controller becomes active
- If active disconnects, next connected becomes active
- Show "Press A to activate" if multiple connected

---

## Testing Checklist

### Game
- [x] Movement with left stick (all directions, analog speed)
- [x] Camera orbit with right stick
- [x] Jump with A button
- [x] Interact with X button
- [x] Pause with Start button
- [x] Navigate title screen
- [x] Navigate pause menu
- [x] Controller disconnect handling
- [x] Controller reconnect handling
- [x] Switch between controller and keyboard/mouse
- [x] Cursor auto-hide when using controller
- [x] Cursor show on mouse movement

### Settings (Future Work)
- [ ] Sensitivity adjustment UI
- [ ] Deadzone adjustment UI
- [ ] Invert Y-axis UI toggle
- [ ] Settings persist after restart

---

## Timeline Estimate

| Phase | Effort | Dependencies |
|-------|--------|--------------|
| Phase 1: Core Infrastructure | 4-6 hours | None |
| Phase 2: Game Movement & Camera | 4-6 hours | Phase 1 |
| Phase 3: Menu Navigation | 3-4 hours | Phase 1 |
| Phase 4: Polish & Settings | 2-3 hours | Phases 2-3 |

**Total Estimated Effort**: 13-19 hours

---

## Future Enhancements (Out of Scope)

- PlayStation controller button prompts
- Steam Input API integration
- Custom button remapping
- Haptic feedback/vibration patterns
- Local multiplayer support
- Gyro aiming (PlayStation/Switch)

---

## References

- [Bevy Gamepad Example](https://github.com/bevyengine/bevy/blob/main/examples/input/gamepad_input.rs)
- [Bevy Input Documentation](https://docs.rs/bevy/latest/bevy/input/gamepad/index.html)
- [XInput on Windows](https://docs.microsoft.com/en-us/windows/win32/xinput/getting-started-with-xinput)
