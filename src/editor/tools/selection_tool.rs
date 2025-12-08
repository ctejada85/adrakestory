//! Selection tool for selecting and manipulating objects.

use crate::editor::cursor::CursorState;
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
    cursor_state: Res<CursorState>,
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

    // Get cursor world position for entity selection
    let cursor_world_pos = cursor_state.position;

    // First, try to select an entity (entities take priority)
    if let Some(world_pos) = cursor_world_pos {
        let selection_radius = 0.5;
        let mut closest_entity_index: Option<usize> = None;
        let mut closest_distance = f32::MAX;

        for (index, entity_data) in editor_state.current_map.entities.iter().enumerate() {
            let (ex, ey, ez) = entity_data.position;
            let entity_pos = Vec3::new(ex, ey, ez);
            let distance = world_pos.distance(entity_pos);

            if distance < selection_radius && distance < closest_distance {
                closest_distance = distance;
                closest_entity_index = Some(index);
            }
        }

        // If we found an entity nearby, select/deselect it
        if let Some(entity_idx) = closest_entity_index {
            // Clear voxel selection when selecting entities
            editor_state.selected_voxels.clear();

            if editor_state.selected_entities.contains(&entity_idx) {
                editor_state.selected_entities.remove(&entity_idx);
                info!("Deselected entity at index {}", entity_idx);
            } else {
                editor_state.selected_entities.clear(); // Single selection for now
                editor_state.selected_entities.insert(entity_idx);
                info!("Selected entity at index {}", entity_idx);
            }

            // Trigger highlight update
            update_events.send(UpdateSelectionHighlights);
            return;
        }
    }

    // If no entity was clicked, try voxel selection
    // Get cursor grid position
    let Some(grid_pos) = cursor_state.grid_pos else {
        return;
    };

    // Clear entity selection when selecting voxels
    editor_state.selected_entities.clear();

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

/// Calculate rotated position around pivot (used by rotation preview and confirmation)
pub fn rotate_position(pos: (i32, i32, i32), pivot: Vec3, axis: RotationAxis, angle: i32) -> (i32, i32, i32) {
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
