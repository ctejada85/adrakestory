//! Voxel placement and removal tools.

use crate::editor::cursor::CursorState;
use crate::editor::history::{EditorAction, EditorHistory};
use crate::editor::state::{EditorState, EditorTool};
use crate::systems::game::map::format::VoxelData;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContexts;

/// Bundle of input resources for voxel tools
#[derive(bevy::ecs::system::SystemParam)]
pub struct VoxelToolInput<'w> {
    pub mouse_button: Res<'w, ButtonInput<MouseButton>>,
    pub keyboard: Res<'w, ButtonInput<KeyCode>>,
}

/// Resource tracking drag-to-place state for voxel tool
#[derive(Resource, Default)]
pub struct VoxelDragState {
    /// Whether we're currently in a drag-place operation
    pub is_dragging: bool,
    /// Last grid position we placed a voxel at
    pub last_placed_pos: Option<(i32, i32, i32)>,
    /// Last cursor grid position (to detect movement)
    pub last_cursor_grid_pos: Option<(i32, i32, i32)>,
    /// Screen position where the drag started (to detect actual mouse movement)
    pub drag_start_screen_pos: Option<Vec2>,
}

/// Handle voxel placement when the tool is active
pub fn handle_voxel_placement(
    cursor_state: Res<CursorState>,
    mut editor_state: ResMut<EditorState>,
    mut history: ResMut<EditorHistory>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    mut drag_state: ResMut<VoxelDragState>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    // Check if voxel place tool is active
    let (voxel_type, pattern) = match &editor_state.active_tool {
        EditorTool::VoxelPlace {
            voxel_type,
            pattern,
        } => (*voxel_type, *pattern),
        _ => {
            // Reset drag state if tool changed
            drag_state.is_dragging = false;
            drag_state.last_placed_pos = None;
            drag_state.last_cursor_grid_pos = None;
            drag_state.drag_start_screen_pos = None;
            return;
        }
    };

    // Check if pointer is over any UI area (panels, buttons, etc.)
    // Also check is_using_pointer() for active interactions like dragging resize handles
    let ctx = contexts.ctx_mut();
    if ctx.is_pointer_over_area() || ctx.is_using_pointer() {
        return;
    }

    // Handle mouse release - stop drag placement
    if mouse_button.just_released(MouseButton::Left) {
        drag_state.is_dragging = false;
        drag_state.last_placed_pos = None;
        drag_state.last_cursor_grid_pos = None;
        drag_state.drag_start_screen_pos = None;
        return;
    }

    // Check if left mouse button was just pressed - start drag
    if mouse_button.just_pressed(MouseButton::Left) {
        drag_state.is_dragging = true;

        // Use placement_grid_pos instead of grid_pos for face-aware placement on initial click
        let Some(grid_pos) = cursor_state.placement_grid_pos else {
            return;
        };

        // Track both the placed position and the cursor grid position
        drag_state.last_placed_pos = Some(grid_pos);
        drag_state.last_cursor_grid_pos = cursor_state.grid_pos;
        
        // Record the screen position where the drag started
        if let Ok(window) = window_query.get_single() {
            drag_state.drag_start_screen_pos = window.cursor_position();
        }

        // Check if voxel already exists at this position
        let voxel_exists = editor_state
            .current_map
            .world
            .voxels
            .iter()
            .any(|v| v.pos == grid_pos);

        if voxel_exists {
            info!("Voxel already exists at {:?}", grid_pos);
            return;
        }

        // Create new voxel data
        let voxel_data = VoxelData {
            pos: grid_pos,
            voxel_type,
            pattern: Some(pattern),
            rotation_state: None,
        };

        // Add to map
        editor_state
            .current_map
            .world
            .voxels
            .push(voxel_data.clone());
        editor_state.mark_modified();

        // Record action in history
        history.push(EditorAction::PlaceVoxel {
            pos: grid_pos,
            data: voxel_data,
        });

        info!("Placed {:?} voxel at {:?}", voxel_type, grid_pos);
    }

    // Handle middle mouse button - quick remove voxel (like using remove tool)
    if mouse_button.just_pressed(MouseButton::Middle) {
        if let Some(grid_pos) = cursor_state.grid_pos {
            try_remove_voxel(&mut editor_state, &mut history, grid_pos);
        }
    }
}

/// Minimum mouse movement in pixels required to trigger drag placement/removal
/// This prevents accidental double actions when the cursor grid position changes
/// due to underlying geometry changes (e.g., after placing/removing a voxel)
const DRAG_MOVEMENT_THRESHOLD: f32 = 5.0;

/// Handle continuous drag placement while mouse is held
/// During drag, voxels are placed adjacent to the last placed voxel in the direction of cursor movement
pub fn handle_voxel_drag_placement(
    cursor_state: Res<CursorState>,
    mut editor_state: ResMut<EditorState>,
    mut history: ResMut<EditorHistory>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    mut drag_state: ResMut<VoxelDragState>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    // Only process if we're in drag-place mode
    if !drag_state.is_dragging {
        return;
    }

    // Check if voxel place tool is active
    let (voxel_type, pattern) = match &editor_state.active_tool {
        EditorTool::VoxelPlace {
            voxel_type,
            pattern,
        } => (*voxel_type, *pattern),
        _ => {
            drag_state.is_dragging = false;
            drag_state.last_placed_pos = None;
            drag_state.last_cursor_grid_pos = None;
            drag_state.drag_start_screen_pos = None;
            return;
        }
    };

    // Stop drag if mouse is released
    if !mouse_button.pressed(MouseButton::Left) {
        drag_state.is_dragging = false;
        drag_state.last_placed_pos = None;
        drag_state.last_cursor_grid_pos = None;
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
    if let (Some(start_pos), Ok(window)) = (drag_state.drag_start_screen_pos, window_query.get_single()) {
        if let Some(current_pos) = window.cursor_position() {
            let distance = (current_pos - start_pos).length();
            if distance < DRAG_MOVEMENT_THRESHOLD {
                return;
            }
        }
    }

    // Get current cursor grid position (the voxel being pointed at, not placement pos)
    let Some(current_cursor_pos) = cursor_state.grid_pos else {
        return;
    };

    // Only process if cursor moved to a different grid position
    if drag_state.last_cursor_grid_pos == Some(current_cursor_pos) {
        return;
    }

    // Get the last placed position
    let Some(last_placed) = drag_state.last_placed_pos else {
        // If we don't have a last placed position, use placement_grid_pos
        if let Some(grid_pos) = cursor_state.placement_grid_pos {
            drag_state.last_cursor_grid_pos = Some(current_cursor_pos);
            try_place_voxel(
                &mut editor_state,
                &mut history,
                grid_pos,
                voxel_type,
                pattern,
                &mut drag_state,
            );
        }
        return;
    };

    // Calculate movement direction from the last cursor position
    let last_cursor = drag_state.last_cursor_grid_pos.unwrap_or(last_placed);

    // Calculate the delta movement
    let dx = current_cursor_pos.0 - last_cursor.0;
    let dy = current_cursor_pos.1 - last_cursor.1;
    let dz = current_cursor_pos.2 - last_cursor.2;

    // Update last cursor position
    drag_state.last_cursor_grid_pos = Some(current_cursor_pos);

    // Determine the dominant direction of movement and place voxel adjacent to last placed
    // We place in the direction the cursor is moving, one step at a time
    let new_pos = if dx.abs() >= dy.abs() && dx.abs() >= dz.abs() {
        // X is dominant
        (last_placed.0 + dx.signum(), last_placed.1, last_placed.2)
    } else if dy.abs() >= dx.abs() && dy.abs() >= dz.abs() {
        // Y is dominant
        (last_placed.0, last_placed.1 + dy.signum(), last_placed.2)
    } else {
        // Z is dominant
        (last_placed.0, last_placed.1, last_placed.2 + dz.signum())
    };

    // Try to place at the calculated position
    try_place_voxel(
        &mut editor_state,
        &mut history,
        new_pos,
        voxel_type,
        pattern,
        &mut drag_state,
    );
}

/// Helper function to place a voxel and update drag state
fn try_place_voxel(
    editor_state: &mut ResMut<EditorState>,
    history: &mut ResMut<EditorHistory>,
    grid_pos: (i32, i32, i32),
    voxel_type: crate::systems::game::components::VoxelType,
    pattern: crate::systems::game::map::format::SubVoxelPattern,
    drag_state: &mut ResMut<VoxelDragState>,
) {
    // Check if voxel already exists at this position
    let voxel_exists = editor_state
        .current_map
        .world
        .voxels
        .iter()
        .any(|v| v.pos == grid_pos);

    if voxel_exists {
        return;
    }

    // Update last placed position
    drag_state.last_placed_pos = Some(grid_pos);

    // Create new voxel data
    let voxel_data = VoxelData {
        pos: grid_pos,
        voxel_type,
        pattern: Some(pattern),
        rotation_state: None,
    };

    // Add to map
    editor_state
        .current_map
        .world
        .voxels
        .push(voxel_data.clone());
    editor_state.mark_modified();

    // Record action in history
    history.push(EditorAction::PlaceVoxel {
        pos: grid_pos,
        data: voxel_data,
    });

    info!("Drag-placed {:?} voxel at {:?}", voxel_type, grid_pos);
}

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
        && (input.keyboard.just_pressed(KeyCode::Delete) || input.keyboard.just_pressed(KeyCode::Backspace))
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

/// Resource tracking drag-to-remove state for voxel remove tool
#[derive(Resource, Default)]
pub struct VoxelRemoveDragState {
    /// Whether we're currently in a drag-remove operation
    pub is_dragging: bool,
    /// Last grid position we removed a voxel at (to avoid duplicates)
    pub last_grid_pos: Option<(i32, i32, i32)>,
    /// Screen position where the drag started (to detect actual mouse movement)
    pub drag_start_screen_pos: Option<Vec2>,
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
    if let (Some(start_pos), Ok(window)) = (drag_state.drag_start_screen_pos, window_query.get_single()) {
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

/// Helper function to remove a voxel at a position
fn try_remove_voxel(
    editor_state: &mut ResMut<EditorState>,
    history: &mut ResMut<EditorHistory>,
    grid_pos: (i32, i32, i32),
) {
    // Find and remove voxel at this position
    if let Some(index) = editor_state
        .current_map
        .world
        .voxels
        .iter()
        .position(|v| v.pos == grid_pos)
    {
        let voxel_data = editor_state.current_map.world.voxels.remove(index);
        editor_state.mark_modified();

        // Record action in history
        history.push(EditorAction::RemoveVoxel {
            pos: grid_pos,
            data: voxel_data.clone(),
        });

        info!("Removed voxel at {:?}", grid_pos);
    }
}
