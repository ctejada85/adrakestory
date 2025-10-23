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
pub mod renderer;
pub mod state;
pub mod tools;
pub mod ui;

pub use cursor::handle_keyboard_cursor_movement;
pub use file_io::{FileSavedEvent, SaveFileDialogReceiver, SaveMapAsEvent, SaveMapEvent};
pub use history::{EditorAction, EditorHistory};
pub use renderer::{MapRenderState, RenderMapEvent};
pub use state::{EditorState, EditorTool};
