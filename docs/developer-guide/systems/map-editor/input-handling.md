# Input Handling in Map Editor

## Overview

The map editor uses a **unified, event-driven input handling architecture** with clear separation between input reading and action execution. This document describes the current architecture and the patterns used to prevent conflicts between UI interactions and canvas operations.

## Current Architecture (Post-Refactoring)

As of October 2025, the map editor uses a unified input system with two main components:

1. **[`handle_keyboard_input()`](../../../../src/editor/tools/input.rs:105)** - Single system that reads all keyboard input and emits semantic events
2. **[`handle_transformation_operations()`](../../../../src/editor/tools/input.rs:234)** - Single system that executes operations based on events

This replaces the previous architecture of 15+ scattered input handler systems. See [Input Refactoring Summary](input-refactoring-summary.md) for details.

## The Problem

When a user clicks on UI elements (buttons, menus, panels), without proper input filtering, the canvas tools would also process these clicks, causing unintended actions like placing voxels or selecting objects. This creates a poor user experience where UI interactions trigger unwanted canvas operations.

## The Solution: Input Priority System

The map editor uses egui's input capture system to determine whether the UI or the canvas should handle input events. This is implemented through two key methods:

- **`wants_pointer_input()`** - Returns `true` when the UI wants to handle mouse/pointer events
- **`wants_keyboard_input()`** - Returns `true` when the UI wants to handle keyboard events

### When UI Captures Input

egui captures input when:
- Mouse is hovering over any UI panel
- A text field has focus
- A dropdown menu is open
- Any UI widget is being interacted with
- Menu items are being clicked
- Buttons or controls are being activated

## Unified Input System Pattern

### Single Keyboard Input Handler

The unified input system uses a single entry point for all keyboard input:

```rust
pub fn handle_keyboard_input(
    editor_state: Res<EditorState>,
    active_transform: Res<ActiveTransform>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut input_events: EventWriter<EditorInputEvent>,
) {
    // Single UI focus check for ALL keyboard input
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    // Context-aware input mapping based on current mode
    match active_transform.mode {
        TransformMode::None => {
            // Selection mode shortcuts
            if keyboard.just_pressed(KeyCode::KeyG) {
                input_events.send(EditorInputEvent::StartMove);
            }
            // ... other shortcuts
        }
        TransformMode::Move => {
            // Move mode shortcuts
            if keyboard.just_pressed(KeyCode::ArrowUp) {
                input_events.send(EditorInputEvent::UpdateMoveOffset(IVec3::new(0, 0, -1)));
            }
            // ... other move controls
        }
        TransformMode::Rotate => {
            // Rotate mode shortcuts
            // ... rotation controls
        }
    }
}
```

### Benefits of Unified Approach

1. **Single UI Focus Check**: Only one `wants_keyboard_input()` check instead of 7+
2. **Context-Aware**: Different key mappings based on current mode
3. **Centralized**: All keyboard shortcuts visible in one place
4. **Maintainable**: Easy to add new shortcuts or modify existing ones
5. **Testable**: Can test input mapping separately from execution

## Legacy Pattern (For Reference)

### For Mouse Input Handlers (Still Used)

Mouse input handlers still use the individual checking pattern:

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

## Current System Implementations

### Unified Input Systems (New Architecture)

1. **Keyboard Input Handler** ([`handle_keyboard_input()`](../../../../src/editor/tools/input.rs:105))
   - Single system for ALL keyboard input
   - Context-aware key mapping based on mode
   - One UI focus check for all shortcuts

2. **Transformation Operations** ([`handle_transformation_operations()`](../../../../src/editor/tools/input.rs:234))
   - Executes operations based on events
   - Handles both keyboard events and UI button events
   - Separated from input reading

### Mouse Input Systems (Individual Pattern)

3. **Voxel Placement** ([`handle_voxel_placement()`](../../../../src/editor/tools/voxel_tool.rs:9))
   - Checks `wants_pointer_input()` before placing voxels on left-click

4. **Voxel Removal** ([`handle_voxel_removal()`](../../../../src/editor/tools/voxel_tool.rs:77))
   - Checks `wants_pointer_input()` for mouse clicks
   - Keyboard deletion now handled by unified system

5. **Entity Placement** ([`handle_entity_placement()`](../../../../src/editor/tools/entity_tool.rs:10))
   - Checks `wants_pointer_input()` before placing entities on left-click

6. **Selection** ([`handle_selection()`](../../../../src/editor/tools/selection_tool.rs:101))
   - Checks `wants_pointer_input()` before selecting voxels on left-click

### Rendering Systems (No Input Checks Needed)

7. **Selection Highlights** ([`render_selection_highlights()`](../../../../src/editor/tools/selection_tool.rs:142))
8. **Transform Preview** ([`render_transform_preview()`](../../../../src/editor/tools/selection_tool.rs:209))
9. **Rotation Preview** ([`render_rotation_preview()`](../../../../src/editor/tools/selection_tool.rs:714))

## Best Practices

1. **Always Check First**: Input checking should be one of the first operations in your handler, right after checking if the tool is active.

2. **Early Return**: Use early returns when UI wants input to keep code clean and avoid deep nesting.

3. **Separate Checks**: When handling both mouse and keyboard, check each separately rather than combining them.

4. **Add EguiContexts Parameter**: Don't forget to add `mut contexts: EguiContexts` to your system parameters.

5. **Import Statement**: Include `use bevy_egui::EguiContexts;` at the top of your file.

6. **UI Events Don't Need Checks**: Events triggered by UI buttons (like `StartMoveOperation` from the "Move" button) don't need UI focus checks because they're intentional user actions from the UI itself.

## Testing Guidelines

When implementing or modifying input handlers, test the following scenarios:

### Menu Interactions
- Click on menu items (File, Edit, View, Tools, Help)
- Verify no canvas operations trigger

### Toolbar Buttons
- Click on quick action buttons and tool selection buttons
- Verify no canvas operations trigger

### Properties Panel
- Interact with controls in the properties panel
- Click buttons, adjust sliders, use dropdowns
- Verify no canvas operations trigger

### Dialog Boxes
- Click buttons in dialogs (New Map, Open, etc.)
- Verify no canvas operations trigger

### Checkboxes and Sliders
- Interact with UI controls in the View menu
- Verify no canvas operations trigger

### Text Input
- Type in text fields (if any)
- Verify keyboard shortcuts don't trigger while typing

**Expected Behavior**: In all cases, canvas operations should NOT trigger when interacting with UI elements.

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

## Historical Context

### Input System Refactoring (October 2025)

The map editor underwent a major refactoring to unify input handling:

**Before**: 15+ scattered input handler systems, each with duplicate UI focus checks
**After**: 2 unified systems with single UI focus check and event-driven architecture

**Benefits**:
- Reduced system count by 72% (18 → 5 total systems)
- Eliminated code duplication (7+ UI checks → 1)
- Improved maintainability (all shortcuts in one place)
- Better separation of concerns (input reading vs execution)

See [Input Refactoring Summary](input-refactoring-summary.md) for complete details.

### Keyboard Input Fix (January 2025)

Initially, keyboard shortcuts were not working when voxels were selected. The root cause was that egui was consuming keyboard events before they could reach the game systems.

The solution was to add UI focus checks using `EguiContexts::wants_keyboard_input()` before processing keyboard input. This pattern was later unified into the single input handler system.

### UI Input Propagation Fix (January 2025)

When clicking on UI controls, the canvas was also processing these mouse events, causing unintended actions.

The solution was to add `wants_pointer_input()` checks to all mouse input handlers. This pattern is still used for mouse input systems.

## Performance Considerations

### Optimization Strategies

The `wants_keyboard_input()` and `wants_pointer_input()` checks are very fast (just checking boolean flags) and have negligible performance impact.

### Future Improvements

Consider creating a helper resource or system that caches the UI focus state each frame to avoid multiple `ctx_mut()` calls:

```rust
#[derive(Resource)]
pub struct UIFocusState {
    pub wants_keyboard: bool,
    pub wants_pointer: bool,
}

// Update once per frame
pub fn update_ui_focus_state(
    mut state: ResMut<UIFocusState>,
    mut contexts: EguiContexts,
) {
    let ctx = contexts.ctx_mut();
    state.wants_keyboard = ctx.wants_keyboard_input();
    state.wants_pointer = ctx.wants_pointer_input();
}
```

Then systems can just check the resource instead of calling `ctx_mut()`.

## Why Not Global Input Blocking?

We check UI focus in each system rather than globally because:
1. Different systems may have different requirements
2. Some systems should work even when UI has focus (e.g., camera controls)
3. More explicit and easier to debug
4. Allows fine-grained control per system

## Related Documentation

- [Input Refactoring Summary](input-refactoring-summary.md) - Details of the unified input system
- [Input Refactoring Plan](input-refactoring-plan.md) - Original design document
- [Map Editor Architecture](architecture.md) - Overall editor architecture
- [Map Editor Controls](../../../user-guide/map-editor/controls.md) - User-facing controls guide
- [Archived: Keyboard Input Fix](archive/keyboard-input-fix.md) - Historical fix
- [Archived: UI Input Propagation Fix](archive/ui-input-propagation-fix.md) - Historical fix

---

**Document Version**: 3.0.0
**Last Updated**: 2025-10-22
**Status**: Updated for unified input architecture