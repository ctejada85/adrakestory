//! Selection tool for selecting and manipulating objects.

use crate::editor::history::{EditorAction, EditorHistory};
use crate::editor::renderer::RenderMapEvent;
use crate::editor::state::{EditorState, EditorTool};
use crate::systems::game::map::format::VoxelData;
use crate::systems::game::map::geometry::RotationAxis;
use bevy::prelude::*;
use bevy_egui::EguiContexts;

/// Marker component for selection highlight visuals
#[derive(Component)]
pub struct SelectionHighlight {
    pub voxel_pos: (i32, i32, i32),
}

/// Marker component for transform preview visuals
#[derive(Component)]
pub struct TransformPreview {
    pub original_pos: (i32, i32, i32),
    pub preview_pos: (i32, i32, i32),
    pub is_valid: bool, // false if collision detected
}

/// Resource tracking active transformation
#[derive(Resource)]
pub struct ActiveTransform {
    pub mode: TransformMode,
    pub selected_voxels: Vec<VoxelData>,
    pub pivot: Vec3,
    pub current_offset: IVec3,
    pub rotation_axis: RotationAxis,
    pub rotation_angle: i32, // In 90-degree increments (0, 1, 2, 3)
}

impl Default for ActiveTransform {
    fn default() -> Self {
        Self {
            mode: TransformMode::None,
            selected_voxels: Vec::new(),
            pivot: Vec3::ZERO,
            current_offset: IVec3::ZERO,
            rotation_axis: RotationAxis::Y,
            rotation_angle: 0,
        }
    }
}

/// Transform operation mode
#[derive(Debug, Clone, PartialEq, Default)]
pub enum TransformMode {
    #[default]
    None,
    Move,
    Rotate,
}

/// Event to trigger selection highlight update
#[derive(Event)]
pub struct UpdateSelectionHighlights;

/// Event to trigger deletion of selected voxels
#[derive(Event)]
pub struct DeleteSelectedVoxels;

/// Event to start move operation
#[derive(Event)]
pub struct StartMoveOperation;

/// Event to start rotate operation
#[derive(Event)]
pub struct StartRotateOperation;

/// Event to set rotation axis
#[derive(Event)]
pub struct SetRotationAxis {
    pub axis: RotationAxis,
}

/// Event to confirm transformation
#[derive(Event)]
pub struct ConfirmTransform;

/// Event to cancel transformation
#[derive(Event)]
pub struct CancelTransform;

/// Event to update transform preview
#[derive(Event)]
pub struct UpdateTransformPreview {
    pub offset: IVec3,
}

/// Event to update rotation
#[derive(Event)]
pub struct UpdateRotation {
    pub delta: i32, // +1 or -1 for 90-degree rotations
}

/// Handle selection when the tool is active
pub fn handle_selection(
    mut editor_state: ResMut<EditorState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    mut update_events: EventWriter<UpdateSelectionHighlights>,
) {
    // Check if select tool is active
    if !matches!(editor_state.active_tool, EditorTool::Select) {
        return;
    }

    // Check if UI wants pointer input (user is interacting with UI elements)
    let ui_wants_input = contexts.ctx_mut().wants_pointer_input();
    if ui_wants_input {
        return;
    }

    // Check if left mouse button was just pressed
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    // Get cursor grid position
    let Some(grid_pos) = editor_state.cursor_grid_pos else {
        return;
    };

    // Toggle selection of voxel at this position
    if editor_state.selected_voxels.contains(&grid_pos) {
        editor_state.selected_voxels.remove(&grid_pos);
        info!("Deselected voxel at {:?}", grid_pos);
    } else {
        editor_state.selected_voxels.insert(grid_pos);
        info!("Selected voxel at {:?}", grid_pos);
    }

    // Trigger highlight update
    update_events.send(UpdateSelectionHighlights);
}

/// Render selection highlights for selected voxels
pub fn render_selection_highlights(
    mut commands: Commands,
    editor_state: Res<EditorState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_highlights: Query<Entity, With<SelectionHighlight>>,
    mut update_events: EventReader<UpdateSelectionHighlights>,
) {
    // Only update if event received
    if update_events.read().count() == 0 {
        return;
    }

    // Despawn existing highlights
    for entity in existing_highlights.iter() {
        commands.entity(entity).despawn();
    }

    // Don't render if no selection
    if editor_state.selected_voxels.is_empty() {
        return;
    }

    // Create highlight material (bright yellow with emission)
    let highlight_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 0.0, 0.6),
        emissive: LinearRgba::new(1.0, 1.0, 0.0, 1.0),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Create wireframe cube mesh (slightly larger than voxel)
    let highlight_mesh = meshes.add(Cuboid::new(1.05, 1.05, 1.05));

    // Spawn highlight for each selected voxel
    for &pos in &editor_state.selected_voxels {
        spawn_selection_highlight(
            &mut commands,
            &highlight_mesh,
            &highlight_material,
            pos,
        );
    }

    info!(
        "Rendered {} selection highlights",
        editor_state.selected_voxels.len()
    );
}

/// Spawn a single selection highlight at the given position
fn spawn_selection_highlight(
    commands: &mut Commands,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    pos: (i32, i32, i32),
) {
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material.clone()),
        Transform::from_xyz(pos.0 as f32, pos.1 as f32, pos.2 as f32),
        SelectionHighlight { voxel_pos: pos },
    ));
}

/// Handle deletion of selected voxels via Delete/Backspace keys
pub fn handle_delete_selected(
    mut editor_state: ResMut<EditorState>,
    mut history: ResMut<EditorHistory>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut render_events: EventWriter<RenderMapEvent>,
    mut update_events: EventWriter<UpdateSelectionHighlights>,
    mut delete_events: EventReader<DeleteSelectedVoxels>,
) {
    // Only active when Select tool is active
    if !matches!(editor_state.active_tool, EditorTool::Select) {
        return;
    }

    // Check if UI wants keyboard input
    let ui_wants_input = contexts.ctx_mut().wants_keyboard_input();
    
    // Check for Delete or Backspace key (only if UI doesn't want input), or delete event from UI
    let should_delete = (!ui_wants_input && (keyboard.just_pressed(KeyCode::Delete)
        || keyboard.just_pressed(KeyCode::Backspace)))
        || delete_events.read().count() > 0;

    if !should_delete {
        return;
    }

    // Nothing to delete
    if editor_state.selected_voxels.is_empty() {
        info!("No voxels selected to delete");
        return;
    }

    // Create batch action for undo/redo
    let mut actions = Vec::new();
    let selected_count = editor_state.selected_voxels.len();

    // Collect selected positions to avoid borrow checker issues
    let selected_positions: Vec<(i32, i32, i32)> =
        editor_state.selected_voxels.iter().copied().collect();

    // Find and remove each selected voxel
    for pos in selected_positions {
        if let Some(index) = editor_state
            .current_map
            .world
            .voxels
            .iter()
            .position(|v| v.pos == pos)
        {
            let voxel_data = editor_state.current_map.world.voxels[index].clone();
            actions.push(EditorAction::RemoveVoxel {
                pos,
                data: voxel_data,
            });
            editor_state.current_map.world.voxels.remove(index);
        }
    }

    // Push batch action to history
    if !actions.is_empty() {
        history.push(EditorAction::Batch {
            description: format!("Delete {} voxel{}", actions.len(), if actions.len() == 1 { "" } else { "s" }),
            actions,
        });

        editor_state.mark_modified();
        info!("Deleted {} selected voxels", selected_count);
    }

    // Clear selection
    editor_state.selected_voxels.clear();

    // Trigger re-render
    render_events.send(RenderMapEvent);
    update_events.send(UpdateSelectionHighlights);
}

/// Start move operation for selected voxels
pub fn start_move_operation(
    mut active_transform: ResMut<ActiveTransform>,
    editor_state: Res<EditorState>,
    mut start_events: EventReader<StartMoveOperation>,
) {
    // Only process if event received
    if start_events.read().count() == 0 {
        return;
    }

    // Check if select tool is active
    if !matches!(editor_state.active_tool, EditorTool::Select) {
        warn!("Move operation requires Select tool to be active");
        return;
    }

    // Check if there are selected voxels
    if editor_state.selected_voxels.is_empty() {
        warn!("No voxels selected for move operation");
        return;
    }

    // Collect selected voxel data
    let mut selected_voxels = Vec::new();
    let mut sum_pos = Vec3::ZERO;
    
    for &pos in &editor_state.selected_voxels {
        if let Some(voxel) = editor_state
            .current_map
            .world
            .voxels
            .iter()
            .find(|v| v.pos == pos)
        {
            selected_voxels.push(voxel.clone());
            sum_pos += Vec3::new(pos.0 as f32, pos.1 as f32, pos.2 as f32);
        }
    }

    // Calculate pivot (center of selection)
    let pivot = sum_pos / selected_voxels.len() as f32;

    // Initialize transform state
    active_transform.mode = TransformMode::Move;
    active_transform.selected_voxels = selected_voxels;
    active_transform.pivot = pivot;
    active_transform.current_offset = IVec3::ZERO;

    info!("Started move operation with {} voxels", active_transform.selected_voxels.len());
}

/// Handle arrow key movement during move operation
pub fn handle_arrow_key_movement(
    active_transform: Res<ActiveTransform>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut preview_events: EventWriter<UpdateTransformPreview>,
) {
    // Only active during move mode
    if active_transform.mode != TransformMode::Move {
        return;
    }

    // Check if UI wants keyboard input
    let ui_wants_input = contexts.ctx_mut().wants_keyboard_input();
    if ui_wants_input {
        return;
    }

    let mut offset = IVec3::ZERO;
    let step = if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
        5 // Shift modifier for 5-unit movement
    } else {
        1 // Normal 1-unit movement
    };

    // Check arrow keys
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        offset.z -= step;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        offset.z += step;
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        offset.x -= step;
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        offset.x += step;
    }

    // Check Y-axis movement (Page Up/Down or custom keys)
    if keyboard.just_pressed(KeyCode::PageUp) {
        offset.y += step;
    }
    if keyboard.just_pressed(KeyCode::PageDown) {
        offset.y -= step;
    }

    // Send update event if offset changed
    if offset != IVec3::ZERO {
        preview_events.send(UpdateTransformPreview { offset });
    }
}

/// Update transform preview based on offset
pub fn update_transform_preview(
    mut active_transform: ResMut<ActiveTransform>,
    mut preview_events: EventReader<UpdateTransformPreview>,
) {
    for event in preview_events.read() {
        active_transform.current_offset += event.offset;
        info!("Transform offset updated to: {:?}", active_transform.current_offset);
    }
}

/// Render transform preview meshes
pub fn render_transform_preview(
    mut commands: Commands,
    active_transform: Res<ActiveTransform>,
    editor_state: Res<EditorState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_previews: Query<Entity, With<TransformPreview>>,
) {
    // Only render during active transform
    if active_transform.mode == TransformMode::None {
        // Clean up any existing previews
        for entity in existing_previews.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Check if transform state changed
    if !active_transform.is_changed() {
        return;
    }

    // Despawn existing previews
    for entity in existing_previews.iter() {
        commands.entity(entity).despawn();
    }

    // Calculate new positions and check collisions
    let offset = active_transform.current_offset;
    let original_positions: std::collections::HashSet<_> = 
        active_transform.selected_voxels.iter().map(|v| v.pos).collect();

    // Create preview mesh
    let preview_mesh = meshes.add(Cuboid::new(0.95, 0.95, 0.95));

    for voxel in &active_transform.selected_voxels {
        let new_pos = (
            voxel.pos.0 + offset.x,
            voxel.pos.1 + offset.y,
            voxel.pos.2 + offset.z,
        );

        // Check for collision (position occupied by non-selected voxel)
        let is_valid = !editor_state
            .current_map
            .world
            .voxels
            .iter()
            .any(|v| v.pos == new_pos && !original_positions.contains(&v.pos));

        // Create material based on validity
        let material = materials.add(StandardMaterial {
            base_color: if is_valid {
                Color::srgba(0.0, 1.0, 0.0, 0.3) // Green for valid
            } else {
                Color::srgba(1.0, 0.0, 0.0, 0.3) // Red for collision
            },
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });

        // Spawn preview
        commands.spawn((
            Mesh3d(preview_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(new_pos.0 as f32, new_pos.1 as f32, new_pos.2 as f32),
            TransformPreview {
                original_pos: voxel.pos,
                preview_pos: new_pos,
                is_valid,
            },
        ));
    }
}

/// Confirm transformation and apply changes
pub fn confirm_transform(
    mut active_transform: ResMut<ActiveTransform>,
    mut editor_state: ResMut<EditorState>,
    mut history: ResMut<EditorHistory>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    mut confirm_events: EventReader<ConfirmTransform>,
    mut render_events: EventWriter<RenderMapEvent>,
    mut update_events: EventWriter<UpdateSelectionHighlights>,
    preview_query: Query<&TransformPreview>,
) {
    // Only active during transform
    if active_transform.mode == TransformMode::None {
        return;
    }

    // Check if UI wants input
    let ui_wants_keyboard = contexts.ctx_mut().wants_keyboard_input();
    let ui_wants_pointer = contexts.ctx_mut().wants_pointer_input();
    
    // Check for confirmation input (only if UI doesn't want input)
    let should_confirm = (!ui_wants_keyboard && keyboard.just_pressed(KeyCode::Enter))
        || (!ui_wants_pointer && mouse_button.just_pressed(MouseButton::Left))
        || confirm_events.read().count() > 0;

    if !should_confirm {
        return;
    }

    // Check if all previews are valid (no collisions)
    let has_collision = preview_query.iter().any(|p| !p.is_valid);
    if has_collision {
        warn!("Cannot confirm move: collision detected");
        return;
    }

    // Apply the transformation
    let offset = active_transform.current_offset;
    let mut moved_voxels = Vec::new();

    for voxel in &active_transform.selected_voxels {
        let old_pos = voxel.pos;
        let new_pos = (
            old_pos.0 + offset.x,
            old_pos.1 + offset.y,
            old_pos.2 + offset.z,
        );

        // Find and update voxel in map
        if let Some(map_voxel) = editor_state
            .current_map
            .world
            .voxels
            .iter_mut()
            .find(|v| v.pos == old_pos)
        {
            map_voxel.pos = new_pos;
            moved_voxels.push((old_pos, new_pos));
        }
    }

    // Create history action
    if !moved_voxels.is_empty() {
        history.push(EditorAction::Batch {
            description: format!("Move {} voxel{}", moved_voxels.len(), if moved_voxels.len() == 1 { "" } else { "s" }),
            actions: moved_voxels
                .iter()
                .map(|(old_pos, new_pos)| {
                    let voxel_data = active_transform
                        .selected_voxels
                        .iter()
                        .find(|v| v.pos == *old_pos)
                        .unwrap()
                        .clone();
                    
                    // Create a remove and place action pair
                    EditorAction::Batch {
                        description: format!("Move voxel from {:?} to {:?}", old_pos, new_pos),
                        actions: vec![
                            EditorAction::RemoveVoxel {
                                pos: *old_pos,
                                data: voxel_data.clone(),
                            },
                            EditorAction::PlaceVoxel {
                                pos: *new_pos,
                                data: VoxelData {
                                    pos: *new_pos,
                                    ..voxel_data
                                },
                            },
                        ],
                    }
                })
                .collect(),
        });

        editor_state.mark_modified();
        info!("Moved {} voxels by offset {:?}", moved_voxels.len(), offset);
    }

    // Update selection to new positions
    editor_state.selected_voxels.clear();
    for (_, new_pos) in moved_voxels {
        editor_state.selected_voxels.insert(new_pos);
    }

    // Reset transform state
    *active_transform = ActiveTransform::default();

    // Trigger updates
    render_events.send(RenderMapEvent);
    update_events.send(UpdateSelectionHighlights);
}

/// Cancel transformation
pub fn cancel_transform(
    mut active_transform: ResMut<ActiveTransform>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    mut cancel_events: EventReader<CancelTransform>,
) {
    // Only active during transform
    if active_transform.mode == TransformMode::None {
        return;
    }

    // Check if UI wants input
    let ui_wants_keyboard = contexts.ctx_mut().wants_keyboard_input();
    let ui_wants_pointer = contexts.ctx_mut().wants_pointer_input();
    
    // Check for cancellation input (only if UI doesn't want input)
    let should_cancel = (!ui_wants_keyboard && keyboard.just_pressed(KeyCode::Escape))
        || (!ui_wants_pointer && mouse_button.just_pressed(MouseButton::Right))
        || cancel_events.read().count() > 0;

    if should_cancel {
        info!("Transform operation cancelled");
        *active_transform = ActiveTransform::default();
    }
}

/// Handle G key to start move operation
pub fn handle_move_shortcut(
    active_transform: Res<ActiveTransform>,
    editor_state: Res<EditorState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut start_events: EventWriter<StartMoveOperation>,
) {
    // Only when Select tool is active and not already transforming
    if !matches!(editor_state.active_tool, EditorTool::Select) {
        return;
    }

    if active_transform.mode != TransformMode::None {
        return;
    }

    // Check if UI wants keyboard input
    let ui_wants_input = contexts.ctx_mut().wants_keyboard_input();
    if ui_wants_input {
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyG) {
        start_events.send(StartMoveOperation);
    }
}


/// Start rotate operation for selected voxels
pub fn start_rotate_operation(
    mut active_transform: ResMut<ActiveTransform>,
    editor_state: Res<EditorState>,
    mut start_events: EventReader<StartRotateOperation>,
) {
    // Only process if event received
    if start_events.read().count() == 0 {
        return;
    }

    // Check if select tool is active
    if !matches!(editor_state.active_tool, EditorTool::Select) {
        warn!("Rotate operation requires Select tool to be active");
        return;
    }

    // Check if there are selected voxels
    if editor_state.selected_voxels.is_empty() {
        warn!("No voxels selected for rotate operation");
        return;
    }

    // Collect selected voxel data
    let mut selected_voxels = Vec::new();
    let mut sum_pos = Vec3::ZERO;
    
    for &pos in &editor_state.selected_voxels {
        if let Some(voxel) = editor_state
            .current_map
            .world
            .voxels
            .iter()
            .find(|v| v.pos == pos)
        {
            selected_voxels.push(voxel.clone());
            sum_pos += Vec3::new(pos.0 as f32, pos.1 as f32, pos.2 as f32);
        }
    }

    // Calculate pivot (center of selection)
    let pivot = sum_pos / selected_voxels.len() as f32;

    // Initialize transform state
    active_transform.mode = TransformMode::Rotate;
    active_transform.selected_voxels = selected_voxels;
    active_transform.pivot = pivot;
    active_transform.rotation_axis = RotationAxis::Y; // Default to Y axis
    active_transform.rotation_angle = 0;

    info!("Started rotate operation with {} voxels around {:?} axis", 
          active_transform.selected_voxels.len(), 
          active_transform.rotation_axis);
}

/// Handle R key to start rotate operation
pub fn handle_rotate_shortcut(
    active_transform: Res<ActiveTransform>,
    editor_state: Res<EditorState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut start_events: EventWriter<StartRotateOperation>,
) {
    // Only when Select tool is active and not already transforming
    if !matches!(editor_state.active_tool, EditorTool::Select) {
        return;
    }

    if active_transform.mode != TransformMode::None {
        return;
    }

    // Check if UI wants keyboard input
    let ui_wants_input = contexts.ctx_mut().wants_keyboard_input();
    if ui_wants_input {
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        start_events.send(StartRotateOperation);
    }
}

/// Handle axis selection during rotate operation (X, Y, Z keys)
pub fn handle_rotation_axis_selection(
    active_transform: Res<ActiveTransform>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut axis_events: EventWriter<SetRotationAxis>,
) {
    // Only active during rotate mode
    if active_transform.mode != TransformMode::Rotate {
        return;
    }

    // Check if UI wants keyboard input
    let ui_wants_input = contexts.ctx_mut().wants_keyboard_input();
    if ui_wants_input {
        return;
    }

    // Check for axis selection keys
    if keyboard.just_pressed(KeyCode::KeyX) {
        axis_events.send(SetRotationAxis { axis: RotationAxis::X });
    } else if keyboard.just_pressed(KeyCode::KeyY) {
        axis_events.send(SetRotationAxis { axis: RotationAxis::Y });
    } else if keyboard.just_pressed(KeyCode::KeyZ) {
        axis_events.send(SetRotationAxis { axis: RotationAxis::Z });
    }
}

/// Update rotation axis
pub fn update_rotation_axis(
    mut active_transform: ResMut<ActiveTransform>,
    mut axis_events: EventReader<SetRotationAxis>,
) {
    for event in axis_events.read() {
        active_transform.rotation_axis = event.axis;
        active_transform.rotation_angle = 0; // Reset angle when changing axis
        info!("Rotation axis changed to: {:?}", active_transform.rotation_axis);
    }
}

/// Handle arrow key rotation during rotate operation
pub fn handle_arrow_key_rotation(
    active_transform: Res<ActiveTransform>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut rotation_events: EventWriter<UpdateRotation>,
) {
    // Only active during rotate mode
    if active_transform.mode != TransformMode::Rotate {
        return;
    }

    // Check if UI wants keyboard input
    let ui_wants_input = contexts.ctx_mut().wants_keyboard_input();
    if ui_wants_input {
        return;
    }

    // Arrow keys rotate in 90-degree increments
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        rotation_events.send(UpdateRotation { delta: -1 });
    } else if keyboard.just_pressed(KeyCode::ArrowRight) {
        rotation_events.send(UpdateRotation { delta: 1 });
    }
}

/// Update rotation angle
pub fn update_rotation(
    mut active_transform: ResMut<ActiveTransform>,
    mut rotation_events: EventReader<UpdateRotation>,
) {
    for event in rotation_events.read() {
        active_transform.rotation_angle = (active_transform.rotation_angle + event.delta).rem_euclid(4);
        info!("Rotation angle updated to: {} ({}°)", 
              active_transform.rotation_angle, 
              active_transform.rotation_angle * 90);
    }
}

/// Calculate rotated position around pivot
fn rotate_position(pos: (i32, i32, i32), pivot: Vec3, axis: RotationAxis, angle: i32) -> (i32, i32, i32) {
    // Convert to Vec3 relative to pivot
    let rel_pos = Vec3::new(
        pos.0 as f32 - pivot.x,
        pos.1 as f32 - pivot.y,
        pos.2 as f32 - pivot.z,
    );

    // Rotate based on axis and angle (in 90-degree increments)
    let rotated = match axis {
        RotationAxis::X => {
            // Rotate around X axis (affects Y and Z)
            match angle {
                0 => rel_pos,
                1 => Vec3::new(rel_pos.x, -rel_pos.z, rel_pos.y),   // 90° CW
                2 => Vec3::new(rel_pos.x, -rel_pos.y, -rel_pos.z),  // 180°
                3 => Vec3::new(rel_pos.x, rel_pos.z, -rel_pos.y),   // 270° CW
                _ => rel_pos,
            }
        }
        RotationAxis::Y => {
            // Rotate around Y axis (affects X and Z)
            match angle {
                0 => rel_pos,
                1 => Vec3::new(rel_pos.z, rel_pos.y, -rel_pos.x),   // 90° CW
                2 => Vec3::new(-rel_pos.x, rel_pos.y, -rel_pos.z),  // 180°
                3 => Vec3::new(-rel_pos.z, rel_pos.y, rel_pos.x),   // 270° CW
                _ => rel_pos,
            }
        }
        RotationAxis::Z => {
            // Rotate around Z axis (affects X and Y)
            match angle {
                0 => rel_pos,
                1 => Vec3::new(-rel_pos.y, rel_pos.x, rel_pos.z),   // 90° CW
                2 => Vec3::new(-rel_pos.x, -rel_pos.y, rel_pos.z),  // 180°
                3 => Vec3::new(rel_pos.y, -rel_pos.x, rel_pos.z),   // 270° CW
                _ => rel_pos,
            }
        }
    };

    // Convert back to world coordinates and round to integers
    (
        (rotated.x + pivot.x).round() as i32,
        (rotated.y + pivot.y).round() as i32,
        (rotated.z + pivot.z).round() as i32,
    )
}

/// Render rotation preview meshes
pub fn render_rotation_preview(
    mut commands: Commands,
    active_transform: Res<ActiveTransform>,
    editor_state: Res<EditorState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_previews: Query<Entity, With<TransformPreview>>,
) {
    // Only render during rotate mode
    if active_transform.mode != TransformMode::Rotate {
        return;
    }

    // Check if transform state changed
    if !active_transform.is_changed() {
        return;
    }

    // Despawn existing previews
    for entity in existing_previews.iter() {
        commands.entity(entity).despawn();
    }

    // Calculate rotated positions and check collisions
    // Create a set of original positions to use as "buffer space" - these positions
    // are considered empty during rotation since the voxels are being moved
    let original_positions: std::collections::HashSet<_> =
        active_transform.selected_voxels.iter().map(|v| v.pos).collect();

    // Create sub-voxel mesh for detailed preview
    const SUB_VOXEL_SIZE: f32 = 1.0 / 8.0;
    let sub_voxel_mesh = meshes.add(Cuboid::new(SUB_VOXEL_SIZE, SUB_VOXEL_SIZE, SUB_VOXEL_SIZE));

    for voxel in &active_transform.selected_voxels {
        let new_pos = rotate_position(
            voxel.pos,
            active_transform.pivot,
            active_transform.rotation_axis,
            active_transform.rotation_angle,
        );

        // Check for collision: position is valid if it's either:
        // 1. Empty (no voxel at that position), OR
        // 2. Occupied by a voxel that's part of the selection being rotated (buffer space)
        // This ensures voxels can rotate into each other's original positions
        let is_valid = !editor_state
            .current_map
            .world
            .voxels
            .iter()
            .any(|v| v.pos == new_pos && !original_positions.contains(&v.pos));

        // Get the rotated geometry for preview
        use crate::systems::game::map::format::{RotationState, SubVoxelPattern};
        let pattern = voxel.pattern.unwrap_or(SubVoxelPattern::Full);
        let rotation_state = Some(RotationState::new(
            active_transform.rotation_axis,
            active_transform.rotation_angle,
        ));
        let geometry = pattern.geometry_with_rotation(rotation_state);

        // Create material based on validity
        let material = materials.add(StandardMaterial {
            base_color: if is_valid {
                Color::srgba(0.0, 0.5, 1.0, 0.3) // Blue for valid rotation
            } else {
                Color::srgba(1.0, 0.0, 0.0, 0.3) // Red for collision
            },
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });

        // Spawn preview sub-voxels showing the rotated geometry
        for (sub_x, sub_y, sub_z) in geometry.occupied_positions() {
            let offset = -0.5 + SUB_VOXEL_SIZE * 0.5;
            let world_x = new_pos.0 as f32 + offset + (sub_x as f32 * SUB_VOXEL_SIZE);
            let world_y = new_pos.1 as f32 + offset + (sub_y as f32 * SUB_VOXEL_SIZE);
            let world_z = new_pos.2 as f32 + offset + (sub_z as f32 * SUB_VOXEL_SIZE);

            commands.spawn((
                Mesh3d(sub_voxel_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(world_x, world_y, world_z),
                TransformPreview {
                    original_pos: voxel.pos,
                    preview_pos: new_pos,
                    is_valid,
                },
            ));
        }
    }
}

/// Confirm rotation and apply changes
pub fn confirm_rotation(
    mut active_transform: ResMut<ActiveTransform>,
    mut editor_state: ResMut<EditorState>,
    mut history: ResMut<EditorHistory>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    mut confirm_events: EventReader<ConfirmTransform>,
    mut render_events: EventWriter<RenderMapEvent>,
    mut update_events: EventWriter<UpdateSelectionHighlights>,
    preview_query: Query<&TransformPreview>,
) {
    // Only active during rotate mode
    if active_transform.mode != TransformMode::Rotate {
        return;
    }

    // Check if UI wants input
    let ui_wants_keyboard = contexts.ctx_mut().wants_keyboard_input();
    let ui_wants_pointer = contexts.ctx_mut().wants_pointer_input();
    
    // Check for confirmation input (only if UI doesn't want input)
    let should_confirm = (!ui_wants_keyboard && keyboard.just_pressed(KeyCode::Enter))
        || (!ui_wants_pointer && mouse_button.just_pressed(MouseButton::Left))
        || confirm_events.read().count() > 0;

    if !should_confirm {
        return;
    }

    // Check if all previews are valid (no collisions)
    let has_collision = preview_query.iter().any(|p| !p.is_valid);
    if has_collision {
        warn!("Cannot confirm rotation: collision detected");
        return;
    }

    // Apply the rotation
    let mut rotated_voxels = Vec::new();

    for voxel in &active_transform.selected_voxels {
        let old_pos = voxel.pos;
        let new_pos = rotate_position(
            old_pos,
            active_transform.pivot,
            active_transform.rotation_axis,
            active_transform.rotation_angle,
        );

        // Find and update voxel in map
        if let Some(map_voxel) = editor_state
            .current_map
            .world
            .voxels
            .iter_mut()
            .find(|v| v.pos == old_pos)
        {
            map_voxel.pos = new_pos;
            
            // Update or create rotation state for the voxel
            if map_voxel.pattern.is_some() {
                use crate::systems::game::map::format::RotationState;
                
                // Compose with existing rotation or create new rotation state
                map_voxel.rotation_state = Some(
                    if let Some(existing_rotation) = map_voxel.rotation_state {
                        existing_rotation.compose(
                            active_transform.rotation_axis,
                            active_transform.rotation_angle,
                        )
                    } else {
                        RotationState::new(
                            active_transform.rotation_axis,
                            active_transform.rotation_angle,
                        )
                    }
                );
            }
            
            rotated_voxels.push((old_pos, new_pos));
        }
    }

    // Create history action
    if !rotated_voxels.is_empty() {
        history.push(EditorAction::Batch {
            description: format!(
                "Rotate {} voxel{} {}° around {:?} axis",
                rotated_voxels.len(),
                if rotated_voxels.len() == 1 { "" } else { "s" },
                active_transform.rotation_angle * 90,
                active_transform.rotation_axis
            ),
            actions: rotated_voxels
                .iter()
                .map(|(old_pos, new_pos)| {
                    let voxel_data = active_transform
                        .selected_voxels
                        .iter()
                        .find(|v| v.pos == *old_pos)
                        .unwrap()
                        .clone();
                    
                    // Update rotation state for the new voxel
                    use crate::systems::game::map::format::RotationState;
                    let new_rotation_state = if voxel_data.pattern.is_some() {
                        Some(
                            if let Some(existing_rotation) = voxel_data.rotation_state {
                                existing_rotation.compose(
                                    active_transform.rotation_axis,
                                    active_transform.rotation_angle,
                                )
                            } else {
                                RotationState::new(
                                    active_transform.rotation_axis,
                                    active_transform.rotation_angle,
                                )
                            }
                        )
                    } else {
                        voxel_data.rotation_state
                    };
                    
                    // Create a remove and place action pair
                    EditorAction::Batch {
                        description: format!("Rotate voxel from {:?} to {:?}", old_pos, new_pos),
                        actions: vec![
                            EditorAction::RemoveVoxel {
                                pos: *old_pos,
                                data: voxel_data.clone(),
                            },
                            EditorAction::PlaceVoxel {
                                pos: *new_pos,
                                data: VoxelData {
                                    pos: *new_pos,
                                    rotation_state: new_rotation_state,
                                    ..voxel_data
                                },
                            },
                        ],
                    }
                })
                .collect(),
        });

        editor_state.mark_modified();
        info!(
            "Rotated {} voxels {}° around {:?} axis",
            rotated_voxels.len(),
            active_transform.rotation_angle * 90,
            active_transform.rotation_axis
        );
    }

    // Update selection to new positions
    editor_state.selected_voxels.clear();
    for (_, new_pos) in rotated_voxels {
        editor_state.selected_voxels.insert(new_pos);
    }

    // Reset transform state
    *active_transform = ActiveTransform::default();

    // Trigger updates
    render_events.send(RenderMapEvent);
    update_events.send(UpdateSelectionHighlights);
}
