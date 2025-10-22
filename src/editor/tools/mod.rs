//! Editor tools for map manipulation.

pub mod entity_tool;
pub mod selection_tool;
pub mod voxel_tool;

pub use entity_tool::handle_entity_placement;
pub use selection_tool::{
    cancel_transform, confirm_rotation, confirm_transform, handle_arrow_key_movement,
    handle_arrow_key_rotation, handle_delete_selected, handle_deselect_shortcut,
    handle_move_shortcut, handle_rotate_shortcut, handle_rotation_axis_selection,
    handle_selection, render_rotation_preview, render_selection_highlights,
    render_transform_preview, start_move_operation, start_rotate_operation, update_rotation,
    update_rotation_axis, update_transform_preview, ActiveTransform, CancelTransform,
    ConfirmTransform, DeleteSelectedVoxels, SetRotationAxis, StartMoveOperation,
    StartRotateOperation, TransformMode, UpdateRotation, UpdateSelectionHighlights,
    UpdateTransformPreview,
};
pub use voxel_tool::{handle_voxel_placement, handle_voxel_removal};

// Re-export RotationAxis from geometry module for convenience
pub use crate::systems::game::map::geometry::RotationAxis;
