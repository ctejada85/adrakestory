//! Keyboard-based cursor movement and selection.

use super::CursorState;
use crate::editor::state::{EditorState, EditorTool, KeyboardEditMode};
use crate::editor::tools::{ActiveTransform, TransformMode};
use bevy::prelude::*;
use bevy_egui::EguiContexts;

/// System to move cursor position using keyboard (arrow keys, space, C)
/// Allows navigating the 3D grid step-by-step without using the mouse
/// Only active when keyboard edit mode is enabled
pub fn handle_keyboard_cursor_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut cursor_state: ResMut<CursorState>,
    editor_state: Res<EditorState>,
    _active_transform: Res<ActiveTransform>,
    keyboard_mode: Res<KeyboardEditMode>,
) {
    // Only allow keyboard cursor movement when in keyboard edit mode
    if !keyboard_mode.enabled {
        return;
    }

    // Check if UI wants keyboard input (user is typing in text fields, etc.)
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    // Block keyboard cursor movement for Camera tool
    if matches!(editor_state.active_tool, EditorTool::Camera) {
        return;
    }

    // Block cursor movement during active Move or Rotate operations
    // This keeps the cursor stationary while transforming selections
    if _active_transform.mode != TransformMode::None {
        return;
    }

    // Get current cursor position or default to (0, 0, 0)
    let current_pos = cursor_state.grid_pos.unwrap_or((0, 0, 0));

    // Calculate movement step (1 or 5 with Shift for fast movement)
    let step = if keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        5
    } else {
        1
    };

    let mut new_pos = current_pos;
    let mut moved = false;

    // Horizontal movement (X/Z plane)
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        new_pos.2 -= step; // Move forward (negative Z)
        moved = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        new_pos.2 += step; // Move backward (positive Z)
        moved = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        new_pos.0 -= step; // Move left (negative X)
        moved = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        new_pos.0 += step; // Move right (positive X)
        moved = true;
    }

    // Vertical movement (Y axis)
    if keyboard.just_pressed(KeyCode::Space) {
        new_pos.1 += step; // Move up
        moved = true;
    }
    if keyboard.just_pressed(KeyCode::KeyC) {
        new_pos.1 -= step; // Move down
        moved = true;
    }

    // Update cursor position if moved
    if moved {
        cursor_state.grid_pos = Some(new_pos);
        cursor_state.position = Some(Vec3::new(
            new_pos.0 as f32,
            new_pos.1 as f32,
            new_pos.2 as f32,
        ));

        // For keyboard movement, assume placement on top (+Y direction)
        cursor_state.hit_face_normal = Some(Vec3::Y);
        let placement_grid = (new_pos.0, new_pos.1 + 1, new_pos.2);
        cursor_state.placement_grid_pos = Some(placement_grid);
        cursor_state.placement_pos = Some(Vec3::new(
            placement_grid.0 as f32,
            placement_grid.1 as f32,
            placement_grid.2 as f32,
        ));

        info!("Cursor moved to grid position: {:?}", new_pos);
    }
}

/// System to handle keyboard-based selection (Enter key)
/// Allows selecting voxels with Enter when in keyboard edit mode
pub fn handle_keyboard_selection(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    cursor_state: Res<CursorState>,
    mut editor_state: ResMut<EditorState>,
    keyboard_mode: Res<KeyboardEditMode>,
    mut update_events: EventWriter<crate::editor::tools::UpdateSelectionHighlights>,
) {
    // Only allow keyboard selection when in keyboard edit mode
    if !keyboard_mode.enabled {
        return;
    }

    // Check if UI wants keyboard input (user is typing in text fields, etc.)
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    // Check if select tool is active
    if !matches!(editor_state.active_tool, EditorTool::Select) {
        return;
    }

    // Check if Enter key was just pressed
    if !keyboard.just_pressed(KeyCode::Enter) {
        return;
    }

    // Get cursor grid position
    let Some(grid_pos) = cursor_state.grid_pos else {
        return;
    };

    // Toggle selection of voxel at this position
    if editor_state.selected_voxels.contains(&grid_pos) {
        editor_state.selected_voxels.remove(&grid_pos);
        info!("Deselected voxel at {:?} (keyboard)", grid_pos);
    } else {
        editor_state.selected_voxels.insert(grid_pos);
        info!("Selected voxel at {:?} (keyboard)", grid_pos);
    }

    // Trigger highlight update
    update_events.send(crate::editor::tools::UpdateSelectionHighlights);
}
