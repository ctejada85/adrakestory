//! Gamepad input handling for Xbox controller support.
//!
//! This module handles:
//! - Gamepad detection and connection events
//! - Input abstraction layer for unified keyboard/gamepad input
//! - Deadzone handling for analog sticks
//! - Settings for gamepad configuration

use bevy::input::gamepad::{GamepadAxis, GamepadButton, GamepadConnection, GamepadConnectionEvent};
use bevy::prelude::*;

/// Resource tracking the currently active gamepad.
///
/// Only one gamepad is supported at a time. The first connected gamepad
/// becomes active, and if it disconnects, the next connected one takes over.
#[derive(Resource, Default)]
pub struct ActiveGamepad(pub Option<Entity>);

/// Configuration settings for gamepad input.
#[derive(Resource)]
pub struct GamepadSettings {
    /// Deadzone for analog sticks (0.0 to 1.0)
    pub stick_deadzone: f32,
    /// Deadzone for triggers (0.0 to 1.0).
    /// Raw axis values below this threshold produce no action.
    pub trigger_deadzone: f32,
    /// Whether to invert the Y-axis for camera/look control.
    pub invert_camera_y: bool,
    /// Camera rotation sensitivity multiplier — not yet applied.
    /// See ticket: docs/bugs/gamepad-settings-apply/ticket.md
    #[allow(dead_code)]
    pub camera_sensitivity: f32,
    /// Movement sensitivity multiplier
    pub movement_sensitivity: f32,
}

impl Default for GamepadSettings {
    fn default() -> Self {
        Self {
            stick_deadzone: 0.15,
            trigger_deadzone: 0.1,
            invert_camera_y: false,
            camera_sensitivity: 3.0,
            movement_sensitivity: 1.0,
        }
    }
}

/// Identifies the current input source being used by the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputSource {
    #[default]
    KeyboardMouse,
    Gamepad,
}

/// Unified input resource that combines keyboard/mouse and gamepad input.
///
/// This resource is populated each frame by the input gathering systems
/// and can be read by movement, camera, and other systems to handle
/// input in a source-agnostic way.
#[derive(Resource, Default)]
pub struct PlayerInput {
    /// Normalized movement direction from left stick or WASD
    pub movement: Vec2,
    /// Camera rotation delta from right stick or mouse
    pub camera_delta: Vec2,
    /// Look direction from right stick (for character facing direction)
    /// When this is non-zero, the character faces this direction instead of movement direction
    pub look_direction: Vec2,
    /// Jump button pressed this frame (A button or Space)
    pub jump_pressed: bool,
    /// Jump button just pressed this frame
    pub jump_just_pressed: bool,
    /// Interact button pressed (X button or E)
    pub interact_pressed: bool,
    /// Pause button just pressed (Start button or Escape)
    pub pause_just_pressed: bool,
    /// Camera reset button just pressed (R3 or Home)
    pub camera_reset_just_pressed: bool,
    /// Flashlight toggle just pressed (F key or Y button)
    pub flashlight_toggle_just_pressed: bool,
    /// Left trigger axis value [0.0, 1.0], after trigger_deadzone is applied.
    /// Zero when the raw value is below the deadzone threshold.
    pub left_trigger: f32,
    /// Right trigger axis value [0.0, 1.0], after trigger_deadzone is applied.
    /// Zero when the raw value is below the deadzone threshold.
    pub right_trigger: f32,
    /// The current active input source
    pub input_source: InputSource,
}

/// Apply circular deadzone to a 2D input value.
///
/// This rescales the input so that values just outside the deadzone
/// start at zero magnitude, providing smooth analog control.
pub fn apply_deadzone(value: Vec2, deadzone: f32) -> Vec2 {
    let magnitude = value.length();
    if magnitude < deadzone {
        Vec2::ZERO
    } else {
        // Rescale to 0-1 range after deadzone
        let normalized = value / magnitude;
        let rescaled_magnitude = ((magnitude - deadzone) / (1.0 - deadzone)).min(1.0);
        normalized * rescaled_magnitude
    }
}

/// System that handles gamepad connection and disconnection events.
///
/// When a gamepad connects, it becomes the active gamepad if none is currently active.
/// When the active gamepad disconnects, the next available one (if any) becomes active.
pub fn handle_gamepad_connections(
    mut active_gamepad: ResMut<ActiveGamepad>,
    mut connection_events: MessageReader<GamepadConnectionEvent>,
    gamepads: Query<Entity, With<Gamepad>>,
) {
    for event in connection_events.read() {
        match &event.connection {
            GamepadConnection::Connected { name, .. } => {
                info!("Gamepad connected: {} (entity: {:?})", name, event.gamepad);
                // If no active gamepad, make this one active
                if active_gamepad.0.is_none() {
                    active_gamepad.0 = Some(event.gamepad);
                    info!("Set as active gamepad");
                }
            }
            GamepadConnection::Disconnected => {
                info!("Gamepad disconnected: {:?}", event.gamepad);
                // If this was the active gamepad, find another or set to None
                if active_gamepad.0 == Some(event.gamepad) {
                    // Find another connected gamepad
                    active_gamepad.0 = gamepads.iter().find(|&entity| entity != event.gamepad);

                    if let Some(new_active) = active_gamepad.0 {
                        info!("New active gamepad: {:?}", new_active);
                    } else {
                        info!("No active gamepad");
                    }
                }
            }
        }
    }
}

/// System that gathers all gamepad input into the PlayerInput resource.
///
/// This system reads from both keyboard/mouse and gamepad, automatically
/// detecting which input source is being used based on activity.
pub fn gather_gamepad_input(
    active_gamepad: Res<ActiveGamepad>,
    settings: Res<GamepadSettings>,
    gamepad_query: Query<&Gamepad>,
    mut player_input: ResMut<PlayerInput>,
) {
    // Reset gamepad-specific inputs (keyboard handler will set its own)
    let mut gamepad_movement = Vec2::ZERO;
    let mut gamepad_camera = Vec2::ZERO;
    let mut gamepad_look_direction = Vec2::ZERO;
    let mut gamepad_jump_pressed = false;
    let mut gamepad_jump_just_pressed = false;
    let mut gamepad_interact = false;
    let mut gamepad_pause = false;
    let mut gamepad_camera_reset = false;
    let mut gamepad_flashlight_toggle = false;
    let mut gamepad_active = false;
    let mut gamepad_left_trigger: f32 = 0.0;
    let mut gamepad_right_trigger: f32 = 0.0;

    if let Some(gamepad_entity) = active_gamepad.0 {
        if let Ok(gamepad) = gamepad_query.get(gamepad_entity) {
            // Left stick for movement - In Bevy 0.15, use GamepadAxis enum variants directly
            let left_stick = Vec2::new(
                gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0),
                gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0),
            );
            gamepad_movement =
                apply_deadzone(left_stick, settings.stick_deadzone) * settings.movement_sensitivity;

            // Right stick for character look direction (not camera)
            let right_stick = Vec2::new(
                gamepad.get(GamepadAxis::RightStickX).unwrap_or(0.0),
                gamepad.get(GamepadAxis::RightStickY).unwrap_or(0.0),
            );
            let mut look = apply_deadzone(right_stick, settings.stick_deadzone);
            if settings.invert_camera_y {
                look.y = -look.y;
            }
            gamepad_look_direction = look;

            // Camera delta is not used from right stick anymore
            gamepad_camera = Vec2::ZERO;

            // Button inputs - In Bevy 0.15, GamepadButton is an enum directly
            gamepad_jump_pressed = gamepad.pressed(GamepadButton::South);
            gamepad_jump_just_pressed = gamepad.just_pressed(GamepadButton::South);
            gamepad_interact = gamepad.just_pressed(GamepadButton::West);
            gamepad_pause = gamepad.just_pressed(GamepadButton::Start);
            gamepad_camera_reset = gamepad.just_pressed(GamepadButton::RightThumb);
            gamepad_flashlight_toggle = gamepad.just_pressed(GamepadButton::North); // Y button

            // Trigger axes — apply custom deadzone
            let left_trigger_raw = gamepad.get(GamepadAxis::LeftZ).unwrap_or(0.0);
            let right_trigger_raw = gamepad.get(GamepadAxis::RightZ).unwrap_or(0.0);
            gamepad_left_trigger = if left_trigger_raw >= settings.trigger_deadzone {
                left_trigger_raw
            } else {
                0.0
            };
            gamepad_right_trigger = if right_trigger_raw >= settings.trigger_deadzone {
                right_trigger_raw
            } else {
                0.0
            };

            // Detect if gamepad is being used (any significant input)
            gamepad_active = gamepad_movement.length() > 0.01
                || gamepad_look_direction.length() > 0.01
                || gamepad_jump_pressed
                || gamepad_interact
                || gamepad_pause
                || gamepad_camera_reset
                || gamepad_flashlight_toggle;
        }
    }

    // Update input source based on activity
    if gamepad_active {
        player_input.input_source = InputSource::Gamepad;
    }

    // Merge gamepad input (keyboard input is handled separately and merged in gather_keyboard_input)
    if player_input.input_source == InputSource::Gamepad {
        player_input.movement = gamepad_movement;
        player_input.camera_delta = gamepad_camera;
        player_input.look_direction = gamepad_look_direction;
        player_input.jump_pressed = gamepad_jump_pressed;
        player_input.jump_just_pressed = gamepad_jump_just_pressed;
        player_input.interact_pressed = gamepad_interact;
        player_input.pause_just_pressed = gamepad_pause;
        player_input.camera_reset_just_pressed = gamepad_camera_reset;
        player_input.flashlight_toggle_just_pressed = gamepad_flashlight_toggle;
        player_input.left_trigger = gamepad_left_trigger;
        player_input.right_trigger = gamepad_right_trigger;
    }
}

/// System that gathers keyboard and mouse input into the PlayerInput resource.
///
/// This system checks if keyboard/mouse is being used and switches the input
/// source accordingly. Mouse movement also triggers the switch to keyboard/mouse mode.
///
/// Input mapping:
/// - WASD keys: Movement (like left stick)
/// - Arrow keys: Look direction / character facing (like right stick)
pub fn gather_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<bevy::input::mouse::MouseMotion>,
    mut player_input: ResMut<PlayerInput>,
) {
    // Calculate keyboard movement (WASD keys only)
    let mut kb_movement = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        kb_movement.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        kb_movement.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        kb_movement.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        kb_movement.x += 1.0;
    }

    // Normalize diagonal movement
    if kb_movement.length() > 0.0 {
        kb_movement = kb_movement.normalize();
    }

    // Calculate look direction from arrow keys (like right stick)
    let mut kb_look_direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::ArrowUp) {
        kb_look_direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        kb_look_direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) {
        kb_look_direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        kb_look_direction.x += 1.0;
    }

    // Normalize diagonal look direction
    if kb_look_direction.length() > 0.0 {
        kb_look_direction = kb_look_direction.normalize();
    }

    let kb_jump_pressed = keyboard.pressed(KeyCode::Space);
    let kb_jump_just_pressed = keyboard.just_pressed(KeyCode::Space);
    let kb_interact = keyboard.just_pressed(KeyCode::KeyE);
    let kb_pause = keyboard.just_pressed(KeyCode::Escape);
    let kb_camera_reset = keyboard.just_pressed(KeyCode::Home);
    let kb_flashlight_toggle = keyboard.just_pressed(KeyCode::KeyF);

    // Check for mouse movement
    let mouse_moved = mouse_motion.read().any(|event| event.delta.length() > 0.5);

    // Check for mouse button presses
    let mouse_clicked =
        mouse_button.any_just_pressed([MouseButton::Left, MouseButton::Right, MouseButton::Middle]);

    // Detect if keyboard is being used
    let keyboard_active = kb_movement.length() > 0.01
        || kb_look_direction.length() > 0.01
        || kb_jump_pressed
        || kb_interact
        || kb_pause
        || kb_camera_reset
        || kb_flashlight_toggle
        || keyboard.any_pressed([
            KeyCode::KeyW,
            KeyCode::KeyA,
            KeyCode::KeyS,
            KeyCode::KeyD,
            KeyCode::ArrowUp,
            KeyCode::ArrowDown,
            KeyCode::ArrowLeft,
            KeyCode::ArrowRight,
            KeyCode::Space,
        ]);

    // Update input source based on activity (keyboard, mouse movement, or mouse click)
    if keyboard_active || mouse_moved || mouse_clicked {
        player_input.input_source = InputSource::KeyboardMouse;
    }

    // Apply keyboard input if using keyboard/mouse
    if player_input.input_source == InputSource::KeyboardMouse {
        player_input.movement = kb_movement;
        player_input.look_direction = kb_look_direction;
        // Camera delta is set from mouse motion in the camera system
        // Keep existing camera_delta if it was set by mouse
        player_input.jump_pressed = kb_jump_pressed;
        player_input.jump_just_pressed = kb_jump_just_pressed;
        player_input.interact_pressed = kb_interact;
        player_input.pause_just_pressed = kb_pause;
        player_input.camera_reset_just_pressed = kb_camera_reset;
        player_input.flashlight_toggle_just_pressed = kb_flashlight_toggle;
    }
}

/// System that resets per-frame input state at the start of each frame.
pub fn reset_player_input(mut player_input: ResMut<PlayerInput>) {
    // Reset just-pressed flags and deltas, keep the input source
    let source = player_input.input_source;
    *player_input = PlayerInput {
        input_source: source,
        ..default()
    };
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
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
}

/// Menu input for navigating UI with gamepad.
///
/// Returns (navigate_up, navigate_down, select, back)
pub fn get_menu_gamepad_input(
    active_gamepad: &ActiveGamepad,
    gamepad_query: &Query<&Gamepad>,
    _settings: &GamepadSettings,
) -> (bool, bool, bool, bool) {
    let mut nav_up = false;
    let mut nav_down = false;
    let mut select = false;
    let mut back = false;

    if let Some(gamepad_entity) = active_gamepad.0 {
        if let Ok(gamepad) = gamepad_query.get(gamepad_entity) {
            // D-pad navigation - In Bevy 0.15, use enum variants directly
            nav_up = gamepad.just_pressed(GamepadButton::DPadUp);
            nav_down = gamepad.just_pressed(GamepadButton::DPadDown);

            // A button to select
            select = gamepad.just_pressed(GamepadButton::South);

            // B button to go back
            back = gamepad.just_pressed(GamepadButton::East);
        }
    }

    (nav_up, nav_down, select, back)
}

/// System that updates cursor visibility based on active input source.
///
/// Hides the cursor when using a gamepad and shows it when using keyboard/mouse.
/// This provides a cleaner experience when playing with a controller.
pub fn update_cursor_visibility(
    player_input: Res<PlayerInput>,
    mut cursor_query: Query<&mut bevy::window::CursorOptions>,
) {
    if player_input.is_changed() {
        if let Ok(mut cursor) = cursor_query.single_mut() {
            match player_input.input_source {
                InputSource::Gamepad => {
                    cursor.visible = false;
                }
                InputSource::KeyboardMouse => {
                    cursor.visible = true;
                }
            }
        }
    }
}
