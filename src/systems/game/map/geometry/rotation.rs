//! Rotation operations for SubVoxelGeometry.

use super::sub_voxel_geometry::SubVoxelGeometry;
use super::RotationAxis;

impl SubVoxelGeometry {
    /// Rotate this geometry around the given axis by the specified angle.
    ///
    /// # Arguments
    /// * `axis` - The axis to rotate around (X, Y, or Z)
    /// * `angle` - The rotation angle in 90° increments (0-3 for 0°, 90°, 180°, 270°)
    ///
    /// # Returns
    /// A new `SubVoxelGeometry` with the rotated positions.
    pub fn rotate(&self, axis: RotationAxis, angle: i32) -> Self {
        let mut result = Self::new();
        let angle = angle.rem_euclid(4);

        // No rotation needed
        if angle == 0 {
            return self.clone();
        }

        // Rotate each occupied sub-voxel
        for (x, y, z) in self.occupied_positions() {
            let (nx, ny, nz) = rotate_point(x, y, z, axis, angle);
            result.set_occupied(nx, ny, nz);
        }

        result
    }
}

/// Rotate a single point around an axis.
///
/// This uses standard 3D rotation matrices for 90° increments.
/// Coordinates are centered around (3.5, 3.5, 3.5) for rotation.
pub fn rotate_point(x: i32, y: i32, z: i32, axis: RotationAxis, angle: i32) -> (i32, i32, i32) {
    // Center coordinates around origin (subtract 3.5, but we use integer math)
    // We'll work with doubled coordinates to avoid floating point
    let cx = x * 2 - 7;
    let cy = y * 2 - 7;
    let cz = z * 2 - 7;

    let (rx, ry, rz) = match axis {
        RotationAxis::X => match angle {
            1 => (cx, -cz, cy),  // 90° CW around X
            2 => (cx, -cy, -cz), // 180° around X
            3 => (cx, cz, -cy),  // 270° CW around X
            _ => (cx, cy, cz),
        },
        RotationAxis::Y => match angle {
            1 => (cz, cy, -cx),  // 90° CW around Y
            2 => (-cx, cy, -cz), // 180° around Y
            3 => (-cz, cy, cx),  // 270° CW around Y
            _ => (cx, cy, cz),
        },
        RotationAxis::Z => match angle {
            1 => (-cy, cx, cz),  // 90° CW around Z
            2 => (-cx, -cy, cz), // 180° around Z
            3 => (cy, -cx, cz),  // 270° CW around Z
            _ => (cx, cy, cz),
        },
    };

    // Translate back (add 3.5 and convert back to integer)
    ((rx + 7) / 2, (ry + 7) / 2, (rz + 7) / 2)
}
