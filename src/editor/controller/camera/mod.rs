//! First-person camera for controller editing mode.
//!
//! Provides a fly-through camera similar to Minecraft Creative mode.

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use std::f32::consts::PI;

/// Event to toggle between orbit and controller camera modes.
#[derive(Message)]
pub struct ControllerModeToggleEvent;

/// Tracks which camera mode is active.
/// Note: Always defaults to FirstPerson as all input methods (gamepad, keyboard, mouse)
/// now work simultaneously without needing mode switching.
#[derive(Resource, PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum ControllerCameraMode {
    /// Traditional orbit camera (mouse-based) - DEPRECATED, kept for compatibility
    Orbit,
    /// First-person flying camera (all input methods work together)
    #[default]
    FirstPerson,
}

/// Component for the first-person controller camera.
#[derive(Component)]
pub struct ControllerCamera {
    /// Position in world space
    pub position: Vec3,
    /// Yaw angle (horizontal rotation) in radians
    pub yaw: f32,
    /// Pitch angle (vertical rotation) in radians
    pub pitch: f32,
    /// Base movement speed (units per second)
    pub speed: f32,
    /// Sprint speed multiplier
    pub sprint_multiplier: f32,
    /// Look sensitivity
    pub sensitivity: f32,
    /// Whether sprinting is active
    pub is_sprinting: bool,
    /// Vertical velocity for flying
    pub vertical_velocity: f32,
}

impl Default for ControllerCamera {
    fn default() -> Self {
        Self {
            position: Vec3::new(5.0, 5.0, 5.0),
            yaw: -PI / 4.0,
            pitch: -0.3,
            speed: 8.0,
            sprint_multiplier: 2.5,
            sensitivity: 2.0,
            is_sprinting: false,
            vertical_velocity: 0.0,
        }
    }
}

impl ControllerCamera {
    /// Create a new controller camera at a position.
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Get the forward direction vector (horizontal plane only).
    pub fn forward(&self) -> Vec3 {
        Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos()).normalize()
    }

    /// Get the full forward direction including pitch.
    pub fn forward_3d(&self) -> Vec3 {
        Vec3::new(
            self.pitch.cos() * self.yaw.sin(),
            self.pitch.sin(),
            self.pitch.cos() * self.yaw.cos(),
        )
        .normalize()
    }

    /// Get the right direction vector.
    pub fn right(&self) -> Vec3 {
        Vec3::new(self.yaw.cos(), 0.0, -self.yaw.sin()).normalize()
    }

    /// Get the current movement speed (accounting for sprint).
    pub fn current_speed(&self) -> f32 {
        if self.is_sprinting {
            self.speed * self.sprint_multiplier
        } else {
            self.speed
        }
    }

    /// Apply movement from stick input.
    pub fn apply_movement(&mut self, movement: Vec2, vertical: f32, dt: f32) {
        let speed = self.current_speed();

        // Forward/backward and strafe movement
        let forward = self.forward();
        let right = self.right();

        self.position += forward * movement.y * speed * dt;
        self.position += right * movement.x * speed * dt;

        // Vertical movement (flying up/down)
        self.position.y += vertical * speed * dt;
    }

    /// Apply look rotation from stick input.
    pub fn apply_look(&mut self, look: Vec2, dt: f32) {
        self.yaw += look.x * self.sensitivity * dt;
        self.pitch -= look.y * self.sensitivity * dt;

        // Clamp pitch to prevent flipping
        self.pitch = self.pitch.clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);

        // Keep yaw in reasonable range
        if self.yaw > PI * 2.0 {
            self.yaw -= PI * 2.0;
        } else if self.yaw < -PI * 2.0 {
            self.yaw += PI * 2.0;
        }
    }

    /// Calculate the camera transform.
    pub fn calculate_transform(&self) -> Transform {
        let mut transform = Transform::from_translation(self.position);
        transform.rotation = Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0);
        transform
    }
}

/// System to toggle between camera modes.
/// Note: This is now mostly a no-op since we always stay in FirstPerson mode,
/// but kept for compatibility. All input methods work simultaneously.
pub fn toggle_controller_mode(
    _keyboard: Res<ButtonInput<KeyCode>>,
    _gamepads: Query<&Gamepad>,
    _mode: ResMut<ControllerCameraMode>,
    mut _contexts: EguiContexts,
    _toggle_events: MessageWriter<ControllerModeToggleEvent>,
) {
    // Mode toggling is disabled - all input methods now work together in FirstPerson mode.
    // The mode is always FirstPerson by default.
}

/// System to update the controller camera based on input.
/// Note: This system is now a no-op as the main EditorCamera handles all input
/// (gamepad, keyboard, mouse) together in handle_camera_input.
pub fn update_controller_camera(
    _mode: Res<ControllerCameraMode>,
    _gamepads: Query<&Gamepad>,
    _keyboard: Res<ButtonInput<KeyCode>>,
    _time: Res<Time>,
    _camera_query: Query<(&mut Transform, &mut ControllerCamera), With<Camera3d>>,
    mut _contexts: EguiContexts,
) {
    // Camera input is now handled by handle_camera_input in camera.rs
    // which processes all input methods (gamepad, keyboard, mouse) together.
}

/// System to sync controller camera position when switching modes.
pub fn sync_camera_on_mode_switch(
    mut toggle_events: MessageReader<ControllerModeToggleEvent>,
    mode: Res<ControllerCameraMode>,
    mut camera_query: Query<(&Transform, Option<&mut ControllerCamera>), With<Camera3d>>,
    editor_cam: Single<&crate::editor::camera::EditorCamera>,
) {
    for _ in toggle_events.read() {
        match *mode {
            ControllerCameraMode::FirstPerson => {
                // Switching to first-person: copy fly camera position
                for (_, controller_cam) in camera_query.iter_mut() {
                    if let Some(mut ctrl) = controller_cam {
                        ctrl.position = editor_cam.position;
                        ctrl.yaw = editor_cam.yaw;
                        ctrl.pitch = editor_cam.pitch;
                    }
                }
            }
            ControllerCameraMode::Orbit => {
                // No longer using orbit mode - this is a no-op
            }
        }
    }
}

#[cfg(test)]
mod tests;
