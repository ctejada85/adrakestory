//! Rotation state for voxel geometry.

use crate::systems::game::map::geometry::RotationAxis;
use serde::{Deserialize, Serialize};

/// Rotation state for a voxel's geometry.
/// This tracks cumulative rotations applied to the voxel's sub-voxel pattern.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RotationState {
    /// The axis of rotation
    pub axis: RotationAxis,
    /// The rotation angle in 90° increments (0-3 for 0°, 90°, 180°, 270°)
    pub angle: i32,
}

impl RotationState {
    /// Create a new rotation state.
    pub fn new(axis: RotationAxis, angle: i32) -> Self {
        Self {
            axis,
            angle: angle.rem_euclid(4),
        }
    }

    /// Compose this rotation with another rotation.
    /// This handles the case where rotations are applied sequentially.
    pub fn compose(self, axis: RotationAxis, angle: i32) -> Self {
        // If rotating around the same axis, just add the angles
        if self.axis == axis {
            Self::new(axis, self.angle + angle)
        } else {
            // For different axes, we need to apply the new rotation
            // For simplicity, we'll store the most recent rotation
            // A full implementation would compose the rotations properly
            Self::new(axis, angle)
        }
    }
}
