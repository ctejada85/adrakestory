//! Editor camera system with first-person fly controls (Minecraft Creative mode style).

use bevy::prelude::*;
use bevy_egui::EguiContexts;

/// Resource to track camera control state
#[derive(Resource, Default)]
pub struct GamepadCameraState {
    /// Whether gamepad is currently controlling the camera
    pub active: bool,
    /// Distance in front of camera for the cursor (fallback when no raycast hit)
    pub cursor_distance: f32,
    /// Position in world space where actions will be performed
    pub action_position: Option<Vec3>,
    /// Grid position for placement actions
    pub action_grid_pos: Option<(i32, i32, i32)>,
    /// Grid position of the voxel being looked at (for removal)
    pub target_voxel_pos: Option<(i32, i32, i32)>,
}

impl GamepadCameraState {
    pub fn new() -> Self {
        Self {
            active: false,
            cursor_distance: 5.0,
            action_position: None,
            action_grid_pos: None,
            target_voxel_pos: None,
        }
    }
}

/// Component for the editor camera - first-person fly camera
#[derive(Component)]
pub struct EditorCamera {
    /// Camera position in world space
    pub position: Vec3,

    /// Camera rotation (yaw, pitch) in radians
    pub yaw: f32,
    pub pitch: f32,

    /// Movement speed (units per second)
    pub move_speed: f32,

    /// Mouse look sensitivity
    pub look_sensitivity: f32,
}

impl Default for EditorCamera {
    fn default() -> Self {
        Self {
            position: Vec3::new(5.0, 5.0, 10.0),
            yaw: 0.0,
            pitch: -0.3, // Slightly looking down
            move_speed: 15.0,
            look_sensitivity: 0.003,
        }
    }
}

impl EditorCamera {
    /// Create a new editor camera with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new editor camera at a specific position looking at a target
    pub fn looking_at(position: Vec3, target: Vec3) -> Self {
        let direction = (target - position).normalize();
        let yaw = direction.x.atan2(direction.z);
        let pitch = (-direction.y).asin().clamp(-1.5, 1.5);
        
        Self {
            position,
            yaw,
            pitch,
            ..Default::default()
        }
    }

    /// Get the camera's current position
    pub fn calculate_position(&self) -> Vec3 {
        self.position
    }

    /// Get the forward direction vector (where camera is looking)
    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            -self.yaw.sin() * self.pitch.cos(),
            -self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        ).normalize()
    }

    /// Get the right direction vector
    pub fn right(&self) -> Vec3 {
        Vec3::new(self.yaw.cos(), 0.0, -self.yaw.sin())
    }

    /// Get the forward direction for movement (ignores pitch, stays horizontal)
    pub fn forward_horizontal(&self) -> Vec3 {
        Vec3::new(-self.yaw.sin(), 0.0, -self.yaw.cos())
    }

    /// Apply mouse look rotation
    pub fn apply_look(&mut self, delta: Vec2) {
        self.yaw -= delta.x * self.look_sensitivity;
        self.pitch += delta.y * self.look_sensitivity;
        
        // Clamp pitch to avoid flipping
        self.pitch = self.pitch.clamp(-1.5, 1.5);
    }

    /// Move the camera by a delta vector
    pub fn move_by(&mut self, delta: Vec3) {
        self.position += delta;
    }

    /// Reset camera to default position
    pub fn reset(&mut self) {
        let default = Self::default();
        self.position = default.position;
        self.yaw = default.yaw;
        self.pitch = default.pitch;
    }

    /// Set camera to look at a specific point from a specific position
    pub fn set_view(&mut self, position: Vec3, target: Vec3) {
        self.position = position;
        let direction = (target - position).normalize();
        self.yaw = direction.x.atan2(direction.z);
        self.pitch = (-direction.y).asin().clamp(-1.5, 1.5);
    }
    
    // Legacy compatibility methods (used by some existing code)
    
    /// Get rotation as Vec2 (yaw, pitch) - for compatibility
    pub fn rotation(&self) -> Vec2 {
        Vec2::new(self.yaw, self.pitch)
    }
}

/// System to update camera transform from EditorCamera state
pub fn update_editor_camera(
    mut query: Query<(&EditorCamera, &mut Transform), With<Camera3d>>,
) {
    for (editor_cam, mut transform) in query.iter_mut() {
        transform.translation = editor_cam.position;
        
        // Calculate look direction from yaw and pitch
        let forward = editor_cam.forward();
        let target = editor_cam.position + forward;
        transform.look_at(target, Vec3::Y);
    }
}

/// Resource to track camera input state
#[derive(Resource, Default)]
pub struct CameraInputState {
    /// Last mouse position for delta calculation
    pub last_mouse_pos: Option<Vec2>,
}

/// System to handle camera input (WASD movement, mouse look, gamepad)
pub fn handle_camera_input(
    mut camera_query: Query<&mut EditorCamera>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut gamepad_state: ResMut<GamepadCameraState>,
    editor_state: Res<crate::editor::state::EditorState>,
    mut contexts: EguiContexts,
    mut windows: Query<&mut Window>,
    time: Res<Time>,
) {
    let Ok(mut camera) = camera_query.get_single_mut() else {
        return;
    };

    let dt = time.delta_secs();

    // Check if any gamepad has significant input
    let mut gamepad_has_input = false;
    for gamepad in gamepads.iter() {
        let left_x = gamepad.get(bevy::input::gamepad::GamepadAxis::LeftStickX).unwrap_or(0.0);
        let left_y = gamepad.get(bevy::input::gamepad::GamepadAxis::LeftStickY).unwrap_or(0.0);
        let right_x = gamepad.get(bevy::input::gamepad::GamepadAxis::RightStickX).unwrap_or(0.0);
        let right_y = gamepad.get(bevy::input::gamepad::GamepadAxis::RightStickY).unwrap_or(0.0);
        
        if left_x.abs() > 0.15 || left_y.abs() > 0.15 || right_x.abs() > 0.15 || right_y.abs() > 0.15 {
            gamepad_has_input = true;
            break;
        }
    }

    // Switch to gamepad mode if gamepad input detected
    if gamepad_has_input && !gamepad_state.active {
        gamepad_state.active = true;
        if let Ok(mut window) = windows.get_single_mut() {
            window.cursor_options.visible = false;
        }
    }

    // Switch back to mouse/keyboard mode if mouse input detected
    let mouse_moved = mouse_motion.len() > 0;
    let mouse_clicked = mouse_button.any_just_pressed([MouseButton::Left, MouseButton::Right, MouseButton::Middle]);
    if (mouse_moved || mouse_clicked) && gamepad_state.active {
        gamepad_state.active = false;
        if let Ok(mut window) = windows.get_single_mut() {
            window.cursor_options.visible = true;
        }
    }

    // Update action position using raycasting (used for both gamepad and keyboard/mouse)
    {
        let camera_pos = camera.position;
        let forward = camera.forward();
        
        let ray = Ray3d {
            origin: camera_pos,
            direction: Dir3::new(forward).unwrap_or(Dir3::NEG_Z),
        };
        
        // Raycast against voxels to find what we're looking at
        if let Some((voxel_pos, hit_info)) = 
            crate::editor::cursor::raycasting::find_closest_voxel_intersection_with_face(&editor_state, &ray) 
        {
            let placement_pos = (
                voxel_pos.0 + hit_info.face_normal.x as i32,
                voxel_pos.1 + hit_info.face_normal.y as i32,
                voxel_pos.2 + hit_info.face_normal.z as i32,
            );
            
            gamepad_state.action_position = Some(Vec3::new(
                placement_pos.0 as f32,
                placement_pos.1 as f32,
                placement_pos.2 as f32,
            ));
            gamepad_state.action_grid_pos = Some(placement_pos);
            gamepad_state.target_voxel_pos = Some(voxel_pos);
        } else if let Some(ground_pos) = crate::editor::cursor::raycasting::intersect_ground_plane(&ray) {
            let grid_pos = (
                ground_pos.x.round() as i32,
                0,
                ground_pos.z.round() as i32,
            );
            gamepad_state.action_position = Some(Vec3::new(
                grid_pos.0 as f32,
                0.0,
                grid_pos.2 as f32,
            ));
            gamepad_state.action_grid_pos = Some(grid_pos);
            gamepad_state.target_voxel_pos = None;
        } else {
            let action_pos = camera_pos + forward * gamepad_state.cursor_distance;
            gamepad_state.action_position = Some(action_pos);
            gamepad_state.action_grid_pos = Some((
                action_pos.x.round() as i32,
                action_pos.y.round() as i32,
                action_pos.z.round() as i32,
            ));
            gamepad_state.target_voxel_pos = None;
        }
    }

    // Check if pointer is over UI
    let ctx = contexts.ctx_mut();
    let pointer_over_ui = ctx.is_pointer_over_area() || ctx.is_using_pointer();
    let wants_keyboard = ctx.wants_keyboard_input();

    // === WASD Movement (keyboard) ===
    if !wants_keyboard {
        let mut movement = Vec3::ZERO;
        
        // Forward/backward
        if keyboard.pressed(KeyCode::KeyW) {
            movement += camera.forward_horizontal();
        }
        if keyboard.pressed(KeyCode::KeyS) {
            movement -= camera.forward_horizontal();
        }
        
        // Strafe left/right
        if keyboard.pressed(KeyCode::KeyA) {
            movement -= camera.right();
        }
        if keyboard.pressed(KeyCode::KeyD) {
            movement += camera.right();
        }
        
        // Up/down (Space/Ctrl)
        if keyboard.pressed(KeyCode::Space) {
            movement.y += 1.0;
        }
        if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
            movement.y -= 1.0;
        }
        
        // Apply movement
        if movement.length_squared() > 0.0 {
            movement = movement.normalize() * camera.move_speed * dt;
            camera.move_by(movement);
        }
    }

    // === Mouse Look ===
    if !pointer_over_ui {
        // Mouse look when right-click is held (similar to many 3D editors)
        if mouse_button.pressed(MouseButton::Right) {
            for event in mouse_motion.read() {
                camera.apply_look(event.delta);
            }
        } else {
            // Consume events to prevent accumulation
            mouse_motion.clear();
        }
    } else {
        mouse_motion.clear();
    }

    // === Keyboard shortcuts ===
    if keyboard.just_pressed(KeyCode::Home) {
        camera.reset();
    }

    // === Q/E Pattern Cycling ===
    // Handled in a separate system

    // === Gamepad input for Minecraft Creative-style camera ===
    let deadzone = 0.15;
    let look_speed = 3.0;
    let move_speed = 20.0;

    for gamepad in gamepads.iter() {
        // Right stick for looking
        let right_x = gamepad.get(bevy::input::gamepad::GamepadAxis::RightStickX).unwrap_or(0.0);
        let right_y = gamepad.get(bevy::input::gamepad::GamepadAxis::RightStickY).unwrap_or(0.0);
        let right_stick = Vec2::new(right_x, right_y);
        
        if right_stick.length() > deadzone {
            let scaled = (right_stick.length() - deadzone) / (1.0 - deadzone);
            let look_input = right_stick.normalize() * scaled;
            
            camera.yaw -= look_input.x * look_speed * dt;
            camera.pitch -= look_input.y * look_speed * dt;
            camera.pitch = camera.pitch.clamp(-1.5, 1.5);
        }

        // Left stick for flying movement
        let left_x = gamepad.get(bevy::input::gamepad::GamepadAxis::LeftStickX).unwrap_or(0.0);
        let left_y = gamepad.get(bevy::input::gamepad::GamepadAxis::LeftStickY).unwrap_or(0.0);
        let left_stick = Vec2::new(left_x, left_y);
        
        if left_stick.length() > deadzone {
            let scaled = (left_stick.length() - deadzone) / (1.0 - deadzone);
            let move_input = left_stick.normalize() * scaled;
            
            let forward = camera.forward_horizontal();
            let right = camera.right();
            
            let movement = (forward * move_input.y + right * move_input.x) * move_speed * dt;
            camera.move_by(movement);
        }

        // A button = fly up, B button = fly down
        if gamepad.pressed(bevy::input::gamepad::GamepadButton::South) {
            camera.position.y += move_speed * dt;
        }
        if gamepad.pressed(bevy::input::gamepad::GamepadButton::East) {
            camera.position.y -= move_speed * dt;
        }

        // D-pad for fine movement
        if gamepad.pressed(bevy::input::gamepad::GamepadButton::DPadUp) {
            let forward = camera.forward_horizontal();
            camera.move_by(forward * move_speed * dt * 0.5);
        }
        if gamepad.pressed(bevy::input::gamepad::GamepadButton::DPadDown) {
            let forward = camera.forward_horizontal();
            camera.move_by(-forward * move_speed * dt * 0.5);
        }
        if gamepad.pressed(bevy::input::gamepad::GamepadButton::DPadLeft) {
            let right = camera.right();
            camera.move_by(-right * move_speed * dt * 0.5);
        }
        if gamepad.pressed(bevy::input::gamepad::GamepadButton::DPadRight) {
            let right = camera.right();
            camera.move_by(right * move_speed * dt * 0.5);
        }

        // Y button to reset camera
        if gamepad.just_pressed(bevy::input::gamepad::GamepadButton::North) {
            camera.reset();
        }
    }
}

/// System to handle gamepad voxel actions (RT to execute tool action, LT to remove)
/// Note: Mouse actions are handled by the tool systems in tools/ module
pub fn handle_gamepad_voxel_actions(
    gamepad_state: Res<GamepadCameraState>,
    gamepads: Query<&Gamepad>,
    mut editor_state: ResMut<crate::editor::state::EditorState>,
    mut history: ResMut<crate::editor::history::EditorHistory>,
    mut contexts: EguiContexts,
    time: Res<Time>,
    mut cooldown: Local<f32>,
) {
    // Update cooldown
    *cooldown = (*cooldown - time.delta_secs()).max(0.0);
    if *cooldown > 0.0 {
        return;
    }

    let Some(grid_pos) = gamepad_state.action_grid_pos else {
        return;
    };

    // Check if UI is being interacted with
    let ctx = contexts.ctx_mut();
    let _pointer_over_ui = ctx.is_pointer_over_area() || ctx.is_using_pointer();

    // Determine if primary or secondary action is triggered
    let mut primary_action = false;
    let mut secondary_action = false;

    // Gamepad triggers only - mouse is handled by tool systems
    for gamepad in gamepads.iter() {
        let rt_axis = gamepad.get(bevy::input::gamepad::GamepadAxis::RightZ).unwrap_or(0.0);
        let lt_axis = gamepad.get(bevy::input::gamepad::GamepadAxis::LeftZ).unwrap_or(0.0);
        let rt_button = gamepad.pressed(bevy::input::gamepad::GamepadButton::RightTrigger2);
        let lt_button = gamepad.pressed(bevy::input::gamepad::GamepadButton::LeftTrigger2);
        
        if rt_axis > 0.5 || rt_button {
            primary_action = true;
        }
        if lt_axis > 0.5 || lt_button {
            secondary_action = true;
        }
    }

    // Primary action = execute main action of current tool
    if primary_action {
        match editor_state.active_tool.clone() {
            crate::editor::state::EditorTool::VoxelPlace { voxel_type, pattern } => {
                let exists = editor_state
                    .current_map
                    .world
                    .voxels
                    .iter()
                    .any(|v| v.pos == grid_pos);

                if !exists {
                    let voxel_data = crate::systems::game::map::format::VoxelData {
                        pos: grid_pos,
                        voxel_type,
                        pattern: Some(pattern),
                        rotation_state: None,
                    };

                    editor_state.current_map.world.voxels.push(voxel_data.clone());
                    editor_state.mark_modified();

                    history.push(crate::editor::history::EditorAction::PlaceVoxel {
                        pos: grid_pos,
                        data: voxel_data,
                    });

                    *cooldown = 0.15;
                }
            }
            crate::editor::state::EditorTool::VoxelRemove => {
                let remove_pos = gamepad_state.target_voxel_pos.unwrap_or(grid_pos);
                
                if let Some(idx) = editor_state
                    .current_map
                    .world
                    .voxels
                    .iter()
                    .position(|v| v.pos == remove_pos)
                {
                    let removed = editor_state.current_map.world.voxels.remove(idx);
                    editor_state.mark_modified();

                    history.push(crate::editor::history::EditorAction::RemoveVoxel {
                        pos: remove_pos,
                        data: removed,
                    });

                    *cooldown = 0.15;
                }
            }
            crate::editor::state::EditorTool::EntityPlace { entity_type } => {
                use crate::systems::game::map::format::EntityData;
                
                let entity_data = EntityData {
                    entity_type,
                    position: (grid_pos.0 as f32 + 0.5, grid_pos.1 as f32, grid_pos.2 as f32 + 0.5),
                    properties: std::collections::HashMap::new(),
                };

                editor_state.current_map.entities.push(entity_data.clone());
                editor_state.mark_modified();

                history.push(crate::editor::history::EditorAction::PlaceEntity {
                    index: editor_state.current_map.entities.len() - 1,
                    data: entity_data,
                });

                *cooldown = 0.15;
            }
            crate::editor::state::EditorTool::Select => {
                if let Some(target_pos) = gamepad_state.target_voxel_pos {
                    if editor_state.selected_voxels.contains(&target_pos) {
                        editor_state.selected_voxels.remove(&target_pos);
                    } else {
                        editor_state.selected_voxels.insert(target_pos);
                    }
                    *cooldown = 0.2;
                }
            }
            crate::editor::state::EditorTool::Camera => {
                // Camera tool has no action
            }
        }
    }

    // Secondary action = always remove voxel
    if secondary_action {
        let remove_pos = gamepad_state.target_voxel_pos.unwrap_or(grid_pos);
        
        if let Some(idx) = editor_state
            .current_map
            .world
            .voxels
            .iter()
            .position(|v| v.pos == remove_pos)
        {
            let removed = editor_state.current_map.world.voxels.remove(idx);
            editor_state.mark_modified();

            history.push(crate::editor::history::EditorAction::RemoveVoxel {
                pos: remove_pos,
                data: removed,
            });

            *cooldown = 0.15;
        }
    }
}

/// System to handle RB/LB and Q/E for cycling through patterns/entities
pub fn handle_gamepad_tool_cycling(
    gamepads: Query<&Gamepad>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut editor_state: ResMut<crate::editor::state::EditorState>,
    mut contexts: EguiContexts,
    time: Res<Time>,
    mut cooldown: Local<f32>,
) {
    // Update cooldown
    *cooldown = (*cooldown - time.delta_secs()).max(0.0);
    if *cooldown > 0.0 {
        return;
    }

    // Check if UI wants keyboard input
    let ctx = contexts.ctx_mut();
    let wants_keyboard = ctx.wants_keyboard_input();

    let mut next_pressed = false;
    let mut prev_pressed = false;

    // Gamepad bumpers
    for gamepad in gamepads.iter() {
        if gamepad.just_pressed(bevy::input::gamepad::GamepadButton::RightTrigger) {
            next_pressed = true;
        }
        if gamepad.just_pressed(bevy::input::gamepad::GamepadButton::LeftTrigger) {
            prev_pressed = true;
        }
    }

    // Q/E keys (only when UI doesn't want keyboard)
    if !wants_keyboard {
        if keyboard.just_pressed(KeyCode::KeyE) {
            next_pressed = true;
        }
        if keyboard.just_pressed(KeyCode::KeyQ) {
            prev_pressed = true;
        }
    }

    if !next_pressed && !prev_pressed {
        return;
    }

    match &mut editor_state.active_tool {
        crate::editor::state::EditorTool::VoxelPlace { pattern, .. } => {
            use crate::systems::game::map::format::SubVoxelPattern;
            
            const PATTERNS: [SubVoxelPattern; 10] = [
                SubVoxelPattern::Full,
                SubVoxelPattern::PlatformXZ,
                SubVoxelPattern::PlatformXY,
                SubVoxelPattern::PlatformYZ,
                SubVoxelPattern::StaircaseX,
                SubVoxelPattern::StaircaseNegX,
                SubVoxelPattern::StaircaseZ,
                SubVoxelPattern::StaircaseNegZ,
                SubVoxelPattern::Pillar,
                SubVoxelPattern::Fence,
            ];

            let current_idx = PATTERNS.iter().position(|p| p == pattern).unwrap_or(0);
            let new_idx = if next_pressed {
                (current_idx + 1) % PATTERNS.len()
            } else {
                (current_idx + PATTERNS.len() - 1) % PATTERNS.len()
            };
            *pattern = PATTERNS[new_idx];
            *cooldown = 0.2;
        }
        crate::editor::state::EditorTool::EntityPlace { entity_type } => {
            use crate::systems::game::map::format::EntityType;
            
            const ENTITIES: [EntityType; 6] = [
                EntityType::PlayerSpawn,
                EntityType::Npc,
                EntityType::Enemy,
                EntityType::Item,
                EntityType::Trigger,
                EntityType::LightSource,
            ];

            let current_idx = ENTITIES.iter().position(|e| e == entity_type).unwrap_or(0);
            let new_idx = if next_pressed {
                (current_idx + 1) % ENTITIES.len()
            } else {
                (current_idx + ENTITIES.len() - 1) % ENTITIES.len()
            };
            *entity_type = ENTITIES[new_idx];
            *cooldown = 0.2;
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
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

        const PATTERNS: [SubVoxelPattern; 10] = [
            SubVoxelPattern::Full,
            SubVoxelPattern::PlatformXZ,
            SubVoxelPattern::PlatformXY,
            SubVoxelPattern::PlatformYZ,
            SubVoxelPattern::StaircaseX,
            SubVoxelPattern::StaircaseNegX,
            SubVoxelPattern::StaircaseZ,
            SubVoxelPattern::StaircaseNegZ,
            SubVoxelPattern::Pillar,
            SubVoxelPattern::Fence,
        ];

        // Test forward cycling wraps correctly
        let current = 9;
        let next = (current + 1) % PATTERNS.len();
        assert_eq!(next, 0);
        assert_eq!(PATTERNS[next], SubVoxelPattern::Full);

        // Test backward cycling wraps correctly
        let current = 0;
        let prev = (current + PATTERNS.len() - 1) % PATTERNS.len();
        assert_eq!(prev, 9);
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

        const PATTERNS: [SubVoxelPattern; 10] = [
            SubVoxelPattern::Full,
            SubVoxelPattern::PlatformXZ,
            SubVoxelPattern::PlatformXY,
            SubVoxelPattern::PlatformYZ,
            SubVoxelPattern::StaircaseX,
            SubVoxelPattern::StaircaseNegX,
            SubVoxelPattern::StaircaseZ,
            SubVoxelPattern::StaircaseNegZ,
            SubVoxelPattern::Pillar,
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
}
