use super::*;

#[test]
fn test_camera_default() {
    let camera = EditorCamera::default();
    assert_eq!(camera.position, Vec3::new(5.0, 5.0, 10.0));
    assert_eq!(camera.yaw, 0.0);
    assert_eq!(camera.pitch, -0.3);
    assert_eq!(camera.move_speed, 15.0);
}

#[test]
fn test_camera_calculate_position() {
    let camera = EditorCamera {
        position: Vec3::new(1.0, 2.0, 3.0),
        ..Default::default()
    };

    let pos = camera.calculate_position();
    assert_eq!(pos, Vec3::new(1.0, 2.0, 3.0));
}

#[test]
fn test_camera_forward_vector() {
    // Camera looking straight ahead (yaw=0, pitch=0)
    let camera = EditorCamera {
        yaw: 0.0,
        pitch: 0.0,
        ..Default::default()
    };

    let forward = camera.forward();
    // Should be looking in -Z direction
    assert!(forward.z < -0.9);
    assert!(forward.x.abs() < 0.1);
    assert!(forward.y.abs() < 0.1);
}

#[test]
fn test_camera_forward_with_yaw() {
    // Camera looking to the side (yaw = PI/2)
    let camera = EditorCamera {
        yaw: std::f32::consts::PI / 2.0,
        pitch: 0.0,
        ..Default::default()
    };

    let forward = camera.forward();
    // Should be looking in -X direction
    assert!(forward.x < -0.9);
    assert!(forward.z.abs() < 0.1);
}

#[test]
fn test_camera_right_vector() {
    let camera = EditorCamera {
        yaw: 0.0,
        ..Default::default()
    };

    let right = camera.right();
    // When yaw is 0, cos(0)=1 and -sin(0)=0, so right = (1, 0, 0)
    assert!(right.x > 0.9);
    assert!(right.y.abs() < 0.01);
    assert!(right.z.abs() < 0.1);
}

#[test]
fn test_camera_apply_look() {
    let mut camera = EditorCamera::default();
    let initial_yaw = camera.yaw;
    let initial_pitch = camera.pitch;

    camera.apply_look(Vec2::new(100.0, 50.0));

    assert!(camera.yaw != initial_yaw);
    assert!(camera.pitch != initial_pitch);
}

#[test]
fn test_camera_apply_look_pitch_clamp() {
    let mut camera = EditorCamera::default();

    // Try to look way up (positive delta.y increases pitch)
    camera.apply_look(Vec2::new(0.0, 10000.0));
    assert!(camera.pitch <= 1.5);

    // Try to look way down (negative delta.y decreases pitch)
    camera.apply_look(Vec2::new(0.0, -10000.0));
    assert!(camera.pitch >= -1.5);
}

#[test]
fn test_camera_move_by() {
    let mut camera = EditorCamera::default();
    let initial_pos = camera.position;

    camera.move_by(Vec3::new(1.0, 2.0, 3.0));

    assert_eq!(camera.position, initial_pos + Vec3::new(1.0, 2.0, 3.0));
}

#[test]
fn test_camera_reset() {
    let mut camera = EditorCamera::default();

    // Modify camera state
    camera.position = Vec3::new(100.0, 100.0, 100.0);
    camera.yaw = 3.14;
    camera.pitch = 1.0;

    camera.reset();

    let default = EditorCamera::default();
    assert_eq!(camera.position, default.position);
    assert_eq!(camera.yaw, default.yaw);
    assert_eq!(camera.pitch, default.pitch);
}

#[test]
fn test_camera_set_view() {
    let mut camera = EditorCamera::default();

    let position = Vec3::new(10.0, 5.0, 10.0);
    let target = Vec3::ZERO;

    camera.set_view(position, target);

    assert_eq!(camera.position, position);
    // Yaw and pitch should be set to look at target
}

#[test]
fn test_camera_looking_at_constructor() {
    let position = Vec3::new(5.0, 5.0, 5.0);
    let target = Vec3::ZERO;

    let camera = EditorCamera::looking_at(position, target);

    assert_eq!(camera.position, position);
}

#[test]
fn test_camera_forward_horizontal() {
    let camera = EditorCamera {
        yaw: 0.0,
        pitch: 0.5, // Looking down
        ..Default::default()
    };

    let forward_h = camera.forward_horizontal();
    // Should ignore pitch, only use yaw
    assert!(forward_h.y.abs() < 0.01);
    assert!(forward_h.z < -0.9);
}

#[test]
fn test_gamepad_camera_state_default() {
    let state = GamepadCameraState::default();
    assert!(!state.active);
    assert!(state.action_position.is_none());
    assert!(state.action_grid_pos.is_none());
    assert!(state.target_voxel_pos.is_none());
}

#[test]
fn test_gamepad_camera_state_new() {
    let state = GamepadCameraState::new();
    assert!(!state.active);
    assert_eq!(state.cursor_distance, 5.0);
    assert!(state.action_position.is_none());
    assert!(state.action_grid_pos.is_none());
    assert!(state.target_voxel_pos.is_none());
}

#[test]
fn test_pattern_cycling_array_coverage() {
    use crate::systems::game::map::format::SubVoxelPattern;

    const PATTERNS: [SubVoxelPattern; 8] = [
        SubVoxelPattern::Full,
        SubVoxelPattern::PlatformXZ,
        SubVoxelPattern::PlatformXY,
        SubVoxelPattern::PlatformYZ,
        SubVoxelPattern::Staircase,
        SubVoxelPattern::Pillar,
        SubVoxelPattern::CenterCube,
        SubVoxelPattern::Fence,
    ];

    // Test forward cycling wraps correctly
    let current = 7;
    let next = (current + 1) % PATTERNS.len();
    assert_eq!(next, 0);
    assert_eq!(PATTERNS[next], SubVoxelPattern::Full);

    // Test backward cycling wraps correctly
    let current = 0;
    let prev = (current + PATTERNS.len() - 1) % PATTERNS.len();
    assert_eq!(prev, 7);
    assert_eq!(PATTERNS[prev], SubVoxelPattern::Fence);
}

#[test]
fn test_entity_cycling_array_coverage() {
    use crate::systems::game::map::format::EntityType;

    const ENTITIES: [EntityType; 6] = [
        EntityType::PlayerSpawn,
        EntityType::Npc,
        EntityType::Enemy,
        EntityType::Item,
        EntityType::Trigger,
        EntityType::LightSource,
    ];

    // Test forward cycling wraps correctly
    let current = 5;
    let next = (current + 1) % ENTITIES.len();
    assert_eq!(next, 0);
    assert_eq!(ENTITIES[next], EntityType::PlayerSpawn);

    // Test backward cycling wraps correctly
    let current = 0;
    let prev = (current + ENTITIES.len() - 1) % ENTITIES.len();
    assert_eq!(prev, 5);
    assert_eq!(ENTITIES[prev], EntityType::LightSource);
}

#[test]
fn test_pattern_cycling_finds_current() {
    use crate::systems::game::map::format::SubVoxelPattern;

    const PATTERNS: [SubVoxelPattern; 8] = [
        SubVoxelPattern::Full,
        SubVoxelPattern::PlatformXZ,
        SubVoxelPattern::PlatformXY,
        SubVoxelPattern::PlatformYZ,
        SubVoxelPattern::Staircase,
        SubVoxelPattern::Pillar,
        SubVoxelPattern::CenterCube,
        SubVoxelPattern::Fence,
    ];

    for (idx, pattern) in PATTERNS.iter().enumerate() {
        let found_idx = PATTERNS.iter().position(|p| p == pattern);
        assert_eq!(found_idx, Some(idx));
    }
}

#[test]
fn test_entity_cycling_finds_current() {
    use crate::systems::game::map::format::EntityType;

    const ENTITIES: [EntityType; 6] = [
        EntityType::PlayerSpawn,
        EntityType::Npc,
        EntityType::Enemy,
        EntityType::Item,
        EntityType::Trigger,
        EntityType::LightSource,
    ];

    for (idx, entity) in ENTITIES.iter().enumerate() {
        let found_idx = ENTITIES.iter().position(|e| e == entity);
        assert_eq!(found_idx, Some(idx));
    }
}
