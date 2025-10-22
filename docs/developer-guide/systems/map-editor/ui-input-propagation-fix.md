# UI Input Propagation Fix

## Issue

When clicking on UI controls in the map editor (menu items, toolbar buttons, properties panel controls), the canvas was also processing these mouse events, causing unintended actions such as:

- Placing voxels when clicking toolbar buttons
- Selecting objects when clicking menu items
- Triggering canvas operations when interacting with dialogs

## Root Cause

The mouse input handlers in the map editor tools were not checking if the UI (egui) wanted to handle pointer input before processing mouse events. This caused both the UI and the canvas to respond to the same click event.

## Solution

Added `wants_pointer_input()` checks to all mouse input handlers, following the same pattern already used for keyboard input with `wants_keyboard_input()`. This ensures that:

1. When the user interacts with UI elements, the canvas ignores the input
2. When the user clicks on the canvas (not on UI), the tools process the input normally

## Files Modified

### 1. [`src/editor/tools/voxel_tool.rs`](../../../src/editor/tools/voxel_tool.rs)

**Changes:**
- Added `use bevy_egui::EguiContexts;` import
- Added `mut contexts: EguiContexts` parameter to [`handle_voxel_placement()`](../../../src/editor/tools/voxel_tool.rs:9)
- Added UI pointer input check before processing left-click for voxel placement
- Added `mut contexts: EguiContexts` parameter to [`handle_voxel_removal()`](../../../src/editor/tools/voxel_tool.rs:77)
- Added separate checks for pointer and keyboard input before processing removal

**Code Pattern:**
```rust
// Check if UI wants pointer input (user is interacting with UI elements)
let ui_wants_input = contexts.ctx_mut().wants_pointer_input();
if ui_wants_input {
    return;
}
```

### 2. [`src/editor/tools/entity_tool.rs`](../../../src/editor/tools/entity_tool.rs)

**Changes:**
- Added `use bevy_egui::EguiContexts;` import
- Added `mut contexts: EguiContexts` parameter to [`handle_entity_placement()`](../../../src/editor/tools/entity_tool.rs:10)
- Added UI pointer input check before processing left-click for entity placement

### 3. [`src/editor/tools/selection_tool.rs`](../../../src/editor/tools/selection_tool.rs)

**Changes:**
- Added `mut contexts: EguiContexts` parameter to [`handle_selection()`](../../../src/editor/tools/selection_tool.rs:101)
- Added UI pointer input check before processing left-click for selection
- Updated [`confirm_transform()`](../../../src/editor/tools/selection_tool.rs:473) to check both keyboard and pointer input separately
- Updated [`cancel_transform()`](../../../src/editor/tools/selection_tool.rs:588) to check both keyboard and pointer input separately
- Updated [`confirm_rotation()`](../../../src/editor/tools/selection_tool.rs:935) to check both keyboard and pointer input separately

**Code Pattern for Mixed Input:**
```rust
// Check if UI wants input
let ui_wants_keyboard = contexts.ctx_mut().wants_keyboard_input();
let ui_wants_pointer = contexts.ctx_mut().wants_pointer_input();

// Check for confirmation input (only if UI doesn't want input)
let should_confirm = (!ui_wants_keyboard && keyboard.just_pressed(KeyCode::Enter))
    || (!ui_wants_pointer && mouse_button.just_pressed(MouseButton::Left))
    || confirm_events.read().count() > 0;
```

## Documentation Created

### [`docs/developer-guide/systems/map-editor/input-handling.md`](input-handling.md)

Comprehensive documentation covering:
- Overview of the input priority system
- Implementation patterns for mouse, keyboard, and mixed input handlers
- List of all current implementations
- Best practices and common mistakes
- Testing guidelines

## Testing Recommendations

To verify the fix works correctly, test the following scenarios:

### Menu Interactions
- [ ] Click on "File" menu items (New, Open, Save, Quit)
- [ ] Click on "Edit" menu items (Undo, Redo)
- [ ] Click on "View" menu items (Show Grid, Snap to Grid)
- [ ] Click on "Tools" menu items (Voxel Place, Voxel Remove, Entity Place, Select, Camera)
- [ ] Click on "Help" menu items (Keyboard Shortcuts, About)

### Toolbar Buttons
- [ ] Click quick action buttons (New, Open, Save)
- [ ] Click undo/redo buttons
- [ ] Click tool selection buttons (Place, Remove, Select)
- [ ] Click grid control checkboxes

### Properties Panel
- [ ] Click buttons in the properties panel
- [ ] Interact with sliders and other controls
- [ ] Click delete button for selected voxels
- [ ] Click move/rotate buttons

### Dialogs
- [ ] Click buttons in the New Map dialog
- [ ] Click buttons in the Open dialog
- [ ] Click buttons in confirmation dialogs
- [ ] Click buttons in the About dialog

### Expected Behavior
In all cases above, **no canvas operations should trigger**. The canvas should only respond to clicks when:
- The mouse is over the 3D viewport area
- Not hovering over any UI elements
- Not clicking on any UI controls

## Impact

This fix ensures a clean separation between UI interactions and canvas operations, providing a professional user experience where:

1. UI controls work as expected without side effects
2. Canvas operations only occur when intended
3. The editor behaves predictably and intuitively

## Related Issues

This fix follows the same pattern established for keyboard input handling, which was already checking `wants_keyboard_input()` in several systems. The mouse input handlers were simply missing the equivalent check for pointer input.

## Future Considerations

When adding new input handlers to the map editor:

1. Always check `wants_pointer_input()` before processing mouse events
2. Always check `wants_keyboard_input()` before processing keyboard events
3. Refer to the [Input Handling documentation](input-handling.md) for the correct pattern
4. Test with UI interactions to ensure no propagation occurs

## Verification

The changes have been verified to compile successfully:
```bash
cargo check --bin map_editor
```

Result: ✅ Passed (Exit code: 0)