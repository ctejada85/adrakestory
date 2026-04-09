//! Sub-voxel patterns for voxel geometry.

use super::rotation::{apply_orientation_matrix, OrientationMatrix};
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
    /// Canonical staircase: stairs ascending in the +X direction.
    ///
    /// This is the single canonical staircase variant. Facing direction is controlled
    /// via the voxel's `rotation` field. Deserialises from the old `"StaircaseX"` name
    /// via a `#[serde(alias)]` for backward compatibility.
    #[serde(alias = "StaircaseX")] // Old name before rename — kept for backward compat
    Staircase,
    /// Backward-compat alias for `Staircase` with an implicit Y+180° pre-bake.
    ///
    /// **Load-time alias only.** This variant is never written on save.
    /// The map loader's `normalise_staircase_variants()` pass converts it to
    /// `Staircase` with the Y+180° rotation absorbed into the voxel's orientation matrix.
    StaircaseNegX,
    /// Backward-compat alias for `Staircase` with an implicit Y+90° pre-bake.
    ///
    /// **Load-time alias only.** This variant is never written on save.
    /// The map loader's `normalise_staircase_variants()` pass converts it to
    /// `Staircase` with the Y+90° rotation absorbed into the voxel's orientation matrix.
    StaircaseZ,
    /// Backward-compat alias for `Staircase` with an implicit Y+270° pre-bake.
    ///
    /// **Load-time alias only.** This variant is never written on save.
    /// The map loader's `normalise_staircase_variants()` pass converts it to
    /// `Staircase` with the Y+270° rotation absorbed into the voxel's orientation matrix.
    StaircaseNegZ,

    /// Full-height 2×8×2 column (pillar).
    ///
    /// Spans the full voxel height — 32 sub-voxels at x∈{3,4}, z∈{3,4}.
    /// Stacking vertically produces zero gap between adjacent cells.
    #[serde(alias = "Pillar")] // Old name before rename — kept for backward compat
    Pillar,

    /// Small 2×2×2 centred cube (symmetric, no orientation).
    ///
    /// Occupies sub-voxels (3,3,3)–(4,4,4): 8 sub-voxels centred in the voxel cell.
    /// This carries the geometry that was previously (and incorrectly) named `Pillar`.
    CenterCube,

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
            Self::Staircase => SubVoxelGeometry::staircase_x(),
            Self::StaircaseNegX => {
                // Staircase rotated 180° around Y (pre-bake preserved for legacy geometry calls)
                SubVoxelGeometry::staircase_x().rotate(crate::editor::tools::RotationAxis::Y, 2)
            }
            Self::StaircaseZ => {
                // Staircase rotated 90° around Y (pre-bake preserved for legacy geometry calls)
                SubVoxelGeometry::staircase_x().rotate(crate::editor::tools::RotationAxis::Y, 1)
            }
            Self::StaircaseNegZ => {
                // Staircase rotated 270° around Y (pre-bake preserved for legacy geometry calls)
                SubVoxelGeometry::staircase_x().rotate(crate::editor::tools::RotationAxis::Y, 3)
            }
            Self::Pillar => SubVoxelGeometry::column_2x2(),
            Self::CenterCube => SubVoxelGeometry::center_cube(),
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

    /// Get the geometry representation of this pattern with an orientation matrix applied.
    ///
    /// # Arguments
    /// * `orientation` - Optional reference to a 3×3 orientation matrix from `MapData::orientations`.
    ///   `None` means identity (no rotation).
    ///
    /// # Returns
    /// The geometry with the orientation applied, or base geometry for `None`.
    pub fn geometry_with_rotation(
        &self,
        orientation: Option<&OrientationMatrix>,
    ) -> SubVoxelGeometry {
        let base_geometry = self.geometry();

        if let Some(matrix) = orientation {
            apply_orientation_matrix(base_geometry, matrix)
        } else {
            base_geometry
        }
    }
}

#[cfg(test)]
mod tests;
