//! Entity placement tool.

use crate::editor::cursor::CursorState;
use crate::editor::history::{EditorAction, EditorHistory};
use crate::editor::state::{EditorState, EditorTool};
use crate::systems::game::map::format::EntityData;
use bevy::prelude::*;
use bevy_egui::EguiContexts;
use std::collections::HashMap;

/// Handle entity placement when the tool is active
pub fn handle_entity_placement(
    cursor_state: Res<CursorState>,
    mut editor_state: ResMut<EditorState>,
    mut history: ResMut<EditorHistory>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
) {
    // Check if entity place tool is active
    let entity_type = match &editor_state.active_tool {
        EditorTool::EntityPlace { entity_type } => *entity_type,
        _ => return,
    };

    // Check if pointer is over any UI area (panels, buttons, backgrounds, etc.)
    // Also check is_using_pointer() for active interactions like dragging resize handles
    let ctx = contexts.ctx_mut();
    if ctx.is_pointer_over_area() || ctx.is_using_pointer() {
        return;
    }

    // Check if left mouse button was just pressed
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    // Get cursor grid position for grid-aligned entity placement
    let Some(grid_pos) = cursor_state.placement_grid_pos else {
        return;
    };

    // Convert grid position to world coordinates (centered on grid cell)
    let position = (grid_pos.0 as f32, grid_pos.1 as f32, grid_pos.2 as f32);

    // Create new entity data
    let entity_data = EntityData {
        entity_type,
        position,
        properties: HashMap::new(),
    };

    // Add to map
    let index = editor_state.current_map.entities.len();
    editor_state.current_map.entities.push(entity_data.clone());
    editor_state.mark_modified();

    // Record action in history
    history.push(EditorAction::PlaceEntity {
        index,
        data: entity_data,
    });

    info!("Placed {:?} entity at grid {:?}", entity_type, grid_pos);
}
