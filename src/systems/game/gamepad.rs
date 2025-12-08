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
    /// Deadzone for triggers (0.0 to 1.0)
    pub trigger_deadzone: f32,
    /// Whether to invert the Y-axis for camera control
    pub invert_camera_y: bool,
    /// Camera rotation sensitivity multiplier
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
    mut connection_events: EventReader<GamepadConnectionEvent>,
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
    let mut gamepad_jump_pressed = false;
    let mut gamepad_jump_just_pressed = false;
    let mut gamepad_interact = false;
    let mut gamepad_pause = false;
    let mut gamepad_camera_reset = false;
    let mut gamepad_active = false;

    if let Some(gamepad_entity) = active_gamepad.0 {
        if let Ok(gamepad) = gamepad_query.get(gamepad_entity) {
            // Left stick for movement - In Bevy 0.15, use GamepadAxis enum variants directly
            let left_stick = Vec2::new(
                gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0),
                gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0),
            );
            gamepad_movement =
                apply_deadzone(left_stick, settings.stick_deadzone) * settings.movement_sensitivity;

            // Right stick for camera
            let right_stick = Vec2::new(
                gamepad.get(GamepadAxis::RightStickX).unwrap_or(0.0),
                gamepad.get(GamepadAxis::RightStickY).unwrap_or(0.0),
            );
            gamepad_camera =
                apply_deadzone(right_stick, settings.stick_deadzone) * settings.camera_sensitivity;

            // Invert Y if configured
            if settings.invert_camera_y {
                gamepad_camera.y = -gamepad_camera.y;
            }

            // Button inputs - In Bevy 0.15, GamepadButton is an enum directly
            gamepad_jump_pressed = gamepad.pressed(GamepadButton::South);
            gamepad_jump_just_pressed = gamepad.just_pressed(GamepadButton::South);
            gamepad_interact = gamepad.just_pressed(GamepadButton::West);
            gamepad_pause = gamepad.just_pressed(GamepadButton::Start);
            gamepad_camera_reset = gamepad.just_pressed(GamepadButton::RightThumb);

            // Detect if gamepad is being used (any significant input)
            gamepad_active = gamepad_movement.length() > 0.01
                || gamepad_camera.length() > 0.01
                || gamepad_jump_pressed
                || gamepad_interact
                || gamepad_pause
                || gamepad_camera_reset;
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
        player_input.jump_pressed = gamepad_jump_pressed;
        player_input.jump_just_pressed = gamepad_jump_just_pressed;
        player_input.interact_pressed = gamepad_interact;
        player_input.pause_just_pressed = gamepad_pause;
        player_input.camera_reset_just_pressed = gamepad_camera_reset;
    }
}

/// System that gathers keyboard and mouse input into the PlayerInput resource.
///
/// This system checks if keyboard/mouse is being used and switches the input
/// source accordingly.
pub fn gather_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_input: ResMut<PlayerInput>,
) {
    // Calculate keyboard movement
    let mut kb_movement = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW)
        || keyboard.pressed(KeyCode::ArrowUp)
        || keyboard.pressed(KeyCode::PageUp)
    {
        kb_movement.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS)
        || keyboard.pressed(KeyCode::ArrowDown)
        || keyboard.pressed(KeyCode::PageDown)
    {
        kb_movement.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA)
        || keyboard.pressed(KeyCode::ArrowLeft)
        || keyboard.pressed(KeyCode::Home)
    {
        kb_movement.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD)
        || keyboard.pressed(KeyCode::ArrowRight)
        || keyboard.pressed(KeyCode::End)
    {
        kb_movement.x += 1.0;
    }

    // Normalize diagonal movement
    if kb_movement.length() > 0.0 {
        kb_movement = kb_movement.normalize();
    }

    let kb_jump_pressed = keyboard.pressed(KeyCode::Space);
    let kb_jump_just_pressed = keyboard.just_pressed(KeyCode::Space);
    let kb_interact = keyboard.just_pressed(KeyCode::KeyE);
    let kb_pause = keyboard.just_pressed(KeyCode::Escape);
    let kb_camera_reset = keyboard.just_pressed(KeyCode::Home);

    // Detect if keyboard is being used
    let keyboard_active = kb_movement.length() > 0.01
        || kb_jump_pressed
        || kb_interact
        || kb_pause
        || kb_camera_reset
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

    // Update input source based on activity
    if keyboard_active {
        player_input.input_source = InputSource::KeyboardMouse;
    }

    // Apply keyboard input if using keyboard/mouse
    if player_input.input_source == InputSource::KeyboardMouse {
        player_input.movement = kb_movement;
        // Camera delta is set from mouse motion in the camera system
        // Keep existing camera_delta if it was set by mouse
        player_input.jump_pressed = kb_jump_pressed;
        player_input.jump_just_pressed = kb_jump_just_pressed;
        player_input.interact_pressed = kb_interact;
        player_input.pause_just_pressed = kb_pause;
        player_input.camera_reset_just_pressed = kb_camera_reset;
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
