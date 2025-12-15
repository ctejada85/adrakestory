//! Grid mesh creation and spawning.

use super::bounds::calculate_frustum_culled_bounds;
use super::{EditorGrid, InfiniteGridConfig};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::primitives::Frustum;

/// Create an infinite grid mesh based on camera position with optional frustum culling
pub fn create_infinite_grid_mesh(
    config: &InfiniteGridConfig,
    camera_pos: Vec3,
    frustum: Option<&Frustum>,
) -> Mesh {
    let bounds = calculate_frustum_culled_bounds(
        camera_pos,
        config.render_distance,
        config.spacing,
        frustum,
    );

    let mut positions = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    let regular_color = [
        config.color.to_srgba().red,
        config.color.to_srgba().green,
        config.color.to_srgba().blue,
        config.opacity,
    ];

    let major_color = [
        config.major_color.to_srgba().red,
        config.major_color.to_srgba().green,
        config.major_color.to_srgba().blue,
        config.opacity * 1.5,
    ];

    // Helper to add a line
    let mut add_line = |start: Vec3, end: Vec3, is_major: bool| {
        let start_idx = positions.len() as u32;
        positions.push([start.x, start.y, start.z]);
        positions.push([end.x, end.y, end.z]);

        let color = if is_major { major_color } else { regular_color };
        colors.push(color);
        colors.push(color);

        indices.push(start_idx);
        indices.push(start_idx + 1);
    };

    // Grid lines parallel to X axis (running along width)
    // Lines at half-integer positions: -0.5, 0.5, 1.5, 2.5, ... (voxel boundaries)
    let mut z = bounds.min_z;
    while z <= bounds.max_z {
        // Major lines at integer boundaries (every Nth voxel)
        let z_voxel = (z + 0.5).floor() as i32;
        let is_major = z_voxel % config.major_line_interval == 0;
        add_line(
            Vec3::new(bounds.min_x, 0.0, z),
            Vec3::new(bounds.max_x, 0.0, z),
            is_major,
        );
        z += config.spacing;
    }

    // Grid lines parallel to Z axis (running along depth)
    // Lines at half-integer positions: -0.5, 0.5, 1.5, 2.5, ... (voxel boundaries)
    let mut x = bounds.min_x;
    while x <= bounds.max_x {
        // Major lines at integer boundaries (every Nth voxel)
        let x_voxel = (x + 0.5).floor() as i32;
        let is_major = x_voxel % config.major_line_interval == 0;
        add_line(
            Vec3::new(x, 0.0, bounds.min_z),
            Vec3::new(x, 0.0, bounds.max_z),
            is_major,
        );
        x += config.spacing;
    }

    let mut mesh = Mesh::new(PrimitiveTopology::LineList, Default::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Spawn the infinite editor grid
pub fn spawn_infinite_grid(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    config: &InfiniteGridConfig,
    camera_pos: Vec3,
    frustum: Option<&Frustum>,
) -> Entity {
    let mesh = create_infinite_grid_mesh(config, camera_pos, frustum);

    commands
        .spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: config.color,
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::default(),
            EditorGrid,
        ))
        .id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_infinite_grid_mesh() {
        let config = InfiniteGridConfig::default();
        let camera_pos = Vec3::ZERO;
        let mesh = create_infinite_grid_mesh(&config, camera_pos, None);

        // Verify mesh was created
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        assert!(mesh.indices().is_some());
    }
}
