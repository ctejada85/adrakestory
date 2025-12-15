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
