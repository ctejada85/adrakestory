//! Loading screen system for displaying map loading progress.

mod components;
mod systems;

pub use systems::{cleanup_loading_screen, setup_loading_screen, update_loading_progress};
