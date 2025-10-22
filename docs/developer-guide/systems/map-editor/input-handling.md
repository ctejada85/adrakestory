# Input Handling in Map Editor

## Overview

The map editor uses a layered input handling approach to prevent conflicts between UI interactions and canvas operations. This document describes the pattern that all input handlers should follow.

## The Problem

When a user clicks on UI elements (buttons, menus, panels), without proper input filtering, the canvas tools would also process these clicks, causing unintended actions like placing voxels or selecting objects.

## The Solution: Input Priority System

The map editor uses egui's input capture system to determine whether the UI or the canvas should handle input events. This is implemented through two key methods:

- `wants_pointer_input()` - Returns `true` when the UI wants to handle mouse/pointer events
- `wants_keyboard_input()` - Returns `true` when the UI wants to handle keyboard events

## Implementation Pattern

### For Mouse Input Handlers

All systems that handle mouse input must check if the UI wants pointer input before processing mouse events:

```rust
use bevy::prelude::*;
use bevy_egui::EguiContexts;

pub fn handle_mouse_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    // ... other parameters
) {
    // Check if UI wants pointer input (user is interacting with UI elements)
    let ui_wants_input = contexts.ctx_mut().wants_pointer_input();
    if ui_wants_input {
        return; // Don't process canvas input when UI is active
    }

    // Now safe to process mouse input for canvas operations
    if mouse_button.just_pressed(MouseButton::Left) {
        // Handle canvas click
    }
}
```

### For Keyboard Input Handlers

Similarly, keyboard input handlers must check if the UI wants keyboard input:

```rust
pub fn handle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    // ... other parameters
) {
    // Check if UI wants keyboard input
    let ui_wants_input = contexts.ctx_mut().wants_keyboard_input();
    if ui_wants_input {
        return; // Don't process canvas input when UI is active
    }

    // Now safe to process keyboard input for canvas operations
    if keyboard.just_pressed(KeyCode::Delete) {
        // Handle deletion
    }
}
```

### For Mixed Input Handlers

When a system handles both mouse and keyboard input, check both conditions separately:

```rust
pub fn handle_mixed_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    // ... other parameters
) {
    // Check UI input state
    let ui_wants_pointer = contexts.ctx_mut().wants_pointer_input();
    let ui_wants_keyboard = contexts.ctx_mut().wants_keyboard_input();

    // Handle mouse input (only if UI doesn't want pointer input)
    if !ui_wants_pointer && mouse_button.just_pressed(MouseButton::Left) {
        // Handle mouse click
    }

    // Handle keyboard input (only if UI doesn't want keyboard input)
    if !ui_wants_keyboard && keyboard.just_pressed(KeyCode::Delete) {
        // Handle key press
    }
}
```

## Current Implementations

The following systems implement this pattern:

### Tool Systems

1. **Voxel Placement** ([`handle_voxel_placement()`](../../../src/editor/tools/voxel_tool.rs:9))
   - Checks `wants_pointer_input()` before placing voxels on left-click

2. **Voxel Removal** ([`handle_voxel_removal()`](../../../src/editor/tools/voxel_tool.rs:77))
   - Checks both `wants_pointer_input()` and `wants_keyboard_input()`
   - Handles mouse clicks and Delete/Backspace keys

3. **Entity Placement** ([`handle_entity_placement()`](../../../src/editor/tools/entity_tool.rs:10))
   - Checks `wants_pointer_input()` before placing entities on left-click

4. **Selection** ([`handle_selection()`](../../../src/editor/tools/selection_tool.rs:101))
   - Checks `wants_pointer_input()` before selecting voxels on left-click

### Transform Operations

5. **Transform Confirmation** ([`confirm_transform()`](../../../src/editor/tools/selection_tool.rs:473), [`confirm_rotation()`](../../../src/editor/tools/selection_tool.rs:935))
   - Checks both input types for Enter key and left-click confirmation

6. **Transform Cancellation** ([`cancel_transform()`](../../../src/editor/tools/selection_tool.rs:588))
   - Checks both input types for Escape key and right-click cancellation

7. **Delete Selected** ([`handle_delete_selected()`](../../../src/editor/tools/selection_tool.rs:202))
   - Checks `wants_keyboard_input()` for Delete/Backspace keys

8. **Move/Rotate Shortcuts** ([`handle_move_shortcut()`](../../../src/editor/tools/selection_tool.rs:615), [`handle_rotate_shortcut()`](../../../src/editor/tools/selection_tool.rs:699))
   - Check `wants_keyboard_input()` for G and R keys

9. **Arrow Key Operations** ([`handle_arrow_key_movement()`](../../../src/editor/tools/selection_tool.rs:332), [`handle_arrow_key_rotation()`](../../../src/editor/tools/selection_tool.rs:767))
   - Check `wants_keyboard_input()` for arrow key navigation

10. **Rotation Axis Selection** ([`handle_rotation_axis_selection()`](../../../src/editor/tools/selection_tool.rs:727))
    - Checks `wants_keyboard_input()` for X, Y, Z keys

## Best Practices

1. **Always Check First**: Input checking should be one of the first operations in your handler, right after checking if the tool is active.

2. **Early Return**: Use early returns when UI wants input to keep code clean and avoid deep nesting.

3. **Separate Checks**: When handling both mouse and keyboard, check each separately rather than combining them.

4. **Add EguiContexts Parameter**: Don't forget to add `mut contexts: EguiContexts` to your system parameters.

5. **Import Statement**: Include `use bevy_egui::EguiContexts;` at the top of your file.

## Testing

When implementing or modifying input handlers, test the following scenarios:

1. **Menu Interactions**: Click on menu items (File, Edit, View, Tools, Help)
2. **Toolbar Buttons**: Click on quick action buttons and tool selection buttons
3. **Properties Panel**: Interact with controls in the properties panel
4. **Dialog Boxes**: Click buttons in dialogs (New Map, Open, etc.)
5. **Checkboxes and Sliders**: Interact with UI controls in the View menu
6. **Text Input**: Type in text fields (if any)

In all cases, canvas operations should NOT trigger when interacting with UI elements.

## Common Mistakes

### ❌ Wrong: Not checking UI input state
```rust
pub fn handle_input(mouse_button: Res<ButtonInput<MouseButton>>) {
    if mouse_button.just_pressed(MouseButton::Left) {
        // This will trigger even when clicking UI!
    }
}
```

### ✅ Correct: Checking UI input state
```rust
pub fn handle_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
) {
    let ui_wants_input = contexts.ctx_mut().wants_pointer_input();
    if ui_wants_input {
        return;
    }
    
    if mouse_button.just_pressed(MouseButton::Left) {
        // Safe: only triggers for canvas clicks
    }
}
```

### ❌ Wrong: Checking after processing
```rust
pub fn handle_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        // Already processed the input!
        let ui_wants_input = contexts.ctx_mut().wants_pointer_input();
        if !ui_wants_input {
            // Too late
        }
    }
}
```

### ✅ Correct: Checking before processing
```rust
pub fn handle_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
) {
    let ui_wants_input = contexts.ctx_mut().wants_pointer_input();
    if ui_wants_input {
        return;
    }
    
    if mouse_button.just_pressed(MouseButton::Left) {
        // Process input
    }
}
```

## Related Documentation

- [Map Editor Architecture](architecture.md)
- [Map Editor Controls](../../../user-guide/map-editor/controls.md)
- [Keyboard Input Fix](keyboard-input-fix.md)