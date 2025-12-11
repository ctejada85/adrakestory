//! Cursor ray casting system for detecting voxel positions from mouse input.

use crate::editor::camera::EditorCamera;
use crate::editor::state::{EditorState, EditorTool, KeyboardEditMode};
use crate::editor::tools::{ActiveTransform, TransformMode};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContexts;

/// Resource to track cursor position separately from editor state.
/// This prevents cursor updates from triggering change detection on EditorState.
#[derive(Resource, Default)]
pub struct CursorState {
    /// Current cursor position in world space (voxel being pointed at)
    pub position: Option<Vec3>,

    /// Current cursor grid position (voxel being pointed at)
    pub grid_pos: Option<(i32, i32, i32)>,

    /// Face normal of the hit surface
    pub hit_face_normal: Option<Vec3>,

    /// Position where a new voxel would be placed (adjacent to hit face)
    pub placement_pos: Option<Vec3>,

    /// Grid position where a new voxel would be placed
    pub placement_grid_pos: Option<(i32, i32, i32)>,
}

impl CursorState {
    /// Create a new cursor state
    pub fn new() -> Self {
        Self::default()
    }
}

/// System to toggle keyboard edit mode (I to enter, Escape to exit)
/// Note: Escape only exits keyboard mode when there are no selections in Select tool
pub fn toggle_keyboard_edit_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut keyboard_mode: ResMut<KeyboardEditMode>,
    mut contexts: EguiContexts,
    editor_state: Res<EditorState>,
) {
    // Don't toggle if UI wants keyboard input
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    // Enter keyboard edit mode with I key
    if keyboard.just_pressed(KeyCode::KeyI) {
        keyboard_mode.enable();
        info!("Keyboard edit mode ENABLED");
    }

    // Exit keyboard edit mode with Escape key
    // BUT: Only if we're not in Select tool with active selections
    // (In Select tool, Escape should clear selections first, handled by handle_selection_mode_input)
    if keyboard.just_pressed(KeyCode::Escape) {
        // Check if we're in Select tool with selections
        let has_selections = matches!(editor_state.active_tool, EditorTool::Select)
            && !editor_state.selected_voxels.is_empty();

        // Only exit keyboard mode if there are no selections
        if !has_selections {
            keyboard_mode.disable();
            info!("Keyboard edit mode DISABLED");
        }
        // If there are selections, let handle_selection_mode_input clear them
        // and keep keyboard mode active
    }
}

/// System to update cursor position and grid position from mouse input
/// Only updates when keyboard edit mode is disabled
pub fn update_cursor_position(
    mut cursor_state: ResMut<CursorState>,
    editor_state: Res<EditorState>,
    camera_query: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    keyboard_mode: Res<KeyboardEditMode>,
    mut contexts: EguiContexts,
) {
    // Don't update cursor from mouse when in keyboard edit mode
    if keyboard_mode.enabled {
        return;
    }

    // Don't update if mouse is over UI
    if contexts.ctx_mut().is_pointer_over_area() {
        return;
    }

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let Ok(window) = window_query.get_single() else {
        return;
    };

    // Get cursor position in window
    let Some(cursor_position) = window.cursor_position() else {
        cursor_state.position = None;
        cursor_state.grid_pos = None;
        return;
    };

    // Convert cursor position to world ray
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        cursor_state.position = None;
        cursor_state.grid_pos = None;
        return;
    };

    // Find the closest voxel that the ray intersects with face information
    let closest_voxel_hit = find_closest_voxel_intersection_with_face(&editor_state, &ray);

    if let Some((voxel_pos, hit_info)) = closest_voxel_hit {
        // Set cursor to the intersected voxel
        cursor_state.grid_pos = Some(voxel_pos);
        cursor_state.position = Some(Vec3::new(
            voxel_pos.0 as f32,
            voxel_pos.1 as f32,
            voxel_pos.2 as f32,
        ));
        cursor_state.hit_face_normal = Some(hit_info.face_normal);

        // Calculate adjacent placement position
        let offset = hit_info.face_normal;
        let placement_grid = (
            voxel_pos.0 + offset.x as i32,
            voxel_pos.1 + offset.y as i32,
            voxel_pos.2 + offset.z as i32,
        );
        cursor_state.placement_grid_pos = Some(placement_grid);
        cursor_state.placement_pos = Some(Vec3::new(
            placement_grid.0 as f32,
            placement_grid.1 as f32,
            placement_grid.2 as f32,
        ));
    } else {
        // No voxel intersection, fall back to ground plane intersection
        if let Some(ground_pos) = intersect_ground_plane(&ray) {
            // Keep cursor position as exact intersection for free movement
            cursor_state.position = Some(ground_pos);

            // Snap grid position to nearest integer coordinates
            let grid_x = ground_pos.x.round() as i32;
            let grid_y = 0;
            let grid_z = ground_pos.z.round() as i32;
            cursor_state.grid_pos = Some((grid_x, grid_y, grid_z));
            cursor_state.hit_face_normal = Some(Vec3::Y); // Upward

            // Placement position snaps to grid (integer coordinates)
            cursor_state.placement_grid_pos = Some((grid_x, grid_y, grid_z));
            cursor_state.placement_pos =
                Some(Vec3::new(grid_x as f32, grid_y as f32, grid_z as f32));
        } else {
            cursor_state.position = None;
            cursor_state.grid_pos = None;
            cursor_state.hit_face_normal = None;
            cursor_state.placement_pos = None;
            cursor_state.placement_grid_pos = None;
        }
    }
}

/// Information about a ray-box intersection
#[derive(Debug, Clone, Copy)]
struct RayHitInfo {
    distance: f32,
    face_normal: Vec3,
}

/// Find the closest voxel that the ray intersects with face information
fn find_closest_voxel_intersection_with_face(
    editor_state: &EditorState,
    ray: &Ray3d,
) -> Option<((i32, i32, i32), RayHitInfo)> {
    let mut closest_distance = f32::MAX;
    let mut closest_result = None;

    // Check each voxel in the map
    for voxel_data in &editor_state.current_map.world.voxels {
        let voxel_pos = voxel_data.pos;

        // Check if ray intersects this voxel's bounding box
        if let Some(hit_info) = ray_box_intersection_with_face(
            ray,
            Vec3::new(voxel_pos.0 as f32, voxel_pos.1 as f32, voxel_pos.2 as f32),
            Vec3::splat(1.0), // Voxel size is 1x1x1
        ) {
            if hit_info.distance < closest_distance {
                closest_distance = hit_info.distance;
                closest_result = Some((voxel_pos, hit_info));
            }
        }
    }

    closest_result
}

/// Ray-box intersection test (AABB) with face detection
/// Returns hit information including which face was hit
fn ray_box_intersection_with_face(
    ray: &Ray3d,
    box_center: Vec3,
    box_size: Vec3,
) -> Option<RayHitInfo> {
    let box_min = box_center - box_size * 0.5;
    let box_max = box_center + box_size * 0.5;

    let ray_origin = ray.origin;
    let ray_dir = ray.direction.normalize();

    // Calculate intersection distances for each axis
    let mut tmin = f32::NEG_INFINITY;
    let mut tmax = f32::INFINITY;
    let mut hit_axis = 0; // 0=X, 1=Y, 2=Z
    let mut hit_min_face = true; // true if hit min face, false if hit max face

    // X axis
    if ray_dir.x.abs() > 0.0001 {
        let tx1 = (box_min.x - ray_origin.x) / ray_dir.x;
        let tx2 = (box_max.x - ray_origin.x) / ray_dir.x;
        let tx_min = tx1.min(tx2);
        let tx_max = tx1.max(tx2);

        if tx_min > tmin {
            tmin = tx_min;
            hit_axis = 0;
            hit_min_face = tx1 < tx2;
        }
        tmax = tmax.min(tx_max);
    } else if ray_origin.x < box_min.x || ray_origin.x > box_max.x {
        return None;
    }

    // Y axis
    if ray_dir.y.abs() > 0.0001 {
        let ty1 = (box_min.y - ray_origin.y) / ray_dir.y;
        let ty2 = (box_max.y - ray_origin.y) / ray_dir.y;
        let ty_min = ty1.min(ty2);
        let ty_max = ty1.max(ty2);

        if ty_min > tmin {
            tmin = ty_min;
            hit_axis = 1;
            hit_min_face = ty1 < ty2;
        }
        tmax = tmax.min(ty_max);
    } else if ray_origin.y < box_min.y || ray_origin.y > box_max.y {
        return None;
    }

    // Z axis
    if ray_dir.z.abs() > 0.0001 {
        let tz1 = (box_min.z - ray_origin.z) / ray_dir.z;
        let tz2 = (box_max.z - ray_origin.z) / ray_dir.z;
        let tz_min = tz1.min(tz2);
        let tz_max = tz1.max(tz2);

        if tz_min > tmin {
            tmin = tz_min;
            hit_axis = 2;
            hit_min_face = tz1 < tz2;
        }
        tmax = tmax.min(tz_max);
    } else if ray_origin.z < box_min.z || ray_origin.z > box_max.z {
        return None;
    }

    // Check if there's a valid intersection
    if tmax >= tmin && tmax >= 0.0 {
        let distance = if tmin >= 0.0 { tmin } else { tmax };

        // Calculate face normal based on hit axis and face
        let face_normal = match (hit_axis, hit_min_face) {
            (0, true) => Vec3::NEG_X,
            (0, false) => Vec3::X,
            (1, true) => Vec3::NEG_Y,
            (1, false) => Vec3::Y,
            (2, true) => Vec3::NEG_Z,
            (2, false) => Vec3::Z,
            _ => Vec3::Y, // fallback
        };

        Some(RayHitInfo {
            distance,
            face_normal,
        })
    } else {
        None
    }
}

/// Intersect ray with ground plane (y=0) as fallback
fn intersect_ground_plane(ray: &Ray3d) -> Option<Vec3> {
    let ray_origin = ray.origin;
    let ray_direction = ray.direction.normalize();

    // Check if ray is parallel to ground
    if ray_direction.y.abs() < 0.001 {
        return None;
    }

    // Calculate t where ray intersects y=0
    let t = -ray_origin.y / ray_direction.y;

    if t < 0.0 {
        return None;
    }

    // Calculate world position at intersection
    Some(ray_origin + ray_direction * t)
}

/// System to move cursor position using keyboard (arrow keys, space, C)
/// Allows navigating the 3D grid step-by-step without using the mouse
/// Only active when keyboard edit mode is enabled
pub fn handle_keyboard_cursor_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut cursor_state: ResMut<CursorState>,
    editor_state: Res<EditorState>,
    _active_transform: Res<ActiveTransform>,
    keyboard_mode: Res<KeyboardEditMode>,
) {
    // Only allow keyboard cursor movement when in keyboard edit mode
    if !keyboard_mode.enabled {
        return;
    }

    // Check if UI wants keyboard input (user is typing in text fields, etc.)
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    // Block keyboard cursor movement for Camera tool
    if matches!(editor_state.active_tool, EditorTool::Camera) {
        return;
    }

    // Block cursor movement during active Move or Rotate operations
    // This keeps the cursor stationary while transforming selections
    if _active_transform.mode != TransformMode::None {
        return;
    }

    // Get current cursor position or default to (0, 0, 0)
    let current_pos = cursor_state.grid_pos.unwrap_or((0, 0, 0));

    // Calculate movement step (1 or 5 with Shift for fast movement)
    let step = if keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        5
    } else {
        1
    };

    let mut new_pos = current_pos;
    let mut moved = false;

    // Horizontal movement (X/Z plane)
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        new_pos.2 -= step; // Move forward (negative Z)
        moved = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        new_pos.2 += step; // Move backward (positive Z)
        moved = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        new_pos.0 -= step; // Move left (negative X)
        moved = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        new_pos.0 += step; // Move right (positive X)
        moved = true;
    }

    // Vertical movement (Y axis)
    if keyboard.just_pressed(KeyCode::Space) {
        new_pos.1 += step; // Move up
        moved = true;
    }
    if keyboard.just_pressed(KeyCode::KeyC) {
        new_pos.1 -= step; // Move down
        moved = true;
    }

    // Update cursor position if moved
    if moved {
        cursor_state.grid_pos = Some(new_pos);
        cursor_state.position = Some(Vec3::new(
            new_pos.0 as f32,
            new_pos.1 as f32,
            new_pos.2 as f32,
        ));

        // For keyboard movement, assume placement on top (+Y direction)
        cursor_state.hit_face_normal = Some(Vec3::Y);
        let placement_grid = (new_pos.0, new_pos.1 + 1, new_pos.2);
        cursor_state.placement_grid_pos = Some(placement_grid);
        cursor_state.placement_pos = Some(Vec3::new(
            placement_grid.0 as f32,
            placement_grid.1 as f32,
            placement_grid.2 as f32,
        ));

        info!("Cursor moved to grid position: {:?}", new_pos);
    }
}

/// System to handle keyboard-based selection (Enter key)
/// Allows selecting voxels with Enter when in keyboard edit mode
pub fn handle_keyboard_selection(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    cursor_state: Res<CursorState>,
    mut editor_state: ResMut<EditorState>,
    keyboard_mode: Res<KeyboardEditMode>,
    mut update_events: EventWriter<crate::editor::tools::UpdateSelectionHighlights>,
) {
    // Only allow keyboard selection when in keyboard edit mode
    if !keyboard_mode.enabled {
        return;
    }

    // Check if UI wants keyboard input (user is typing in text fields, etc.)
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    // Check if select tool is active
    if !matches!(editor_state.active_tool, EditorTool::Select) {
        return;
    }

    // Check if Enter key was just pressed
    if !keyboard.just_pressed(KeyCode::Enter) {
        return;
    }

    // Get cursor grid position
    let Some(grid_pos) = cursor_state.grid_pos else {
        return;
    };

    // Toggle selection of voxel at this position
    if editor_state.selected_voxels.contains(&grid_pos) {
        editor_state.selected_voxels.remove(&grid_pos);
        info!("Deselected voxel at {:?} (keyboard)", grid_pos);
    } else {
        editor_state.selected_voxels.insert(grid_pos);
        info!("Selected voxel at {:?} (keyboard)", grid_pos);
    }

    // Trigger highlight update
    update_events.send(crate::editor::tools::UpdateSelectionHighlights);
}

/// System to handle tool switching with keyboard shortcuts
/// B or 1 = VoxelPlace, V or 2 = Select, X = VoxelRemove, E = EntityPlace, C = Camera
pub fn handle_tool_switching(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut editor_state: ResMut<EditorState>,
    mut tool_memory: ResMut<crate::editor::state::ToolMemory>,
) {
    // Check if UI wants keyboard input (user is typing in text fields, etc.)
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    // Helper to save current tool parameters before switching
    let save_current_params = |editor_state: &EditorState, tool_memory: &mut crate::editor::state::ToolMemory| {
        match &editor_state.active_tool {
            EditorTool::VoxelPlace { voxel_type, pattern } => {
                tool_memory.voxel_type = *voxel_type;
                tool_memory.voxel_pattern = *pattern;
            }
            EditorTool::EntityPlace { entity_type } => {
                tool_memory.entity_type = entity_type.clone();
            }
            _ => {}
        }
    };

    // Switch to VoxelPlace tool with B or 1 key
    if (keyboard.just_pressed(KeyCode::Digit1) || keyboard.just_pressed(KeyCode::KeyB))
        && !matches!(editor_state.active_tool, EditorTool::VoxelPlace { .. })
    {
        save_current_params(&editor_state, &mut tool_memory);
        editor_state.active_tool = EditorTool::VoxelPlace {
            voxel_type: tool_memory.voxel_type,
            pattern: tool_memory.voxel_pattern,
        };
        info!("Switched to VoxelPlace tool");
    }

    // Switch to Select tool with V or 2 key
    if (keyboard.just_pressed(KeyCode::Digit2) || keyboard.just_pressed(KeyCode::KeyV))
        && !matches!(editor_state.active_tool, EditorTool::Select)
    {
        save_current_params(&editor_state, &mut tool_memory);
        editor_state.active_tool = EditorTool::Select;
        info!("Switched to Select tool");
    }

    // Switch to VoxelRemove tool with X key
    if keyboard.just_pressed(KeyCode::KeyX)
        && !matches!(editor_state.active_tool, EditorTool::VoxelRemove)
    {
        save_current_params(&editor_state, &mut tool_memory);
        editor_state.active_tool = EditorTool::VoxelRemove;
        info!("Switched to VoxelRemove tool");
    }

    // Switch to EntityPlace tool with E key
    if keyboard.just_pressed(KeyCode::KeyE)
        && !matches!(editor_state.active_tool, EditorTool::EntityPlace { .. })
    {
        save_current_params(&editor_state, &mut tool_memory);
        editor_state.active_tool = EditorTool::EntityPlace {
            entity_type: tool_memory.entity_type.clone(),
        };
        info!("Switched to EntityPlace tool");
    }

    // Switch to Camera tool with C key
    if keyboard.just_pressed(KeyCode::KeyC)
        && !matches!(editor_state.active_tool, EditorTool::Camera)
    {
        save_current_params(&editor_state, &mut tool_memory);
        editor_state.active_tool = EditorTool::Camera;
        info!("Switched to Camera tool");
    }
}
