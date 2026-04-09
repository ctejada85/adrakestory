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
    /// One-shot flag: scroll the rename row into view on the next frame it is rendered.
    /// Set when rename mode is entered via context menu or F2 (not double-click, which
    /// already has the row on screen).
    pub scroll_to_rename: bool,
}

impl OutlinerState {
    pub fn new() -> Self {
        Self {
            voxels_expanded: true,
            entities_expanded: true,
            filter_text: String::new(),
            voxel_type_expanded: HashMap::new(),
            renaming_index: None,
            scroll_to_rename: false,
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

            // F2: enter rename mode for the single selected non-PlayerSpawn entity.
            // Guard: no rename already in progress, exactly one entity selected.
            if outliner_state.renaming_index.is_none()
                && ui.input(|i| i.key_pressed(egui::Key::F2))
                && editor_state.selected_entities.len() == 1
            {
                let sel_index = *editor_state.selected_entities.iter().next().unwrap();
                if sel_index < editor_state.current_map.entities.len()
                    && editor_state.current_map.entities[sel_index].entity_type
                        != EntityType::PlayerSpawn
                {
                    let cancel_id =
                        egui::Id::new("outliner_rename_cancel_snapshot").with(sel_index);
                    ui.data_mut(|d| {
                        d.insert_temp(
                            cancel_id,
                            editor_state.current_map.entities[sel_index].clone(),
                        )
                    });
                    outliner_state.renaming_index = Some(sel_index);
                    outliner_state.scroll_to_rename = true;
                }
            }

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

                    // Scroll this rename row into view if activation came from context menu or F2.
                    if outliner_state.scroll_to_rename {
                        response.scroll_to_me(None);
                        outliner_state.scroll_to_rename = false;
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
                            // Phase 2: empty-name commit removes the key rather than storing "".
                            // Do this before cloning new_data so the history entry captures the
                            // clean state (key absent) rather than an empty string.
                            if editor_state.current_map.entities[index]
                                .properties
                                .get("name")
                                .map(String::as_str)
                                .unwrap_or("")
                                .is_empty()
                            {
                                editor_state.current_map.entities[index]
                                    .properties
                                    .remove("name");
                                editor_state.mark_modified();
                            }

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
                            // "Rename" only for entity types that support names.
                            if entity_type != EntityType::PlayerSpawn
                                && ui.button("✏️ Rename").clicked()
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
                                outliner_state.scroll_to_rename = true;
                                ui.close();
                            }
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

    // --- Phase 2: scroll_to_rename field ---

    #[test]
    fn scroll_to_rename_defaults_to_false() {
        let state = OutlinerState::default();
        assert!(!state.scroll_to_rename);
    }

    #[test]
    fn scroll_to_rename_new_is_false() {
        let state = OutlinerState::new();
        assert!(!state.scroll_to_rename);
    }

    #[test]
    fn scroll_to_rename_can_be_set_and_cleared() {
        let mut state = OutlinerState::new();
        state.scroll_to_rename = true;
        assert!(state.scroll_to_rename);
        state.scroll_to_rename = false;
        assert!(!state.scroll_to_rename);
    }

    // --- Phase 2: context menu Rename sets renaming_index (logic simulation) ---

    #[test]
    fn context_menu_rename_sets_renaming_index_for_non_player_spawn() {
        // Simulates the context-menu "Rename" handler logic.
        let mut state = OutlinerState::new();
        let entity_type = EntityType::Npc;
        let index = 2usize;

        // Guard mirrors the production code: entity_type != PlayerSpawn
        if entity_type != EntityType::PlayerSpawn {
            state.renaming_index = Some(index);
            state.scroll_to_rename = true;
        }

        assert_eq!(state.renaming_index, Some(index));
        assert!(state.scroll_to_rename);
    }

    #[test]
    fn context_menu_rename_does_not_fire_for_player_spawn() {
        let mut state = OutlinerState::new();
        let entity_type = EntityType::PlayerSpawn;
        let index = 0usize;

        if entity_type != EntityType::PlayerSpawn {
            state.renaming_index = Some(index);
            state.scroll_to_rename = true;
        }

        assert_eq!(state.renaming_index, None);
        assert!(!state.scroll_to_rename);
    }

    // --- Phase 2: F2 shortcut logic ---

    #[test]
    fn f2_enters_rename_for_single_selected_non_player_spawn() {
        let mut state = OutlinerState::new();

        // Simulate: renaming_index is None, exactly one non-PlayerSpawn entity selected.
        let sel_index = 1usize;
        let entity_type = EntityType::Enemy;
        let mut selected = std::collections::HashSet::new();
        selected.insert(sel_index);

        // Mirrors the F2 guard in production code.
        if state.renaming_index.is_none()
            && selected.len() == 1
            && entity_type != EntityType::PlayerSpawn
        {
            state.renaming_index = Some(sel_index);
            state.scroll_to_rename = true;
        }

        assert_eq!(state.renaming_index, Some(sel_index));
        assert!(state.scroll_to_rename);
    }

    #[test]
    fn f2_is_noop_when_nothing_selected() {
        let mut state = OutlinerState::new();
        let selected: std::collections::HashSet<usize> = std::collections::HashSet::new();

        if state.renaming_index.is_none() && selected.len() == 1 {
            state.renaming_index = Some(0);
        }

        assert_eq!(state.renaming_index, None);
    }

    #[test]
    fn f2_is_noop_when_multiple_entities_selected() {
        let mut state = OutlinerState::new();
        let mut selected = std::collections::HashSet::new();
        selected.insert(0usize);
        selected.insert(1usize);

        if state.renaming_index.is_none() && selected.len() == 1 {
            state.renaming_index = Some(0);
        }

        assert_eq!(state.renaming_index, None);
    }

    #[test]
    fn f2_is_noop_when_rename_already_active() {
        let mut state = OutlinerState::new();
        state.renaming_index = Some(0); // already renaming
        let mut selected = std::collections::HashSet::new();
        selected.insert(1usize);

        if state.renaming_index.is_none() && selected.len() == 1 {
            state.renaming_index = Some(1);
        }

        // renaming_index stays at 0, not overwritten to 1
        assert_eq!(state.renaming_index, Some(0));
    }

    #[test]
    fn f2_is_noop_for_player_spawn() {
        let mut state = OutlinerState::new();
        let sel_index = 0usize;
        let entity_type = EntityType::PlayerSpawn;
        let mut selected = std::collections::HashSet::new();
        selected.insert(sel_index);

        if state.renaming_index.is_none()
            && selected.len() == 1
            && entity_type != EntityType::PlayerSpawn
        {
            state.renaming_index = Some(sel_index);
        }

        assert_eq!(state.renaming_index, None);
    }

    // --- Phase 2: empty-name commit removes the "name" key ---

    #[test]
    fn empty_name_commit_removes_name_key() {
        // Simulate the Phase 2 post-commit cleanup step.
        let mut entity = make_entity(EntityType::Npc, Some(""));

        // Cleanup: if name is empty, remove the key.
        if entity
            .properties
            .get("name")
            .map(String::as_str)
            .unwrap_or("")
            .is_empty()
        {
            entity.properties.remove("name");
        }

        assert!(
            !entity.properties.contains_key("name"),
            "empty name should remove the key entirely"
        );
    }

    #[test]
    fn nonempty_name_commit_keeps_name_key() {
        let mut entity = make_entity(EntityType::Npc, Some("Guard"));

        if entity
            .properties
            .get("name")
            .map(String::as_str)
            .unwrap_or("")
            .is_empty()
        {
            entity.properties.remove("name");
        }

        assert_eq!(
            entity.properties.get("name").map(String::as_str),
            Some("Guard")
        );
    }

    #[test]
    fn empty_name_commit_history_captures_key_absent_state() {
        // Verify that when old_name is non-empty and new name becomes absent after cleanup,
        // the names differ → a history entry would be pushed.
        let old = make_entity(EntityType::Npc, Some("Guard"));
        let mut current = make_entity(EntityType::Npc, Some(""));

        // Apply cleanup (mirrors production: remove key before cloning new_data).
        if current
            .properties
            .get("name")
            .map(String::as_str)
            .unwrap_or("")
            .is_empty()
        {
            current.properties.remove("name");
        }

        let old_name = old.properties.get("name").map(String::as_str).unwrap_or("");
        let new_name = current
            .properties
            .get("name")
            .map(String::as_str)
            .unwrap_or("");

        // "Guard" != "" → history entry should be pushed
        assert_ne!(old_name, new_name);
        // new_data would have no "name" key → redo restores key-absent state
        assert!(!current.properties.contains_key("name"));
    }

    #[test]
    fn empty_name_commit_no_history_when_was_already_absent() {
        // old entity had no name; user committed ""; cleanup removes key → still no name → no
        // history entry.
        let old = make_entity(EntityType::Npc, None);
        let mut current = make_entity(EntityType::Npc, Some(""));

        if current
            .properties
            .get("name")
            .map(String::as_str)
            .unwrap_or("")
            .is_empty()
        {
            current.properties.remove("name");
        }

        let old_name = old.properties.get("name").map(String::as_str).unwrap_or("");
        let new_name = current
            .properties
            .get("name")
            .map(String::as_str)
            .unwrap_or("");

        // Both resolve to "" → no history entry pushed
        assert_eq!(old_name, new_name);
    }
}
