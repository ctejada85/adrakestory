//! Properties panel UI for editing object properties.

use crate::editor::cursor::CursorState;
use crate::editor::state::{EditorState, EditorTool};
use crate::editor::tools::{
    ActiveTransform, CancelTransform, ConfirmTransform, DeleteSelectedVoxels, StartMoveOperation,
    StartRotateOperation, TransformMode,
};
use crate::systems::game::components::VoxelType;
use crate::systems::game::map::format::{EntityType, SubVoxelPattern};
use bevy::prelude::*;
use bevy_egui::egui;

/// Bundle of event writers for transform operations
#[derive(bevy::ecs::system::SystemParam)]
pub struct TransformEvents<'w> {
    pub delete: EventWriter<'w, DeleteSelectedVoxels>,
    pub move_start: EventWriter<'w, StartMoveOperation>,
    pub rotate_start: EventWriter<'w, StartRotateOperation>,
    pub confirm: EventWriter<'w, ConfirmTransform>,
    pub cancel: EventWriter<'w, CancelTransform>,
}

/// Render the right-side properties panel
pub fn render_properties_panel(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    cursor_state: &CursorState,
    active_transform: &ActiveTransform,
    events: &mut TransformEvents,
) {
    egui::SidePanel::right("properties")
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.heading("Properties");
            ui.separator();

            // Tool-specific properties
            render_tool_properties(ui, editor_state, active_transform, events);

            ui.separator();

            // Selected entity properties (if any entity is selected)
            if !editor_state.selected_entities.is_empty() {
                render_selected_entity_properties(ui, editor_state);
                ui.separator();
            }

            // Map information
            render_map_info(ui, editor_state);

            ui.separator();

            // Cursor information
            render_cursor_info(ui, cursor_state);
        });
}

/// Render properties specific to the active tool
fn render_tool_properties(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    active_transform: &ActiveTransform,
    events: &mut TransformEvents,
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
                        ui.selectable_value(
                            pattern,
                            SubVoxelPattern::PlatformXZ,
                            "Platform (Horizontal)",
                        );
                        ui.selectable_value(
                            pattern,
                            SubVoxelPattern::PlatformXY,
                            "Platform (Wall Z)",
                        );
                        ui.selectable_value(
                            pattern,
                            SubVoxelPattern::PlatformYZ,
                            "Platform (Wall X)",
                        );
                        ui.selectable_value(pattern, SubVoxelPattern::StaircaseX, "Staircase (+X)");
                        ui.selectable_value(
                            pattern,
                            SubVoxelPattern::StaircaseNegX,
                            "Staircase (-X)",
                        );
                        ui.selectable_value(pattern, SubVoxelPattern::StaircaseZ, "Staircase (+Z)");
                        ui.selectable_value(
                            pattern,
                            SubVoxelPattern::StaircaseNegZ,
                            "Staircase (-Z)",
                        );
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
                        ui.selectable_value(entity_type, EntityType::Npc, "NPC");
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
                        ui.label("Mode: Move");
                        ui.label(format!(
                            "Offset: ({}, {}, {})",
                            active_transform.current_offset.x,
                            active_transform.current_offset.y,
                            active_transform.current_offset.z
                        ));
                        ui.label(format!(
                            "Voxels: {}",
                            active_transform.selected_voxels.len()
                        ));
                    }
                    TransformMode::Rotate => {
                        ui.label("Mode: Rotate");
                        ui.label(format!("Axis: {:?}", active_transform.rotation_axis));
                        ui.label(format!("Angle: {}°", active_transform.rotation_angle * 90));
                        ui.label(format!(
                            "Voxels: {}",
                            active_transform.selected_voxels.len()
                        ));
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
                        events.confirm.send(ConfirmTransform);
                    }

                    if ui.button("✗ Cancel").clicked() {
                        events.cancel.send(CancelTransform);
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
                            events.move_start.send(StartMoveOperation);
                            info!(
                                "Move button clicked for {} voxels",
                                editor_state.selected_voxels.len()
                            );
                        }

                        if ui.button("🔃 Rotate").clicked() {
                            events.rotate_start.send(StartRotateOperation);
                            info!(
                                "Rotate button clicked for {} voxels",
                                editor_state.selected_voxels.len()
                            );
                        }

                        if ui.button("🗑 Delete").clicked() {
                            events.delete.send(DeleteSelectedVoxels);
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
fn render_cursor_info(ui: &mut egui::Ui, cursor_state: &CursorState) {
    ui.label("Cursor");
    ui.separator();

    if let Some(grid_pos) = cursor_state.grid_pos {
        ui.label(format!(
            "Grid: ({}, {}, {})",
            grid_pos.0, grid_pos.1, grid_pos.2
        ));
    } else {
        ui.label("Grid: -");
    }

    if let Some(world_pos) = cursor_state.position {
        ui.label(format!(
            "World: ({:.2}, {:.2}, {:.2})",
            world_pos.x, world_pos.y, world_pos.z
        ));
    } else {
        ui.label("World: -");
    }
}

/// Render properties for selected entities
fn render_selected_entity_properties(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    ui.label("Selected Entity");
    ui.separator();

    // For simplicity, edit first selected entity
    let index = match editor_state.selected_entities.iter().next() {
        Some(&idx) => idx,
        None => return,
    };

    // Need to check bounds and get entity info
    if index >= editor_state.current_map.entities.len() {
        ui.label("Invalid entity selection");
        return;
    }

    // Display entity type
    let entity_type = editor_state.current_map.entities[index].entity_type;
    ui.label(format!("Type: {:?}", entity_type));

    // Position editing
    ui.horizontal(|ui| {
        ui.label("Position:");
    });

    let mut position_changed = false;
    let (mut x, mut y, mut z) = editor_state.current_map.entities[index].position;

    ui.horizontal(|ui| {
        ui.label("X:");
        if ui.add(egui::DragValue::new(&mut x).speed(0.1)).changed() {
            position_changed = true;
        }
        ui.label("Y:");
        if ui.add(egui::DragValue::new(&mut y).speed(0.1)).changed() {
            position_changed = true;
        }
        ui.label("Z:");
        if ui.add(egui::DragValue::new(&mut z).speed(0.1)).changed() {
            position_changed = true;
        }
    });

    if position_changed {
        editor_state.current_map.entities[index].position = (x, y, z);
        editor_state.mark_modified();
    }

    // NPC-specific properties
    if entity_type == EntityType::Npc {
        ui.separator();
        ui.label("NPC Properties:");

        // Name
        let current_name = editor_state.current_map.entities[index]
            .properties
            .get("name")
            .cloned()
            .unwrap_or_else(|| "NPC".to_string());
        let mut name = current_name.clone();

        ui.horizontal(|ui| {
            ui.label("Name:");
            if ui.text_edit_singleline(&mut name).changed() {
                editor_state.current_map.entities[index]
                    .properties
                    .insert("name".to_string(), name);
                editor_state.mark_modified();
            }
        });

        // Radius
        let current_radius: f32 = editor_state.current_map.entities[index]
            .properties
            .get("radius")
            .and_then(|r| r.parse().ok())
            .unwrap_or(0.3);
        let mut radius = current_radius;

        ui.horizontal(|ui| {
            ui.label("Radius:");
            if ui
                .add(egui::Slider::new(&mut radius, 0.1..=1.0).step_by(0.05))
                .changed()
            {
                editor_state.current_map.entities[index]
                    .properties
                    .insert("radius".to_string(), format!("{:.2}", radius));
                editor_state.mark_modified();
            }
        });
    }

    ui.separator();

    // Delete button
    if ui.button("🗑 Delete Entity").clicked() {
        editor_state.current_map.entities.remove(index);
        editor_state.selected_entities.clear();
        editor_state.mark_modified();
        info!("Deleted entity at index {}", index);
    }
}
