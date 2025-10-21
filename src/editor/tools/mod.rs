//! Editor tools for map manipulation.

pub mod entity_tool;
pub mod selection_tool;
pub mod voxel_tool;

pub use entity_tool::handle_entity_placement;
pub use selection_tool::{
    cancel_transform, confirm_transform, handle_arrow_key_movement, handle_delete_selected,
    handle_move_shortcut, handle_selection, render_selection_highlights,
    render_transform_preview, start_move_operation, update_transform_preview, ActiveTransform,
    CancelTransform, ConfirmTransform, DeleteSelectedVoxels, StartMoveOperation,
    TransformMode, UpdateSelectionHighlights, UpdateTransformPreview,
};
pub use voxel_tool::{handle_voxel_placement, handle_voxel_removal};
