//! Map Editor for A Drake's Story
//!
//! A standalone GUI application for creating and editing map files.

mod file_handlers;
mod lighting;
mod setup;
mod status_bar;
mod ui_system;

use adrakestory::editor::play::{
    handle_play_map, handle_stop_game, poll_game_process, PlayMapEvent, PlayTestState,
    StopGameEvent,
};
use adrakestory::editor::recent_files::{OpenRecentFileEvent, RecentFiles};
use adrakestory::editor::shortcuts::{handle_global_shortcuts, handle_redo, handle_undo};
use adrakestory::editor::tools::ActiveTransform;
use adrakestory::editor::tools::DragSelectState;
use adrakestory::editor::tools::VoxelDragState;
use adrakestory::editor::tools::VoxelRemoveDragState;
use adrakestory::editor::ui::dialogs::AppExitEvent;
use adrakestory::editor::{camera, cursor, file_io, grid, renderer, state, tools, ui};
use adrakestory::editor::{
    handle_keyboard_cursor_movement, handle_keyboard_selection, handle_play_shortcuts,
    handle_tool_switching, toggle_keyboard_edit_mode,
};
use adrakestory::editor::{
    CursorState, EditorHistory, EditorState, KeyboardEditMode, MapRenderState, RedoEvent,
    RenderMapEvent, UndoEvent,
};
use adrakestory::editor::{FileSavedEvent, SaveFileDialogReceiver, SaveMapAsEvent, SaveMapEvent};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use grid::InfiniteGridConfig;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Map Editor - A Drake's Story".to_string(),
                resolution: (1600.0, 900.0).into(),
                // Prevent window from closing immediately - we handle it manually
                prevent_default_event_handling: false,
                ..default()
            }),
            close_when_requested: false, // Don't auto-close, we'll handle it
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .init_resource::<EditorState>()
        .init_resource::<CursorState>()
        .init_resource::<EditorHistory>()
        .init_resource::<state::EditorUIState>()
        .init_resource::<state::ToolMemory>()
        .init_resource::<camera::CameraInputState>()
        .init_resource::<camera::GamepadCameraState>()
        .init_resource::<ui::dialogs::FileDialogReceiver>()
        .init_resource::<SaveFileDialogReceiver>()
        .init_resource::<MapRenderState>()
        .init_resource::<InfiniteGridConfig>()
        .init_resource::<ActiveTransform>()
        .init_resource::<KeyboardEditMode>()
        .init_resource::<DragSelectState>()
        .init_resource::<VoxelDragState>()
        .init_resource::<VoxelRemoveDragState>()
        .init_resource::<ui::OutlinerState>()
        .init_resource::<PlayTestState>()
        .insert_resource(RecentFiles::load()) // Load recent files from disk
        .add_event::<ui::dialogs::FileSelectedEvent>()
        .add_event::<SaveMapEvent>()
        .add_event::<SaveMapAsEvent>()
        .add_event::<FileSavedEvent>()
        .add_event::<RenderMapEvent>()
        .add_event::<ui::dialogs::MapDataChangedEvent>()
        .add_event::<OpenRecentFileEvent>()
        .add_event::<PlayMapEvent>()
        .add_event::<StopGameEvent>()
        .add_event::<UndoEvent>()
        .add_event::<RedoEvent>()
        .add_event::<tools::UpdateSelectionHighlights>()
        // New unified input event
        .add_event::<tools::EditorInputEvent>()
        // Keep these events for UI button compatibility
        .add_event::<tools::DeleteSelectedVoxels>()
        .add_event::<tools::StartMoveOperation>()
        .add_event::<tools::StartRotateOperation>()
        .add_event::<tools::ConfirmTransform>()
        .add_event::<tools::CancelTransform>()
        .add_event::<tools::UpdateTransformPreview>()
        .add_event::<tools::UpdateRotation>()
        .add_event::<tools::SetRotationAxis>()
        .add_event::<AppExitEvent>()
        .add_systems(Startup, setup::setup_editor)
        .add_systems(Update, lighting::update_lighting_on_map_change)
        .add_systems(Update, ui_system::render_ui)
        .add_systems(Update, ui::dialogs::check_file_dialog_result)
        .add_systems(Update, ui::dialogs::handle_file_selected)
        .add_systems(Update, ui::dialogs::handle_window_close_request)
        .add_systems(Update, ui::dialogs::handle_app_exit)
        .add_systems(Update, file_io::handle_save_map)
        .add_systems(Update, file_io::handle_save_map_as)
        .add_systems(Update, file_io::check_save_dialog_result)
        .add_systems(Update, file_io::handle_file_saved)
        // Play/test systems
        .add_systems(Update, handle_play_map)
        .add_systems(Update, handle_stop_game)
        .add_systems(Update, poll_game_process)
        .add_systems(
            Update,
            adrakestory::editor::recent_files::update_recent_on_save,
        )
        .add_systems(Update, file_handlers::handle_open_recent_file)
        // Global keyboard shortcuts (Ctrl+S, Ctrl+Z, etc.) - must run after render_ui
        .add_systems(
            Update,
            handle_global_shortcuts.after(ui_system::render_ui),
        )
        .add_systems(Update, handle_undo.after(handle_global_shortcuts))
        .add_systems(Update, handle_redo.after(handle_global_shortcuts))
        // Keyboard handling systems - must run after render_ui for correct egui state
        .add_systems(
            Update,
            (
                toggle_keyboard_edit_mode,
                handle_tool_switching,
                handle_play_shortcuts,
                cursor::update_cursor_position,
                handle_keyboard_cursor_movement.after(cursor::update_cursor_position),
                handle_keyboard_selection,
            )
                .after(ui_system::render_ui),
        )
        .add_systems(Update, renderer::detect_map_changes)
        .add_systems(Update, renderer::render_map_system)
        .add_systems(Update, renderer::render_entities_system)
        .add_systems(Update, camera::handle_camera_input.after(ui_system::render_ui))
        .add_systems(Update, camera::update_editor_camera)
        .add_systems(Update, camera::handle_gamepad_voxel_actions.after(camera::handle_camera_input))
        .add_systems(Update, camera::handle_gamepad_tool_cycling.after(camera::handle_camera_input))
        .add_systems(Update, grid::update_infinite_grid)
        .add_systems(Update, grid::update_grid_visibility)
        .add_systems(Update, grid::update_cursor_indicator)
        .add_systems(Update, tools::handle_voxel_placement.after(ui_system::render_ui))
        .add_systems(
            Update,
            tools::handle_voxel_drag_placement.after(tools::handle_voxel_placement),
        )
        .add_systems(Update, tools::handle_voxel_removal.after(ui_system::render_ui))
        .add_systems(
            Update,
            tools::handle_voxel_drag_removal.after(tools::handle_voxel_removal),
        )
        .add_systems(Update, tools::handle_entity_placement.after(ui_system::render_ui))
        .add_systems(Update, tools::handle_selection.after(ui_system::render_ui))
        .add_systems(
            Update,
            tools::handle_drag_selection.after(tools::handle_selection),
        )
        // Unified input handling systems - must run in order:
        // 1. handle_keyboard_input reads keyboard and sends EditorInputEvent
        // 2. handle_transformation_operations processes those events
        .add_systems(Update, tools::handle_keyboard_input)
        .add_systems(
            Update,
            tools::handle_transformation_operations.after(tools::handle_keyboard_input),
        )
        // Keep rendering systems
        .add_systems(Update, tools::render_selection_highlights)
        .add_systems(Update, tools::render_transform_preview)
        .add_systems(Update, tools::render_rotation_preview)
        .run();
}
