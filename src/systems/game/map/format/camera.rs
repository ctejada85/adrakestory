//! Camera configuration structures.

use serde::{Deserialize, Serialize};

/// Camera configuration for the map.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CameraData {
    /// Camera position (x, y, z)
    pub position: (f32, f32, f32),
    /// Point the camera looks at (x, y, z)
    pub look_at: (f32, f32, f32),
    /// Additional rotation offset in radians
    pub rotation_offset: f32,
}

impl Default for CameraData {
    fn default() -> Self {
        Self {
            position: (1.5, 8.0, 5.5),
            look_at: (1.5, 0.0, 1.5),
            rotation_offset: -std::f32::consts::FRAC_PI_2,
        }
    }
}
