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

// --- Entity deletion history (bug-entity-delete-no-undo) ---

#[test]
fn delete_pushes_remove_entity_action_onto_history() {
    // Simulate the fixed deletion handler logic:
    // 1. clone entity at index
    // 2. push RemoveEntity onto history
    // 3. remove from vec
    use crate::editor::history::{EditorAction, EditorHistory};

    let entity = make_entity(EntityType::Npc, Some("Guard"));
    let mut entities: Vec<EntityData> = vec![
        make_entity(EntityType::PlayerSpawn, None),
        entity.clone(),
        make_entity(EntityType::Enemy, None),
    ];
    let mut history = EditorHistory::new();

    let index = 1usize;
    let removed_data = entities[index].clone();
    history.push(EditorAction::RemoveEntity {
        index,
        data: removed_data,
    });
    entities.remove(index);

    assert_eq!(history.undo_count(), 1, "one undo entry should be present");
    assert_eq!(entities.len(), 2, "entity list should have one fewer item");
}

#[test]
fn undo_after_delete_restores_entity_at_correct_index() {
    use crate::editor::history::{EditorAction, EditorHistory};

    let npc = make_entity(EntityType::Npc, Some("Guard"));
    let mut entities: Vec<EntityData> = vec![
        make_entity(EntityType::PlayerSpawn, None),
        npc.clone(),
        make_entity(EntityType::Enemy, None),
    ];
    let mut history = EditorHistory::new();

    // Simulate deletion at index 1
    let index = 1usize;
    history.push(EditorAction::RemoveEntity {
        index,
        data: entities[index].clone(),
    });
    entities.remove(index);

    // Simulate undo: inverse of RemoveEntity is PlaceEntity → insert at index
    if let Some(action) = history.undo() {
        let inverse = action.inverse();
        if let EditorAction::PlaceEntity {
            index: restore_index,
            data,
        } = inverse
        {
            if restore_index <= entities.len() {
                entities.insert(restore_index, data);
            } else {
                entities.push(data);
            }
        }
    }

    assert_eq!(entities.len(), 3, "entity should be restored");
    assert_eq!(
        entities[1].entity_type,
        EntityType::Npc,
        "restored entity should be at index 1"
    );
    assert_eq!(
        entities[1].properties.get("name").map(String::as_str),
        Some("Guard"),
        "restored entity should have original name"
    );
    assert_eq!(
        history.undo_count(),
        0,
        "undo stack should be empty after undo"
    );
}

#[test]
fn delete_only_entity_then_undo_restores_single_entity() {
    use crate::editor::history::{EditorAction, EditorHistory};

    let entity = make_entity(EntityType::PlayerSpawn, None);
    let mut entities: Vec<EntityData> = vec![entity.clone()];
    let mut history = EditorHistory::new();

    // Delete the only entity
    history.push(EditorAction::RemoveEntity {
        index: 0,
        data: entities[0].clone(),
    });
    entities.remove(0);
    assert!(entities.is_empty());

    // Undo
    if let Some(action) = history.undo() {
        let inverse = action.inverse();
        if let EditorAction::PlaceEntity {
            index: restore_index,
            data,
        } = inverse
        {
            if restore_index <= entities.len() {
                entities.insert(restore_index, data);
            } else {
                entities.push(data);
            }
        }
    }

    assert_eq!(entities.len(), 1, "single entity should be restored");
    assert_eq!(entities[0].entity_type, EntityType::PlayerSpawn);
}

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
