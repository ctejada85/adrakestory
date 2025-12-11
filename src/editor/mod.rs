//! Map editor module for A Drake's Story.
//!
//! This module provides a standalone GUI application for creating and editing
//! map files in RON format. It uses bevy_egui for the UI and reuses the game's
//! rendering code for 3D preview.

pub mod camera;
pub mod cursor;
pub mod file_io;
pub mod grid;
pub mod history;
pub mod play;
pub mod recent_files;
pub mod renderer;
pub mod state;
pub mod tools;
pub mod ui;

pub use cursor::{
    handle_keyboard_cursor_movement, handle_keyboard_selection, handle_play_shortcuts,
    handle_tool_switching, toggle_keyboard_edit_mode, CursorState,
};
pub use file_io::{FileSavedEvent, SaveFileDialogReceiver, SaveMapAsEvent, SaveMapEvent};
pub use history::{EditorAction, EditorHistory};
pub use play::{PlayMapEvent, PlayTestState, StopGameEvent};
pub use recent_files::{OpenRecentFileEvent, RecentFiles};
pub use renderer::{
    render_entities_system, EditorChunk, EditorEntityMarker, MapRenderState, RenderMapEvent,
};
pub use state::{EditorState, EditorTool, KeyboardEditMode, ToolMemory};
