//! Editor tools for map manipulation.

pub mod entity_tool;
pub mod selection_tool;
pub mod voxel_tool;

pub use entity_tool::handle_entity_placement;
pub use selection_tool::{
    handle_delete_selected, handle_selection, render_selection_highlights,
    DeleteSelectedVoxels, UpdateSelectionHighlights,
};
pub use voxel_tool::{handle_voxel_placement, handle_voxel_removal};
