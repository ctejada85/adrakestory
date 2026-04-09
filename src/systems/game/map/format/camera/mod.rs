//! Camera configuration structures.

use serde::{Deserialize, Serialize};

/// Camera configuration for the map.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CameraData {
    /// Camera position in world space (x, y, z).
    pub position: (f32, f32, f32),
    /// World-space point the camera initially looks at (x, y, z).
    pub look_at: (f32, f32, f32),
    /// Additional Y-axis rotation applied around `look_at` after the initial
    /// `looking_at` transform is established. Value is in radians.
    /// `-π/2` (≈ `-1.5707964`) rotates 90° to the left; `π/2` rotates 90° right.
    pub rotation_offset: f32,
    /// Speed at which the camera follows the player (exponential decay rate).
    /// Higher values produce a more responsive follow.
    /// When absent from the map file, the engine default (`15.0`) is used.
    #[serde(default)]
    pub follow_speed: Option<f32>,
    /// Speed at which the camera interpolates toward its target rotation
    /// (exponential decay rate).
    /// When absent from the map file, the engine default (`5.0`) is used.
    #[serde(default)]
    pub rotation_speed: Option<f32>,
    /// Vertical field of view in degrees.
    /// When absent, the engine default (~60°) is used. Recommended range: 5–150.
    #[serde(default)]
    pub fov_degrees: Option<f32>,
}

impl Default for CameraData {
    fn default() -> Self {
        Self {
            position: (1.5, 8.0, 5.5),
            look_at: (1.5, 0.0, 1.5),
            rotation_offset: -std::f32::consts::FRAC_PI_2,
            follow_speed: None,
            rotation_speed: None,
            fov_degrees: None,
        }
    }
}

#[cfg(test)]
mod tests;
