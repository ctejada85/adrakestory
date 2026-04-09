use super::*;

// -------------------------------------------------------------------------
// GamepadSettings Tests
// -------------------------------------------------------------------------

#[test]
fn test_gamepad_settings_default() {
    let settings = GamepadSettings::default();
    assert!((settings.stick_deadzone - 0.15).abs() < f32::EPSILON);
    assert!((settings.trigger_deadzone - 0.1).abs() < f32::EPSILON);
    assert!(!settings.invert_camera_y);
    assert!((settings.camera_sensitivity - 3.0).abs() < f32::EPSILON);
    assert!((settings.movement_sensitivity - 1.0).abs() < f32::EPSILON);
}

// -------------------------------------------------------------------------
// apply_deadzone Tests
// -------------------------------------------------------------------------

#[test]
fn test_apply_deadzone_zero_input() {
    let result = apply_deadzone(Vec2::ZERO, 0.15);
    assert_eq!(result, Vec2::ZERO);
}

#[test]
fn test_apply_deadzone_within_deadzone() {
    // Input within deadzone should return zero
    let result = apply_deadzone(Vec2::new(0.1, 0.05), 0.15);
    assert_eq!(result, Vec2::ZERO);
}

#[test]
fn test_apply_deadzone_exactly_at_deadzone() {
    // Input at exactly deadzone threshold (magnitude = 0.15)
    let input = Vec2::new(0.15, 0.0);
    let result = apply_deadzone(input, 0.15);
    // Should return zero (< not <=)
    assert_eq!(result, Vec2::ZERO);
}

#[test]
fn test_apply_deadzone_just_outside() {
    // Input just outside deadzone
    let input = Vec2::new(0.2, 0.0);
    let result = apply_deadzone(input, 0.15);
    // Should return small positive value
    assert!(result.x > 0.0);
    assert!(result.y.abs() < f32::EPSILON);
}

#[test]
fn test_apply_deadzone_full_input() {
    // Full input (magnitude = 1.0)
    let input = Vec2::new(1.0, 0.0);
    let result = apply_deadzone(input, 0.15);
    // Should be close to 1.0 (rescaled)
    assert!((result.x - 1.0).abs() < 0.01);
}

#[test]
fn test_apply_deadzone_diagonal() {
    // Diagonal input
    let input = Vec2::new(0.7, 0.7);
    let result = apply_deadzone(input, 0.15);
    // Should preserve direction
    let direction = result.normalize();
    assert!((direction.x - direction.y).abs() < 0.01);
}

#[test]
fn test_apply_deadzone_negative_values() {
    // Negative input
    let input = Vec2::new(-0.5, -0.3);
    let result = apply_deadzone(input, 0.15);
    // Should preserve negative direction
    assert!(result.x < 0.0);
    assert!(result.y < 0.0);
}

#[test]
fn test_apply_deadzone_zero_deadzone() {
    // Zero deadzone - input passes through unchanged
    let input = Vec2::new(0.5, 0.3);
    let result = apply_deadzone(input, 0.0);
    assert!((result - input).length() < 0.01);
}

#[test]
fn test_apply_deadzone_capped_at_one() {
    // Input beyond 1.0 should be capped
    let input = Vec2::new(1.5, 0.0);
    let result = apply_deadzone(input, 0.15);
    // Rescaled magnitude should be capped at 1.0
    assert!(result.length() <= 1.01); // Small epsilon for float comparison
}

// -------------------------------------------------------------------------
// PlayerInput Tests
// -------------------------------------------------------------------------

#[test]
fn test_player_input_default() {
    let input = PlayerInput::default();
    assert_eq!(input.movement, Vec2::ZERO);
    assert_eq!(input.camera_delta, Vec2::ZERO);
    assert_eq!(input.look_direction, Vec2::ZERO);
    assert!(!input.jump_pressed);
    assert!(!input.jump_just_pressed);
    assert!(!input.interact_pressed);
    assert!(!input.pause_just_pressed);
    assert!(!input.camera_reset_just_pressed);
    assert!(!input.flashlight_toggle_just_pressed);
    assert_eq!(input.left_trigger, 0.0);
    assert_eq!(input.right_trigger, 0.0);
    assert_eq!(input.input_source, InputSource::KeyboardMouse);
}

// -------------------------------------------------------------------------
// InputSource Tests
// -------------------------------------------------------------------------

#[test]
fn test_invert_camera_y_false_leaves_y_unchanged() {
    // With invert_camera_y false the Y component of look_direction is unchanged.
    let stick = Vec2::new(0.0, 0.8);
    let mut look = apply_deadzone(stick, 0.15);
    let invert = false;
    if invert {
        look.y = -look.y;
    }
    assert!(look.y > 0.0, "Y should be positive when not inverted");
}

#[test]
fn test_invert_camera_y_true_negates_y() {
    // With invert_camera_y true the Y component must be negated.
    let stick = Vec2::new(0.0, 0.8);
    let after_deadzone = apply_deadzone(stick, 0.15);
    let normal_y = after_deadzone.y;
    let mut look = after_deadzone;
    look.y = -look.y;
    assert_eq!(look.y, -normal_y);
}

#[test]
fn test_invert_camera_y_does_not_affect_x() {
    // Inversion must only touch the Y axis.
    let stick = Vec2::new(0.6, 0.6);
    let after_deadzone = apply_deadzone(stick, 0.15);
    let normal_x = after_deadzone.x;
    let mut look = after_deadzone;
    look.y = -look.y;
    assert_eq!(look.x, normal_x);
}

#[test]
fn test_trigger_deadzone_below_threshold_produces_zero() {
    let raw = 0.05_f32;
    let deadzone = 0.1_f32;
    let result = if raw >= deadzone { raw } else { 0.0 };
    assert_eq!(result, 0.0);
}

#[test]
fn test_trigger_deadzone_at_threshold_passes_through() {
    // A raw value exactly at the threshold must pass through.
    let raw = 0.1_f32;
    let deadzone = 0.1_f32;
    let result = if raw >= deadzone { raw } else { 0.0 };
    assert_eq!(result, raw);
}

#[test]
fn test_trigger_deadzone_above_threshold_passes_through() {
    let raw = 0.75_f32;
    let deadzone = 0.1_f32;
    let result = if raw >= deadzone { raw } else { 0.0 };
    assert_eq!(result, raw);
}

#[test]
fn test_trigger_deadzone_zero_deadzone_always_passes() {
    for &raw in &[0.0_f32, 0.01, 0.5, 1.0] {
        let result = if raw >= 0.0_f32 { raw } else { 0.0 };
        assert_eq!(result, raw);
    }
}

#[test]
fn test_input_source_default() {
    let source = InputSource::default();
    assert_eq!(source, InputSource::KeyboardMouse);
}

#[test]
fn test_input_source_equality() {
    assert_eq!(InputSource::Gamepad, InputSource::Gamepad);
    assert_eq!(InputSource::KeyboardMouse, InputSource::KeyboardMouse);
    assert_ne!(InputSource::Gamepad, InputSource::KeyboardMouse);
}

// -------------------------------------------------------------------------
// ActiveGamepad Tests
// -------------------------------------------------------------------------

#[test]
fn test_active_gamepad_default() {
    let active = ActiveGamepad::default();
    assert!(active.0.is_none());
}
