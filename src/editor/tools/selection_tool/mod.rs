//! Selection tool for selecting and manipulating objects.
//!
//! This module provides:
//! - Voxel and entity selection via click/drag
//! - Selection highlighting
//! - Transform preview rendering for move/rotate operations

mod highlights;
mod preview;
mod selection;

pub use highlights::render_selection_highlights;
pub use preview::{render_rotation_preview, render_transform_preview, rotate_position};
pub use selection::{handle_drag_selection, handle_selection};

use crate::editor::camera::EditorCamera;
use crate::systems::game::map::format::VoxelData;
use crate::systems::game::map::geometry::RotationAxis;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// Bundle of queries needed for viewport raycasting
#[derive(bevy::ecs::system::SystemParam)]
pub struct ViewportRaycast<'w, 's> {
    pub camera: Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<EditorCamera>>,
    pub window: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
}

/// Marker component for selection highlight visuals
#[derive(Component)]
pub struct SelectionHighlight {
    pub voxel_pos: (i32, i32, i32),
}

/// Marker component for transform preview visuals
#[derive(Component)]
pub struct TransformPreview {
    pub original_pos: (i32, i32, i32),
    pub preview_pos: (i32, i32, i32),
    pub is_valid: bool, // false if collision detected
}

/// Resource tracking active transformation
#[derive(Resource)]
pub struct ActiveTransform {
    pub mode: TransformMode,
    pub selected_voxels: Vec<VoxelData>,
    pub pivot: Vec3,
    pub current_offset: IVec3,
    pub rotation_axis: RotationAxis,
    pub rotation_angle: i32, // In 90-degree increments (0, 1, 2, 3)
}

impl Default for ActiveTransform {
    fn default() -> Self {
        Self {
            mode: TransformMode::None,
            selected_voxels: Vec::new(),
            pivot: Vec3::ZERO,
            current_offset: IVec3::ZERO,
            rotation_axis: RotationAxis::Y,
            rotation_angle: 0,
        }
    }
}

/// Transform operation mode
#[derive(Debug, Clone, PartialEq, Default)]
pub enum TransformMode {
    #[default]
    None,
    Move,
    Rotate,
}

/// Resource tracking drag-to-select state
#[derive(Resource, Default)]
pub struct DragSelectState {
    /// Whether we're currently in a drag-select operation
    pub is_dragging: bool,
    /// Last grid position we added to selection (to avoid duplicates)
    pub last_grid_pos: Option<(i32, i32, i32)>,
    /// The initial grid position when click started (for toggle detection)
    pub start_grid_pos: Option<(i32, i32, i32)>,
    /// Whether the cursor moved to a different voxel during drag
    pub did_drag_move: bool,
    /// Whether the starting voxel was already selected (for toggle on release)
    pub start_was_selected: bool,
}

/// Event to trigger selection highlight update
#[derive(Event)]
pub struct UpdateSelectionHighlights;

/// Event to trigger deletion of selected voxels
#[derive(Event)]
pub struct DeleteSelectedVoxels;

/// Event to start move operation
#[derive(Event)]
pub struct StartMoveOperation;

/// Event to start rotate operation
#[derive(Event)]
pub struct StartRotateOperation;

/// Event to set rotation axis
#[derive(Event)]
pub struct SetRotationAxis {
    pub axis: RotationAxis,
}

/// Event to confirm transformation
#[derive(Event)]
pub struct ConfirmTransform;

/// Event to cancel transformation
#[derive(Event)]
pub struct CancelTransform;

/// Event to update transform preview
#[derive(Event)]
pub struct UpdateTransformPreview {
    pub offset: IVec3,
}

/// Event to update rotation
#[derive(Event)]
pub struct UpdateRotation {
    pub delta: i32, // +1 or -1 for 90-degree rotations
}
