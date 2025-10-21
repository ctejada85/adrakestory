# Keyboard Input Fix for Map Editor

## Issue
Keyboard shortcuts (G key, arrow keys, Delete, etc.) were not working in the map editor when voxels were selected.

## Root Cause
The egui UI library was consuming keyboard events before they could reach the game systems. This is a common issue in Bevy applications using egui - when UI elements have focus or when the mouse is over UI panels, egui captures keyboard input to prevent accidental game actions while typing in text fields.

## Solution
Added UI focus checks using `EguiContexts::wants_keyboard_input()` before processing keyboard input in all relevant systems:

### Modified Systems
1. **`handle_delete_selected()`** - Delete/Backspace key handling
2. **`handle_arrow_key_movement()`** - Arrow key movement during move operation
3. **`confirm_transform()`** - Enter key to confirm transformation
4. **`cancel_transform()`** - Escape key to cancel transformation
5. **`handle_move_shortcut()`** - G key to start move operation

### Code Pattern
```rust
// Check if UI wants keyboard input
let ui_wants_input = contexts.ctx_mut().wants_keyboard_input();

// Only process keyboard input if UI doesn't want it
if !ui_wants_input && keyboard.just_pressed(KeyCode::SomeKey) {
    // Process game input
}
```

## Testing the Fix

### Test 1: Basic Keyboard Input
1. Launch map editor: `cargo run --bin map_editor`
2. Place a voxel using Voxel tool (V key)
3. Switch to Select tool (S key)
4. Click on voxel to select it
5. **Press G key** - Should enter move mode
6. **Press arrow keys** - Should see ghost preview moving
7. **Press Enter** - Should confirm move
8. **Press Ctrl+Z** - Should undo move

**Expected**: All keyboard shortcuts work when viewport has focus

### Test 2: UI Focus Behavior
1. Select a voxel
2. Click in a text field in the properties panel (e.g., map name)
3. **Press G key** - Should type 'g' in text field, NOT start move mode
4. Click outside text field (in viewport)
5. **Press G key** - Should now start move mode

**Expected**: Keyboard shortcuts don't interfere with text input

### Test 3: Move Operation
1. Select a voxel
2. Press G to enter move mode
3. Verify "Transform Mode: Move" appears in properties panel
4. Press arrow keys to move preview
5. Press Enter to confirm
6. Verify voxel moved to new position

**Expected**: Full move operation works correctly

## When UI Captures Input

egui captures keyboard input when:
- Mouse is hovering over any UI panel
- A text field has focus
- A dropdown menu is open
- Any UI widget is being interacted with

## Best Practices

### Always Check UI Focus
```rust
pub fn my_keyboard_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
) {
    // Always check first
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }
    
    // Now safe to process keyboard input
    if keyboard.just_pressed(KeyCode::MyKey) {
        // Handle input
    }
}
```

### UI Events Don't Need Checks
Events triggered by UI buttons (like `StartMoveOperation` from the "Move" button) don't need UI focus checks because they're intentional user actions from the UI itself.

```rust
// This is fine - event comes from UI button
if start_events.read().count() > 0 {
    // Start move operation
}
```

### Mouse Input Considerations
Mouse clicks may also need similar checks using `wants_pointer_input()` if you want to prevent clicking through UI panels:

```rust
if contexts.ctx_mut().wants_pointer_input() {
    return; // UI wants mouse input
}
```

## Related Files
- [`src/editor/tools/selection_tool.rs`](../../../src/editor/tools/selection_tool.rs) - All keyboard input systems
- [`src/bin/map_editor.rs`](../../../src/bin/map_editor.rs) - System registration

## Additional Notes

### Why Not Global Input Blocking?
We check UI focus in each system rather than globally because:
1. Different systems may have different requirements
2. Some systems should work even when UI has focus (e.g., camera controls)
3. More explicit and easier to debug
4. Allows fine-grained control per system

### Performance Impact
The `wants_keyboard_input()` check is very fast (just checking a boolean flag) and has negligible performance impact.

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