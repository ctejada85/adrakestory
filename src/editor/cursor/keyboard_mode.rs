//! Keyboard mode toggle and tool/play shortcuts.

use crate::editor::play::{PlayMapEvent, PlayTestState, StopGameEvent};
use crate::editor::state::{EditorState, EditorTool, KeyboardEditMode};
use bevy::prelude::*;
use bevy_egui::EguiContexts;

/// System to toggle keyboard edit mode (I to enter, Escape to exit)
/// Note: Escape only exits keyboard mode when there are no selections in Select tool
pub fn toggle_keyboard_edit_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut keyboard_mode: ResMut<KeyboardEditMode>,
    mut contexts: EguiContexts,
    editor_state: Res<EditorState>,
) {
    // Don't toggle if UI wants keyboard input
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    // Enter keyboard edit mode with I key
    if keyboard.just_pressed(KeyCode::KeyI) {
        keyboard_mode.enable();
        info!("Keyboard edit mode ENABLED");
    }

    // Exit keyboard edit mode with Escape key
    // BUT: Only if we're not in Select tool with active selections
    // (In Select tool, Escape should clear selections first, handled by handle_selection_mode_input)
    if keyboard.just_pressed(KeyCode::Escape) {
        // Check if we're in Select tool with selections
        let has_selections = matches!(editor_state.active_tool, EditorTool::Select)
            && !editor_state.selected_voxels.is_empty();

        // Only exit keyboard mode if there are no selections
        if !has_selections {
            keyboard_mode.disable();
            info!("Keyboard edit mode DISABLED");
        }
        // If there are selections, let handle_selection_mode_input clear them
        // and keep keyboard mode active
    }
}

/// System to handle tool switching with keyboard shortcuts
/// B or 1 = VoxelPlace, V or 2 = Select, X = VoxelRemove, E = EntityPlace, C = Camera
pub fn handle_tool_switching(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut editor_state: ResMut<EditorState>,
    mut tool_memory: ResMut<crate::editor::state::ToolMemory>,
) {
    // Check if UI wants keyboard input (user is typing in text fields, etc.)
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    // Helper to save current tool parameters before switching
    let save_current_params =
        |editor_state: &EditorState, tool_memory: &mut crate::editor::state::ToolMemory| {
            match &editor_state.active_tool {
                EditorTool::VoxelPlace {
                    voxel_type,
                    pattern,
                } => {
                    tool_memory.voxel_type = *voxel_type;
                    tool_memory.voxel_pattern = *pattern;
                }
                EditorTool::EntityPlace { entity_type } => {
                    tool_memory.entity_type = *entity_type;
                }
                _ => {}
            }
        };

    // Switch to VoxelPlace tool with B or 1 key
    if (keyboard.just_pressed(KeyCode::Digit1) || keyboard.just_pressed(KeyCode::KeyB))
        && !matches!(editor_state.active_tool, EditorTool::VoxelPlace { .. })
    {
        save_current_params(&editor_state, &mut tool_memory);
        editor_state.active_tool = EditorTool::VoxelPlace {
            voxel_type: tool_memory.voxel_type,
            pattern: tool_memory.voxel_pattern,
        };
        info!("Switched to VoxelPlace tool");
    }

    // Switch to Select tool with V or 2 key
    if (keyboard.just_pressed(KeyCode::Digit2) || keyboard.just_pressed(KeyCode::KeyV))
        && !matches!(editor_state.active_tool, EditorTool::Select)
    {
        save_current_params(&editor_state, &mut tool_memory);
        editor_state.active_tool = EditorTool::Select;
        info!("Switched to Select tool");
    }

    // Switch to VoxelRemove tool with X key
    if keyboard.just_pressed(KeyCode::KeyX)
        && !matches!(editor_state.active_tool, EditorTool::VoxelRemove)
    {
        save_current_params(&editor_state, &mut tool_memory);
        editor_state.active_tool = EditorTool::VoxelRemove;
        info!("Switched to VoxelRemove tool");
    }

    // Switch to EntityPlace tool with E key
    if keyboard.just_pressed(KeyCode::KeyE)
        && !matches!(editor_state.active_tool, EditorTool::EntityPlace { .. })
    {
        save_current_params(&editor_state, &mut tool_memory);
        editor_state.active_tool = EditorTool::EntityPlace {
            entity_type: tool_memory.entity_type,
        };
        info!("Switched to EntityPlace tool");
    }

    // Switch to Camera tool with C key
    if keyboard.just_pressed(KeyCode::KeyC)
        && !matches!(editor_state.active_tool, EditorTool::Camera)
    {
        save_current_params(&editor_state, &mut tool_memory);
        editor_state.active_tool = EditorTool::Camera;
        info!("Switched to Camera tool");
    }
}

/// System to handle play/stop shortcuts (F5 to play, Shift+F5 to stop)
pub fn handle_play_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    play_state: Res<PlayTestState>,
    mut play_events: EventWriter<PlayMapEvent>,
    mut stop_events: EventWriter<StopGameEvent>,
    mut contexts: EguiContexts,
) {
    // Don't handle shortcuts if UI wants keyboard input
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    // F5 key handling
    if keyboard.just_pressed(KeyCode::F5) {
        let shift_held =
            keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

        if shift_held {
            // Shift+F5 to Stop
            if play_state.is_running {
                stop_events.send(StopGameEvent);
                info!("Stop game triggered via Shift+F5");
            }
        } else {
            // F5 to Play
            if !play_state.is_running {
                play_events.send(PlayMapEvent);
                info!("Play game triggered via F5");
            }
        }
    }
}
