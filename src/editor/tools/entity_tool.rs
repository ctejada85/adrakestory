//! Entity placement tool.

use crate::editor::history::{EditorAction, EditorHistory};
use crate::editor::state::{EditorState, EditorTool};
use crate::systems::game::map::format::EntityData;
use bevy::prelude::*;
use std::collections::HashMap;

/// Handle entity placement when the tool is active
pub fn handle_entity_placement(
    mut editor_state: ResMut<EditorState>,
    mut history: ResMut<EditorHistory>,
    mouse_button: Res<ButtonInput<MouseButton>>,
) {
    // Check if entity place tool is active
    let entity_type = match &editor_state.active_tool {
        EditorTool::EntityPlace { entity_type } => *entity_type,
        _ => return,
    };

    // Check if left mouse button was just pressed
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    // Get cursor position (use world position for entities, not grid)
    let Some(cursor_pos) = editor_state.cursor_position else {
        return;
    };

    // Create new entity data
    let entity_data = EntityData {
        entity_type,
        position: (cursor_pos.x, cursor_pos.y, cursor_pos.z),
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

    info!("Placed {:?} entity at {:?}", entity_type, cursor_pos);
}
