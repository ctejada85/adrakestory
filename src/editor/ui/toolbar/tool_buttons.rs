//! Tool button rendering for the toolbar.

use crate::editor::state::{EditorState, EditorTool, ToolMemory};
use bevy_egui::egui;

/// Render the tool selection buttons
pub fn render_tool_buttons(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    tool_memory: &mut ToolMemory,
) {
    // Get current tool state for highlighting
    let is_select = matches!(editor_state.active_tool, EditorTool::Select);
    let is_voxel_place = matches!(editor_state.active_tool, EditorTool::VoxelPlace { .. });
    let is_voxel_remove = matches!(editor_state.active_tool, EditorTool::VoxelRemove);
    let is_entity_place = matches!(editor_state.active_tool, EditorTool::EntityPlace { .. });
    let is_camera = matches!(editor_state.active_tool, EditorTool::Camera);

    // Tool button style helper
    let tool_button = |ui: &mut egui::Ui, icon: &str, tooltip: &str, is_active: bool| -> bool {
        let button = egui::Button::new(icon).min_size(egui::vec2(28.0, 24.0));

        let response = if is_active {
            ui.add(button.fill(egui::Color32::from_rgb(70, 100, 150)))
        } else {
            ui.add(button)
        };

        response.on_hover_text(tooltip).clicked()
    };

    // Save current tool parameters before switching
    let save_current_params = |editor_state: &EditorState, tool_memory: &mut ToolMemory| {
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

    // Select Tool (V / 2)
    if tool_button(
        ui,
        "🔲",
        "Select Tool (V)\nClick to select voxels/entities",
        is_select,
    ) && !is_select
    {
        save_current_params(editor_state, tool_memory);
        editor_state.active_tool = EditorTool::Select;
    }

    // Voxel Place Tool (B / 1)
    if tool_button(
        ui,
        "✏️",
        "Voxel Place Tool (B)\nClick to place voxels",
        is_voxel_place,
    ) && !is_voxel_place
    {
        save_current_params(editor_state, tool_memory);
        // Restore remembered voxel_type and pattern
        editor_state.active_tool = EditorTool::VoxelPlace {
            voxel_type: tool_memory.voxel_type,
            pattern: tool_memory.voxel_pattern,
        };
    }

    // Voxel Remove Tool (X)
    if tool_button(
        ui,
        "🗑️",
        "Voxel Remove Tool (X)\nClick to remove voxels",
        is_voxel_remove,
    ) && !is_voxel_remove
    {
        save_current_params(editor_state, tool_memory);
        editor_state.active_tool = EditorTool::VoxelRemove;
    }

    // Entity Place Tool (E)
    if tool_button(
        ui,
        "📍",
        "Entity Place Tool (E)\nClick to place entities",
        is_entity_place,
    ) && !is_entity_place
    {
        save_current_params(editor_state, tool_memory);
        // Restore remembered entity type
        editor_state.active_tool = EditorTool::EntityPlace {
            entity_type: tool_memory.entity_type,
        };
    }

    // Camera Tool (C)
    if tool_button(
        ui,
        "📷",
        "Camera Tool (C)\nDrag to control camera",
        is_camera,
    ) && !is_camera
    {
        save_current_params(editor_state, tool_memory);
        editor_state.active_tool = EditorTool::Camera;
    }
}
