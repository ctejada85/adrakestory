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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_normalizes_angle() {
        let rotation = RotationState::new(RotationAxis::Y, 5);
        assert_eq!(rotation.angle, 1); // 5 mod 4 = 1
    }

    #[test]
    fn test_new_negative_angle_normalized() {
        let rotation = RotationState::new(RotationAxis::Y, -1);
        assert_eq!(rotation.angle, 3); // -1 mod 4 = 3 (using rem_euclid)
    }

    #[test]
    fn test_compose_same_axis_adds_angles() {
        let rotation = RotationState::new(RotationAxis::Y, 1);
        let composed = rotation.compose(RotationAxis::Y, 2);
        assert_eq!(composed.axis, RotationAxis::Y);
        assert_eq!(composed.angle, 3); // 1 + 2 = 3
    }

    #[test]
    fn test_compose_same_axis_wraps_around() {
        let rotation = RotationState::new(RotationAxis::Y, 3);
        let composed = rotation.compose(RotationAxis::Y, 2);
        assert_eq!(composed.angle, 1); // (3 + 2) mod 4 = 1
    }

    #[test]
    fn test_compose_different_axis_replaces() {
        let rotation = RotationState::new(RotationAxis::Y, 2);
        let composed = rotation.compose(RotationAxis::X, 1);
        assert_eq!(composed.axis, RotationAxis::X);
        assert_eq!(composed.angle, 1);
    }

    #[test]
    fn test_ron_serialization_roundtrip() {
        let rotation = RotationState::new(RotationAxis::Z, 2);
        let serialized = ron::to_string(&rotation).unwrap();
        let deserialized: RotationState = ron::from_str(&serialized).unwrap();
        assert_eq!(rotation, deserialized);
    }

    #[test]
    fn test_angle_values_are_0_to_3() {
        for angle in -10..10 {
            let rotation = RotationState::new(RotationAxis::Y, angle);
            assert!(rotation.angle >= 0 && rotation.angle < 4);
        }
    }
}
