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
mod tests {
    use super::*;

    #[test]
    fn camera_data_defaults_produce_none_optionals() {
        let ron = r#"(
            position: (1.0, 2.0, 3.0),
            look_at: (0.0, 0.0, 0.0),
            rotation_offset: 0.0,
        )"#;
        let cd: CameraData = ron::from_str(ron).expect("parse failed");
        assert!(cd.follow_speed.is_none());
        assert!(cd.rotation_speed.is_none());
        assert!(cd.fov_degrees.is_none());
    }

    #[test]
    fn camera_data_follow_speed_round_trips() {
        let ron = r#"(
            position: (1.0, 2.0, 3.0),
            look_at: (0.0, 0.0, 0.0),
            rotation_offset: 0.0,
            follow_speed: Some(8.0),
        )"#;
        let cd: CameraData = ron::from_str(ron).expect("parse failed");
        assert_eq!(cd.follow_speed, Some(8.0));
    }

    #[test]
    fn camera_data_rotation_speed_round_trips() {
        let ron = r#"(
            position: (1.0, 2.0, 3.0),
            look_at: (0.0, 0.0, 0.0),
            rotation_offset: 0.0,
            rotation_speed: Some(2.5),
        )"#;
        let cd: CameraData = ron::from_str(ron).expect("parse failed");
        assert_eq!(cd.rotation_speed, Some(2.5));
    }

    #[test]
    fn camera_data_fov_degrees_round_trips() {
        let ron = r#"(
            position: (1.0, 2.0, 3.0),
            look_at: (0.0, 0.0, 0.0),
            rotation_offset: 0.0,
            fov_degrees: Some(90.0),
        )"#;
        let cd: CameraData = ron::from_str(ron).expect("parse failed");
        assert_eq!(cd.fov_degrees, Some(90.0));
    }

    #[test]
    fn camera_data_all_optional_fields_present() {
        let ron = r#"(
            position: (5.0, 10.0, 5.0),
            look_at: (0.0, 0.0, 0.0),
            rotation_offset: -1.5707964,
            follow_speed: Some(5.0),
            rotation_speed: Some(3.0),
            fov_degrees: Some(75.0),
        )"#;
        let cd: CameraData = ron::from_str(ron).expect("parse failed");
        assert_eq!(cd.follow_speed, Some(5.0));
        assert_eq!(cd.rotation_speed, Some(3.0));
        assert_eq!(cd.fov_degrees, Some(75.0));
    }

    #[test]
    fn camera_data_default_impl_has_none_optionals() {
        let cd = CameraData::default();
        assert!(cd.follow_speed.is_none());
        assert!(cd.rotation_speed.is_none());
        assert!(cd.fov_degrees.is_none());
    }
}
