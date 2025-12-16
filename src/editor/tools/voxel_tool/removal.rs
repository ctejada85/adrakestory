//! Voxel removal handling.

use super::drag_state::VoxelRemoveDragState;
use super::{try_remove_voxel, VoxelToolInput, DRAG_MOVEMENT_THRESHOLD};
use crate::editor::cursor::CursorState;
use crate::editor::history::EditorHistory;
use crate::editor::state::{EditorState, EditorTool};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContexts;

/// Handle voxel removal when the tool is active
pub fn handle_voxel_removal(
    cursor_state: Res<CursorState>,
    mut editor_state: ResMut<EditorState>,
    mut history: ResMut<EditorHistory>,
    input: VoxelToolInput,
    mut contexts: EguiContexts,
    mut drag_state: ResMut<VoxelRemoveDragState>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    // Check if voxel remove tool is active
    if !matches!(editor_state.active_tool, EditorTool::VoxelRemove) {
        // Reset drag state if tool changed
        drag_state.is_dragging = false;
        drag_state.last_grid_pos = None;
        drag_state.drag_start_screen_pos = None;
        return;
    }

    // Check if pointer is over any UI area
    let ctx = contexts.ctx_mut();
    let pointer_over_ui = ctx.is_pointer_over_area() || ctx.is_using_pointer();
    let ui_wants_keyboard = ctx.wants_keyboard_input();

    // Handle mouse release - stop drag removal
    if input.mouse_button.just_released(MouseButton::Left) {
        drag_state.is_dragging = false;
        drag_state.last_grid_pos = None;
        drag_state.drag_start_screen_pos = None;
        return;
    }

    // Handle keyboard delete (not draggable)
    if !ui_wants_keyboard
        && (input.keyboard.just_pressed(KeyCode::Delete)
            || input.keyboard.just_pressed(KeyCode::Backspace))
    {
        if let Some(grid_pos) = cursor_state.grid_pos {
            try_remove_voxel(&mut editor_state, &mut history, grid_pos);
        }
        return;
    }

    // Check if left mouse button was just pressed - start drag
    if !pointer_over_ui && input.mouse_button.just_pressed(MouseButton::Left) {
        drag_state.is_dragging = true;

        let Some(grid_pos) = cursor_state.grid_pos else {
            return;
        };

        drag_state.last_grid_pos = Some(grid_pos);

        // Record the screen position where the drag started
        if let Ok(window) = window_query.get_single() {
            drag_state.drag_start_screen_pos = window.cursor_position();
        }

        try_remove_voxel(&mut editor_state, &mut history, grid_pos);
    }
}

/// Handle continuous drag removal while mouse is held
pub fn handle_voxel_drag_removal(
    cursor_state: Res<CursorState>,
    mut editor_state: ResMut<EditorState>,
    mut history: ResMut<EditorHistory>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    mut drag_state: ResMut<VoxelRemoveDragState>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    // Only process if we're in drag-remove mode
    if !drag_state.is_dragging {
        return;
    }

    // Check if voxel remove tool is active
    if !matches!(editor_state.active_tool, EditorTool::VoxelRemove) {
        drag_state.is_dragging = false;
        drag_state.last_grid_pos = None;
        drag_state.drag_start_screen_pos = None;
        return;
    }

    // Stop drag if mouse is released
    if !mouse_button.pressed(MouseButton::Left) {
        drag_state.is_dragging = false;
        drag_state.last_grid_pos = None;
        drag_state.drag_start_screen_pos = None;
        return;
    }

    // Check if pointer is over any UI area
    let ctx = contexts.ctx_mut();
    if ctx.is_pointer_over_area() || ctx.is_using_pointer() {
        return;
    }

    // Check if mouse has actually moved enough from the drag start position
    // This prevents false drag triggers when grid_pos changes due to geometry changes
    // (e.g., after removing a voxel, the cursor now points at the voxel behind it)
    if let (Some(start_pos), Ok(window)) =
        (drag_state.drag_start_screen_pos, window_query.get_single())
    {
        if let Some(current_pos) = window.cursor_position() {
            let distance = (current_pos - start_pos).length();
            if distance < DRAG_MOVEMENT_THRESHOLD {
                return;
            }
        }
    }

    // Get current cursor grid position
    let Some(grid_pos) = cursor_state.grid_pos else {
        return;
    };

    // Only remove if this is a different position than last time
    if drag_state.last_grid_pos == Some(grid_pos) {
        return;
    }

    // Update last position
    drag_state.last_grid_pos = Some(grid_pos);

    // Try to remove voxel at this position
    try_remove_voxel(&mut editor_state, &mut history, grid_pos);
}
