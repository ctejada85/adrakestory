//! Cursor indicator for voxel placement.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

/// Component for the cursor indicator
#[derive(Component)]
pub struct CursorIndicator;

/// Spawn a cursor indicator mesh
pub fn spawn_cursor_indicator(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    // Create a wireframe cube mesh
    let mesh = create_cursor_mesh();

    commands
        .spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 0.0, 0.5),
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_scale(Vec3::splat(0.0)), // Start hidden
            CursorIndicator,
        ))
        .id()
}

/// Create a wireframe cube mesh for the cursor
pub fn create_cursor_mesh() -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();

    // Cube vertices (centered at voxel position)
    let offset = -0.5;
    let size = 1.0;
    let vertices = [
        [offset, offset, offset],
        [offset + size, offset, offset],
        [offset + size, offset + size, offset],
        [offset, offset + size, offset],
        [offset, offset, offset + size],
        [offset + size, offset, offset + size],
        [offset + size, offset + size, offset + size],
        [offset, offset + size, offset + size],
    ];

    positions.extend_from_slice(&vertices);

    // Cube edges
    let edges = [
        // Bottom face
        [0, 1],
        [1, 2],
        [2, 3],
        [3, 0],
        // Top face
        [4, 5],
        [5, 6],
        [6, 7],
        [7, 4],
        // Vertical edges
        [0, 4],
        [1, 5],
        [2, 6],
        [3, 7],
    ];

    for edge in edges.iter() {
        indices.push(edge[0]);
        indices.push(edge[1]);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::LineList, Default::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Update cursor indicator position
pub fn update_cursor_indicator(
    cursor_state: Res<crate::editor::cursor::CursorState>,
    editor_state: Res<crate::editor::state::EditorState>,
    gamepad_state: Option<Res<crate::editor::camera::GamepadCameraState>>,
    mut cursor_query: Query<&mut Transform, With<CursorIndicator>>,
) {
    for mut transform in cursor_query.iter_mut() {
        // If gamepad is active, use gamepad action position
        if let Some(ref gp_state) = gamepad_state {
            if gp_state.active {
                if let Some(grid_pos) = gp_state.action_grid_pos {
                    // Show cursor at gamepad action position (centered on voxel)
                    transform.translation = Vec3::new(
                        grid_pos.0 as f32 + 0.5,
                        grid_pos.1 as f32 + 0.5,
                        grid_pos.2 as f32 + 0.5,
                    );
                    transform.scale = Vec3::splat(1.0);
                    continue;
                }
            }
        }

        // For VoxelPlace tool, show placement position (adjacent to hit face)
        if matches!(
            editor_state.active_tool,
            crate::editor::state::EditorTool::VoxelPlace { .. }
        ) {
            if let Some(placement_pos) = cursor_state.placement_pos {
                transform.translation = placement_pos;
                transform.scale = Vec3::splat(1.0);
                continue;
            }
        }

        // For other tools (Select, Remove, etc.), show original cursor position
        if let Some(grid_pos) = cursor_state.grid_pos {
            // Show cursor at grid position (centered on voxel)
            transform.translation =
                Vec3::new(grid_pos.0 as f32, grid_pos.1 as f32, grid_pos.2 as f32);
            transform.scale = Vec3::splat(1.0);
        } else {
            // Hide cursor
            transform.scale = Vec3::splat(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_cursor_mesh() {
        let mesh = create_cursor_mesh();

        // Verify mesh was created with correct topology
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        assert!(mesh.indices().is_some());
    }
}
