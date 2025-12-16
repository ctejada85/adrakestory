//! Controller support for map editor.
//!
//! This module provides Xbox controller support for the map editor,
//! enabling a Minecraft Creative Mode-style editing experience.
//!
//! Features:
//! - First-person camera flying through the map
//! - Hotbar system for quick item switching
//! - Direct voxel placement/removal with triggers
//! - Full-screen item palette for selecting blocks and entities

pub mod camera;
pub mod cursor;
pub mod hotbar;
pub mod input;
pub mod palette;

pub use camera::{
    update_controller_camera, ControllerCamera, ControllerCameraMode, ControllerModeToggleEvent,
};
pub use cursor::{update_controller_cursor, ControllerCursor};
pub use hotbar::{HotbarItem, PaletteCategory};
pub use input::{
    handle_controller_editing, handle_controller_hotbar, handle_controller_palette,
    ControllerEditMode,
};
pub use palette::render_controller_palette;
