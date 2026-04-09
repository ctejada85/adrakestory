use super::*;

// ToolMemory tests
#[test]
fn test_tool_memory_default() {
    let memory = ToolMemory::default();
    assert_eq!(memory.voxel_type, VoxelType::Grass);
    assert_eq!(memory.voxel_pattern, SubVoxelPattern::Full);
    assert_eq!(memory.entity_type, EntityType::PlayerSpawn);
}

// EditorState tests
#[test]
fn test_editor_state_default() {
    let state = EditorState::default();
    assert!(state.current_map.world.voxels.is_empty());
    assert!(state.file_path.is_none());
    assert!(!state.is_modified);
    assert!(state.selected_voxels.is_empty());
    assert!(state.selected_entities.is_empty());
    assert!(state.show_grid);
    assert_eq!(state.grid_opacity, 0.3);
    assert!(state.snap_to_grid);
    assert!(state.show_entity_labels);
    assert!(state.outliner_scroll_to.is_none());
}

#[test]
fn test_editor_state_new() {
    let state = EditorState::new();
    assert!(!state.is_modified);
    assert!(state.file_path.is_none());
}

#[test]
fn test_editor_state_with_map() {
    let mut map = MapData::empty_map();
    map.world.width = 100;
    map.world.height = 50;
    map.world.depth = 75;

    let state = EditorState::with_map(map);
    assert_eq!(state.current_map.world.width, 100);
    assert_eq!(state.current_map.world.height, 50);
    assert_eq!(state.current_map.world.depth, 75);
    assert!(!state.is_modified);
}

#[test]
fn test_mark_modified() {
    let mut state = EditorState::new();
    assert!(!state.is_modified);

    state.mark_modified();
    assert!(state.is_modified);
}

#[test]
fn test_clear_modified() {
    let mut state = EditorState::new();
    state.mark_modified();
    assert!(state.is_modified);

    state.clear_modified();
    assert!(!state.is_modified);
}

#[test]
fn test_get_display_name_untitled() {
    let state = EditorState::new();
    assert_eq!(state.get_display_name(), "Untitled");
}

#[test]
fn test_get_display_name_with_path() {
    let mut state = EditorState::new();
    state.file_path = Some(PathBuf::from("/maps/test_map.ron"));
    assert_eq!(state.get_display_name(), "test_map.ron");
}

#[test]
fn test_get_window_title_untitled_unmodified() {
    let state = EditorState::new();
    assert_eq!(state.get_window_title(), "Untitled - Map Editor");
}

#[test]
fn test_get_window_title_untitled_modified() {
    let mut state = EditorState::new();
    state.mark_modified();
    assert_eq!(state.get_window_title(), "Untitled* - Map Editor");
}

#[test]
fn test_get_window_title_with_file_modified() {
    let mut state = EditorState::new();
    state.file_path = Some(PathBuf::from("my_map.ron"));
    state.mark_modified();
    assert_eq!(state.get_window_title(), "my_map.ron* - Map Editor");
}

#[test]
fn test_clear_selections() {
    let mut state = EditorState::new();
    state.selected_voxels.insert((0, 0, 0));
    state.selected_voxels.insert((1, 2, 3));
    state.selected_entities.insert(0);
    state.selected_entities.insert(5);

    assert_eq!(state.selected_voxels.len(), 2);
    assert_eq!(state.selected_entities.len(), 2);

    state.clear_selections();

    assert!(state.selected_voxels.is_empty());
    assert!(state.selected_entities.is_empty());
}

// EditorTool tests
#[test]
fn test_editor_tool_name() {
    assert_eq!(
        EditorTool::VoxelPlace {
            voxel_type: VoxelType::Grass,
            pattern: SubVoxelPattern::Full
        }
        .name(),
        "Voxel Place"
    );
    assert_eq!(EditorTool::VoxelRemove.name(), "Voxel Remove");
    assert_eq!(
        EditorTool::EntityPlace {
            entity_type: EntityType::PlayerSpawn
        }
        .name(),
        "Entity Place"
    );
    assert_eq!(EditorTool::Select.name(), "Select");
    assert_eq!(EditorTool::Camera.name(), "Camera");
}

#[test]
fn test_editor_tool_description() {
    assert_eq!(
        EditorTool::VoxelPlace {
            voxel_type: VoxelType::Grass,
            pattern: SubVoxelPattern::Full
        }
        .description(),
        "Click to place voxels"
    );
    assert_eq!(
        EditorTool::VoxelRemove.description(),
        "Click to remove voxels"
    );
    assert_eq!(
        EditorTool::EntityPlace {
            entity_type: EntityType::PlayerSpawn
        }
        .description(),
        "Click to place entities"
    );
    assert_eq!(EditorTool::Select.description(), "Click to select objects");
    assert_eq!(EditorTool::Camera.description(), "Drag to move camera");
}

#[test]
fn test_editor_tool_equality() {
    let tool1 = EditorTool::VoxelPlace {
        voxel_type: VoxelType::Grass,
        pattern: SubVoxelPattern::Full,
    };
    let tool2 = EditorTool::VoxelPlace {
        voxel_type: VoxelType::Grass,
        pattern: SubVoxelPattern::Full,
    };
    let tool3 = EditorTool::VoxelPlace {
        voxel_type: VoxelType::Stone,
        pattern: SubVoxelPattern::Full,
    };

    assert_eq!(tool1, tool2);
    assert_ne!(tool1, tool3);
}

// EditorUIState tests
#[test]
fn test_editor_ui_state_default() {
    let ui_state = EditorUIState::default();
    assert!(!ui_state.file_dialog_open);
    assert!(!ui_state.new_map_dialog_open);
    assert!(!ui_state.unsaved_changes_dialog_open);
    assert!(ui_state.pending_action.is_none());
    assert!(!ui_state.about_dialog_open);
    assert!(!ui_state.shortcuts_help_open);
    assert!(!ui_state.error_dialog_open);
    assert!(ui_state.error_message.is_empty());
}

// KeyboardEditMode tests
#[test]
fn test_keyboard_edit_mode_default() {
    let mode = KeyboardEditMode::default();
    assert!(!mode.enabled);
}

#[test]
fn test_keyboard_edit_mode_new() {
    let mode = KeyboardEditMode::new();
    assert!(!mode.enabled);
}

#[test]
fn test_keyboard_edit_mode_enable() {
    let mut mode = KeyboardEditMode::new();
    mode.enable();
    assert!(mode.enabled);
}

#[test]
fn test_keyboard_edit_mode_disable() {
    let mut mode = KeyboardEditMode::new();
    mode.enable();
    assert!(mode.enabled);
    mode.disable();
    assert!(!mode.enabled);
}

#[test]
fn test_keyboard_edit_mode_toggle() {
    let mut mode = KeyboardEditMode::new();
    assert!(!mode.enabled);
    mode.toggle();
    assert!(mode.enabled);
    mode.toggle();
    assert!(!mode.enabled);
}
