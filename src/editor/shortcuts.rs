//! Global keyboard shortcuts for the map editor.
//!
//! This module provides keyboard shortcut handling for common editor operations
//! such as Save (Ctrl+S), Open (Ctrl+O), New (Ctrl+N), and Undo/Redo (Ctrl+Z/Y).

use crate::editor::file_io::{SaveMapAsEvent, SaveMapEvent};
use crate::editor::history::{EditorAction, EditorHistory};
use crate::editor::renderer::RenderMapEvent;
use crate::editor::state::{EditorState, EditorUIState, PendingAction};
use crate::editor::ui::dialogs::MapDataChangedEvent;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::EguiContexts;

/// Event to request an undo operation
#[derive(Event)]
pub struct UndoEvent;

/// Event to request a redo operation
#[derive(Event)]
pub struct RedoEvent;

/// Bundle of event writers for shortcut-triggered actions
#[derive(SystemParam)]
pub struct ShortcutEvents<'w> {
    pub save: EventWriter<'w, SaveMapEvent>,
    pub save_as: EventWriter<'w, SaveMapAsEvent>,
    pub undo: EventWriter<'w, UndoEvent>,
    pub redo: EventWriter<'w, RedoEvent>,
}

/// System to handle global keyboard shortcuts for the editor
///
/// Handles:
/// - Ctrl+S: Save
/// - Ctrl+Shift+S: Save As
/// - Ctrl+O: Open
/// - Ctrl+N: New
/// - Ctrl+Z: Undo
/// - Ctrl+Y / Ctrl+Shift+Z: Redo
pub fn handle_global_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    editor_state: Res<EditorState>,
    mut ui_state: ResMut<EditorUIState>,
    mut events: ShortcutEvents,
) {
    // Don't handle shortcuts if egui wants keyboard input (text fields, etc.)
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    let ctrl_pressed =
        keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    let shift_pressed =
        keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    if !ctrl_pressed {
        return;
    }

    // Ctrl+S: Save / Ctrl+Shift+S: Save As
    if keyboard.just_pressed(KeyCode::KeyS) {
        if shift_pressed {
            // Ctrl+Shift+S: Save As
            events.save_as.send(SaveMapAsEvent);
            info!("Save As triggered via Ctrl+Shift+S");
        } else {
            // Ctrl+S: Save
            events.save.send(SaveMapEvent);
            info!("Save triggered via Ctrl+S");
        }
    }

    // Ctrl+O: Open
    if keyboard.just_pressed(KeyCode::KeyO) && !shift_pressed {
        if editor_state.is_modified {
            ui_state.unsaved_changes_dialog_open = true;
            ui_state.pending_action = Some(PendingAction::OpenMap);
        } else {
            ui_state.file_dialog_open = true;
        }
        info!("Open triggered via Ctrl+O");
    }

    // Ctrl+N: New
    if keyboard.just_pressed(KeyCode::KeyN) && !shift_pressed {
        if editor_state.is_modified {
            ui_state.unsaved_changes_dialog_open = true;
            ui_state.pending_action = Some(PendingAction::NewMap);
        } else {
            ui_state.new_map_dialog_open = true;
        }
        info!("New triggered via Ctrl+N");
    }

    // Ctrl+Z: Undo / Ctrl+Shift+Z: Redo
    if keyboard.just_pressed(KeyCode::KeyZ) {
        if shift_pressed {
            // Ctrl+Shift+Z: Redo
            events.redo.send(RedoEvent);
            info!("Redo triggered via Ctrl+Shift+Z");
        } else {
            // Ctrl+Z: Undo
            events.undo.send(UndoEvent);
            info!("Undo triggered via Ctrl+Z");
        }
    }

    // Ctrl+Y: Redo (alternative)
    if keyboard.just_pressed(KeyCode::KeyY) && !shift_pressed {
        events.redo.send(RedoEvent);
        info!("Redo triggered via Ctrl+Y");
    }
}

/// System to handle undo events and apply undo operations
pub fn handle_undo(
    mut undo_events: EventReader<UndoEvent>,
    mut editor_state: ResMut<EditorState>,
    mut history: ResMut<EditorHistory>,
    mut render_events: EventWriter<RenderMapEvent>,
    mut map_changed_events: EventWriter<MapDataChangedEvent>,
) {
    for _event in undo_events.read() {
        if let Some(action) = history.undo() {
            // Apply the inverse of the action
            apply_action_inverse(&action, &mut editor_state);
            editor_state.mark_modified();

            // Trigger re-render
            render_events.send(RenderMapEvent);
            map_changed_events.send(MapDataChangedEvent);

            info!("Undo: {}", action.description());
        } else {
            info!("Nothing to undo");
        }
    }
}

/// System to handle redo events and apply redo operations
pub fn handle_redo(
    mut redo_events: EventReader<RedoEvent>,
    mut editor_state: ResMut<EditorState>,
    mut history: ResMut<EditorHistory>,
    mut render_events: EventWriter<RenderMapEvent>,
    mut map_changed_events: EventWriter<MapDataChangedEvent>,
) {
    for _event in redo_events.read() {
        if let Some(action) = history.redo() {
            // Apply the action directly (redo means re-do the original action)
            apply_action(&action, &mut editor_state);
            editor_state.mark_modified();

            // Trigger re-render
            render_events.send(RenderMapEvent);
            map_changed_events.send(MapDataChangedEvent);

            info!("Redo: {}", action.description());
        } else {
            info!("Nothing to redo");
        }
    }
}

/// Apply an editor action to the editor state
fn apply_action(action: &EditorAction, editor_state: &mut EditorState) {
    match action {
        EditorAction::PlaceVoxel { pos, data } => {
            // Add or update voxel
            if let Some(existing) = editor_state
                .current_map
                .world
                .voxels
                .iter_mut()
                .find(|v| v.pos == *pos)
            {
                *existing = data.clone();
            } else {
                editor_state.current_map.world.voxels.push(data.clone());
            }
        }
        EditorAction::RemoveVoxel { pos, .. } => {
            // Remove voxel at position
            editor_state
                .current_map
                .world
                .voxels
                .retain(|v| v.pos != *pos);
        }
        EditorAction::PlaceEntity { index, data } => {
            // Insert entity at index
            if *index <= editor_state.current_map.entities.len() {
                editor_state
                    .current_map
                    .entities
                    .insert(*index, data.clone());
            } else {
                editor_state.current_map.entities.push(data.clone());
            }
        }
        EditorAction::RemoveEntity { index, .. } => {
            // Remove entity at index
            if *index < editor_state.current_map.entities.len() {
                editor_state.current_map.entities.remove(*index);
            }
        }
        EditorAction::ModifyEntity {
            index, new_data, ..
        } => {
            // Update entity data
            if let Some(entity) = editor_state.current_map.entities.get_mut(*index) {
                *entity = new_data.clone();
            }
        }
        EditorAction::ModifyMetadata { new, .. } => {
            editor_state.current_map.metadata = new.clone();
        }
        EditorAction::Batch { actions, .. } => {
            // Apply all actions in order
            for sub_action in actions {
                apply_action(sub_action, editor_state);
            }
        }
    }
}

/// Apply the inverse of an editor action (for undo)
fn apply_action_inverse(action: &EditorAction, editor_state: &mut EditorState) {
    // Use the action's inverse method to get the reversed action, then apply it
    let inverse = action.inverse();
    apply_action(&inverse, editor_state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::game::components::VoxelType;
    use crate::systems::game::map::format::{MapData, SubVoxelPattern, VoxelData};

    fn create_test_editor_state() -> EditorState {
        EditorState {
            current_map: MapData::empty_map(),
            ..Default::default()
        }
    }

    #[test]
    fn test_apply_place_voxel() {
        let mut state = create_test_editor_state();
        let action = EditorAction::PlaceVoxel {
            pos: (1, 2, 3),
            data: VoxelData {
                pos: (1, 2, 3),
                voxel_type: VoxelType::Grass,
                pattern: Some(SubVoxelPattern::Full),
                rotation_state: None,
            },
        };

        apply_action(&action, &mut state);
        assert_eq!(state.current_map.world.voxels.len(), 1);
        assert_eq!(state.current_map.world.voxels[0].pos, (1, 2, 3));
    }

    #[test]
    fn test_apply_remove_voxel() {
        let mut state = create_test_editor_state();
        state.current_map.world.voxels.push(VoxelData {
            pos: (1, 2, 3),
            voxel_type: VoxelType::Grass,
            pattern: Some(SubVoxelPattern::Full),
            rotation_state: None,
        });

        let action = EditorAction::RemoveVoxel {
            pos: (1, 2, 3),
            data: VoxelData {
                pos: (1, 2, 3),
                voxel_type: VoxelType::Grass,
                pattern: Some(SubVoxelPattern::Full),
                rotation_state: None,
            },
        };

        apply_action(&action, &mut state);
        assert!(state.current_map.world.voxels.is_empty());
    }

    #[test]
    fn test_undo_via_inverse() {
        let mut state = create_test_editor_state();

        // Place a voxel
        let place_action = EditorAction::PlaceVoxel {
            pos: (1, 2, 3),
            data: VoxelData {
                pos: (1, 2, 3),
                voxel_type: VoxelType::Grass,
                pattern: Some(SubVoxelPattern::Full),
                rotation_state: None,
            },
        };
        apply_action(&place_action, &mut state);
        assert_eq!(state.current_map.world.voxels.len(), 1);

        // Undo by applying inverse
        apply_action_inverse(&place_action, &mut state);
        assert!(state.current_map.world.voxels.is_empty());
    }
}
