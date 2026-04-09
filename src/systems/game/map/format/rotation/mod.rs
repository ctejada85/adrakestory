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
mod tests;
