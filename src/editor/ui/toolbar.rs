//! Toolbar UI with menu bar and quick actions.

use crate::editor::file_io::{SaveMapAsEvent, SaveMapEvent};
use crate::editor::history::EditorHistory;
use crate::editor::recent_files::{OpenRecentFileEvent, RecentFiles};
use crate::editor::state::{EditorState, EditorTool, EditorUIState, ToolMemory};
use crate::systems::game::components::VoxelType;
use crate::systems::game::map::format::{EntityType, SubVoxelPattern};
use bevy::prelude::*;
use bevy_egui::egui;

/// Render the top toolbar with menu bar and quick actions
#[allow(clippy::too_many_arguments)]
pub fn render_toolbar(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    ui_state: &mut EditorUIState,
    tool_memory: &mut ToolMemory,
    history: &EditorHistory,
    recent_files: &mut RecentFiles,
    save_events: &mut EventWriter<SaveMapEvent>,
    save_as_events: &mut EventWriter<SaveMapAsEvent>,
    open_recent_events: &mut EventWriter<OpenRecentFileEvent>,
) {
    // Menu bar panel
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            render_file_menu(
                ui,
                editor_state,
                ui_state,
                recent_files,
                save_events,
                save_as_events,
                open_recent_events,
            );
            render_edit_menu(ui, history);
            render_view_menu(ui, editor_state);
            render_tools_menu(ui, editor_state, tool_memory);
            render_help_menu(ui, ui_state);

            // Spacer to push map name to the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let map_name = &editor_state.current_map.metadata.name;
                let modified = if editor_state.is_modified { " *" } else { "" };
                ui.label(format!("{}{}", map_name, modified));
            });
        });
    });

    // Horizontal toolbar panel (below menu bar)
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;

            // === Tool Buttons ===
            render_tool_buttons(ui, editor_state, tool_memory);

            ui.separator();

            // === Context-Sensitive Options ===
            render_tool_options(ui, editor_state, tool_memory);

            ui.separator();

            // === View Toggles ===
            render_view_toggles(ui, editor_state);
        });
    });
}

/// Render the tool selection buttons
fn render_tool_buttons(ui: &mut egui::Ui, editor_state: &mut EditorState, tool_memory: &mut ToolMemory) {
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
    // This is called when any tool button is clicked
    let save_current_params = |editor_state: &EditorState, tool_memory: &mut ToolMemory| {
        match &editor_state.active_tool {
            EditorTool::VoxelPlace { voxel_type, pattern } => {
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
    ) && !is_select {
        save_current_params(editor_state, tool_memory);
        editor_state.active_tool = EditorTool::Select;
    }

    // Voxel Place Tool (B / 1)
    if tool_button(
        ui,
        "✏️",
        "Voxel Place Tool (B)\nClick to place voxels",
        is_voxel_place,
    ) && !is_voxel_place {
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
    ) && !is_voxel_remove {
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
    ) && !is_camera {
        save_current_params(editor_state, tool_memory);
        editor_state.active_tool = EditorTool::Camera;
    }
}

/// Render context-sensitive tool options (type, pattern dropdowns)
fn render_tool_options(ui: &mut egui::Ui, editor_state: &mut EditorState, tool_memory: &mut ToolMemory) {
    match &mut editor_state.active_tool {
        EditorTool::VoxelPlace {
            voxel_type,
            pattern,
        } => {
            // Voxel Type dropdown
            ui.label("Type:");
            let type_changed = egui::ComboBox::from_id_salt("toolbar_voxel_type")
                .selected_text(format!("{:?}", voxel_type))
                .width(80.0)
                .show_ui(ui, |ui| {
                    let mut changed = false;
                    changed |= ui.selectable_value(voxel_type, VoxelType::Grass, "🟩 Grass").changed();
                    changed |= ui.selectable_value(voxel_type, VoxelType::Dirt, "🟫 Dirt").changed();
                    changed |= ui.selectable_value(voxel_type, VoxelType::Stone, "⬜ Stone").changed();
                    changed
                }).inner.unwrap_or(false);

            // Pattern dropdown
            ui.label("Pattern:");
            let pattern_changed = egui::ComboBox::from_id_salt("toolbar_pattern")
                .selected_text(pattern_short_name(pattern))
                .width(100.0)
                .show_ui(ui, |ui| {
                    let mut changed = false;
                    changed |= ui.selectable_value(pattern, SubVoxelPattern::Full, "■ Full").changed();
                    changed |= ui.selectable_value(pattern, SubVoxelPattern::PlatformXZ, "▬ Platform").changed();
                    changed |= ui.selectable_value(pattern, SubVoxelPattern::StaircaseX, "⌐ Stairs +X").changed();
                    changed |= ui.selectable_value(pattern, SubVoxelPattern::StaircaseNegX, "⌐ Stairs -X").changed();
                    changed |= ui.selectable_value(pattern, SubVoxelPattern::StaircaseZ, "⌐ Stairs +Z").changed();
                    changed |= ui.selectable_value(pattern, SubVoxelPattern::StaircaseNegZ, "⌐ Stairs -Z").changed();
                    changed |= ui.selectable_value(pattern, SubVoxelPattern::Pillar, "│ Pillar").changed();
                    changed |= ui.selectable_value(pattern, SubVoxelPattern::PlatformXY, "▐ Wall Z").changed();
                    changed |= ui.selectable_value(pattern, SubVoxelPattern::PlatformYZ, "▌ Wall X").changed();
                    changed |= ui.selectable_value(pattern, SubVoxelPattern::Fence, "┼ Fence").changed();
                    changed
                }).inner.unwrap_or(false);

            // Update tool memory when parameters change
            if type_changed {
                tool_memory.voxel_type = *voxel_type;
            }
            if pattern_changed {
                tool_memory.voxel_pattern = *pattern;
            }
        }

        EditorTool::EntityPlace { entity_type } => {
            // Entity Type dropdown
            ui.label("Entity:");
            let entity_changed = egui::ComboBox::from_id_salt("toolbar_entity_type")
                .selected_text(entity_type_display(entity_type))
                .width(120.0)
                .show_ui(ui, |ui| {
                    let mut changed = false;
                    changed |= ui.selectable_value(entity_type, EntityType::PlayerSpawn, "🟢 Player Spawn").changed();
                    changed |= ui.selectable_value(entity_type, EntityType::Npc, "🔵 NPC").changed();
                    changed |= ui.selectable_value(entity_type, EntityType::Enemy, "🔴 Enemy").changed();
                    changed |= ui.selectable_value(entity_type, EntityType::Item, "🟡 Item").changed();
                    changed |= ui.selectable_value(entity_type, EntityType::Trigger, "🟣 Trigger").changed();
                    changed
                }).inner.unwrap_or(false);

            // Update tool memory when entity type changes
            if entity_changed {
                tool_memory.entity_type = *entity_type;
            }
        }

        EditorTool::Select => {
            // Show selection info
            let voxel_count = editor_state.selected_voxels.len();
            let entity_count = editor_state.selected_entities.len();

            if voxel_count > 0 || entity_count > 0 {
                ui.label(format!(
                    "Selected: {} voxel{}, {} entit{}",
                    voxel_count,
                    if voxel_count == 1 { "" } else { "s" },
                    entity_count,
                    if entity_count == 1 { "y" } else { "ies" }
                ));

                if ui
                    .button("🗑️ Delete")
                    .on_hover_text("Delete selected (Del)")
                    .clicked()
                {
                    // Deletion is handled by keyboard input system
                    info!("Delete button clicked");
                }

                if ui
                    .button("Clear")
                    .on_hover_text("Clear selection (Esc)")
                    .clicked()
                {
                    editor_state.clear_selections();
                }
            } else {
                ui.label("Click to select");
            }
        }

        EditorTool::VoxelRemove => {
            ui.label("Click voxels to remove");
        }

        EditorTool::Camera => {
            ui.label("RMB: Orbit | MMB: Pan | Scroll: Zoom");
        }
    }
}

/// Render view toggle buttons
fn render_view_toggles(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    // Grid toggle
    let grid_icon = if editor_state.show_grid { "▦" } else { "▢" };
    let grid_text = format!("{} Grid", grid_icon);
    if ui
        .selectable_label(editor_state.show_grid, grid_text)
        .on_hover_text("Toggle grid (G)")
        .clicked()
    {
        editor_state.show_grid = !editor_state.show_grid;
        info!("Grid toggled: {}", editor_state.show_grid);
    }

    // Snap toggle
    let snap_icon = if editor_state.snap_to_grid {
        "⊞"
    } else {
        "⊟"
    };
    let snap_text = format!("{} Snap", snap_icon);
    if ui
        .selectable_label(editor_state.snap_to_grid, snap_text)
        .on_hover_text("Toggle snap to grid (Shift+G)")
        .clicked()
    {
        editor_state.snap_to_grid = !editor_state.snap_to_grid;
        info!("Snap toggled: {}", editor_state.snap_to_grid);
    }
}

/// Get a short display name for a pattern
fn pattern_short_name(pattern: &SubVoxelPattern) -> &'static str {
    match pattern {
        SubVoxelPattern::Full => "Full",
        SubVoxelPattern::PlatformXZ => "Platform",
        SubVoxelPattern::PlatformXY => "Wall Z",
        SubVoxelPattern::PlatformYZ => "Wall X",
        SubVoxelPattern::StaircaseX => "Stairs +X",
        SubVoxelPattern::StaircaseNegX => "Stairs -X",
        SubVoxelPattern::StaircaseZ => "Stairs +Z",
        SubVoxelPattern::StaircaseNegZ => "Stairs -Z",
        SubVoxelPattern::Pillar => "Pillar",
        SubVoxelPattern::Fence => "Fence",
    }
}

/// Get a display string for an entity type
fn entity_type_display(entity_type: &EntityType) -> &'static str {
    match entity_type {
        EntityType::PlayerSpawn => "🟢 Player Spawn",
        EntityType::Npc => "🔵 NPC",
        EntityType::Enemy => "🔴 Enemy",
        EntityType::Item => "🟡 Item",
        EntityType::Trigger => "🟣 Trigger",
    }
}

// === Menu Rendering Functions ===

fn render_file_menu(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    ui_state: &mut EditorUIState,
    recent_files: &mut RecentFiles,
    save_events: &mut EventWriter<SaveMapEvent>,
    save_as_events: &mut EventWriter<SaveMapAsEvent>,
    open_recent_events: &mut EventWriter<OpenRecentFileEvent>,
) {
    ui.menu_button("File", |ui| {
        if ui.button("📄 New").clicked() {
            if editor_state.is_modified {
                ui_state.unsaved_changes_dialog_open = true;
                ui_state.pending_action = Some(crate::editor::state::PendingAction::NewMap);
            } else {
                ui_state.new_map_dialog_open = true;
            }
            ui.close_menu();
        }

        if ui.button("📁 Open...").clicked() {
            if editor_state.is_modified {
                ui_state.unsaved_changes_dialog_open = true;
                ui_state.pending_action = Some(crate::editor::state::PendingAction::OpenMap);
            } else {
                ui_state.file_dialog_open = true;
            }
            ui.close_menu();
        }

        // Recent Files submenu
        ui.menu_button("🕐 Recent Files", |ui| {
            if recent_files.is_empty() {
                ui.label("No recent files");
            } else {
                for path in recent_files.files.iter() {
                    let display_name = RecentFiles::get_display_name(path);
                    let tooltip = path.display().to_string();

                    if ui.button(&display_name).on_hover_text(&tooltip).clicked() {
                        if editor_state.is_modified {
                            ui_state.unsaved_changes_dialog_open = true;
                            ui_state.pending_action = Some(
                                crate::editor::state::PendingAction::OpenRecentFile(path.clone()),
                            );
                        } else {
                            open_recent_events.send(OpenRecentFileEvent { path: path.clone() });
                        }
                        ui.close_menu();
                    }
                }

                ui.separator();

                if ui.button("🗑 Clear Recent Files").clicked() {
                    recent_files.clear();
                    ui.close_menu();
                }
            }
        });

        ui.separator();

        if ui.button("💾 Save").clicked() {
            save_events.send(SaveMapEvent);
            ui.close_menu();
        }

        if ui.button("💾 Save As...").clicked() {
            save_as_events.send(SaveMapAsEvent);
            ui.close_menu();
        }

        ui.separator();

        if ui.button("🚪 Quit").clicked() {
            if editor_state.is_modified {
                ui_state.unsaved_changes_dialog_open = true;
                ui_state.pending_action = Some(crate::editor::state::PendingAction::Quit);
            } else {
                info!("Quit clicked");
            }
            ui.close_menu();
        }
    });
}

fn render_edit_menu(ui: &mut egui::Ui, history: &EditorHistory) {
    ui.menu_button("Edit", |ui| {
        let can_undo = history.can_undo();
        let can_redo = history.can_redo();

        ui.add_enabled_ui(can_undo, |ui| {
            let undo_text = if let Some(desc) = history.undo_description() {
                format!("↶ Undo {}", desc)
            } else {
                "↶ Undo".to_string()
            };

            if ui.button(undo_text).clicked() {
                info!("Undo clicked");
                ui.close_menu();
            }
        });

        ui.add_enabled_ui(can_redo, |ui| {
            let redo_text = if let Some(desc) = history.redo_description() {
                format!("↷ Redo {}", desc)
            } else {
                "↷ Redo".to_string()
            };

            if ui.button(redo_text).clicked() {
                info!("Redo clicked");
                ui.close_menu();
            }
        });
    });
}

fn render_view_menu(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    ui.menu_button("View", |ui| {
        if ui
            .checkbox(&mut editor_state.show_grid, "▦ Show Grid")
            .clicked()
        {
            info!("Grid visibility: {}", editor_state.show_grid);
        }

        if ui
            .checkbox(&mut editor_state.snap_to_grid, "⊞ Snap to Grid")
            .clicked()
        {
            info!("Snap to grid: {}", editor_state.snap_to_grid);
        }

        ui.separator();

        ui.label("Grid Opacity");
        ui.add(egui::Slider::new(&mut editor_state.grid_opacity, 0.0..=1.0));
    });
}

fn render_tools_menu(ui: &mut egui::Ui, editor_state: &mut EditorState, tool_memory: &mut ToolMemory) {
    ui.menu_button("Tools", |ui| {
        // Helper to save current tool parameters before switching
        let save_current_params = |editor_state: &EditorState, tool_memory: &mut ToolMemory| {
            match &editor_state.active_tool {
                EditorTool::VoxelPlace { voxel_type, pattern } => {
                    tool_memory.voxel_type = *voxel_type;
                    tool_memory.voxel_pattern = *pattern;
                }
                EditorTool::EntityPlace { entity_type } => {
                    tool_memory.entity_type = *entity_type;
                }
                _ => {}
            }
        };

        let is_select = matches!(editor_state.active_tool, EditorTool::Select);
        if ui.selectable_label(is_select, "🔲 Select (V)").clicked() {
            if !is_select {
                save_current_params(editor_state, tool_memory);
                editor_state.active_tool = EditorTool::Select;
            }
            ui.close_menu();
        }

        let is_voxel_place = matches!(editor_state.active_tool, EditorTool::VoxelPlace { .. });
        if ui
            .selectable_label(is_voxel_place, "✏️ Voxel Place (B)")
            .clicked()
        {
            if !is_voxel_place {
                save_current_params(editor_state, tool_memory);
                editor_state.active_tool = EditorTool::VoxelPlace {
                    voxel_type: tool_memory.voxel_type,
                    pattern: tool_memory.voxel_pattern,
                };
            }
            ui.close_menu();
        }

        let is_voxel_remove = matches!(editor_state.active_tool, EditorTool::VoxelRemove);
        if ui
            .selectable_label(is_voxel_remove, "🗑️ Voxel Remove (X)")
            .clicked()
        {
            if !is_voxel_remove {
                save_current_params(editor_state, tool_memory);
                editor_state.active_tool = EditorTool::VoxelRemove;
            }
            ui.close_menu();
        }

        let is_entity_place = matches!(editor_state.active_tool, EditorTool::EntityPlace { .. });
        if ui
            .selectable_label(is_entity_place, "📍 Entity Place (E)")
            .clicked()
        {
            if !is_entity_place {
                save_current_params(editor_state, tool_memory);
                editor_state.active_tool = EditorTool::EntityPlace {
                    entity_type: tool_memory.entity_type,
                };
            }
            ui.close_menu();
        }

        let is_camera = matches!(editor_state.active_tool, EditorTool::Camera);
        if ui.selectable_label(is_camera, "📷 Camera (C)").clicked() {
            if !is_camera {
                save_current_params(editor_state, tool_memory);
                editor_state.active_tool = EditorTool::Camera;
            }
            ui.close_menu();
        }
    });
}

fn render_help_menu(ui: &mut egui::Ui, ui_state: &mut EditorUIState) {
    ui.menu_button("Help", |ui| {
        if ui.button("⌨️ Keyboard Shortcuts").clicked() {
            ui_state.shortcuts_help_open = true;
            ui.close_menu();
        }

        if ui.button("ℹ️ About").clicked() {
            ui_state.about_dialog_open = true;
            ui.close_menu();
        }
    });
}
