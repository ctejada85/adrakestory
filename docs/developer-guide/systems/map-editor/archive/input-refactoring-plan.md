# Input System Refactoring Plan

## Overview

This document outlines the plan to refactor the map editor's input handling from multiple scattered systems to a unified architecture with clear separation of concerns.

## Current Architecture Problems

### Scattered Input Handling
Currently, input handling is spread across 7+ separate systems in [`selection_tool.rs`](../../../../src/editor/tools/selection_tool.rs):

1. [`handle_delete_selected()`](../../../../src/editor/tools/selection_tool.rs:209) - Delete/Backspace keys
2. [`handle_move_shortcut()`](../../../../src/editor/tools/selection_tool.rs:624) - G key
3. [`handle_rotate_shortcut()`](../../../../src/editor/tools/selection_tool.rs:708) - R key
4. [`handle_arrow_key_movement()`](../../../../src/editor/tools/selection_tool.rs:339) - Arrow keys during move
5. [`handle_arrow_key_rotation()`](../../../../src/editor/tools/selection_tool.rs:818) - Arrow keys during rotate
6. [`handle_rotation_axis_selection()`](../../../../src/editor/tools/selection_tool.rs:778) - X/Y/Z keys
7. [`handle_deselect_shortcut()`](../../../../src/editor/tools/selection_tool.rs:735) - Escape key

### Issues with Current Approach

1. **Code Duplication**: Each system independently checks `wants_keyboard_input()`
2. **Scattered Logic**: Hard to see all keyboard shortcuts in one place
3. **Maintenance Burden**: Adding new shortcuts requires creating new systems
4. **Testing Difficulty**: Can't test input handling separately from execution
5. **Unclear Responsibilities**: Input reading mixed with action execution

## Proposed Architecture

### Two-System Design

```
┌─────────────────────────────────────────────────────────────┐
│                    User Input (Keyboard)                     │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│              handle_keyboard_input() System                  │
│  • Single UI focus check                                     │
│  • Context-aware key mapping                                 │
│  • Emits semantic EditorInputEvent                           │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼ EditorInputEvent
┌─────────────────────────────────────────────────────────────┐
│           handle_transformation_operations() System          │
│  • Receives EditorInputEvent                                 │
│  • Updates ActiveTransform state                             │
│  • Executes transformations                                  │
│  • Emits render/update events                                │
└─────────────────────────────────────────────────────────────┘
```

### Benefits

1. ✅ **Single Responsibility**: Each system has one clear job
2. ✅ **DRY Principle**: One UI focus check instead of 7+
3. ✅ **Maintainability**: All shortcuts visible in one place
4. ✅ **Extensibility**: Easy to add new shortcuts
5. ✅ **Testability**: Can test input mapping separately from execution
6. ✅ **Performance**: Fewer systems to run each frame

## Implementation Details

### 1. EditorInputEvent Enum

Create a new event type that represents all possible input actions:

```rust
// Location: src/editor/tools/input.rs (new file)

use bevy::prelude::*;
use crate::systems::game::map::geometry::RotationAxis;

/// Semantic events representing user input actions in the editor
#[derive(Event, Debug, Clone)]
pub enum EditorInputEvent {
    // Selection operations
    StartMove,
    StartRotate,
    DeleteSelected,
    DeselectAll,
    
    // Transform operations - Move mode
    UpdateMoveOffset(IVec3),
    ConfirmMove,
    CancelMove,
    
    // Transform operations - Rotate mode
    SetRotationAxis(RotationAxis),
    RotateDelta(i32),  // +1 or -1 for 90-degree rotations
    ConfirmRotate,
    CancelRotate,
    
    // Generic transform operations
    ConfirmTransform,
    CancelTransform,
}
```

### 2. Unified Input Handler System

Replace all individual input systems with one unified handler:

```rust
// Location: src/editor/tools/input.rs

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use crate::editor::state::{EditorState, EditorTool};
use crate::editor::tools::ActiveTransform;
use crate::systems::game::map::geometry::RotationAxis;

/// Single system to handle all keyboard input for the editor
/// Replaces: handle_delete_selected, handle_move_shortcut, handle_rotate_shortcut,
///           handle_arrow_key_movement, handle_arrow_key_rotation,
///           handle_rotation_axis_selection, handle_deselect_shortcut
pub fn handle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    editor_state: Res<EditorState>,
    active_transform: Res<ActiveTransform>,
    mut input_events: EventWriter<EditorInputEvent>,
) {
    // Single UI focus check for ALL keyboard input
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    // Only handle input when Select tool is active
    if !matches!(editor_state.active_tool, EditorTool::Select) {
        return;
    }

    // Context-aware input mapping based on current mode
    match active_transform.mode {
        TransformMode::None => {
            handle_selection_mode_input(&keyboard, &mut input_events);
        }
        TransformMode::Move => {
            handle_move_mode_input(&keyboard, &mut input_events);
        }
        TransformMode::Rotate => {
            handle_rotate_mode_input(&keyboard, &mut input_events);
        }
    }
}

/// Handle input when in selection mode (no active transformation)
fn handle_selection_mode_input(
    keyboard: &ButtonInput<KeyCode>,
    events: &mut EventWriter<EditorInputEvent>,
) {
    // Start operations
    if keyboard.just_pressed(KeyCode::KeyG) {
        events.send(EditorInputEvent::StartMove);
    }
    if keyboard.just_pressed(KeyCode::KeyR) {
        events.send(EditorInputEvent::StartRotate);
    }
    
    // Delete selected
    if keyboard.just_pressed(KeyCode::Delete) || keyboard.just_pressed(KeyCode::Backspace) {
        events.send(EditorInputEvent::DeleteSelected);
    }
    
    // Deselect all
    if keyboard.just_pressed(KeyCode::Escape) {
        events.send(EditorInputEvent::DeselectAll);
    }
}

/// Handle input during move operation
fn handle_move_mode_input(
    keyboard: &ButtonInput<KeyCode>,
    events: &mut EventWriter<EditorInputEvent>,
) {
    // Calculate movement step (1 or 5 with Shift)
    let step = if keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        5
    } else {
        1
    };
    
    // Arrow key movement
    let mut offset = IVec3::ZERO;
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        offset.z -= step;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        offset.z += step;
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        offset.x -= step;
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        offset.x += step;
    }
    
    // Y-axis movement
    if keyboard.just_pressed(KeyCode::PageUp) {
        offset.y += step;
    }
    if keyboard.just_pressed(KeyCode::PageDown) {
        offset.y -= step;
    }
    
    // Send offset update if any movement occurred
    if offset != IVec3::ZERO {
        events.send(EditorInputEvent::UpdateMoveOffset(offset));
    }
    
    // Confirm or cancel
    if keyboard.just_pressed(KeyCode::Enter) {
        events.send(EditorInputEvent::ConfirmTransform);
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        events.send(EditorInputEvent::CancelTransform);
    }
}

/// Handle input during rotate operation
fn handle_rotate_mode_input(
    keyboard: &ButtonInput<KeyCode>,
    events: &mut EventWriter<EditorInputEvent>,
) {
    // Axis selection
    if keyboard.just_pressed(KeyCode::KeyX) {
        events.send(EditorInputEvent::SetRotationAxis(RotationAxis::X));
    }
    if keyboard.just_pressed(KeyCode::KeyY) {
        events.send(EditorInputEvent::SetRotationAxis(RotationAxis::Y));
    }
    if keyboard.just_pressed(KeyCode::KeyZ) {
        events.send(EditorInputEvent::SetRotationAxis(RotationAxis::Z));
    }
    
    // Rotation with arrow keys
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        events.send(EditorInputEvent::RotateDelta(-1));
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        events.send(EditorInputEvent::RotateDelta(1));
    }
    
    // Confirm or cancel
    if keyboard.just_pressed(KeyCode::Enter) {
        events.send(EditorInputEvent::ConfirmTransform);
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        events.send(EditorInputEvent::CancelTransform);
    }
}
```

### 3. Transformation Operations System

Create a dedicated system to handle transformation logic:

```rust
// Location: src/editor/tools/input.rs

/// System to handle transformation operations based on input events
/// Replaces the logic portions of: start_move_operation, start_rotate_operation,
///                                  update_transform_preview, update_rotation,
///                                  update_rotation_axis, confirm_transform,
///                                  confirm_rotation, cancel_transform
pub fn handle_transformation_operations(
    mut input_events: EventReader<EditorInputEvent>,
    mut active_transform: ResMut<ActiveTransform>,
    mut editor_state: ResMut<EditorState>,
    mut history: ResMut<EditorHistory>,
    mut render_events: EventWriter<RenderMapEvent>,
    mut update_events: EventWriter<UpdateSelectionHighlights>,
    preview_query: Query<&TransformPreview>,
) {
    for event in input_events.read() {
        match event {
            EditorInputEvent::StartMove => {
                start_move_operation_internal(&mut active_transform, &editor_state);
            }
            
            EditorInputEvent::StartRotate => {
                start_rotate_operation_internal(&mut active_transform, &editor_state);
            }
            
            EditorInputEvent::DeleteSelected => {
                delete_selected_voxels(
                    &mut editor_state,
                    &mut history,
                    &mut render_events,
                    &mut update_events,
                );
            }
            
            EditorInputEvent::DeselectAll => {
                editor_state.clear_selections();
                update_events.send(UpdateSelectionHighlights);
            }
            
            EditorInputEvent::UpdateMoveOffset(offset) => {
                active_transform.current_offset += *offset;
            }
            
            EditorInputEvent::SetRotationAxis(axis) => {
                active_transform.rotation_axis = *axis;
                active_transform.rotation_angle = 0;
            }
            
            EditorInputEvent::RotateDelta(delta) => {
                active_transform.rotation_angle = 
                    (active_transform.rotation_angle + delta).rem_euclid(4);
            }
            
            EditorInputEvent::ConfirmTransform => {
                match active_transform.mode {
                    TransformMode::Move => {
                        confirm_move_internal(
                            &mut active_transform,
                            &mut editor_state,
                            &mut history,
                            &preview_query,
                            &mut render_events,
                            &mut update_events,
                        );
                    }
                    TransformMode::Rotate => {
                        confirm_rotate_internal(
                            &mut active_transform,
                            &mut editor_state,
                            &mut history,
                            &preview_query,
                            &mut render_events,
                            &mut update_events,
                        );
                    }
                    TransformMode::None => {}
                }
            }
            
            EditorInputEvent::CancelTransform => {
                *active_transform = ActiveTransform::default();
            }
        }
    }
}

// Internal helper functions (extracted from current systems)
fn start_move_operation_internal(
    active_transform: &mut ActiveTransform,
    editor_state: &EditorState,
) {
    // Implementation from current start_move_operation
    // ... (lines 287-336 from selection_tool.rs)
}

fn start_rotate_operation_internal(
    active_transform: &mut ActiveTransform,
    editor_state: &EditorState,
) {
    // Implementation from current start_rotate_operation
    // ... (lines 653-705 from selection_tool.rs)
}

fn delete_selected_voxels(
    editor_state: &mut EditorState,
    history: &mut EditorHistory,
    render_events: &mut EventWriter<RenderMapEvent>,
    update_events: &mut EventWriter<UpdateSelectionHighlights>,
) {
    // Implementation from current handle_delete_selected
    // ... (lines 236-283 from selection_tool.rs)
}

fn confirm_move_internal(
    active_transform: &mut ActiveTransform,
    editor_state: &mut EditorState,
    history: &mut EditorHistory,
    preview_query: &Query<&TransformPreview>,
    render_events: &mut EventWriter<RenderMapEvent>,
    update_events: &mut EventWriter<UpdateSelectionHighlights>,
) {
    // Implementation from current confirm_transform
    // ... (lines 510-593 from selection_tool.rs)
}

fn confirm_rotate_internal(
    active_transform: &mut ActiveTransform,
    editor_state: &mut EditorState,
    history: &mut EditorHistory,
    preview_query: &Query<&TransformPreview>,
    render_events: &mut EventWriter<RenderMapEvent>,
    update_events: &mut EventWriter<UpdateSelectionHighlights>,
) {
    // Implementation from current confirm_rotation
    // ... (lines 1040-1181 from selection_tool.rs)
}
```

### 4. Module Structure Changes

```rust
// src/editor/tools/mod.rs

pub mod entity_tool;
pub mod input;           // NEW: Unified input handling
pub mod selection_tool;  // MODIFIED: Remove input handlers, keep rendering/events
pub mod voxel_tool;

// Export new input system
pub use input::{
    handle_keyboard_input,
    handle_transformation_operations,
    EditorInputEvent,
};

// Keep existing exports for rendering and events
pub use selection_tool::{
    render_selection_highlights,
    render_transform_preview,
    render_rotation_preview,
    ActiveTransform,
    TransformMode,
    UpdateSelectionHighlights,
    // Remove: All the handle_* functions
};
```

### 5. System Registration Changes

```rust
// src/bin/map_editor.rs

fn main() {
    App::new()
        // ... existing setup ...
        .add_event::<EditorInputEvent>()  // NEW
        // Remove old events that are now internal:
        // - StartMoveOperation
        // - StartRotateOperation
        // - UpdateTransformPreview
        // - UpdateRotation
        // - SetRotationAxis
        .add_systems(Update, render_ui)
        .add_systems(Update, ui::dialogs::check_file_dialog_result)
        // ... other systems ...
        
        // NEW: Single input handler
        .add_systems(Update, tools::handle_keyboard_input)
        
        // NEW: Single transformation handler
        .add_systems(Update, tools::handle_transformation_operations)
        
        // Keep rendering systems
        .add_systems(Update, tools::render_selection_highlights)
        .add_systems(Update, tools::render_transform_preview)
        .add_systems(Update, tools::render_rotation_preview)
        
        // REMOVE these systems (replaced by handle_keyboard_input):
        // - tools::handle_delete_selected
        // - tools::handle_move_shortcut
        // - tools::handle_rotate_shortcut
        // - tools::handle_arrow_key_movement
        // - tools::handle_arrow_key_rotation
        // - tools::handle_rotation_axis_selection
        // - tools::handle_deselect_shortcut
        
        // REMOVE these systems (replaced by handle_transformation_operations):
        // - tools::start_move_operation
        // - tools::start_rotate_operation
        // - tools::update_transform_preview
        // - tools::update_rotation
        // - tools::update_rotation_axis
        // - tools::confirm_transform
        // - tools::confirm_rotation
        // - tools::cancel_transform
        
        .run();
}
```

## Migration Strategy

### Phase 1: Create New Systems (Non-Breaking)
1. Create `src/editor/tools/input.rs`
2. Implement `EditorInputEvent` enum
3. Implement `handle_keyboard_input()` system
4. Implement `handle_transformation_operations()` system
5. Add new systems to `map_editor.rs` (alongside old ones)

### Phase 2: Testing
1. Test all keyboard shortcuts work with new systems
2. Verify UI focus checks work correctly
3. Test edge cases (rapid key presses, mode transitions)
4. Compare behavior with old systems

### Phase 3: Cleanup (Breaking)
1. Remove old input handler systems from `selection_tool.rs`
2. Remove old system registrations from `map_editor.rs`
3. Remove old events that are now internal
4. Update exports in `tools/mod.rs`

### Phase 4: Documentation
1. Update [`input-handling.md`](input-handling.md)
2. Update [`architecture.md`](architecture.md)
3. Add code comments explaining the new architecture
4. Update user-facing documentation if needed

## Testing Checklist

### Functional Tests
- [ ] G key starts move operation
- [ ] R key starts rotate operation
- [ ] Arrow keys move selection during move mode
- [ ] Arrow keys rotate during rotate mode
- [ ] X/Y/Z keys change rotation axis
- [ ] Shift+Arrow moves by 5 units
- [ ] Enter confirms transformation
- [ ] Escape cancels transformation
- [ ] Delete/Backspace deletes selected voxels
- [ ] Escape deselects when not transforming

### UI Integration Tests
- [ ] Keyboard shortcuts don't trigger when typing in text fields
- [ ] Keyboard shortcuts don't trigger when clicking UI buttons
- [ ] Keyboard shortcuts don't trigger when interacting with menus
- [ ] UI buttons still work (Move, Rotate, Delete from properties panel)

### Edge Case Tests
- [ ] Rapid key presses don't cause issues
- [ ] Mode transitions work correctly
- [ ] Multiple selections work
- [ ] Undo/redo works with new system
- [ ] Collision detection still works

## Performance Considerations

### Before (Current)
- 17 systems run every frame
- 7+ UI focus checks per frame
- Multiple event readers per frame

### After (Proposed)
- 2 input systems run every frame
- 1 UI focus check per frame
- 1 event reader for transformations

**Expected Performance Improvement**: Minimal but measurable reduction in input handling overhead.

## Rollback Plan

If issues arise during migration:

1. Keep old systems in codebase but commented out
2. Add feature flag to switch between old and new systems
3. Can quickly revert by uncommenting old systems
4. Full rollback: Delete `input.rs` and restore old system registrations

## Future Enhancements

Once this refactoring is complete, future improvements become easier:

1. **Mouse Input Unification**: Apply same pattern to mouse input
2. **Input Remapping**: Easy to add user-configurable keybindings
3. **Input Recording**: Can record/replay input events for testing
4. **Macro System**: Could build macro recording on top of events
5. **Accessibility**: Easier to add alternative input methods

## References

- Current implementation: [`src/editor/tools/selection_tool.rs`](../../../../src/editor/tools/selection_tool.rs)
- Input handling docs: [`input-handling.md`](input-handling.md)
- Architecture docs: [`architecture.md`](architecture.md)

---

**Document Version**: 1.0.0  
**Last Updated**: 2025-10-22  
**Status**: Planning Phase