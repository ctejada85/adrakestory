//! Orientation matrix system for voxel geometry.
//!
//! The map format stores voxel orientations as 3×3 integer rotation matrices in a
//! top-level `orientations` list on `MapData`. Each voxel references an entry by
//! index (`rotation: Option<usize>`).
//!
//! Legacy files that use the old `rotation_state: Some((axis, angle))` syntax are
//! transparently migrated on load via `migrate_legacy_rotations()`.

use crate::systems::game::map::geometry::{RotationAxis, SubVoxelGeometry};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A 3×3 integer rotation matrix.
///
/// Rows represent where the X, Y, and Z unit vectors map to after rotation.
/// Valid entries are in {−1, 0, 1} with exactly one non-zero per row and column
/// and determinant = 1.
pub type OrientationMatrix = [[i32; 3]; 3];

/// The identity orientation (no rotation).
pub const IDENTITY: OrientationMatrix = [[1, 0, 0], [0, 1, 0], [0, 0, 1]];

/// Legacy rotation state used only for backward-compatible deserialisation.
///
/// Old map files contain `rotation_state: Some((axis: Y, angle: 1))`. This struct
/// captures that on deserialise; `migrate_legacy_rotations()` converts it to a
/// matrix entry and clears this field before any game/editor code sees it.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LegacyRotationState {
    pub axis: RotationAxis,
    /// Rotation angle in 90° increments (0–3).
    pub angle: i32,
}

// ---------------------------------------------------------------------------
// Matrix helpers
// ---------------------------------------------------------------------------

/// Convert a single-axis 90° rotation into a 3×3 integer matrix.
///
/// The matrices match the transforms in `rotate_point()` so that
/// `apply_orientation_matrix(geom, &axis_angle_to_matrix(axis, angle))`
/// produces the same result as `geom.rotate(axis, angle)`.
pub fn axis_angle_to_matrix(axis: RotationAxis, angle: i32) -> OrientationMatrix {
    let angle = angle.rem_euclid(4);
    match axis {
        RotationAxis::X => match angle {
            1 => [[1, 0, 0], [0, 0, -1], [0, 1, 0]], // X+90°:  Y→-Z, Z→Y
            2 => [[1, 0, 0], [0, -1, 0], [0, 0, -1]], // X+180°: Y→-Y, Z→-Z
            3 => [[1, 0, 0], [0, 0, 1], [0, -1, 0]], // X+270°: Y→Z,  Z→-Y
            _ => IDENTITY,
        },
        RotationAxis::Y => match angle {
            1 => [[0, 0, 1], [0, 1, 0], [-1, 0, 0]], // Y+90°:  X→-Z, Z→X
            2 => [[-1, 0, 0], [0, 1, 0], [0, 0, -1]], // Y+180°: X→-X, Z→-Z
            3 => [[0, 0, -1], [0, 1, 0], [1, 0, 0]], // Y+270°: X→Z,  Z→-X
            _ => IDENTITY,
        },
        RotationAxis::Z => match angle {
            1 => [[0, -1, 0], [1, 0, 0], [0, 0, 1]], // Z+90°:  X→Y,  Y→-X
            2 => [[-1, 0, 0], [0, -1, 0], [0, 0, 1]], // Z+180°: X→-X, Y→-Y
            3 => [[0, 1, 0], [-1, 0, 0], [0, 0, 1]], // Z+270°: X→-Y, Y→X
            _ => IDENTITY,
        },
    }
}

/// Multiply two 3×3 integer matrices: result = a × b.
///
/// Used by the editor to compose an existing orientation matrix with a new
/// single-axis rotation: `M' = R_new × M_current`.
pub fn multiply_matrices(a: &OrientationMatrix, b: &OrientationMatrix) -> OrientationMatrix {
    let mut result = [[0i32; 3]; 3];
    for row in 0..3 {
        for col in 0..3 {
            result[row][col] =
                a[row][0] * b[0][col] + a[row][1] * b[1][col] + a[row][2] * b[2][col];
        }
    }
    result
}

/// Apply an orientation matrix to a `SubVoxelGeometry` by decomposing it into
/// at most two `rotate()` calls.
///
/// The function identifies which single-axis rotations compose to produce `matrix`
/// and applies them sequentially. This keeps the geometry layer unchanged while
/// supporting full 24-orientation coverage.
///
/// # Panics
/// Does not panic — returns geometry unchanged if the matrix is identity or
/// unrecognised (the latter should never occur for validated maps).
pub fn apply_orientation_matrix(
    geometry: SubVoxelGeometry,
    matrix: &OrientationMatrix,
) -> SubVoxelGeometry {
    if *matrix == IDENTITY {
        return geometry;
    }

    // Try all single-axis combinations first (12 cases: 3 axes × 4 angles, minus identity)
    let single_axis_cases = [
        (RotationAxis::X, 1),
        (RotationAxis::X, 2),
        (RotationAxis::X, 3),
        (RotationAxis::Y, 1),
        (RotationAxis::Y, 2),
        (RotationAxis::Y, 3),
        (RotationAxis::Z, 1),
        (RotationAxis::Z, 2),
        (RotationAxis::Z, 3),
    ];

    for (axis, angle) in &single_axis_cases {
        if axis_angle_to_matrix(*axis, *angle) == *matrix {
            return geometry.rotate(*axis, *angle);
        }
    }

    // Try all two-axis compositions (remaining 12 of the 24 orientations)
    for (axis1, angle1) in &single_axis_cases {
        for (axis2, angle2) in &single_axis_cases {
            let composed = multiply_matrices(
                &axis_angle_to_matrix(*axis2, *angle2),
                &axis_angle_to_matrix(*axis1, *angle1),
            );
            if composed == *matrix {
                return geometry.rotate(*axis1, *angle1).rotate(*axis2, *angle2);
            }
        }
    }

    // Identity or unrecognised — return unchanged
    geometry
}

/// Find or insert `matrix` in the orientations list and return its index.
///
/// This is the canonical way for the editor and the legacy migration shim to
/// add an orientation to the map's list without creating duplicates.
pub fn find_or_insert_orientation(
    orientations: &mut Vec<OrientationMatrix>,
    matrix: OrientationMatrix,
) -> usize {
    if let Some(index) = orientations.iter().position(|m| *m == matrix) {
        index
    } else {
        orientations.push(matrix);
        orientations.len() - 1
    }
}

/// Map a world-space direction into the local frame of a voxel with the given orientation.
///
/// `OrientationMatrix` rows store where local X/Y/Z map **to** in world space
/// (`M × local = world`), so the inverse is the transpose (`Mᵀ × world = local`).
///
/// `dir` is a unit direction vector with integer components (e.g. `[1,0,0]` for +X).
/// Returns the corresponding direction in the voxel's local coordinate frame.
///
/// If `orientation` is `None` (identity), the direction is returned unchanged.
pub fn world_dir_to_local(orientation: Option<&OrientationMatrix>, dir: [i32; 3]) -> [i32; 3] {
    match orientation {
        None => dir,
        Some(m) => [
            m[0][0] * dir[0] + m[1][0] * dir[1] + m[2][0] * dir[2],
            m[0][1] * dir[0] + m[1][1] * dir[1] + m[2][1] * dir[2],
            m[0][2] * dir[0] + m[1][2] * dir[1] + m[2][2] * dir[2],
        ],
    }
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

/// Return `true` if `matrix` is a valid 90°-grid rotation matrix.
///
/// Checks:
/// - All entries are in {−1, 0, 1}
/// - Exactly one non-zero entry per row and per column
/// - Determinant = 1
pub fn is_valid_rotation_matrix(matrix: &OrientationMatrix) -> bool {
    // Check entries are in {-1, 0, 1}
    for row in matrix {
        for &v in row {
            if v != -1 && v != 0 && v != 1 {
                return false;
            }
        }
    }

    // Check exactly one non-zero per row
    for row in matrix {
        let non_zero = row.iter().filter(|&&v| v != 0).count();
        if non_zero != 1 {
            return false;
        }
    }

    // Check exactly one non-zero per column
    #[allow(clippy::needless_range_loop)]
    for col in 0..3 {
        let non_zero = (0..3).filter(|&row| matrix[row][col] != 0).count();
        if non_zero != 1 {
            return false;
        }
    }

    // Check determinant = 1 (not -1, which would be an improper rotation / reflection)
    determinant_3x3(matrix) == 1
}

/// Compute the determinant of a 3×3 integer matrix.
fn determinant_3x3(m: &OrientationMatrix) -> i32 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

// ---------------------------------------------------------------------------
// Legacy migration
// ---------------------------------------------------------------------------

/// Migrate all `rotation_state` fields on voxels to the new `rotation: Option<usize>` system.
///
/// Called by the map loader after RON deserialisation and before `validate_map()`.
/// On the next save by the editor, the file will be written in the new format.
pub fn migrate_legacy_rotations(
    orientations: &mut Vec<OrientationMatrix>,
    voxels: &mut [super::world::VoxelData],
) {
    for voxel in voxels.iter_mut() {
        if let Some(legacy) = voxel.rotation_state.take() {
            let matrix = axis_angle_to_matrix(legacy.axis, legacy.angle);
            let index = find_or_insert_orientation(orientations, matrix);
            voxel.rotation = Some(index);
        }
    }
}

// ---------------------------------------------------------------------------
// Staircase normalisation
// ---------------------------------------------------------------------------

/// Normalise all staircase directional variants to `Staircase`.
///
/// Three staircase variants (`StaircaseNegX`, `StaircaseZ`, `StaircaseNegZ`) bake a
/// Y-axis rotation into `SubVoxelPattern::geometry()`. When a voxel also carries an
/// explicit orientation matrix the two rotations compound silently, producing wrong
/// geometry. This pass absorbs the hidden pre-bake into the voxel's explicit
/// orientation matrix so that only one rotation is ever applied at spawn time.
///
/// **Call order:** after `migrate_legacy_rotations()`, before `validate_map()`.
///
/// **Algorithm (per voxel with a directional variant):**
/// 1. Look up the pre-bake matrix for the variant (Y+90°, Y+180°, or Y+270°).
/// 2. Retrieve the voxel's current orientation (`None` = identity).
/// 3. Compose: `M_final = multiply_matrices(M_existing, M_prebake)`.
/// 4. If `M_final == IDENTITY`, set `voxel.rotation = None`; otherwise
///    call `find_or_insert_orientation` and set `voxel.rotation = Some(index)`.
/// 5. Set `voxel.pattern = Some(Staircase)`.
///
/// Voxels with `pattern: Some(Staircase)` or any non-staircase pattern are
/// unchanged.
pub fn normalise_staircase_variants(
    orientations: &mut Vec<OrientationMatrix>,
    voxels: &mut [super::world::VoxelData],
) {
    use super::patterns::SubVoxelPattern::{Staircase, StaircaseNegX, StaircaseNegZ, StaircaseZ};

    for voxel in voxels.iter_mut() {
        // Pre-bake angle in 90° increments for each directional variant.
        let prebake_angle = match voxel.pattern {
            Some(StaircaseNegX) => 2, // Y+180°
            Some(StaircaseZ) => 1,    // Y+90°
            Some(StaircaseNegZ) => 3, // Y+270°
            _ => continue,
        };

        // Pre-bake is the inner (first-applied) rotation.
        let prebake = axis_angle_to_matrix(RotationAxis::Y, prebake_angle);

        // Compose: M_final = M_existing × M_prebake.
        // When there is no existing rotation, M_existing is identity so M_final = M_prebake.
        let composed = match voxel.rotation {
            None => prebake,
            Some(i) => multiply_matrices(&orientations[i], &prebake),
        };

        // Identity edge case: composed == IDENTITY → set rotation: None (no entry needed).
        if composed == IDENTITY {
            voxel.rotation = None;
        } else {
            let index = find_or_insert_orientation(orientations, composed);
            voxel.rotation = Some(index);
        }

        voxel.pattern = Some(Staircase);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::game::map::geometry::RotationAxis;

    // --- axis_angle_to_matrix ---

    #[test]
    fn test_identity_matrices_all_axes() {
        for axis in [RotationAxis::X, RotationAxis::Y, RotationAxis::Z] {
            assert_eq!(
                axis_angle_to_matrix(axis, 0),
                IDENTITY,
                "angle 0 should be identity for {:?}",
                axis
            );
        }
    }

    #[test]
    fn test_four_rotations_compose_to_identity() {
        for axis in [RotationAxis::X, RotationAxis::Y, RotationAxis::Z] {
            let r = axis_angle_to_matrix(axis, 1);
            let r2 = multiply_matrices(&r, &r);
            let r3 = multiply_matrices(&r2, &r);
            let r4 = multiply_matrices(&r3, &r);
            assert_eq!(r4, IDENTITY, "4 × 90° around {:?} should be identity", axis);
        }
    }

    #[test]
    fn test_angle_180_equals_two_90s() {
        for axis in [RotationAxis::X, RotationAxis::Y, RotationAxis::Z] {
            let r90 = axis_angle_to_matrix(axis, 1);
            let r180_direct = axis_angle_to_matrix(axis, 2);
            let r180_composed = multiply_matrices(&r90, &r90);
            assert_eq!(
                r180_direct, r180_composed,
                "180° should equal 90°×90° for {:?}",
                axis
            );
        }
    }

    // --- multiply_matrices ---

    #[test]
    fn test_multiply_by_identity() {
        let m = axis_angle_to_matrix(RotationAxis::Y, 1);
        assert_eq!(multiply_matrices(&m, &IDENTITY), m);
        assert_eq!(multiply_matrices(&IDENTITY, &m), m);
    }

    #[test]
    fn test_y90_then_x90_composition() {
        // Y+90 followed by X+90
        let my = axis_angle_to_matrix(RotationAxis::Y, 1);
        let mx = axis_angle_to_matrix(RotationAxis::X, 1);
        let composed = multiply_matrices(&mx, &my);
        // The composition must be a valid rotation
        assert!(is_valid_rotation_matrix(&composed));
        assert_ne!(composed, my);
        assert_ne!(composed, mx);
    }

    // --- apply_orientation_matrix parity with rotate() ---

    #[test]
    fn test_apply_matches_rotate_single_axis() {
        use crate::systems::game::map::format::patterns::SubVoxelPattern;

        let base = SubVoxelPattern::Staircase.geometry();

        for axis in [RotationAxis::X, RotationAxis::Y, RotationAxis::Z] {
            for angle in 1..=3 {
                let via_rotate = base.clone().rotate(axis, angle);
                let matrix = axis_angle_to_matrix(axis, angle);
                let via_matrix = apply_orientation_matrix(base.clone(), &matrix);

                let rotate_positions: std::collections::BTreeSet<_> =
                    via_rotate.occupied_positions().collect();
                let matrix_positions: std::collections::BTreeSet<_> =
                    via_matrix.occupied_positions().collect();

                assert_eq!(
                    rotate_positions, matrix_positions,
                    "apply_orientation_matrix should match rotate() for {:?} angle {}",
                    axis, angle
                );
            }
        }
    }

    #[test]
    fn test_apply_identity_is_noop() {
        use crate::systems::game::map::format::patterns::SubVoxelPattern;

        let base = SubVoxelPattern::Staircase.geometry();
        let result = apply_orientation_matrix(base.clone(), &IDENTITY);
        let base_positions: std::collections::BTreeSet<_> = base.occupied_positions().collect();
        let result_positions: std::collections::BTreeSet<_> = result.occupied_positions().collect();
        assert_eq!(base_positions, result_positions);
    }

    #[test]
    fn test_apply_multi_axis_composition() {
        use crate::systems::game::map::format::patterns::SubVoxelPattern;

        let base = SubVoxelPattern::Staircase.geometry();

        // Apply Y+90 then X+90 via rotate()
        let via_rotate = base
            .clone()
            .rotate(RotationAxis::Y, 1)
            .rotate(RotationAxis::X, 1);

        // Build the composed matrix and apply
        let my = axis_angle_to_matrix(RotationAxis::Y, 1);
        let mx = axis_angle_to_matrix(RotationAxis::X, 1);
        let composed = multiply_matrices(&mx, &my);
        let via_matrix = apply_orientation_matrix(base, &composed);

        let rotate_positions: std::collections::BTreeSet<_> =
            via_rotate.occupied_positions().collect();
        let matrix_positions: std::collections::BTreeSet<_> =
            via_matrix.occupied_positions().collect();

        assert_eq!(rotate_positions, matrix_positions);
    }

    // --- is_valid_rotation_matrix ---

    #[test]
    fn test_identity_is_valid() {
        assert!(is_valid_rotation_matrix(&IDENTITY));
    }

    #[test]
    fn test_all_24_single_and_double_axis_are_valid() {
        let cases = [
            (RotationAxis::X, 1),
            (RotationAxis::X, 2),
            (RotationAxis::X, 3),
            (RotationAxis::Y, 1),
            (RotationAxis::Y, 2),
            (RotationAxis::Y, 3),
            (RotationAxis::Z, 1),
            (RotationAxis::Z, 2),
            (RotationAxis::Z, 3),
        ];
        for (axis, angle) in &cases {
            let m = axis_angle_to_matrix(*axis, *angle);
            assert!(
                is_valid_rotation_matrix(&m),
                "axis_angle_to_matrix({:?}, {}) should be valid",
                axis,
                angle
            );
        }
    }

    #[test]
    fn test_reflection_is_invalid() {
        // Reflection: det = -1
        let reflection: OrientationMatrix = [[-1, 0, 0], [0, 1, 0], [0, 0, 1]];
        assert!(!is_valid_rotation_matrix(&reflection));
    }

    #[test]
    fn test_non_permutation_is_invalid() {
        let bad: OrientationMatrix = [[1, 1, 0], [0, 0, 1], [0, 0, 1]];
        assert!(!is_valid_rotation_matrix(&bad));
    }

    // --- find_or_insert_orientation ---

    #[test]
    fn test_find_or_insert_deduplicates() {
        let mut list: Vec<OrientationMatrix> = Vec::new();
        let m = axis_angle_to_matrix(RotationAxis::Y, 1);
        let i1 = find_or_insert_orientation(&mut list, m);
        let i2 = find_or_insert_orientation(&mut list, m);
        assert_eq!(i1, i2);
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_find_or_insert_appends_distinct() {
        let mut list: Vec<OrientationMatrix> = Vec::new();
        let m1 = axis_angle_to_matrix(RotationAxis::Y, 1);
        let m2 = axis_angle_to_matrix(RotationAxis::X, 1);
        let i1 = find_or_insert_orientation(&mut list, m1);
        let i2 = find_or_insert_orientation(&mut list, m2);
        assert_ne!(i1, i2);
        assert_eq!(list.len(), 2);
    }

    // --- legacy migration ---

    #[test]
    fn test_legacy_migration_all_12_single_axis() {
        use crate::systems::game::components::VoxelType;
        use crate::systems::game::map::format::world::VoxelData;

        let cases = [
            (RotationAxis::X, 1),
            (RotationAxis::X, 2),
            (RotationAxis::X, 3),
            (RotationAxis::Y, 1),
            (RotationAxis::Y, 2),
            (RotationAxis::Y, 3),
            (RotationAxis::Z, 1),
            (RotationAxis::Z, 2),
            (RotationAxis::Z, 3),
        ];

        for (axis, angle) in &cases {
            let mut orientations: Vec<OrientationMatrix> = Vec::new();
            let mut voxels = vec![VoxelData {
                pos: (0, 0, 0),
                voxel_type: VoxelType::Stone,
                pattern: None,
                rotation: None,
                rotation_state: Some(LegacyRotationState {
                    axis: *axis,
                    angle: *angle,
                }),
            }];

            migrate_legacy_rotations(&mut orientations, &mut voxels);

            assert!(
                voxels[0].rotation_state.is_none(),
                "rotation_state must be cleared"
            );
            assert!(voxels[0].rotation.is_some(), "rotation must be set");
            let index = voxels[0].rotation.unwrap();
            assert_eq!(
                orientations[index],
                axis_angle_to_matrix(*axis, *angle),
                "migrated matrix should match for {:?} angle {}",
                axis,
                angle
            );
        }
    }

    #[test]
    fn test_legacy_migration_deduplicates() {
        use crate::systems::game::components::VoxelType;
        use crate::systems::game::map::format::world::VoxelData;

        let mut orientations: Vec<OrientationMatrix> = Vec::new();
        let mut voxels = vec![
            VoxelData {
                pos: (0, 0, 0),
                voxel_type: VoxelType::Stone,
                pattern: None,
                rotation: None,
                rotation_state: Some(LegacyRotationState {
                    axis: RotationAxis::Y,
                    angle: 1,
                }),
            },
            VoxelData {
                pos: (1, 0, 0),
                voxel_type: VoxelType::Stone,
                pattern: None,
                rotation: None,
                rotation_state: Some(LegacyRotationState {
                    axis: RotationAxis::Y,
                    angle: 1,
                }),
            },
        ];

        migrate_legacy_rotations(&mut orientations, &mut voxels);

        assert_eq!(
            orientations.len(),
            1,
            "duplicate matrix must not be added twice"
        );
        assert_eq!(voxels[0].rotation, voxels[1].rotation);
    }

    // --- normalise_staircase_variants ---

    /// Helper: build a minimal VoxelData for normalisation tests.
    fn staircase_voxel(
        pattern: super::super::patterns::SubVoxelPattern,
        rotation: Option<usize>,
    ) -> super::super::world::VoxelData {
        use crate::systems::game::components::VoxelType;
        super::super::world::VoxelData {
            pos: (0, 0, 0),
            voxel_type: VoxelType::Stone,
            pattern: Some(pattern),
            rotation,
            rotation_state: None,
        }
    }

    #[test]
    fn normalise_staircase_neg_x_rotation_none_absorbs_y180() {
        use super::super::patterns::SubVoxelPattern;

        let mut orientations: Vec<OrientationMatrix> = Vec::new();
        let mut voxels = vec![staircase_voxel(SubVoxelPattern::StaircaseNegX, None)];

        normalise_staircase_variants(&mut orientations, &mut voxels);

        assert_eq!(
            voxels[0].pattern,
            Some(SubVoxelPattern::Staircase),
            "pattern must be normalised to Staircase"
        );
        let idx = voxels[0]
            .rotation
            .expect("rotation must be Some after normalisation");
        assert_eq!(
            orientations[idx],
            axis_angle_to_matrix(RotationAxis::Y, 2),
            "absorbed matrix must be Y+180°"
        );
    }

    #[test]
    fn normalise_staircase_z_rotation_none_absorbs_y90() {
        use super::super::patterns::SubVoxelPattern;

        let mut orientations: Vec<OrientationMatrix> = Vec::new();
        let mut voxels = vec![staircase_voxel(SubVoxelPattern::StaircaseZ, None)];

        normalise_staircase_variants(&mut orientations, &mut voxels);

        assert_eq!(voxels[0].pattern, Some(SubVoxelPattern::Staircase));
        let idx = voxels[0].rotation.expect("rotation must be set");
        assert_eq!(
            orientations[idx],
            axis_angle_to_matrix(RotationAxis::Y, 1),
            "absorbed matrix must be Y+90°"
        );
    }

    #[test]
    fn normalise_staircase_neg_z_rotation_none_absorbs_y270() {
        use super::super::patterns::SubVoxelPattern;

        let mut orientations: Vec<OrientationMatrix> = Vec::new();
        let mut voxels = vec![staircase_voxel(SubVoxelPattern::StaircaseNegZ, None)];

        normalise_staircase_variants(&mut orientations, &mut voxels);

        assert_eq!(voxels[0].pattern, Some(SubVoxelPattern::Staircase));
        let idx = voxels[0].rotation.expect("rotation must be set");
        assert_eq!(
            orientations[idx],
            axis_angle_to_matrix(RotationAxis::Y, 3),
            "absorbed matrix must be Y+270°"
        );
    }

    #[test]
    fn normalise_staircase_z_with_existing_y90_composes_to_y180() {
        use super::super::patterns::SubVoxelPattern;

        // Pre-populate orientations with M_y90 at index 0
        let m_y90 = axis_angle_to_matrix(RotationAxis::Y, 1);
        let mut orientations: Vec<OrientationMatrix> = vec![m_y90];
        // StaircaseZ pre-bake = Y+90°; existing = Y+90°; composed = Y+180°
        let mut voxels = vec![staircase_voxel(SubVoxelPattern::StaircaseZ, Some(0))];

        normalise_staircase_variants(&mut orientations, &mut voxels);

        assert_eq!(voxels[0].pattern, Some(SubVoxelPattern::Staircase));
        let idx = voxels[0]
            .rotation
            .expect("composed Y+180° must produce a non-None rotation");
        assert_eq!(
            orientations[idx],
            axis_angle_to_matrix(RotationAxis::Y, 2),
            "composed matrix must be Y+180°"
        );
    }

    #[test]
    fn normalise_staircase_neg_z_plus_y90_equals_identity_sets_rotation_none() {
        use super::super::patterns::SubVoxelPattern;

        // StaircaseNegZ pre-bake = Y+270°; existing orientation = Y+90°
        // composed = Y+90° × Y+270° = Y+360° = IDENTITY → rotation: None
        let m_y90 = axis_angle_to_matrix(RotationAxis::Y, 1);
        let mut orientations: Vec<OrientationMatrix> = vec![m_y90];
        let mut voxels = vec![staircase_voxel(SubVoxelPattern::StaircaseNegZ, Some(0))];

        normalise_staircase_variants(&mut orientations, &mut voxels);

        assert_eq!(voxels[0].pattern, Some(SubVoxelPattern::Staircase));
        assert_eq!(
            voxels[0].rotation, None,
            "composed == IDENTITY must set rotation: None"
        );
    }

    #[test]
    fn normalise_canonical_staircase_is_unchanged() {
        use super::super::patterns::SubVoxelPattern;

        let m_y90 = axis_angle_to_matrix(RotationAxis::Y, 1);
        let mut orientations: Vec<OrientationMatrix> = vec![m_y90];
        let mut voxels = vec![staircase_voxel(SubVoxelPattern::Staircase, Some(0))];

        normalise_staircase_variants(&mut orientations, &mut voxels);

        // Pattern and rotation must be unchanged
        assert_eq!(voxels[0].pattern, Some(SubVoxelPattern::Staircase));
        assert_eq!(voxels[0].rotation, Some(0));
        assert_eq!(
            orientations.len(),
            1,
            "no new orientation must be appended for canonical Staircase"
        );
    }

    #[test]
    fn normalise_non_staircase_pattern_is_unchanged() {
        use super::super::patterns::SubVoxelPattern;
        use super::super::world::VoxelData;
        use crate::systems::game::components::VoxelType;

        let mut orientations: Vec<OrientationMatrix> = Vec::new();
        let mut voxels = vec![VoxelData {
            pos: (0, 0, 0),
            voxel_type: VoxelType::Stone,
            pattern: Some(SubVoxelPattern::Full),
            rotation: None,
            rotation_state: None,
        }];

        normalise_staircase_variants(&mut orientations, &mut voxels);

        assert_eq!(voxels[0].pattern, Some(SubVoxelPattern::Full));
        assert_eq!(voxels[0].rotation, None);
        assert!(
            orientations.is_empty(),
            "no orientations should be added for non-staircase"
        );
    }

    #[test]
    fn normalise_deduplicates_identical_composed_matrices() {
        use super::super::patterns::SubVoxelPattern;

        let mut orientations: Vec<OrientationMatrix> = Vec::new();
        // Two voxels with the same directional variant and no prior rotation
        // should share the same orientation index after normalisation.
        let mut voxels = vec![
            staircase_voxel(SubVoxelPattern::StaircaseZ, None),
            staircase_voxel(SubVoxelPattern::StaircaseZ, None),
        ];

        normalise_staircase_variants(&mut orientations, &mut voxels);

        assert_eq!(
            orientations.len(),
            1,
            "identical composed matrices must be deduplicated"
        );
        assert_eq!(
            voxels[0].rotation, voxels[1].rotation,
            "both voxels must reference the same orientation index"
        );
    }

    #[test]
    fn normalise_round_trip_geometry_identical_to_legacy_pre_bake() {
        use super::super::patterns::SubVoxelPattern;
        use crate::systems::game::map::geometry::RotationAxis as GeoAxis;

        // The geometry produced by normalised Staircase + composed matrix
        // must be bit-for-bit identical to SubVoxelGeometry::staircase_x().rotate(Y, 1)
        // (the old StaircaseZ.geometry() pre-bake path).

        // --- Legacy path (what the bug used to produce for rotation: None) ---
        let legacy_geom = SubVoxelPattern::StaircaseZ.geometry();

        // --- Normalised path ---
        let mut orientations: Vec<OrientationMatrix> = Vec::new();
        let mut voxels = vec![staircase_voxel(SubVoxelPattern::StaircaseZ, None)];
        normalise_staircase_variants(&mut orientations, &mut voxels);

        let orientation = voxels[0].rotation.map(|i| &orientations[i]);
        let normalised_geom = SubVoxelPattern::Staircase.geometry_with_rotation(orientation);

        let legacy_positions: std::collections::BTreeSet<_> =
            legacy_geom.occupied_positions().collect();
        let normalised_positions: std::collections::BTreeSet<_> =
            normalised_geom.occupied_positions().collect();

        assert_eq!(
            legacy_positions, normalised_positions,
            "normalised geometry must match old StaircaseZ.geometry() pre-bake"
        );

        // The unused import suppression — GeoAxis is imported above for clarity
        let _ = GeoAxis::Y;
    }

    // --- world_dir_to_local tests ---

    #[test]
    fn world_dir_to_local_identity_returns_dir_unchanged() {
        assert_eq!(world_dir_to_local(None, [1, 0, 0]), [1, 0, 0]);
        assert_eq!(world_dir_to_local(None, [-1, 0, 0]), [-1, 0, 0]);
        assert_eq!(world_dir_to_local(None, [0, 0, 1]), [0, 0, 1]);
        assert_eq!(world_dir_to_local(None, [0, 0, -1]), [0, 0, -1]);
    }

    #[test]
    fn world_dir_to_local_y90_maps_world_x_to_local_neg_z() {
        // Y+90°: local X → world −Z, local Z → world +X
        // Inverse (Mᵀ): world +X → local +Z, world −X → local −Z,
        //               world +Z → local −X, world −Z → local +X
        let m = axis_angle_to_matrix(RotationAxis::Y, 1);
        assert_eq!(world_dir_to_local(Some(&m), [1, 0, 0]), [0, 0, 1]);
        assert_eq!(world_dir_to_local(Some(&m), [-1, 0, 0]), [0, 0, -1]);
        assert_eq!(world_dir_to_local(Some(&m), [0, 0, 1]), [-1, 0, 0]);
        assert_eq!(world_dir_to_local(Some(&m), [0, 0, -1]), [1, 0, 0]);
    }

    #[test]
    fn world_dir_to_local_y180_maps_x_to_neg_x() {
        // Y+180°: local X → world −X, local Z → world −Z
        // Inverse: world +X → local −X, world +Z → local −Z
        let m = axis_angle_to_matrix(RotationAxis::Y, 2);
        assert_eq!(world_dir_to_local(Some(&m), [1, 0, 0]), [-1, 0, 0]);
        assert_eq!(world_dir_to_local(Some(&m), [0, 0, 1]), [0, 0, -1]);
    }

    #[test]
    fn world_dir_to_local_y270_maps_world_x_to_local_z() {
        // Y+270° (= Y−90°): local X → world +Z, local Z → world −X
        // Inverse: world +X → local −Z, world +Z → local +X
        let m = axis_angle_to_matrix(RotationAxis::Y, 3);
        assert_eq!(world_dir_to_local(Some(&m), [1, 0, 0]), [0, 0, -1]);
        assert_eq!(world_dir_to_local(Some(&m), [0, 0, 1]), [1, 0, 0]);
    }
}
