# Fullscreen Toggle Implementation Plan

## Overview
Implement fullscreen mode toggling using `Alt + Enter` keyboard shortcut, allowing users to switch between fullscreen and windowed mode.

## Requirements
- Press `Alt + Enter` to toggle between fullscreen and windowed mode
- Should work from any game state (title screen, gameplay, pause menu, etc.)
- Remember window size when returning from fullscreen to windowed mode

## Implementation Steps

### 1. Create Fullscreen Toggle System

**File:** `src/systems/game/input.rs` (or new file `src/systems/fullscreen.rs`)

Create a new system that:
- Listens for `Alt + Enter` key combination
- Toggles the window mode between `WindowMode::Fullscreen` (or `BorderlessFullscreen`) and `WindowMode::Windowed`

```rust
pub fn toggle_fullscreen_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window>,
) {
    // Check for Alt + Enter
    let alt_pressed = keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight);
    let enter_just_pressed = keyboard.just_pressed(KeyCode::Enter);

    if alt_pressed && enter_just_pressed {
        if let Ok(mut window) = windows.get_single_mut() {
            window.mode = match window.mode {
                WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                _ => WindowMode::Windowed,
            };
        }
    }
}
```

### 2. Register the System

**File:** `src/main.rs` or appropriate plugin file

Add the system to run in the `Update` schedule with a high priority so it runs regardless of game state:

```rust
app.add_systems(Update, toggle_fullscreen_system);
```

### 3. Consider Window Mode Options

Bevy provides several fullscreen modes:
- `WindowMode::Fullscreen` - Exclusive fullscreen (changes display resolution)
- `WindowMode::BorderlessFullscreen` - Borderless fullscreen window (recommended for most cases)
- `WindowMode::SizedFullscreen` - Fullscreen at current window size
- `WindowMode::Windowed` - Regular windowed mode

**Recommendation:** Use `BorderlessFullscreen` as it provides:
- Faster alt-tab switching
- Better multi-monitor support
- No resolution change flicker

### 4. Optional Enhancements

#### 4.1 Persist User Preference
Store the fullscreen preference in a config file so it persists between game sessions.

#### 4.2 Add to Pause Menu / Settings
Add a menu option to toggle fullscreen in addition to the keyboard shortcut.

#### 4.3 Handle Resolution
If using exclusive fullscreen, consider:
- Storing the preferred resolution
- Providing resolution options in settings menu

## File Changes Summary

| File | Change |
|------|--------|
| `src/systems/game/input.rs` | Add `toggle_fullscreen_system` function |
| `src/systems/game/mod.rs` | Export the new system |
| `src/main.rs` | Register the system in the app |

## Testing Checklist

- [ ] `Alt + Enter` toggles to fullscreen from windowed mode
- [ ] `Alt + Enter` toggles back to windowed from fullscreen
- [ ] Works during gameplay
- [ ] Works on title screen
- [ ] Works in pause menu
- [ ] Window size is preserved when returning to windowed mode
- [ ] Left Alt and Right Alt both work

## Dependencies

- Bevy's `Window` component and `WindowMode` enum (already available via `bevy::prelude::*`)

## Estimated Effort

~30 minutes for basic implementation
