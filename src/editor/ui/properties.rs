//! Properties panel UI for editing object properties.

use crate::editor::state::{EditorState, EditorTool};
use crate::editor::tools::{
    ActiveTransform, CancelTransform, ConfirmTransform, DeleteSelectedVoxels, StartMoveOperation,
    StartRotateOperation, TransformMode,
};
use crate::systems::game::components::VoxelType;
use crate::systems::game::map::format::{EntityType, SubVoxelPattern};
use bevy::prelude::*;
use bevy_egui::egui;

/// Render the right-side properties panel
pub fn render_properties_panel(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    active_transform: &ActiveTransform,
    delete_events: &mut EventWriter<DeleteSelectedVoxels>,
    move_events: &mut EventWriter<StartMoveOperation>,
    rotate_events: &mut EventWriter<StartRotateOperation>,
    confirm_events: &mut EventWriter<ConfirmTransform>,
    cancel_events: &mut EventWriter<CancelTransform>,
) {
    egui::SidePanel::right("properties")
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.heading("Properties");
            ui.separator();

            // Tool-specific properties
            render_tool_properties(
                ui,
                editor_state,
                active_transform,
                delete_events,
                move_events,
                rotate_events,
                confirm_events,
                cancel_events,
            );

            ui.separator();

            // Map information
            render_map_info(ui, editor_state);

            ui.separator();

            // Cursor information
            render_cursor_info(ui, editor_state);
        });
}

/// Render properties specific to the active tool
fn render_tool_properties(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    active_transform: &ActiveTransform,
    delete_events: &mut EventWriter<DeleteSelectedVoxels>,
    move_events: &mut EventWriter<StartMoveOperation>,
    rotate_events: &mut EventWriter<StartRotateOperation>,
    confirm_events: &mut EventWriter<ConfirmTransform>,
    cancel_events: &mut EventWriter<CancelTransform>,
) {
    ui.label("Tool Settings");
    ui.separator();

    match &mut editor_state.active_tool {
        EditorTool::VoxelPlace {
            voxel_type,
            pattern,
        } => {
            ui.label("Voxel Place Tool");

            ui.horizontal(|ui| {
                ui.label("Type:");
                egui::ComboBox::from_id_salt("voxel_type")
                    .selected_text(format!("{:?}", voxel_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(voxel_type, VoxelType::Grass, "Grass");
                        ui.selectable_value(voxel_type, VoxelType::Dirt, "Dirt");
                        ui.selectable_value(voxel_type, VoxelType::Stone, "Stone");
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Pattern:");
                egui::ComboBox::from_id_salt("voxel_pattern")
                    .selected_text(format!("{:?}", pattern))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(pattern, SubVoxelPattern::Full, "Full");
                        ui.selectable_value(pattern, SubVoxelPattern::PlatformXZ, "Platform (Horizontal)");
                        ui.selectable_value(pattern, SubVoxelPattern::PlatformXY, "Platform (Wall Z)");
                        ui.selectable_value(pattern, SubVoxelPattern::PlatformYZ, "Platform (Wall X)");
                        ui.selectable_value(pattern, SubVoxelPattern::StaircaseX, "Staircase (+X)");
                        ui.selectable_value(pattern, SubVoxelPattern::StaircaseNegX, "Staircase (-X)");
                        ui.selectable_value(pattern, SubVoxelPattern::StaircaseZ, "Staircase (+Z)");
                        ui.selectable_value(pattern, SubVoxelPattern::StaircaseNegZ, "Staircase (-Z)");
                        ui.selectable_value(pattern, SubVoxelPattern::Pillar, "Pillar");
                    });
            });

            ui.label("Click to place voxel");
        }

        EditorTool::VoxelRemove => {
            ui.label("Voxel Remove Tool");
            ui.label("Click to remove voxel");
            ui.label("Or press Delete/Backspace");
        }

        EditorTool::EntityPlace { entity_type } => {
            ui.label("Entity Place Tool");

            ui.horizontal(|ui| {
                ui.label("Type:");
                egui::ComboBox::from_id_salt("entity_type")
                    .selected_text(format!("{:?}", entity_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(entity_type, EntityType::PlayerSpawn, "Player Spawn");
                        ui.selectable_value(entity_type, EntityType::Enemy, "Enemy");
                        ui.selectable_value(entity_type, EntityType::Item, "Item");
                        ui.selectable_value(entity_type, EntityType::Trigger, "Trigger");
                    });
            });

            ui.label("Click to place entity");
        }

        EditorTool::Select => {
            ui.label("Select Tool");
            
            // Show transform mode status
            if active_transform.mode != TransformMode::None {
                ui.separator();
                ui.colored_label(egui::Color32::YELLOW, "🔄 Transform Mode Active");
                
                match active_transform.mode {
                    TransformMode::Move => {
                        ui.label(format!("Mode: Move"));
                        ui.label(format!(
                            "Offset: ({}, {}, {})",
                            active_transform.current_offset.x,
                            active_transform.current_offset.y,
                            active_transform.current_offset.z
                        ));
                        ui.label(format!("Voxels: {}", active_transform.selected_voxels.len()));
                    }
                    TransformMode::Rotate => {
                        ui.label(format!("Mode: Rotate"));
                        ui.label(format!("Axis: {:?}", active_transform.rotation_axis));
                        ui.label(format!("Angle: {}°", active_transform.rotation_angle * 90));
                        ui.label(format!("Voxels: {}", active_transform.selected_voxels.len()));
                    }
                    TransformMode::None => {}
                }
                
                ui.separator();
                ui.label("Controls:");
                ui.label("• Arrow keys to move");
                ui.label("• Shift+Arrow for 5 units");
                ui.label("• PageUp/Down for Y-axis");
                ui.label("• Enter to confirm");
                ui.label("• Escape to cancel");
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("✓ Confirm").clicked() {
                        confirm_events.send(ConfirmTransform);
                    }
                    
                    if ui.button("✗ Cancel").clicked() {
                        cancel_events.send(CancelTransform);
                    }
                });
            } else {
                ui.label("Click to select voxels");
                ui.label("Press G to move");
                ui.label("Press R to rotate");
                ui.label("Delete/Backspace to remove");

                if !editor_state.selected_voxels.is_empty() {
                    ui.separator();
                    ui.label(format!(
                        "Selected: {} voxel{}",
                        editor_state.selected_voxels.len(),
                        if editor_state.selected_voxels.len() == 1 {
                            ""
                        } else {
                            "s"
                        }
                    ));

                    // Show first few selected positions
                    if editor_state.selected_voxels.len() <= 5 {
                        ui.label("Positions:");
                        for pos in editor_state.selected_voxels.iter().take(5) {
                            ui.label(format!("  ({}, {}, {})", pos.0, pos.1, pos.2));
                        }
                    } else {
                        ui.label(format!(
                            "Showing first 5 of {} positions:",
                            editor_state.selected_voxels.len()
                        ));
                        for pos in editor_state.selected_voxels.iter().take(5) {
                            ui.label(format!("  ({}, {}, {})", pos.0, pos.1, pos.2));
                        }
                        ui.label("  ...");
                    }

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("🔄 Move").clicked() {
                            move_events.send(StartMoveOperation);
                            info!("Move button clicked for {} voxels", editor_state.selected_voxels.len());
                        }
                        
                        if ui.button("🔃 Rotate").clicked() {
                            rotate_events.send(StartRotateOperation);
                            info!("Rotate button clicked for {} voxels", editor_state.selected_voxels.len());
                        }
                        
                        if ui.button("🗑 Delete").clicked() {
                            delete_events.send(DeleteSelectedVoxels);
                            info!(
                                "Delete button clicked for {} voxels",
                                editor_state.selected_voxels.len()
                            );
                        }
                    });
                    
                    ui.horizontal(|ui| {
                        if ui.button("Clear Selection").clicked() {
                            editor_state.clear_selections();
                        }
                    });
                } else {
                    ui.separator();
                    ui.label("No voxels selected");
                }
            }
        }

        EditorTool::Camera => {
            ui.label("Camera Tool");
            ui.label("Right-click drag: Orbit");
            ui.label("Middle-click drag: Pan");
            ui.label("Scroll: Zoom");
            ui.label("Home: Reset camera");
        }
    }
}

/// Render map information section
fn render_map_info(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    ui.label("Map Information");
    ui.separator();

    // Handle name field
    ui.horizontal(|ui| {
        ui.label("Name:");
        if ui
            .text_edit_singleline(&mut editor_state.current_map.metadata.name)
            .changed()
        {
            editor_state.mark_modified();
        }
    });

    // Handle author field
    ui.horizontal(|ui| {
        ui.label("Author:");
        if ui
            .text_edit_singleline(&mut editor_state.current_map.metadata.author)
            .changed()
        {
            editor_state.mark_modified();
        }
    });

    ui.horizontal(|ui| {
        ui.label("Description:");
    });
    if ui
        .text_edit_multiline(&mut editor_state.current_map.metadata.description)
        .changed()
    {
        editor_state.mark_modified();
    }

    ui.separator();

    let world = &editor_state.current_map.world;
    ui.label(format!(
        "Dimensions: {}×{}×{}",
        world.width, world.height, world.depth
    ));
    ui.label(format!("Voxels: {}", world.voxels.len()));
    ui.label(format!(
        "Entities: {}",
        editor_state.current_map.entities.len()
    ));
}

/// Render cursor information section
fn render_cursor_info(ui: &mut egui::Ui, editor_state: &EditorState) {
    ui.label("Cursor");
    ui.separator();

    if let Some(grid_pos) = editor_state.cursor_grid_pos {
        ui.label(format!(
            "Grid: ({}, {}, {})",
            grid_pos.0, grid_pos.1, grid_pos.2
        ));
    } else {
        ui.label("Grid: -");
    }

    if let Some(world_pos) = editor_state.cursor_position {
        ui.label(format!(
            "World: ({:.2}, {:.2}, {:.2})",
            world_pos.x, world_pos.y, world_pos.z
        ));
    } else {
        ui.label("World: -");
    }
}
