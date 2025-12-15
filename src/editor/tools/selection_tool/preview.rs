//! Transform and rotation preview rendering.

use super::{ActiveTransform, TransformMode, TransformPreview};
use crate::editor::state::EditorState;
use crate::systems::game::map::geometry::RotationAxis;
use bevy::prelude::*;

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
    let original_positions: std::collections::HashSet<_> = active_transform
        .selected_voxels
        .iter()
        .map(|v| v.pos)
        .collect();

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
    let original_positions: std::collections::HashSet<_> = active_transform
        .selected_voxels
        .iter()
        .map(|v| v.pos)
        .collect();

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
pub fn rotate_position(
    pos: (i32, i32, i32),
    pivot: Vec3,
    axis: RotationAxis,
    angle: i32,
) -> (i32, i32, i32) {
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
                1 => Vec3::new(rel_pos.x, -rel_pos.z, rel_pos.y), // 90° CW
                2 => Vec3::new(rel_pos.x, -rel_pos.y, -rel_pos.z), // 180°
                3 => Vec3::new(rel_pos.x, rel_pos.z, -rel_pos.y), // 270° CW
                _ => rel_pos,
            }
        }
        RotationAxis::Y => {
            // Rotate around Y axis (affects X and Z)
            match angle {
                0 => rel_pos,
                1 => Vec3::new(rel_pos.z, rel_pos.y, -rel_pos.x), // 90° CW
                2 => Vec3::new(-rel_pos.x, rel_pos.y, -rel_pos.z), // 180°
                3 => Vec3::new(-rel_pos.z, rel_pos.y, rel_pos.x), // 270° CW
                _ => rel_pos,
            }
        }
        RotationAxis::Z => {
            // Rotate around Z axis (affects X and Y)
            match angle {
                0 => rel_pos,
                1 => Vec3::new(-rel_pos.y, rel_pos.x, rel_pos.z), // 90° CW
                2 => Vec3::new(-rel_pos.x, -rel_pos.y, rel_pos.z), // 180°
                3 => Vec3::new(rel_pos.y, -rel_pos.x, rel_pos.z), // 270° CW
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
