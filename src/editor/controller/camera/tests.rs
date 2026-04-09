use super::*;

#[test]
fn test_controller_camera_default() {
    let cam = ControllerCamera::default();
    assert!(cam.position.length() > 0.0);
    assert!((cam.speed - 8.0).abs() < f32::EPSILON);
}

#[test]
fn test_forward_direction() {
    let mut cam = ControllerCamera::default();
    cam.yaw = 0.0;
    cam.pitch = 0.0;

    let forward = cam.forward();
    assert!((forward.z - 1.0).abs() < 0.01);
    assert!(forward.x.abs() < 0.01);
}

#[test]
fn test_right_direction() {
    let mut cam = ControllerCamera::default();
    cam.yaw = 0.0;

    let right = cam.right();
    assert!((right.x - 1.0).abs() < 0.01);
    assert!(right.z.abs() < 0.01);
}

#[test]
fn test_sprint_speed() {
    let mut cam = ControllerCamera::default();
    let normal_speed = cam.current_speed();

    cam.is_sprinting = true;
    let sprint_speed = cam.current_speed();

    assert!(sprint_speed > normal_speed);
    assert!((sprint_speed - normal_speed * cam.sprint_multiplier).abs() < 0.01);
}

#[test]
fn test_pitch_clamping() {
    let mut cam = ControllerCamera::default();

    // Try to look straight up
    cam.apply_look(Vec2::new(0.0, 100.0), 1.0);
    assert!(cam.pitch > -PI / 2.0);
    assert!(cam.pitch < PI / 2.0);

    // Try to look straight down
    cam.apply_look(Vec2::new(0.0, -100.0), 1.0);
    assert!(cam.pitch > -PI / 2.0);
    assert!(cam.pitch < PI / 2.0);
}

#[test]
fn test_controller_camera_new() {
    let position = Vec3::new(10.0, 20.0, 30.0);
    let cam = ControllerCamera::new(position);
    assert_eq!(cam.position, position);
    // Other values should be defaults
    assert!((cam.speed - 8.0).abs() < f32::EPSILON);
}

#[test]
fn test_apply_movement_forward() {
    let mut cam = ControllerCamera::default();
    cam.position = Vec3::ZERO;
    cam.yaw = 0.0; // Looking along +Z

    let initial_pos = cam.position;
    cam.apply_movement(Vec2::new(0.0, 1.0), 0.0, 1.0); // Move forward for 1 second

    // Should move in +Z direction
    assert!(cam.position.z > initial_pos.z);
    assert!((cam.position.x - initial_pos.x).abs() < 0.01);
}

#[test]
fn test_apply_movement_strafe() {
    let mut cam = ControllerCamera::default();
    cam.position = Vec3::ZERO;
    cam.yaw = 0.0; // Looking along +Z

    let initial_pos = cam.position;
    cam.apply_movement(Vec2::new(1.0, 0.0), 0.0, 1.0); // Strafe right for 1 second

    // Should move in +X direction
    assert!(cam.position.x > initial_pos.x);
    assert!((cam.position.z - initial_pos.z).abs() < 0.01);
}

#[test]
fn test_apply_movement_vertical() {
    let mut cam = ControllerCamera::default();
    cam.position = Vec3::ZERO;

    cam.apply_movement(Vec2::ZERO, 1.0, 1.0); // Move up for 1 second
    assert!(cam.position.y > 0.0);

    cam.position = Vec3::ZERO;
    cam.apply_movement(Vec2::ZERO, -1.0, 1.0); // Move down for 1 second
    assert!(cam.position.y < 0.0);
}

#[test]
fn test_apply_look_yaw() {
    let mut cam = ControllerCamera::default();
    cam.yaw = 0.0;

    cam.apply_look(Vec2::new(1.0, 0.0), 1.0); // Look right
    assert!(cam.yaw != 0.0);
}

#[test]
fn test_apply_look_pitch() {
    let mut cam = ControllerCamera::default();
    cam.pitch = 0.0;

    cam.apply_look(Vec2::new(0.0, 1.0), 1.0); // Look up/down
    assert!(cam.pitch != 0.0);
}

#[test]
fn test_yaw_wrapping() {
    let mut cam = ControllerCamera::default();
    cam.yaw = 0.0;

    // Apply many rotations to exceed 2*PI
    for _ in 0..100 {
        cam.apply_look(Vec2::new(1.0, 0.0), 1.0);
    }

    // Yaw should be wrapped to reasonable range
    assert!(cam.yaw.abs() <= PI * 2.0 + 0.1);
}

#[test]
fn test_calculate_transform_position() {
    let cam = ControllerCamera {
        position: Vec3::new(1.0, 2.0, 3.0),
        ..Default::default()
    };

    let transform = cam.calculate_transform();
    assert_eq!(transform.translation, cam.position);
}

#[test]
fn test_forward_3d_at_zero_pitch() {
    let mut cam = ControllerCamera::default();
    cam.yaw = 0.0;
    cam.pitch = 0.0;

    let forward = cam.forward_3d();
    // At zero pitch, forward_3d should be similar to forward (horizontal)
    assert!((forward.z - 1.0).abs() < 0.01);
    assert!(forward.y.abs() < 0.01);
}

#[test]
fn test_forward_3d_looking_up() {
    let mut cam = ControllerCamera::default();
    cam.yaw = 0.0;
    cam.pitch = 0.5; // Looking up

    let forward = cam.forward_3d();
    // Should have positive Y component when looking up
    assert!(forward.y > 0.0);
}

#[test]
fn test_controller_camera_mode_default() {
    let mode = ControllerCameraMode::default();
    assert_eq!(mode, ControllerCameraMode::FirstPerson);
}

#[test]
fn test_movement_speed_with_sprint() {
    let mut cam = ControllerCamera::default();
    cam.position = Vec3::ZERO;
    cam.yaw = 0.0;

    // Move without sprint
    cam.apply_movement(Vec2::new(0.0, 1.0), 0.0, 1.0);
    let normal_distance = cam.position.length();

    // Reset and move with sprint
    cam.position = Vec3::ZERO;
    cam.is_sprinting = true;
    cam.apply_movement(Vec2::new(0.0, 1.0), 0.0, 1.0);
    let sprint_distance = cam.position.length();

    // Sprint should cover more distance
    assert!(sprint_distance > normal_distance);
    assert!((sprint_distance / normal_distance - cam.sprint_multiplier).abs() < 0.1);
}
