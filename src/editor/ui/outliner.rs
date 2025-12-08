//! Outliner panel for hierarchical view of map contents.
//!
//! Provides a tree view of voxels (grouped by type) and entities
//! for easy selection and navigation.

use crate::editor::renderer::RenderMapEvent;
use crate::editor::state::EditorState;
use crate::editor::tools::UpdateSelectionHighlights;
use crate::systems::game::components::VoxelType;
use crate::systems::game::map::format::EntityType;
use bevy::prelude::*;
use bevy_egui::egui;
use std::collections::{BTreeMap, HashMap};

/// State for the outliner panel
#[derive(Resource, Default)]
pub struct OutlinerState {
    /// Whether the voxels section is expanded
    pub voxels_expanded: bool,
    /// Whether the entities section is expanded
    pub entities_expanded: bool,
    /// Filter text for searching
    pub filter_text: String,
    /// Which voxel types are expanded
    pub voxel_type_expanded: HashMap<VoxelType, bool>,
}

impl OutlinerState {
    pub fn new() -> Self {
        Self {
            voxels_expanded: true,
            entities_expanded: true,
            filter_text: String::new(),
            voxel_type_expanded: HashMap::new(),
        }
    }
}

/// Render the outliner panel on the left side
pub fn render_outliner_panel(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    outliner_state: &mut OutlinerState,
    selection_events: &mut EventWriter<UpdateSelectionHighlights>,
    render_events: &mut EventWriter<RenderMapEvent>,
) {
    let response = egui::SidePanel::left("outliner")
        .default_width(200.0)
        .min_width(150.0)
        .max_width(350.0)
        .show(ctx, |ui| {
            // Header with search
            ui.horizontal(|ui| {
                ui.heading("Outliner");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("🔍").on_hover_text("Filter items").clicked() {
                        // Toggle filter visibility (filter is always shown for simplicity)
                    }
                });
            });
            
            // Filter/search box
            ui.horizontal(|ui| {
                ui.label("🔍");
                ui.add(
                    egui::TextEdit::singleline(&mut outliner_state.filter_text)
                        .hint_text("Filter...")
                        .desired_width(ui.available_width() - 10.0)
                );
            });
            
            ui.separator();
            
            // Map name header
            let map_name = &editor_state.current_map.metadata.name;
            ui.label(format!("📁 {}", if map_name.is_empty() { "Untitled" } else { map_name }));
            
            ui.separator();
            
            // Scrollable content area
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // Voxels section
                    render_voxels_section(ui, editor_state, outliner_state, &mut *selection_events);
                    
                    ui.add_space(8.0);
                    
                    // Entities section
                    render_entities_section(ui, editor_state, outliner_state, selection_events, render_events);
                });
        });

    // Store panel width in egui memory for viewport overlays to use
    let panel_width = response.response.rect.width();
    ctx.memory_mut(|mem| {
        mem.data
            .insert_temp(egui::Id::new("outliner").with("__panel_width"), panel_width);
    });
}

/// Render the voxels section of the outliner
fn render_voxels_section(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    outliner_state: &mut OutlinerState,
    selection_events: &mut EventWriter<UpdateSelectionHighlights>,
) {
    let voxel_count = editor_state.current_map.world.voxels.len();
    
    // Collapsible header
    let header = egui::CollapsingHeader::new(format!("🧱 Voxels ({})", voxel_count))
        .default_open(outliner_state.voxels_expanded)
        .show(ui, |ui| {
            if voxel_count == 0 {
                ui.label("No voxels in map");
                return;
            }
            
            // Group voxels by type (use BTreeMap for deterministic ordering)
            let mut voxels_by_type: BTreeMap<VoxelType, Vec<(i32, i32, i32)>> = BTreeMap::new();
            for voxel in &editor_state.current_map.world.voxels {
                voxels_by_type
                    .entry(voxel.voxel_type)
                    .or_default()
                    .push(voxel.pos);
            }
            
            // Filter
            let filter = outliner_state.filter_text.to_lowercase();
            
            // Render each voxel type group
            for (voxel_type, positions) in voxels_by_type.iter() {
                let type_name = format!("{:?}", voxel_type);
                
                // Skip if doesn't match filter
                if !filter.is_empty() && !type_name.to_lowercase().contains(&filter) {
                    continue;
                }
                
                let icon = get_voxel_type_icon(voxel_type);
                let is_expanded = outliner_state.voxel_type_expanded.get(voxel_type).copied().unwrap_or(false);
                
                let header_response = egui::CollapsingHeader::new(format!("{} {} ({})", icon, type_name, positions.len()))
                    .default_open(is_expanded)
                    .show(ui, |ui| {
                        // Show all positions in a scrollable area if there are many
                        let needs_scroll = positions.len() > 50;
                        let mut show_content = |ui: &mut egui::Ui| {
                            for pos in positions.iter() {
                                let is_selected = editor_state.selected_voxels.contains(pos);
                                
                                let label = format!("  ({}, {}, {})", pos.0, pos.1, pos.2);
                                let response = ui.selectable_label(is_selected, label);
                                
                                if response.clicked() {
                                    if is_selected {
                                        editor_state.selected_voxels.remove(pos);
                                    } else {
                                        editor_state.selected_voxels.insert(*pos);
                                    }
                                    selection_events.send(UpdateSelectionHighlights);
                                }
                                
                                // Show position on hover
                                response.on_hover_text(format!("Position: {:?}\nClick to toggle selection", pos));
                            }
                        };
                        
                        if needs_scroll {
                            // Use a bounded scroll area for large lists
                            egui::ScrollArea::vertical()
                                .max_height(200.0)
                                .id_salt(format!("voxel_scroll_{:?}", voxel_type))
                                .show(ui, |ui| {
                                    show_content(ui);
                                });
                        } else {
                            show_content(ui);
                        }
                    });
                
                // Track expansion state
                outliner_state.voxel_type_expanded.insert(*voxel_type, header_response.body_returned.is_some());
            }
            
            // Select all / deselect all buttons
            ui.separator();
            ui.horizontal(|ui| {
                if ui.small_button("Select All").clicked() {
                    for voxel in &editor_state.current_map.world.voxels {
                        editor_state.selected_voxels.insert(voxel.pos);
                    }
                    selection_events.send(UpdateSelectionHighlights);
                }
                if ui.small_button("Deselect").clicked() {
                    editor_state.selected_voxels.clear();
                    selection_events.send(UpdateSelectionHighlights);
                }
            });
        });
    
    outliner_state.voxels_expanded = header.fully_open();
}

/// Render the entities section of the outliner
fn render_entities_section(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    outliner_state: &mut OutlinerState,
    selection_events: &mut EventWriter<UpdateSelectionHighlights>,
    render_events: &mut EventWriter<RenderMapEvent>,
) {
    let entity_count = editor_state.current_map.entities.len();
    
    // Collapsible header
    let header = egui::CollapsingHeader::new(format!("📍 Entities ({})", entity_count))
        .default_open(outliner_state.entities_expanded)
        .show(ui, |ui| {
            if entity_count == 0 {
                ui.label("No entities in map");
                ui.label("Use Entity Place tool to add");
                return;
            }
            
            // Filter
            let filter = outliner_state.filter_text.to_lowercase();
            
            // Track entities to delete (can't modify while iterating)
            let mut entity_to_delete: Option<usize> = None;
            
            // Render each entity
            for (index, entity_data) in editor_state.current_map.entities.iter().enumerate() {
                let icon = get_entity_type_icon(&entity_data.entity_type);
                let type_name = format!("{:?}", entity_data.entity_type);
                
                // Get display name (use custom property or type name)
                let display_name = entity_data.properties
                    .get("name")
                    .cloned()
                    .unwrap_or_else(|| type_name.clone());
                
                // Skip if doesn't match filter
                if !filter.is_empty() 
                    && !type_name.to_lowercase().contains(&filter)
                    && !display_name.to_lowercase().contains(&filter) 
                {
                    continue;
                }
                
                let is_selected = editor_state.selected_entities.contains(&index);
                
                ui.horizontal(|ui| {
                    // Selection toggle
                    let label = format!("{} {}", icon, display_name);
                    let response = ui.selectable_label(is_selected, label);
                    
                    if response.clicked() {
                        // Clear other selections and select this entity
                        editor_state.selected_voxels.clear();
                        if is_selected {
                            editor_state.selected_entities.remove(&index);
                        } else {
                            editor_state.selected_entities.clear();
                            editor_state.selected_entities.insert(index);
                        }
                        selection_events.send(UpdateSelectionHighlights);
                    }
                    
                    // Context menu on right-click
                    response.context_menu(|ui| {
                        if ui.button("🗑️ Delete").clicked() {
                            entity_to_delete = Some(index);
                            ui.close_menu();
                        }
                        // Future: duplicate, rename, etc.
                    });
                    
                    // Hover info
                    let (x, y, z) = entity_data.position;
                    response.on_hover_text(format!(
                        "Type: {:?}\nPosition: ({:.1}, {:.1}, {:.1})\nClick to select\nRight-click for options",
                        entity_data.entity_type, x, y, z
                    ));
                });
            }
            
            // Handle deletion (outside the iteration)
            if let Some(index) = entity_to_delete {
                if index < editor_state.current_map.entities.len() {
                    editor_state.current_map.entities.remove(index);
                    editor_state.selected_entities.clear();
                    editor_state.mark_modified();
                    selection_events.send(UpdateSelectionHighlights);
                    render_events.send(RenderMapEvent);
                    info!("Deleted entity at index {}", index);
                }
            }
            
            ui.separator();
            
            // Quick add buttons
            ui.horizontal(|ui| {
                if ui.small_button("+ Add").clicked() {
                    // This would ideally open a dropdown, but for now just add a player spawn
                    ui.memory_mut(|mem| mem.toggle_popup(ui.make_persistent_id("add_entity_popup")));
                }
                
                if !editor_state.selected_entities.is_empty() && ui.small_button("Deselect").clicked() {
                    editor_state.selected_entities.clear();
                    selection_events.send(UpdateSelectionHighlights);
                }
            });
        });
    
    outliner_state.entities_expanded = header.fully_open();
}

/// Get icon for voxel type
fn get_voxel_type_icon(voxel_type: &VoxelType) -> &'static str {
    match voxel_type {
        VoxelType::Air => "⬛",
        VoxelType::Grass => "🟩",
        VoxelType::Dirt => "🟫",
        VoxelType::Stone => "⬜",
    }
}

/// Get icon for entity type
fn get_entity_type_icon(entity_type: &EntityType) -> &'static str {
    match entity_type {
        EntityType::PlayerSpawn => "🟢",
        EntityType::Npc => "🔵",
        EntityType::Enemy => "🔴",
        EntityType::Item => "🟡",
        EntityType::Trigger => "🟣",
    }
}
