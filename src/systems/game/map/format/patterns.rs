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
    fn test_center_cube_pattern_has_8_positions() {
        let geometry = SubVoxelPattern::CenterCube.geometry();
        let positions: Vec<_> = geometry.occupied_positions().collect();
        // 2x2x2 = 8 sub-voxels
        assert_eq!(positions.len(), 8);
    }

    #[test]
    fn test_pillar_column_has_32_positions() {
        let geometry = SubVoxelPattern::Pillar.geometry();
        let positions: Vec<_> = geometry.occupied_positions().collect();
        // 2x8x2 = 32 sub-voxels
        assert_eq!(positions.len(), 32);
    }

    #[test]
    fn test_pillar_column_spans_full_height() {
        let geometry = SubVoxelPattern::Pillar.geometry();
        let positions: Vec<_> = geometry.occupied_positions().collect();
        // Must contain sub-voxels at both y=0 and y=7 (full height, no gap when stacking)
        let has_y0 = positions.iter().any(|(_, y, _)| *y == 0);
        let has_y7 = positions.iter().any(|(_, y, _)| *y == 7);
        assert!(has_y0, "Pillar must reach y=0");
        assert!(has_y7, "Pillar must reach y=7");
        // All occupied positions must be in x∈{3,4}, z∈{3,4}
        for (x, _, z) in &positions {
            assert!(*x == 3 || *x == 4, "x must be 3 or 4, got {x}");
            assert!(*z == 3 || *z == 4, "z must be 3 or 4, got {z}");
        }
    }

    #[test]
    fn test_center_cube_is_centered() {
        let geometry = SubVoxelPattern::CenterCube.geometry();
        let positions: Vec<_> = geometry.occupied_positions().collect();
        // All positions must be in x∈{3,4}, y∈{3,4}, z∈{3,4}
        for (x, y, z) in &positions {
            assert!(*x == 3 || *x == 4);
            assert!(*y == 3 || *y == 4);
            assert!(*z == 3 || *z == 4);
        }
    }

    #[test]
    fn test_ron_deserialization_pillar_alias() {
        // "Pillar" was the old name — it now deserialises as the new Pillar (column) via alias
        let ron_str = "Pillar";
        let pattern: SubVoxelPattern = ron::from_str(ron_str).unwrap();
        assert_eq!(pattern, SubVoxelPattern::Pillar);
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
        let staircase = SubVoxelPattern::Staircase.geometry();
        let staircase_neg_x = SubVoxelPattern::StaircaseNegX.geometry();
        let staircase_z = SubVoxelPattern::StaircaseZ.geometry();
        let staircase_neg_z = SubVoxelPattern::StaircaseNegZ.geometry();

        let count_x: usize = staircase.occupied_positions().count();
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
        assert!(!SubVoxelPattern::CenterCube.is_fence());
        assert!(!SubVoxelPattern::Staircase.is_fence());
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
        // "Staircase" was the old alias name — now it IS the canonical name
        let ron_str = "Staircase";
        let pattern: SubVoxelPattern = ron::from_str(ron_str).unwrap();
        assert_eq!(pattern, SubVoxelPattern::Staircase);
    }

    #[test]
    fn test_ron_deserialization_staircase_x_alias() {
        // "StaircaseX" is now the backward-compat alias for the renamed canonical variant
        let ron_str = "StaircaseX";
        let pattern: SubVoxelPattern = ron::from_str(ron_str).unwrap();
        assert_eq!(pattern, SubVoxelPattern::Staircase);
    }

    #[test]
    fn test_ron_serialization_roundtrip() {
        let patterns = [
            SubVoxelPattern::Full,
            SubVoxelPattern::Pillar,
            SubVoxelPattern::CenterCube,
            SubVoxelPattern::PlatformXZ,
            SubVoxelPattern::Staircase,
            SubVoxelPattern::Fence,
        ];

        for pattern in patterns {
            let serialized = ron::to_string(&pattern).unwrap();
            let deserialized: SubVoxelPattern = ron::from_str(&serialized).unwrap();
            assert_eq!(pattern, deserialized);
        }
    }

    #[test]
    fn test_staircase_serializes_as_staircase_not_staircase_x() {
        // After rename, Staircase must serialise as "Staircase", not "StaircaseX".
        let serialized = ron::to_string(&SubVoxelPattern::Staircase).unwrap();
        assert_eq!(serialized, "Staircase");
    }
}
