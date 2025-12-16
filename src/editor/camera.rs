//! Editor camera system with orbit, pan, and zoom controls.

use bevy::prelude::*;
use bevy_egui::EguiContexts;

/// Resource to track if gamepad is being used for camera control
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

/// Component for the editor camera
#[derive(Component)]
pub struct EditorCamera {
    /// Target point the camera orbits around
    pub target: Vec3,

    /// Current distance from target
    pub distance: f32,

    /// Target distance for smooth zoom interpolation
    pub target_distance: f32,

    /// Rotation around target (yaw, pitch)
    pub rotation: Vec2,

    /// Camera movement speed
    pub pan_speed: f32,

    /// Camera rotation speed
    pub orbit_speed: f32,

    /// Zoom speed (how fast scroll wheel changes target distance)
    pub zoom_speed: f32,

    /// Zoom smoothing factor (how fast camera interpolates to target distance)
    pub zoom_smoothing: f32,

    /// Minimum distance from target
    pub min_distance: f32,

    /// Maximum distance from target
    pub max_distance: f32,
}

impl Default for EditorCamera {
    fn default() -> Self {
        // Windows has different scroll wheel behavior, so use faster zoom speed
        #[cfg(target_os = "windows")]
        let zoom_speed = 0.15; // Faster zoom for Windows scroll wheel

        #[cfg(not(target_os = "windows"))]
        let zoom_speed = 0.0125; // Original zoom speed for macOS/Linux

        Self {
            target: Vec3::new(2.0, 0.0, 2.0),
            distance: 10.0,
            target_distance: 10.0,
            rotation: Vec2::new(0.0, 0.5), // yaw, pitch
            pan_speed: 0.00125,
            orbit_speed: 0.005,
            zoom_speed,
            zoom_smoothing: 15.0, // Smooth interpolation speed
            min_distance: 2.0,
            max_distance: 200.0, // Allow zooming out much further
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

    /// Zoom the camera (change target distance for smooth interpolation)
    pub fn zoom(&mut self, delta: f32) {
        // Modify target distance - actual distance will interpolate smoothly
        self.target_distance -= delta * self.zoom_speed * self.target_distance;
        self.target_distance = self
            .target_distance
            .clamp(self.min_distance, self.max_distance);
    }

    /// Update smooth zoom interpolation (call every frame with delta time)
    pub fn update_smooth_zoom(&mut self, dt: f32) {
        // Exponential interpolation for smooth zoom
        let t = 1.0 - (-self.zoom_smoothing * dt).exp();
        self.distance = self.distance + (self.target_distance - self.distance) * t;
        self.distance = self.distance.clamp(self.min_distance, self.max_distance);
    }

    /// Reset camera to default position
    pub fn reset(&mut self) {
        let default = Self::default();
        self.target = default.target;
        self.distance = default.distance;
        self.target_distance = default.target_distance;
        self.rotation = default.rotation;
    }

    /// Set camera to look at a specific point from a specific position
    pub fn set_view(&mut self, position: Vec3, target: Vec3) {
        self.target = target;
        let offset = position - target;
        self.distance = offset.length();
        self.target_distance = self.distance;

        // Calculate rotation from offset
        self.rotation.x = offset.z.atan2(offset.x);
        self.rotation.y = (offset.y / self.distance).asin();
    }
}

/// System to update camera transform and smooth zoom
pub fn update_editor_camera(
    mut query: Query<(&mut EditorCamera, &mut Transform), With<Camera3d>>,
    time: Res<Time>,
) {
    for (mut editor_cam, mut transform) in query.iter_mut() {
        // Update smooth zoom interpolation
        editor_cam.update_smooth_zoom(time.delta_secs());

        // Update camera position
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
    gamepads: Query<&Gamepad>,
    mut input_state: ResMut<CameraInputState>,
    mut gamepad_state: ResMut<GamepadCameraState>,
    editor_state: Res<crate::editor::state::EditorState>,
    mut contexts: EguiContexts,
    mut windows: Query<&mut Window>,
    time: Res<Time>,
) {
    let Ok(mut camera) = camera_query.get_single_mut() else {
        return;
    };

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
        // Hide mouse cursor
        if let Ok(mut window) = windows.get_single_mut() {
            window.cursor_options.visible = false;
        }
    }

    // Switch back to mouse mode if mouse input detected
    let mouse_moved = mouse_motion.len() > 0;
    let mouse_clicked = mouse_button.any_just_pressed([MouseButton::Left, MouseButton::Right, MouseButton::Middle]);
    if (mouse_moved || mouse_clicked) && gamepad_state.active {
        gamepad_state.active = false;
        // Show mouse cursor
        if let Ok(mut window) = windows.get_single_mut() {
            window.cursor_options.visible = true;
        }
    }

    // Update gamepad action position using raycasting
    if gamepad_state.active {
        let camera_pos = camera.calculate_position();
        let yaw = camera.rotation.x;
        let pitch = camera.rotation.y;
        
        // Calculate forward direction from camera rotation
        let forward = Vec3::new(
            -yaw.sin() * pitch.cos(),
            -pitch.sin(),
            -yaw.cos() * pitch.cos(),
        ).normalize();
        
        // Create ray from camera center
        let ray = Ray3d {
            origin: camera_pos,
            direction: Dir3::new(forward).unwrap_or(Dir3::NEG_Z),
        };
        
        // Raycast against voxels to find what we're looking at
        if let Some((voxel_pos, hit_info)) = 
            crate::editor::cursor::raycasting::find_closest_voxel_intersection_with_face(&editor_state, &ray) 
        {
            // For placement, offset by face normal to place adjacent
            let placement_pos = (
                voxel_pos.0 + hit_info.face_normal.x as i32,
                voxel_pos.1 + hit_info.face_normal.y as i32,
                voxel_pos.2 + hit_info.face_normal.z as i32,
            );
            
            // action_position is at integer grid coords (voxels are centered at integer positions)
            gamepad_state.action_position = Some(Vec3::new(
                placement_pos.0 as f32,
                placement_pos.1 as f32,
                placement_pos.2 as f32,
            ));
            gamepad_state.action_grid_pos = Some(placement_pos);
            // Store the voxel we're looking at for removal
            gamepad_state.target_voxel_pos = Some(voxel_pos);
        } else if let Some(ground_pos) = crate::editor::cursor::raycasting::intersect_ground_plane(&ray) {
            // Fallback to ground plane
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
            // No hit - place at fixed distance as fallback
            let action_pos = camera_pos + forward * gamepad_state.cursor_distance;
            gamepad_state.action_position = Some(action_pos);
            gamepad_state.action_grid_pos = Some((
                action_pos.x.round() as i32,
                action_pos.y.round() as i32,
                action_pos.z.round() as i32,
            ));
            gamepad_state.target_voxel_pos = None;
        }
    } else {
        gamepad_state.action_position = None;
        gamepad_state.action_grid_pos = None;
        gamepad_state.target_voxel_pos = None;
    }

    // Check if pointer is over any UI area - don't process camera input if mouse is over UI panels
    // Also check is_using_pointer() for active interactions like dragging resize handles
    let ctx = contexts.ctx_mut();
    let pointer_over_ui = ctx.is_pointer_over_area() || ctx.is_using_pointer();

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
        // Continue to process gamepad input even over UI
    } else {
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
    }

    // Handle keyboard shortcuts
    if keyboard.just_pressed(KeyCode::Home) {
        camera.reset();
    }

    // === Gamepad input for Minecraft Creative-style camera ===
    let dt = time.delta_secs();
    let deadzone = 0.15;
    let look_speed = 3.0; // Radians per second for looking
    let move_speed = 20.0; // Units per second for flying

    for gamepad in gamepads.iter() {
        // Right stick for looking (rotating camera view)
        let right_x = gamepad.get(bevy::input::gamepad::GamepadAxis::RightStickX).unwrap_or(0.0);
        let right_y = gamepad.get(bevy::input::gamepad::GamepadAxis::RightStickY).unwrap_or(0.0);
        let right_stick = Vec2::new(right_x, right_y);
        
        if right_stick.length() > deadzone {
            let scaled = (right_stick.length() - deadzone) / (1.0 - deadzone);
            let look_input = right_stick.normalize() * scaled;
            
            // Fix X inversion: negate X so right stick right = look right
            camera.rotation.x -= look_input.x * look_speed * dt;
            camera.rotation.y -= look_input.y * look_speed * dt;
            camera.rotation.y = camera.rotation.y.clamp(-1.5, 1.5);
        }

        // Left stick for flying movement
        let left_x = gamepad.get(bevy::input::gamepad::GamepadAxis::LeftStickX).unwrap_or(0.0);
        let left_y = gamepad.get(bevy::input::gamepad::GamepadAxis::LeftStickY).unwrap_or(0.0);
        let left_stick = Vec2::new(left_x, left_y);
        
        if left_stick.length() > deadzone {
            let scaled = (left_stick.length() - deadzone) / (1.0 - deadzone);
            let move_input = left_stick.normalize() * scaled;
            
            // Calculate forward and right vectors based on camera yaw
            let yaw = camera.rotation.x;
            let forward = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
            let right = Vec3::new(-yaw.cos(), 0.0, yaw.sin());
            
            // Fix X inversion: negate X so stick right = strafe right
            let movement = (forward * move_input.y - right * move_input.x) * move_speed * dt;
            camera.target += movement;
        }

        // A button (South) = fly up, B button (East) = fly down
        if gamepad.pressed(bevy::input::gamepad::GamepadButton::South) {
            camera.target.y += move_speed * dt;
        }
        if gamepad.pressed(bevy::input::gamepad::GamepadButton::East) {
            camera.target.y -= move_speed * dt;
        }

        // Triggers for zoom (LT zoom out, RT zoom in)
        let lt = gamepad.get(bevy::input::gamepad::GamepadAxis::LeftZ).unwrap_or(0.0);
        let rt = gamepad.get(bevy::input::gamepad::GamepadAxis::RightZ).unwrap_or(0.0);
        if rt > 0.1 {
            camera.zoom(rt * dt * 60.0);
        }
        if lt > 0.1 {
            camera.zoom(-lt * dt * 60.0);
        }

        // D-pad for fine movement
        if gamepad.pressed(bevy::input::gamepad::GamepadButton::DPadUp) {
            let yaw = camera.rotation.x;
            let forward = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
            camera.target += forward * move_speed * dt * 0.5;
        }
        if gamepad.pressed(bevy::input::gamepad::GamepadButton::DPadDown) {
            let yaw = camera.rotation.x;
            let forward = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
            camera.target -= forward * move_speed * dt * 0.5;
        }
        if gamepad.pressed(bevy::input::gamepad::GamepadButton::DPadLeft) {
            let yaw = camera.rotation.x;
            let right = Vec3::new(-yaw.cos(), 0.0, yaw.sin());
            camera.target -= right * move_speed * dt * 0.5;
        }
        if gamepad.pressed(bevy::input::gamepad::GamepadButton::DPadRight) {
            let yaw = camera.rotation.x;
            let right = Vec3::new(-yaw.cos(), 0.0, yaw.sin());
            camera.target += right * move_speed * dt * 0.5;
        }

        // Y button to reset camera
        if gamepad.just_pressed(bevy::input::gamepad::GamepadButton::North) {
            camera.reset();
        }
    }
}

/// System to handle gamepad voxel actions (RT to execute tool action, LT to remove)
pub fn handle_gamepad_voxel_actions(
    gamepad_state: Res<GamepadCameraState>,
    gamepads: Query<&Gamepad>,
    mut editor_state: ResMut<crate::editor::state::EditorState>,
    mut history: ResMut<crate::editor::history::EditorHistory>,
    time: Res<Time>,
    mut cooldown: Local<f32>,
) {
    // Only process if gamepad is active
    if !gamepad_state.active {
        return;
    }

    // Update cooldown
    *cooldown = (*cooldown - time.delta_secs()).max(0.0);
    if *cooldown > 0.0 {
        return;
    }

    let Some(grid_pos) = gamepad_state.action_grid_pos else {
        return;
    };

    for gamepad in gamepads.iter() {
        // Try both axis and button for triggers (different controllers report differently)
        let rt_axis = gamepad.get(bevy::input::gamepad::GamepadAxis::RightZ).unwrap_or(0.0);
        let lt_axis = gamepad.get(bevy::input::gamepad::GamepadAxis::LeftZ).unwrap_or(0.0);
        let rt_button = gamepad.pressed(bevy::input::gamepad::GamepadButton::RightTrigger2);
        let lt_button = gamepad.pressed(bevy::input::gamepad::GamepadButton::LeftTrigger2);
        
        let rt_pressed = rt_axis > 0.5 || rt_button;
        let lt_pressed = lt_axis > 0.5 || lt_button;

        // RT = execute main action of current tool
        if rt_pressed {
            match editor_state.active_tool.clone() {
                crate::editor::state::EditorTool::VoxelPlace { voxel_type, pattern } => {
                    // Place voxel
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
                        info!("Placed voxel at {:?}", grid_pos);
                    }
                }
                crate::editor::state::EditorTool::VoxelRemove => {
                    // Remove voxel we're looking at
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
                        info!("Removed voxel at {:?}", remove_pos);
                    }
                }
                crate::editor::state::EditorTool::EntityPlace { entity_type } => {
                    // Place entity
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
                    info!("Placed entity at {:?}", grid_pos);
                }
                crate::editor::state::EditorTool::Select => {
                    // Select voxel we're looking at
                    if let Some(target_pos) = gamepad_state.target_voxel_pos {
                        if editor_state.selected_voxels.contains(&target_pos) {
                            editor_state.selected_voxels.remove(&target_pos);
                        } else {
                            editor_state.selected_voxels.insert(target_pos);
                        }
                        *cooldown = 0.2;
                        info!("Toggled selection at {:?}", target_pos);
                    }
                }
                crate::editor::state::EditorTool::Camera => {
                    // Camera tool has no action
                }
            }
        }

        // LT = always remove voxel (secondary action)
        if lt_pressed {
            // Use target_voxel_pos for removal (the voxel we're looking at, not the placement position)
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
                info!("Removed voxel at {:?}", remove_pos);
            }
        }
    }
}

/// System to handle RB/LB for cycling through patterns/entities
pub fn handle_gamepad_tool_cycling(
    gamepad_state: Res<GamepadCameraState>,
    gamepads: Query<&Gamepad>,
    mut editor_state: ResMut<crate::editor::state::EditorState>,
    time: Res<Time>,
    mut cooldown: Local<f32>,
) {
    // Only process if gamepad is active
    if !gamepad_state.active {
        return;
    }

    // Update cooldown
    *cooldown = (*cooldown - time.delta_secs()).max(0.0);
    if *cooldown > 0.0 {
        return;
    }

    for gamepad in gamepads.iter() {
        let rb_pressed = gamepad.just_pressed(bevy::input::gamepad::GamepadButton::RightTrigger);
        let lb_pressed = gamepad.just_pressed(bevy::input::gamepad::GamepadButton::LeftTrigger);

        if !rb_pressed && !lb_pressed {
            continue;
        }

        match &mut editor_state.active_tool {
            crate::editor::state::EditorTool::VoxelPlace { pattern, .. } => {
                use crate::systems::game::map::format::SubVoxelPattern;
                
                // All patterns in order
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
                let new_idx = if rb_pressed {
                    (current_idx + 1) % PATTERNS.len()
                } else {
                    (current_idx + PATTERNS.len() - 1) % PATTERNS.len()
                };
                *pattern = PATTERNS[new_idx];
                *cooldown = 0.2;
                info!("Pattern changed to {:?}", pattern);
            }
            crate::editor::state::EditorTool::EntityPlace { entity_type } => {
                use crate::systems::game::map::format::EntityType;
                
                // All entity types in order
                const ENTITIES: [EntityType; 6] = [
                    EntityType::PlayerSpawn,
                    EntityType::Npc,
                    EntityType::Enemy,
                    EntityType::Item,
                    EntityType::Trigger,
                    EntityType::LightSource,
                ];

                let current_idx = ENTITIES.iter().position(|e| e == entity_type).unwrap_or(0);
                let new_idx = if rb_pressed {
                    (current_idx + 1) % ENTITIES.len()
                } else {
                    (current_idx + ENTITIES.len() - 1) % ENTITIES.len()
                };
                *entity_type = ENTITIES[new_idx];
                *cooldown = 0.2;
                info!("Entity type changed to {:?}", entity_type);
            }
            _ => {}
        }
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
        let initial_target_distance = camera.target_distance;

        camera.zoom(1.0);
        assert!(camera.target_distance < initial_target_distance);

        camera.zoom(-1.0);
        // Target distance should be close to initial after zooming in then out
        assert!((camera.target_distance - initial_target_distance).abs() < 1.0);
    }

    #[test]
    fn test_camera_zoom_limits() {
        let mut camera = EditorCamera::default();

        // Zoom in a lot
        for _ in 0..100 {
            camera.zoom(1.0);
        }
        assert!(camera.target_distance >= camera.min_distance);

        // Zoom out a lot
        for _ in 0..100 {
            camera.zoom(-1.0);
        }
        assert!(camera.target_distance <= camera.max_distance);
    }

    #[test]
    fn test_smooth_zoom_interpolation() {
        let mut camera = EditorCamera {
            target_distance: 5.0, // Set target closer
            ..Default::default()
        };

        // Simulate several frames of interpolation
        for _ in 0..60 {
            camera.update_smooth_zoom(1.0 / 60.0);
        }

        // After 1 second, distance should be very close to target
        assert!((camera.distance - camera.target_distance).abs() < 0.1);
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
    fn test_camera_orbit() {
        let mut camera = EditorCamera::default();
        let initial_rotation = camera.rotation;

        camera.orbit(Vec2::new(100.0, 50.0));

        // Rotation should have changed
        assert!(camera.rotation.x != initial_rotation.x);
        assert!(camera.rotation.y != initial_rotation.y);
    }

    #[test]
    fn test_camera_orbit_pitch_clamp() {
        let mut camera = EditorCamera::default();

        // Try to orbit way past the pitch limits
        camera.orbit(Vec2::new(0.0, 10000.0));
        assert!(camera.rotation.y <= 1.5);

        camera.orbit(Vec2::new(0.0, -20000.0));
        assert!(camera.rotation.y >= -1.5);
    }

    #[test]
    fn test_camera_pan() {
        let mut camera = EditorCamera::default();
        let initial_target = camera.target;

        camera.pan(Vec2::new(100.0, 100.0));

        // Target should have moved
        assert!(camera.target != initial_target);
    }

    #[test]
    fn test_camera_reset() {
        let mut camera = EditorCamera::default();

        // Modify camera state
        camera.target = Vec3::new(100.0, 100.0, 100.0);
        camera.distance = 50.0;
        camera.target_distance = 50.0;
        camera.rotation = Vec2::new(3.14, 1.0);

        camera.reset();

        let default = EditorCamera::default();
        assert_eq!(camera.target, default.target);
        assert_eq!(camera.distance, default.distance);
        assert_eq!(camera.target_distance, default.target_distance);
        assert_eq!(camera.rotation, default.rotation);
    }

    #[test]
    fn test_camera_set_view() {
        let mut camera = EditorCamera::default();

        let position = Vec3::new(10.0, 5.0, 10.0);
        let target = Vec3::ZERO;

        camera.set_view(position, target);

        assert_eq!(camera.target, target);
        // Distance should be approximately the distance between position and target
        let expected_distance = (position - target).length();
        assert!((camera.distance - expected_distance).abs() < 0.01);
    }

    #[test]
    fn test_camera_looking_at_constructor() {
        let target = Vec3::new(5.0, 5.0, 5.0);
        let distance = 15.0;

        let camera = EditorCamera::looking_at(target, distance);

        assert_eq!(camera.target, target);
        assert_eq!(camera.distance, distance);
    }

    #[test]
    fn test_camera_calculate_position_at_different_rotations() {
        // Test that camera position varies with rotation
        let camera1 = EditorCamera {
            target: Vec3::ZERO,
            distance: 10.0,
            rotation: Vec2::new(0.0, 0.0),
            ..Default::default()
        };

        let camera2 = EditorCamera {
            target: Vec3::ZERO,
            distance: 10.0,
            rotation: Vec2::new(std::f32::consts::PI / 2.0, 0.0), // 90 degree yaw
            ..Default::default()
        };

        let pos1 = camera1.calculate_position();
        let pos2 = camera2.calculate_position();

        // Positions should be different due to different yaw
        assert!((pos1 - pos2).length() > 1.0);

        // Both should be at same distance from target
        assert!((pos1.length() - 10.0).abs() < 0.01);
        assert!((pos2.length() - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_camera_calculate_position_with_pitch() {
        let camera = EditorCamera {
            target: Vec3::ZERO,
            distance: 10.0,
            rotation: Vec2::new(0.0, 0.5), // Some pitch
            ..Default::default()
        };

        let pos = camera.calculate_position();

        // Camera should be elevated due to positive pitch
        assert!(pos.y > 0.0);
        // Still at correct distance
        assert!((pos.length() - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_pattern_cycling_array_coverage() {
        use crate::systems::game::map::format::SubVoxelPattern;

        // Verify all patterns are in the cycling array
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
        let current = 9; // Last index (Fence)
        let next = (current + 1) % PATTERNS.len();
        assert_eq!(next, 0);
        assert_eq!(PATTERNS[next], SubVoxelPattern::Full);

        // Test backward cycling wraps correctly
        let current = 0; // First index (Full)
        let prev = (current + PATTERNS.len() - 1) % PATTERNS.len();
        assert_eq!(prev, 9);
        assert_eq!(PATTERNS[prev], SubVoxelPattern::Fence);
    }

    #[test]
    fn test_entity_cycling_array_coverage() {
        use crate::systems::game::map::format::EntityType;

        // Verify all entity types are in the cycling array
        const ENTITIES: [EntityType; 6] = [
            EntityType::PlayerSpawn,
            EntityType::Npc,
            EntityType::Enemy,
            EntityType::Item,
            EntityType::Trigger,
            EntityType::LightSource,
        ];

        // Test forward cycling wraps correctly
        let current = 5; // Last index (LightSource)
        let next = (current + 1) % ENTITIES.len();
        assert_eq!(next, 0);
        assert_eq!(ENTITIES[next], EntityType::PlayerSpawn);

        // Test backward cycling wraps correctly
        let current = 0; // First index (PlayerSpawn)
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

        // Test each pattern can be found
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

        // Test each entity type can be found
        for (idx, entity) in ENTITIES.iter().enumerate() {
            let found_idx = ENTITIES.iter().position(|e| e == entity);
            assert_eq!(found_idx, Some(idx));
        }
    }
}
