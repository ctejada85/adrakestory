//! Editor tools for map manipulation.

pub mod entity_tool;
pub mod input;
pub mod selection_tool;
pub mod voxel_tool;

pub use entity_tool::handle_entity_placement;

// New unified input handling
pub use input::{
    handle_keyboard_input,
    handle_transformation_operations,
    EditorInputEvent,
};

// Keep selection tool exports for rendering and mouse selection
pub use selection_tool::{
    handle_selection,
    render_rotation_preview,
    render_selection_highlights,
    render_transform_preview,
    ActiveTransform,
    TransformMode,
    UpdateSelectionHighlights,
    // Keep these events for UI button compatibility
    CancelTransform,
    ConfirmTransform,
    DeleteSelectedVoxels,
    SetRotationAxis,
    StartMoveOperation,
    StartRotateOperation,
    UpdateRotation,
    UpdateTransformPreview,
};

pub use voxel_tool::{handle_voxel_placement, handle_voxel_removal};

// Re-export RotationAxis from geometry module for convenience
pub use crate::systems::game::map::geometry::RotationAxis;
