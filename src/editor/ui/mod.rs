//! UI panels and components for the map editor.

pub mod dialogs;
pub mod properties;
pub mod toolbar;
pub mod viewport;

// Note: dialogs functions are used directly from the module, not re-exported
pub use properties::render_properties_panel;
pub use toolbar::render_toolbar;
pub use viewport::render_viewport_controls;
