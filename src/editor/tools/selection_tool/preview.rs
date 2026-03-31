//! Transform and rotation preview rendering.

use super::{ActiveTransform, TransformMode, TransformPreview};
use crate::editor::state::EditorState;
use crate::systems::game::map::format::{
    apply_orientation_matrix, axis_angle_to_matrix, multiply_matrices, world_dir_to_local,
    SubVoxelPattern, IDENTITY,
};
use crate::systems::game::map::geometry::RotationAxis;
use bevy::prelude::*;

/// Render transform previews for move and rotate modes.
///
/// Owns the full lifecycle of [`TransformPreview`] entities.
/// A single cleanup loop fires before any spawning so no entity is despawned twice.
pub fn render_transform_preview(
    mut commands: Commands,
    active_transform: Res<ActiveTransform>,
    editor_state: Res<EditorState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_previews: Query<Entity, With<TransformPreview>>,
) {
    // mode == None: clean up any leftover previews and exit
    if active_transform.mode == TransformMode::None {
        for entity in existing_previews.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Skip update when nothing has changed
    if !active_transform.is_changed() {
        return;
    }

    // Single despawn loop — runs exactly once per frame regardless of mode
    for entity in existing_previews.iter() {
        commands.entity(entity).despawn();
    }

    match active_transform.mode {
        TransformMode::None => {} // handled above
        TransformMode::Move => spawn_move_previews(
            &mut commands,
            &active_transform,
            &editor_state,
            &mut meshes,
            &mut materials,
        ),
        TransformMode::Rotate => spawn_rotation_previews(
            &mut commands,
            &active_transform,
            &editor_state,
            &mut meshes,
            &mut materials,
        ),
    }
}

/// Spawn coarse 1-voxel cube previews for move mode.
fn spawn_move_previews(
    commands: &mut Commands,
    active_transform: &ActiveTransform,
    editor_state: &EditorState,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let offset = active_transform.current_offset;
    let original_positions: std::collections::HashSet<_> = active_transform
        .selected_voxels
        .iter()
        .map(|v| v.pos)
        .collect();

    let preview_mesh = meshes.add(Cuboid::new(0.95, 0.95, 0.95));

    for voxel in &active_transform.selected_voxels {
        let new_pos = (
            voxel.pos.0 + offset.x,
            voxel.pos.1 + offset.y,
            voxel.pos.2 + offset.z,
        );

        let is_valid = !editor_state
            .current_map
            .world
            .voxels
            .iter()
            .any(|v| v.pos == new_pos && !original_positions.contains(&v.pos));

        let material = materials.add(StandardMaterial {
            base_color: if is_valid {
                Color::srgba(0.0, 1.0, 0.0, 0.3)
            } else {
                Color::srgba(1.0, 0.0, 0.0, 0.3)
            },
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });

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

/// Spawn sub-voxel geometry previews for rotate mode.
fn spawn_rotation_previews(
    commands: &mut Commands,
    active_transform: &ActiveTransform,
    editor_state: &EditorState,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let original_positions: std::collections::HashSet<_> = active_transform
        .selected_voxels
        .iter()
        .map(|v| v.pos)
        .collect();

    // Build fence neighbour set for context-aware geometry (same logic as renderer/spawner).
    let fence_positions: std::collections::HashSet<(i32, i32, i32)> = editor_state
        .current_map
        .world
        .voxels
        .iter()
        .filter(|v| v.pattern.map_or(false, |p| p.is_fence()))
        .map(|v| v.pos)
        .collect();

    const SUB_VOXEL_SIZE: f32 = 1.0 / 8.0;
    let sub_voxel_mesh = meshes.add(Cuboid::new(SUB_VOXEL_SIZE, SUB_VOXEL_SIZE, SUB_VOXEL_SIZE));

    for voxel in &active_transform.selected_voxels {
        let new_pos = rotate_position(
            voxel.pos,
            active_transform.pivot,
            active_transform.rotation_axis,
            active_transform.rotation_angle,
        );

        let is_valid = !editor_state
            .current_map
            .world
            .voxels
            .iter()
            .any(|v| v.pos == new_pos && !original_positions.contains(&v.pos));

        let pattern = voxel.pattern.unwrap_or(SubVoxelPattern::Full);
        let delta_matrix = axis_angle_to_matrix(
            active_transform.rotation_axis,
            active_transform.rotation_angle,
        );

        // Existing orientation of this voxel (before the preview rotation).
        let existing_orientation = voxel
            .rotation
            .and_then(|i| editor_state.current_map.orientations.get(i));

        // Compose delta × existing so the preview shows the final rotated result.
        let composed_matrix =
            multiply_matrices(&delta_matrix, existing_orientation.unwrap_or(&IDENTITY));

        // For fence patterns, generate neighbour-aware geometry first in local space,
        // then rotate it to the final composed orientation.
        // Neighbour booleans must be in the fence's LOCAL frame (using its existing orientation),
        // since the voxel's world position hasn't changed during the preview.
        let base_geometry = if pattern.is_fence() {
            let (x, y, z) = voxel.pos;

            let world_dirs: [([i32; 3], (i32, i32, i32)); 6] = [
                ([-1, 0, 0], (x - 1, y, z)),
                ([1, 0, 0], (x + 1, y, z)),
                ([0, 0, -1], (x, y, z - 1)),
                ([0, 0, 1], (x, y, z + 1)),
                ([0, -1, 0], (x, y - 1, z)),
                ([0, 1, 0], (x, y + 1, z)),
            ];

            let mut local_neg_x = false;
            let mut local_pos_x = false;
            let mut local_neg_z = false;
            let mut local_pos_z = false;

            for (world_dir, neighbor_pos) in &world_dirs {
                if fence_positions.contains(neighbor_pos) {
                    match world_dir_to_local(existing_orientation, *world_dir) {
                        [-1, 0, 0] => local_neg_x = true,
                        [1, 0, 0] => local_pos_x = true,
                        [0, 0, -1] => local_neg_z = true,
                        [0, 0, 1] => local_pos_z = true,
                        _ => {}
                    }
                }
            }

            pattern.fence_geometry_with_neighbors((
                local_neg_x,
                local_pos_x,
                local_neg_z,
                local_pos_z,
            ))
        } else {
            pattern.geometry()
        };
        let geometry = apply_orientation_matrix(base_geometry, &composed_matrix);

        let material = materials.add(StandardMaterial {
            base_color: if is_valid {
                Color::srgba(0.0, 0.5, 1.0, 0.3)
            } else {
                Color::srgba(1.0, 0.0, 0.0, 0.3)
            },
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });

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

/// Calculate rotated position around pivot (used by rotation preview and confirmation).
pub fn rotate_position(
    pos: (i32, i32, i32),
    pivot: Vec3,
    axis: RotationAxis,
    angle: i32,
) -> (i32, i32, i32) {
    let rel_pos = Vec3::new(
        pos.0 as f32 - pivot.x,
        pos.1 as f32 - pivot.y,
        pos.2 as f32 - pivot.z,
    );

    let rotated = match axis {
        RotationAxis::X => match angle {
            0 => rel_pos,
            1 => Vec3::new(rel_pos.x, -rel_pos.z, rel_pos.y),
            2 => Vec3::new(rel_pos.x, -rel_pos.y, -rel_pos.z),
            3 => Vec3::new(rel_pos.x, rel_pos.z, -rel_pos.y),
            _ => rel_pos,
        },
        RotationAxis::Y => match angle {
            0 => rel_pos,
            1 => Vec3::new(rel_pos.z, rel_pos.y, -rel_pos.x),
            2 => Vec3::new(-rel_pos.x, rel_pos.y, -rel_pos.z),
            3 => Vec3::new(-rel_pos.z, rel_pos.y, rel_pos.x),
            _ => rel_pos,
        },
        RotationAxis::Z => match angle {
            0 => rel_pos,
            1 => Vec3::new(-rel_pos.y, rel_pos.x, rel_pos.z),
            2 => Vec3::new(-rel_pos.x, -rel_pos.y, rel_pos.z),
            3 => Vec3::new(rel_pos.y, -rel_pos.x, rel_pos.z),
            _ => rel_pos,
        },
    };

    (
        (rotated.x + pivot.x).round() as i32,
        (rotated.y + pivot.y).round() as i32,
        (rotated.z + pivot.z).round() as i32,
    )
}
