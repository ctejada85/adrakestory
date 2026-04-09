use super::*;
use crate::systems::game::components::VoxelType;
use crate::systems::game::map::format::SubVoxelPattern;

#[test]
fn test_history_push_and_undo() {
    let mut history = EditorHistory::new();

    let action = EditorAction::PlaceVoxel {
        pos: (0, 0, 0),
        data: VoxelData {
            pos: (0, 0, 0),
            voxel_type: VoxelType::Grass,
            pattern: Some(SubVoxelPattern::Full),
            rotation: None,
            rotation_state: None,
        },
    };

    history.push(action.clone());
    assert!(history.can_undo());
    assert!(!history.can_redo());

    let undone = history.undo();
    assert!(undone.is_some());
    assert!(!history.can_undo());
    assert!(history.can_redo());
}

#[test]
fn test_history_redo() {
    let mut history = EditorHistory::new();

    let action = EditorAction::PlaceVoxel {
        pos: (0, 0, 0),
        data: VoxelData {
            pos: (0, 0, 0),
            voxel_type: VoxelType::Grass,
            pattern: Some(SubVoxelPattern::Full),
            rotation: None,
            rotation_state: None,
        },
    };

    history.push(action.clone());
    history.undo();

    let redone = history.redo();
    assert!(redone.is_some());
    assert!(history.can_undo());
    assert!(!history.can_redo());
}

#[test]
fn test_history_clear_redo_on_new_action() {
    let mut history = EditorHistory::new();

    let action1 = EditorAction::PlaceVoxel {
        pos: (0, 0, 0),
        data: VoxelData {
            pos: (0, 0, 0),
            voxel_type: VoxelType::Grass,
            pattern: Some(SubVoxelPattern::Full),
            rotation: None,
            rotation_state: None,
        },
    };

    let action2 = EditorAction::PlaceVoxel {
        pos: (1, 0, 0),
        data: VoxelData {
            pos: (1, 0, 0),
            voxel_type: VoxelType::Dirt,
            pattern: Some(SubVoxelPattern::Full),
            rotation: None,
            rotation_state: None,
        },
    };

    history.push(action1);
    history.undo();
    assert!(history.can_redo());

    history.push(action2);
    assert!(!history.can_redo());
}
