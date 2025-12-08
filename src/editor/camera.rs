//! Editor camera system with orbit, pan, and zoom controls.

use bevy::prelude::*;
use bevy_egui::EguiContexts;

/// Component for the editor camera
#[derive(Component)]
pub struct EditorCamera {
    /// Target point the camera orbits around
    pub target: Vec3,

    /// Distance from target
    pub distance: f32,

    /// Rotation around target (yaw, pitch)
    pub rotation: Vec2,

    /// Camera movement speed
    pub pan_speed: f32,

    /// Camera rotation speed
    pub orbit_speed: f32,

    /// Zoom speed
    pub zoom_speed: f32,

    /// Minimum distance from target
    pub min_distance: f32,

    /// Maximum distance from target
    pub max_distance: f32,
}

impl Default for EditorCamera {
    fn default() -> Self {
        Self {
            target: Vec3::new(2.0, 0.0, 2.0),
            distance: 10.0,
            rotation: Vec2::new(0.0, 0.5), // yaw, pitch
            pan_speed: 0.00125,
            orbit_speed: 0.005,
            zoom_speed: 0.0125,
            min_distance: 2.0,
            max_distance: 50.0,
        }
    }
}

impl EditorCamera {
    /// Create a new editor camera with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new editor camera looking at a specific target
    pub fn looking_at(target: Vec3, distance: f32) -> Self {
        Self {
            target,
            distance,
            ..Default::default()
        }
    }

    /// Calculate the camera position based on target, distance, and rotation
    pub fn calculate_position(&self) -> Vec3 {
        let yaw = self.rotation.x;
        let pitch = self.rotation.y.clamp(-1.5, 1.5); // Limit pitch to avoid gimbal lock

        let x = self.distance * pitch.cos() * yaw.sin();
        let y = self.distance * pitch.sin();
        let z = self.distance * pitch.cos() * yaw.cos();

        self.target + Vec3::new(x, y, z)
    }

    /// Orbit the camera around the target
    pub fn orbit(&mut self, delta: Vec2) {
        self.rotation.x += delta.x * self.orbit_speed;
        self.rotation.y += delta.y * self.orbit_speed;

        // Clamp pitch to avoid flipping
        self.rotation.y = self.rotation.y.clamp(-1.5, 1.5);
    }

    /// Pan the camera (move target)
    pub fn pan(&mut self, delta: Vec2) {
        let yaw = self.rotation.x;

        // Calculate right and up vectors
        let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
        let up = Vec3::Y;

        self.target += right * delta.x * self.pan_speed * self.distance;
        self.target += up * delta.y * self.pan_speed * self.distance;
    }

    /// Zoom the camera (change distance)
    pub fn zoom(&mut self, delta: f32) {
        self.distance -= delta * self.zoom_speed * self.distance;
        self.distance = self.distance.clamp(self.min_distance, self.max_distance);
    }

    /// Reset camera to default position
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Set camera to look at a specific point from a specific position
    pub fn set_view(&mut self, position: Vec3, target: Vec3) {
        self.target = target;
        let offset = position - target;
        self.distance = offset.length();

        // Calculate rotation from offset
        self.rotation.x = offset.z.atan2(offset.x);
        self.rotation.y = (offset.y / self.distance).asin();
    }
}

/// System to update camera transform based on EditorCamera component
pub fn update_editor_camera(mut query: Query<(&EditorCamera, &mut Transform), With<Camera3d>>) {
    for (editor_cam, mut transform) in query.iter_mut() {
        let position = editor_cam.calculate_position();
        transform.translation = position;
        transform.look_at(editor_cam.target, Vec3::Y);
    }
}

/// Resource to track camera input state
#[derive(Resource, Default)]
pub struct CameraInputState {
    /// Whether the camera is being orbited
    pub is_orbiting: bool,

    /// Whether the camera is being panned
    pub is_panning: bool,

    /// Last mouse position
    pub last_mouse_pos: Option<Vec2>,
}

/// System to handle camera input
pub fn handle_camera_input(
    mut camera_query: Query<&mut EditorCamera>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
    mut mouse_wheel: EventReader<bevy::input::mouse::MouseWheel>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut input_state: ResMut<CameraInputState>,
    mut contexts: EguiContexts,
) {
    let Ok(mut camera) = camera_query.get_single_mut() else {
        return;
    };

    // Check if pointer is over any UI area - don't process camera input if mouse is over UI panels
    let pointer_over_ui = contexts.ctx_mut().is_pointer_over_area();

    // Handle mouse button state
    let shift_pressed =
        keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let space_pressed = keyboard.pressed(KeyCode::Space);
    let ctrl_pressed =
        keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    let super_pressed =
        keyboard.pressed(KeyCode::SuperLeft) || keyboard.pressed(KeyCode::SuperRight);

    let middle_mouse = mouse_button.pressed(MouseButton::Middle);
    let right_mouse = mouse_button.pressed(MouseButton::Right);
    let left_mouse = mouse_button.pressed(MouseButton::Left);

    // Don't start new camera operations when pointer is over UI
    // But allow continuing existing operations (for smooth drag experience)
    if pointer_over_ui && !input_state.is_orbiting && !input_state.is_panning {
        // Still need to consume events to prevent accumulation
        mouse_motion.clear();
        mouse_wheel.clear();
        return;
    }

    // Panning: Middle mouse OR Right mouse + Shift OR Left mouse + Space OR Left mouse + Cmd/Ctrl
    input_state.is_panning = middle_mouse
        || (right_mouse && shift_pressed)
        || (left_mouse && space_pressed)
        || (left_mouse && (ctrl_pressed || super_pressed));

    // Orbiting: Right mouse (without shift or other modifiers)
    input_state.is_orbiting = right_mouse && !shift_pressed && !input_state.is_panning;

    // Handle mouse motion
    for event in mouse_motion.read() {
        if input_state.is_orbiting {
            camera.orbit(Vec2::new(event.delta.x, -event.delta.y));
        } else if input_state.is_panning {
            camera.pan(Vec2::new(-event.delta.x, event.delta.y));
        }
    }

    // Handle mouse wheel (zoom)
    for event in mouse_wheel.read() {
        camera.zoom(event.y);
    }

    // Handle keyboard shortcuts
    if keyboard.just_pressed(KeyCode::Home) {
        camera.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_position_calculation() {
        let camera = EditorCamera {
            target: Vec3::ZERO,
            distance: 10.0,
            rotation: Vec2::new(0.0, 0.0),
            ..Default::default()
        };

        let pos = camera.calculate_position();
        assert!((pos.length() - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_camera_zoom() {
        let mut camera = EditorCamera::default();
        let initial_distance = camera.distance;

        camera.zoom(1.0);
        assert!(camera.distance < initial_distance);

        camera.zoom(-1.0);
        assert!((camera.distance - initial_distance).abs() < 0.1);
    }

    #[test]
    fn test_camera_zoom_limits() {
        let mut camera = EditorCamera::default();

        // Zoom in a lot
        for _ in 0..100 {
            camera.zoom(1.0);
        }
        assert!(camera.distance >= camera.min_distance);

        // Zoom out a lot
        for _ in 0..100 {
            camera.zoom(-1.0);
        }
        assert!(camera.distance <= camera.max_distance);
    }
}
