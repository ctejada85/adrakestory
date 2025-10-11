//! Grid visualization for the map editor.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

/// Component marking a grid entity
#[derive(Component)]
pub struct EditorGrid;

/// Create a grid mesh for the editor
pub fn create_grid_mesh(width: i32, height: i32, depth: i32, opacity: f32) -> Mesh {
    let mut positions = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    let color = [0.5, 0.5, 0.5, opacity];

    // Helper to add a line
    let mut add_line = |start: Vec3, end: Vec3| {
        let start_idx = positions.len() as u32;
        positions.push([start.x, start.y, start.z]);
        positions.push([end.x, end.y, end.z]);
        colors.push(color);
        colors.push(color);
        indices.push(start_idx);
        indices.push(start_idx + 1);
    };

    // Grid lines parallel to X axis (along width)
    for y in 0..=height {
        for z in 0..=depth {
            let y_pos = y as f32;
            let z_pos = z as f32;
            add_line(
                Vec3::new(0.0, y_pos, z_pos),
                Vec3::new(width as f32, y_pos, z_pos),
            );
        }
    }

    // Grid lines parallel to Y axis (along height)
    for x in 0..=width {
        for z in 0..=depth {
            let x_pos = x as f32;
            let z_pos = z as f32;
            add_line(
                Vec3::new(x_pos, 0.0, z_pos),
                Vec3::new(x_pos, height as f32, z_pos),
            );
        }
    }

    // Grid lines parallel to Z axis (along depth)
    for x in 0..=width {
        for y in 0..=height {
            let x_pos = x as f32;
            let y_pos = y as f32;
            add_line(
                Vec3::new(x_pos, y_pos, 0.0),
                Vec3::new(x_pos, y_pos, depth as f32),
            );
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::LineList, Default::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Spawn the editor grid
pub fn spawn_grid(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    width: i32,
    height: i32,
    depth: i32,
    opacity: f32,
) -> Entity {
    let mesh = create_grid_mesh(width, height, depth, opacity);

    commands
        .spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.5, 0.5, 0.5, opacity),
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::default(),
            EditorGrid,
        ))
        .id()
}

/// Update grid visibility based on editor state
pub fn update_grid_visibility(
    editor_state: Res<crate::editor::state::EditorState>,
    mut grid_query: Query<&mut Visibility, With<EditorGrid>>,
) {
    for mut visibility in grid_query.iter_mut() {
        *visibility = if editor_state.show_grid {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

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
fn create_cursor_mesh() -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();

    // Cube vertices
    let vertices = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
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
    editor_state: Res<crate::editor::state::EditorState>,
    mut cursor_query: Query<&mut Transform, With<CursorIndicator>>,
) {
    for mut transform in cursor_query.iter_mut() {
        if let Some(grid_pos) = editor_state.cursor_grid_pos {
            // Show cursor at grid position
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
    fn test_create_grid_mesh() {
        let mesh = create_grid_mesh(2, 2, 2, 0.5);

        // Verify mesh was created
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        assert!(mesh.indices().is_some());
    }

    #[test]
    fn test_create_cursor_mesh() {
        let mesh = create_cursor_mesh();

        // Verify mesh was created with correct topology
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        assert!(mesh.indices().is_some());
    }
}
