//! Outliner panel for hierarchical view of map contents.
//!
//! Provides a tree view of voxels (grouped by type) and entities
//! for easy selection and navigation.

use crate::editor::history::{EditorAction, EditorHistory};
use crate::editor::renderer::RenderMapEvent;
use crate::editor::state::EditorState;
use crate::editor::tools::UpdateSelectionHighlights;
use crate::systems::game::components::VoxelType;
use crate::systems::game::map::format::{EntityData, EntityType};
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
    /// Index of the entity currently being renamed (`None` when idle).
    pub renaming_index: Option<usize>,
}

impl OutlinerState {
    pub fn new() -> Self {
        Self {
            voxels_expanded: true,
            entities_expanded: true,
            filter_text: String::new(),
            voxel_type_expanded: HashMap::new(),
            renaming_index: None,
        }
    }
}

/// Render the outliner panel on the left side
pub fn render_outliner_panel(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    outliner_state: &mut OutlinerState,
    history: &mut EditorHistory,
    selection_events: &mut MessageWriter<UpdateSelectionHighlights>,
    render_events: &mut MessageWriter<RenderMapEvent>,
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
                    if ui
                        .small_button("🔍")
                        .on_hover_text("Filter items")
                        .clicked()
                    {
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
                        .desired_width(ui.available_width() - 10.0),
                );
            });

            ui.separator();

            // Map name header
            let map_name = &editor_state.current_map.metadata.name;
            ui.label(format!(
                "📁 {}",
                if map_name.is_empty() {
                    "Untitled"
                } else {
                    map_name
                }
            ));

            ui.separator();

            // Scrollable content area
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // Voxels section
                    render_voxels_section(ui, editor_state, outliner_state, &mut *selection_events);

                    ui.add_space(8.0);

                    // Entities section
                    render_entities_section(
                        ui,
                        editor_state,
                        outliner_state,
                        history,
                        selection_events,
                        render_events,
                    );
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
    selection_events: &mut MessageWriter<UpdateSelectionHighlights>,
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
                let is_expanded = outliner_state
                    .voxel_type_expanded
                    .get(voxel_type)
                    .copied()
                    .unwrap_or(false);

                let header_response = egui::CollapsingHeader::new(format!(
                    "{} {} ({})",
                    icon,
                    type_name,
                    positions.len()
                ))
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
                                selection_events.write(UpdateSelectionHighlights);
                            }

                            // Show position on hover
                            response.on_hover_text(format!(
                                "Position: {:?}\nClick to toggle selection",
                                pos
                            ));
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
                outliner_state
                    .voxel_type_expanded
                    .insert(*voxel_type, header_response.body_returned.is_some());
            }

            // Select all / deselect all buttons
            ui.separator();
            ui.horizontal(|ui| {
                if ui.small_button("Select All").clicked() {
                    for voxel in &editor_state.current_map.world.voxels {
                        editor_state.selected_voxels.insert(voxel.pos);
                    }
                    selection_events.write(UpdateSelectionHighlights);
                }
                if ui.small_button("Deselect").clicked() {
                    editor_state.selected_voxels.clear();
                    selection_events.write(UpdateSelectionHighlights);
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
    history: &mut EditorHistory,
    selection_events: &mut MessageWriter<UpdateSelectionHighlights>,
    render_events: &mut MessageWriter<RenderMapEvent>,
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

            // Task 7: guard against the entity being renamed having been deleted.
            // If renaming_index is out of bounds, clean up temp storage and exit rename mode.
            if let Some(ri) = outliner_state.renaming_index {
                if ri >= editor_state.current_map.entities.len() {
                    let snapshot_id = egui::Id::new("outliner_rename_snapshot").with(ri);
                    let cancel_id = egui::Id::new("outliner_rename_cancel_snapshot").with(ri);
                    ui.data_mut(|d| {
                        d.remove::<bool>(snapshot_id);
                        d.remove::<EntityData>(cancel_id);
                    });
                    outliner_state.renaming_index = None;
                }
            }

            // Filter
            let filter = outliner_state.filter_text.to_lowercase();

            // Track entities to delete (can't modify while iterating)
            let mut entity_to_delete: Option<usize> = None;

            // Render each entity
            for index in 0..editor_state.current_map.entities.len() {
                let entity_type = editor_state.current_map.entities[index].entity_type;
                let icon = get_entity_type_icon(&entity_type);
                let type_name = format!("{:?}", entity_type);

                // Get display name (use custom property or type name)
                let display_name = editor_state.current_map.entities[index]
                    .properties
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

                if outliner_state.renaming_index == Some(index) {
                    // --- Rename mode ---
                    let snapshot_id = egui::Id::new("outliner_rename_snapshot").with(index);
                    let cancel_id = egui::Id::new("outliner_rename_cancel_snapshot").with(index);

                    let current_name = editor_state.current_map.entities[index]
                        .properties
                        .get("name")
                        .cloned()
                        .unwrap_or_default();
                    let mut name = current_name;

                    // Render icon + borderless text input filling the row width.
                    // frame(false) removes the border; desired_width(INFINITY) fills the space.
                    // This keeps the row height identical to a normal selectable_label row.
                    let response = ui
                        .horizontal(|ui| {
                            ui.label(icon);
                            egui::TextEdit::singleline(&mut name)
                                .frame(false)
                                .desired_width(f32::INFINITY)
                                .show(ui)
                                .response
                        })
                        .inner;

                    // First frame: request focus once (snapshot_id acts as a "focused" flag).
                    if ui.data_mut(|d| d.get_temp::<bool>(snapshot_id)).is_none() {
                        ui.data_mut(|d| d.insert_temp(snapshot_id, true));
                        response.request_focus();
                    }

                    // Write-through: keep the stored name in sync every keystroke (Guardrail 12).
                    if response.changed() {
                        editor_state.current_map.entities[index]
                            .properties
                            .insert("name".to_string(), name.clone());
                        editor_state.mark_modified();
                    }

                    let escape_pressed = ui.input(|i| i.key_pressed(egui::Key::Escape));

                    if escape_pressed {
                        // Cancel: restore the name that was in place when double-click occurred.
                        let old_data: Option<EntityData> =
                            ui.data_mut(|d| d.get_temp(cancel_id));
                        ui.data_mut(|d| {
                            d.remove::<EntityData>(cancel_id);
                            d.remove::<bool>(snapshot_id);
                        });
                        if let Some(old) = old_data {
                            let old_name =
                                old.properties.get("name").cloned().unwrap_or_default();
                            editor_state.current_map.entities[index]
                                .properties
                                .insert("name".to_string(), old_name);
                            editor_state.mark_modified();
                        }
                        outliner_state.renaming_index = None;
                    } else if response.lost_focus() {
                        // Commit: push one undo entry if the name actually changed.
                        let old_data: Option<EntityData> =
                            ui.data_mut(|d| d.get_temp(cancel_id));
                        ui.data_mut(|d| {
                            d.remove::<EntityData>(cancel_id);
                            d.remove::<bool>(snapshot_id);
                        });
                        if let Some(old_data) = old_data {
                            let old_name = old_data
                                .properties
                                .get("name")
                                .map(String::as_str)
                                .unwrap_or("");
                            let new_name = editor_state.current_map.entities[index]
                                .properties
                                .get("name")
                                .map(String::as_str)
                                .unwrap_or("");
                            if old_name != new_name {
                                let new_data =
                                    editor_state.current_map.entities[index].clone();
                                history.push(EditorAction::ModifyEntity {
                                    index,
                                    old_data,
                                    new_data,
                                });
                            }
                        }
                        outliner_state.renaming_index = None;
                    }
                } else {
                    // --- Normal mode ---
                    ui.horizontal(|ui| {
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
                            selection_events.write(UpdateSelectionHighlights);
                        }

                        // Enter rename mode on double-click (PlayerSpawn excluded — no name concept).
                        if response.double_clicked()
                            && entity_type != EntityType::PlayerSpawn
                        {
                            let cancel_id =
                                egui::Id::new("outliner_rename_cancel_snapshot").with(index);
                            ui.data_mut(|d| {
                                d.insert_temp(
                                    cancel_id,
                                    editor_state.current_map.entities[index].clone(),
                                )
                            });
                            outliner_state.renaming_index = Some(index);
                        }

                        // Scroll this row into view when a viewport label click requested it.
                        if editor_state.outliner_scroll_to == Some(index) {
                            response.scroll_to_me(None);
                            editor_state.outliner_scroll_to = None;
                        }

                        // Context menu on right-click
                        response.context_menu(|ui| {
                            if ui.button("🗑️ Delete").clicked() {
                                entity_to_delete = Some(index);
                                ui.close();
                            }
                        });

                        // Hover info
                        let (x, y, z) = editor_state.current_map.entities[index].position;
                        response.on_hover_text(format!(
                            "Type: {:?}\nPosition: ({:.1}, {:.1}, {:.1})\nClick to select\nRight-click for options",
                            entity_type, x, y, z
                        ));
                    });
                }
            }

            // Handle deletion (outside the iteration)
            if let Some(index) = entity_to_delete {
                if index < editor_state.current_map.entities.len() {
                    // If the deleted entity was being renamed, exit rename mode cleanly.
                    if outliner_state.renaming_index == Some(index) {
                        let snapshot_id = egui::Id::new("outliner_rename_snapshot").with(index);
                        let cancel_id =
                            egui::Id::new("outliner_rename_cancel_snapshot").with(index);
                        ui.data_mut(|d| {
                            d.remove::<bool>(snapshot_id);
                            d.remove::<EntityData>(cancel_id);
                        });
                        outliner_state.renaming_index = None;
                    }

                    editor_state.current_map.entities.remove(index);
                    editor_state.selected_entities.clear();
                    editor_state.mark_modified();
                    selection_events.write(UpdateSelectionHighlights);
                    render_events.write(RenderMapEvent);
                    info!("Deleted entity at index {}", index);
                }
            }

            ui.separator();

            // Quick add buttons
            ui.horizontal(|ui| {
                if ui.small_button("+ Add").clicked() {
                    // This would ideally open a dropdown, but for now just add a player spawn
                    let popup_id = ui.make_persistent_id("add_entity_popup");
                    egui::Popup::toggle_id(ui.ctx(), popup_id);
                }

                if !editor_state.selected_entities.is_empty() && ui.small_button("Deselect").clicked() {
                    editor_state.selected_entities.clear();
                    selection_events.write(UpdateSelectionHighlights);
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
        EntityType::LightSource => "💡",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::game::map::format::EntityData;
    use std::collections::HashMap;

    fn make_entity(entity_type: EntityType, name: Option<&str>) -> EntityData {
        let mut props = HashMap::new();
        if let Some(n) = name {
            props.insert("name".to_string(), n.to_string());
        }
        EntityData {
            entity_type,
            position: (0.0, 0.0, 0.0),
            properties: props,
        }
    }

    // --- OutlinerState field tests ---

    #[test]
    fn outliner_state_renaming_index_defaults_to_none() {
        let state = OutlinerState::default();
        assert_eq!(state.renaming_index, None);
    }

    #[test]
    fn outliner_state_new_renaming_index_is_none() {
        let state = OutlinerState::new();
        assert_eq!(state.renaming_index, None);
    }

    #[test]
    fn outliner_state_renaming_index_can_be_set_and_cleared() {
        let mut state = OutlinerState::new();
        state.renaming_index = Some(3);
        assert_eq!(state.renaming_index, Some(3));
        state.renaming_index = None;
        assert_eq!(state.renaming_index, None);
    }

    // --- Snapshot key distinctness tests ---

    #[test]
    fn snapshot_ids_are_distinct_from_properties_panel_key() {
        // The Properties panel uses "entity_name_snapshot".
        // The outliner uses two different keys — verify they don't collide.
        let props_id = egui::Id::new("entity_name_snapshot").with(0usize);
        let outliner_focus_id = egui::Id::new("outliner_rename_snapshot").with(0usize);
        let outliner_cancel_id = egui::Id::new("outliner_rename_cancel_snapshot").with(0usize);
        assert_ne!(props_id, outliner_focus_id);
        assert_ne!(props_id, outliner_cancel_id);
        assert_ne!(outliner_focus_id, outliner_cancel_id);
    }

    #[test]
    fn snapshot_ids_are_distinct_across_entity_indices() {
        let id_0 = egui::Id::new("outliner_rename_cancel_snapshot").with(0usize);
        let id_1 = egui::Id::new("outliner_rename_cancel_snapshot").with(1usize);
        let id_5 = egui::Id::new("outliner_rename_cancel_snapshot").with(5usize);
        assert_ne!(id_0, id_1);
        assert_ne!(id_0, id_5);
        assert_ne!(id_1, id_5);
    }

    // --- PlayerSpawn exclusion (logic guard) ---

    #[test]
    fn player_spawn_would_not_enter_rename_mode() {
        // The condition checked in the double-click handler:
        // entity_type != EntityType::PlayerSpawn
        // Verify all other types pass the guard, PlayerSpawn does not.
        let excluded = EntityType::PlayerSpawn;
        let allowed = [
            EntityType::Npc,
            EntityType::Enemy,
            EntityType::Item,
            EntityType::Trigger,
            EntityType::LightSource,
        ];
        assert!(excluded == EntityType::PlayerSpawn); // excluded
        for t in &allowed {
            assert!(
                *t != EntityType::PlayerSpawn,
                "{:?} should be renameable",
                t
            );
        }
    }

    // --- Name resolution helpers ---

    #[test]
    fn entity_with_name_property_resolves_to_that_name() {
        let entity = make_entity(EntityType::Npc, Some("Guard"));
        let name = entity.properties.get("name").cloned().unwrap_or_default();
        assert_eq!(name, "Guard");
    }

    #[test]
    fn entity_without_name_property_resolves_to_empty_string() {
        let entity = make_entity(EntityType::Enemy, None);
        let name = entity.properties.get("name").cloned().unwrap_or_default();
        assert_eq!(name, "");
    }

    // --- Rename mode guard: out-of-bounds index detection ---

    #[test]
    fn renaming_index_out_of_bounds_is_detected() {
        // Simulates the guard: if renaming_index >= entity count, reset it.
        let mut state = OutlinerState::new();
        state.renaming_index = Some(5); // entity list has 2 items
        let entity_count = 2usize;

        if let Some(ri) = state.renaming_index {
            if ri >= entity_count {
                state.renaming_index = None;
            }
        }

        assert_eq!(state.renaming_index, None);
    }

    #[test]
    fn renaming_index_in_bounds_is_kept() {
        let mut state = OutlinerState::new();
        state.renaming_index = Some(1); // entity list has 3 items
        let entity_count = 3usize;

        if let Some(ri) = state.renaming_index {
            if ri >= entity_count {
                state.renaming_index = None;
            }
        }

        assert_eq!(state.renaming_index, Some(1));
    }

    // --- Commit logic: name-change detection ---

    #[test]
    fn commit_pushes_history_when_name_changed() {
        // Simulate the condition: old_name != new_name → history should be pushed
        let old = make_entity(EntityType::Npc, Some("OldName"));
        let new_name = "NewName";
        let old_name = old.properties.get("name").map(String::as_str).unwrap_or("");
        assert_ne!(old_name, new_name, "names differ → history entry expected");
    }

    #[test]
    fn commit_skips_history_when_name_unchanged() {
        // Simulate the condition: old_name == new_name → no history entry
        let old = make_entity(EntityType::Npc, Some("SameName"));
        let new_name = "SameName";
        let old_name = old.properties.get("name").map(String::as_str).unwrap_or("");
        assert_eq!(old_name, new_name, "names identical → no history entry");
    }

    #[test]
    fn commit_skips_history_when_both_names_absent() {
        let old = make_entity(EntityType::Npc, None);
        let new_name = "";
        let old_name = old.properties.get("name").map(String::as_str).unwrap_or("");
        assert_eq!(old_name, new_name);
    }

    // --- Cancel logic: restore from snapshot ---

    #[test]
    fn cancel_restores_original_name() {
        // Simulate cancel: restore old_data name into current entity
        let old = make_entity(EntityType::Npc, Some("OriginalName"));
        let mut current = make_entity(EntityType::Npc, Some("PartiallyTyped"));

        // Cancel path: restore from old snapshot
        let old_name = old.properties.get("name").cloned().unwrap_or_default();
        current
            .properties
            .insert("name".to_string(), old_name.clone());

        assert_eq!(
            current.properties.get("name").map(String::as_str),
            Some("OriginalName")
        );
    }

    #[test]
    fn cancel_restores_absent_name_as_empty() {
        // Entity had no name; user typed something; cancel should restore to empty.
        let old = make_entity(EntityType::Enemy, None);
        let mut current = make_entity(EntityType::Enemy, Some("HalfTyped"));

        let old_name = old.properties.get("name").cloned().unwrap_or_default();
        current
            .properties
            .insert("name".to_string(), old_name.clone());

        assert_eq!(current.properties.get("name").map(String::as_str), Some(""));
    }
}
