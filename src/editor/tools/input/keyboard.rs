//! Keyboard input handling for different editor modes.

use super::EditorInputEvent;
use crate::systems::game::map::geometry::RotationAxis;
use bevy::prelude::*;

/// Handle input when in selection mode (no active transformation)
pub fn handle_selection_mode_input(
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

    // Entity movement with arrow keys (grid-aligned movement for entities)
    // Entities should move in full grid units to stay aligned with voxels
    let step = if keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        5.0 // Move 5 grid units with Shift
    } else {
        1.0 // Move 1 grid unit normally
    };

    let mut offset = Vec3::ZERO;
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
    if keyboard.just_pressed(KeyCode::PageUp) {
        offset.y += step;
    }
    if keyboard.just_pressed(KeyCode::PageDown) {
        offset.y -= step;
    }

    if offset != Vec3::ZERO {
        events.send(EditorInputEvent::MoveSelectedEntities(offset));
    }
}

/// Handle input during move operation
pub fn handle_move_mode_input(
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

    // Y-axis movement - Space for jump/up, C for crouch/down
    if keyboard.just_pressed(KeyCode::PageUp) || keyboard.just_pressed(KeyCode::Space) {
        offset.y += step;
    }
    if keyboard.just_pressed(KeyCode::PageDown) || keyboard.just_pressed(KeyCode::KeyC) {
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
pub fn handle_rotate_mode_input(
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
