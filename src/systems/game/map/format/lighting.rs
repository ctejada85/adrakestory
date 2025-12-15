//! Lighting configuration structures.

use serde::{Deserialize, Serialize};

/// Lighting configuration for the map.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LightingData {
    /// Ambient light intensity (0.0 to 1.0)
    pub ambient_intensity: f32,
    /// Optional directional light
    pub directional_light: Option<DirectionalLightData>,
}

impl Default for LightingData {
    fn default() -> Self {
        Self {
            ambient_intensity: 0.3,
            directional_light: Some(DirectionalLightData::default()),
        }
    }
}

/// Directional light configuration.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DirectionalLightData {
    /// Light direction (x, y, z) - will be normalized
    pub direction: (f32, f32, f32),
    /// Light intensity in lux
    pub illuminance: f32,
    /// Light color (r, g, b) in 0.0-1.0 range
    pub color: (f32, f32, f32),
}

impl Default for DirectionalLightData {
    fn default() -> Self {
        Self {
            direction: (-0.5, -1.0, -0.5),
            illuminance: 10000.0,
            color: (1.0, 1.0, 1.0),
        }
    }
}
