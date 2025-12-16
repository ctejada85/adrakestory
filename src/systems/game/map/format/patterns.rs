//! Sub-voxel patterns for voxel geometry.

use super::rotation::RotationState;
use crate::systems::game::map::geometry::SubVoxelGeometry;
use serde::{Deserialize, Serialize};

/// Sub-voxel patterns for different voxel appearances.
/// Patterns with orientation variants support proper rotation transformations.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SubVoxelPattern {
    /// Full 8x8x8 cube of sub-voxels (symmetric, no orientation)
    #[default]
    Full,

    // Platform variants (thin slabs in different orientations)
    /// Thin 8x1x8 platform on XZ plane (horizontal, default)
    #[serde(alias = "Platform")] // For backward compatibility
    PlatformXZ,
    /// Thin 8x8x1 platform on XY plane (vertical wall facing Z)
    PlatformXY,
    /// Thin 1x8x8 platform on YZ plane (vertical wall facing X)
    PlatformYZ,

    // Staircase variants (progressive height in different directions)
    /// Stairs ascending in +X direction (default)
    #[serde(alias = "Staircase")] // For backward compatibility
    StaircaseX,
    /// Stairs ascending in -X direction
    StaircaseNegX,
    /// Stairs ascending in +Z direction
    StaircaseZ,
    /// Stairs ascending in -Z direction
    StaircaseNegZ,

    /// Small 2x2x2 centered column (symmetric, no orientation)
    Pillar,

    /// Fence pattern along X axis (posts at ends with horizontal rails)
    #[serde(alias = "FenceX")]
    #[serde(alias = "FenceZ")]
    #[serde(alias = "FenceCorner")]
    Fence,
}

impl SubVoxelPattern {
    /// Get the base geometry representation of this pattern without any rotation.
    ///
    /// This converts the pattern enum into actual 3D sub-voxel positions.
    pub fn geometry(&self) -> SubVoxelGeometry {
        match self {
            Self::Full => SubVoxelGeometry::full(),
            Self::PlatformXZ => SubVoxelGeometry::platform_horizontal(),
            Self::PlatformXY => {
                // Horizontal platform rotated 90° around X
                SubVoxelGeometry::platform_horizontal()
                    .rotate(crate::editor::tools::RotationAxis::X, 1)
            }
            Self::PlatformYZ => {
                // Horizontal platform rotated 90° around Z
                SubVoxelGeometry::platform_horizontal()
                    .rotate(crate::editor::tools::RotationAxis::Z, 1)
            }
            Self::StaircaseX => SubVoxelGeometry::staircase_x(),
            Self::StaircaseNegX => {
                // Staircase rotated 180° around Y
                SubVoxelGeometry::staircase_x().rotate(crate::editor::tools::RotationAxis::Y, 2)
            }
            Self::StaircaseZ => {
                // Staircase rotated 90° around Y
                SubVoxelGeometry::staircase_x().rotate(crate::editor::tools::RotationAxis::Y, 1)
            }
            Self::StaircaseNegZ => {
                // Staircase rotated 270° around Y
                SubVoxelGeometry::staircase_x().rotate(crate::editor::tools::RotationAxis::Y, 3)
            }
            Self::Pillar => SubVoxelGeometry::pillar(),
            Self::Fence => SubVoxelGeometry::fence_post(), // Default to just a post
        }
    }

    /// Check if this pattern is a fence that needs neighbor-aware geometry.
    pub fn is_fence(&self) -> bool {
        matches!(self, Self::Fence)
    }

    /// Get geometry for fence patterns based on neighboring fences.
    ///
    /// # Arguments
    /// * `neighbors` - Tuple of (has_neg_x, has_pos_x, has_neg_z, has_pos_z) indicating adjacent fences
    pub fn fence_geometry_with_neighbors(
        &self,
        neighbors: (bool, bool, bool, bool),
    ) -> SubVoxelGeometry {
        if !self.is_fence() {
            return self.geometry();
        }
        SubVoxelGeometry::fence_with_connections(neighbors.0, neighbors.1, neighbors.2, neighbors.3)
    }

    /// Get the geometry representation of this pattern with rotation applied.
    ///
    /// This applies the rotation state to the base pattern geometry.
    ///
    /// # Arguments
    /// * `rotation_state` - Optional rotation state to apply
    ///
    /// # Returns
    /// The geometry with rotation applied, or base geometry if no rotation state.
    pub fn geometry_with_rotation(
        &self,
        rotation_state: Option<RotationState>,
    ) -> SubVoxelGeometry {
        let base_geometry = self.geometry();

        if let Some(rotation) = rotation_state {
            base_geometry.rotate(rotation.axis, rotation.angle)
        } else {
            base_geometry
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_default_is_full() {
        assert_eq!(SubVoxelPattern::default(), SubVoxelPattern::Full);
    }

    #[test]
    fn test_full_pattern_geometry_has_512_positions() {
        let geometry = SubVoxelPattern::Full.geometry();
        let positions: Vec<_> = geometry.occupied_positions().collect();
        // 8x8x8 = 512 sub-voxels
        assert_eq!(positions.len(), 512);
    }

    #[test]
    fn test_pillar_pattern_has_8_positions() {
        let geometry = SubVoxelPattern::Pillar.geometry();
        let positions: Vec<_> = geometry.occupied_positions().collect();
        // 2x2x2 = 8 sub-voxels
        assert_eq!(positions.len(), 8);
    }

    #[test]
    fn test_platform_xz_is_thin_slab() {
        let geometry = SubVoxelPattern::PlatformXZ.geometry();
        let positions: Vec<_> = geometry.occupied_positions().collect();
        // 8x1x8 = 64 sub-voxels
        assert_eq!(positions.len(), 64);
        // All positions should have the same Y (layer 0)
        for (_, y, _) in &positions {
            assert_eq!(*y, 0);
        }
    }

    #[test]
    fn test_staircase_patterns_have_same_count() {
        let staircase_x = SubVoxelPattern::StaircaseX.geometry();
        let staircase_neg_x = SubVoxelPattern::StaircaseNegX.geometry();
        let staircase_z = SubVoxelPattern::StaircaseZ.geometry();
        let staircase_neg_z = SubVoxelPattern::StaircaseNegZ.geometry();

        let count_x: usize = staircase_x.occupied_positions().count();
        let count_neg_x: usize = staircase_neg_x.occupied_positions().count();
        let count_z: usize = staircase_z.occupied_positions().count();
        let count_neg_z: usize = staircase_neg_z.occupied_positions().count();

        // All rotations should have the same number of sub-voxels
        assert_eq!(count_x, count_neg_x);
        assert_eq!(count_x, count_z);
        assert_eq!(count_x, count_neg_z);
    }

    #[test]
    fn test_is_fence_returns_true_for_fence() {
        assert!(SubVoxelPattern::Fence.is_fence());
    }

    #[test]
    fn test_is_fence_returns_false_for_non_fence() {
        assert!(!SubVoxelPattern::Full.is_fence());
        assert!(!SubVoxelPattern::Pillar.is_fence());
        assert!(!SubVoxelPattern::StaircaseX.is_fence());
        assert!(!SubVoxelPattern::PlatformXZ.is_fence());
    }

    #[test]
    fn test_geometry_with_rotation_none_returns_base() {
        let base = SubVoxelPattern::Full.geometry();
        let with_none = SubVoxelPattern::Full.geometry_with_rotation(None);

        let base_positions: Vec<_> = base.occupied_positions().collect();
        let none_positions: Vec<_> = with_none.occupied_positions().collect();

        assert_eq!(base_positions, none_positions);
    }

    #[test]
    fn test_ron_deserialization_full() {
        let ron_str = "Full";
        let pattern: SubVoxelPattern = ron::from_str(ron_str).unwrap();
        assert_eq!(pattern, SubVoxelPattern::Full);
    }

    #[test]
    fn test_ron_deserialization_platform_alias() {
        // Test backward compatibility alias
        let ron_str = "Platform";
        let pattern: SubVoxelPattern = ron::from_str(ron_str).unwrap();
        assert_eq!(pattern, SubVoxelPattern::PlatformXZ);
    }

    #[test]
    fn test_ron_deserialization_staircase_alias() {
        // Test backward compatibility alias
        let ron_str = "Staircase";
        let pattern: SubVoxelPattern = ron::from_str(ron_str).unwrap();
        assert_eq!(pattern, SubVoxelPattern::StaircaseX);
    }

    #[test]
    fn test_ron_serialization_roundtrip() {
        let patterns = [
            SubVoxelPattern::Full,
            SubVoxelPattern::Pillar,
            SubVoxelPattern::PlatformXZ,
            SubVoxelPattern::StaircaseX,
            SubVoxelPattern::Fence,
        ];

        for pattern in patterns {
            let serialized = ron::to_string(&pattern).unwrap();
            let deserialized: SubVoxelPattern = ron::from_str(&serialized).unwrap();
            assert_eq!(pattern, deserialized);
        }
    }
}
