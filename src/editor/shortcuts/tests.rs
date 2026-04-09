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
            rotation: None,
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
        rotation: None,
        rotation_state: None,
    });

    let action = EditorAction::RemoveVoxel {
        pos: (1, 2, 3),
        data: VoxelData {
            pos: (1, 2, 3),
            voxel_type: VoxelType::Grass,
            pattern: Some(SubVoxelPattern::Full),
            rotation: None,
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
            rotation: None,
            rotation_state: None,
        },
    };
    apply_action(&place_action, &mut state);
    assert_eq!(state.current_map.world.voxels.len(), 1);

    // Undo by applying inverse
    apply_action_inverse(&place_action, &mut state);
    assert!(state.current_map.world.voxels.is_empty());
}
