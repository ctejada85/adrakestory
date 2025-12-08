//! Infinite grid visualization for the map editor.
//!
//! This module provides an infinite grid that spans in all directions,
//! dynamically regenerating based on camera position for efficient rendering.
//!
//! ## Optimizations
//! - **Distance culling**: Only renders grid within render_distance of camera
//! - **Frustum culling**: Only generates grid lines visible in the camera's view frustum
//! - **Regeneration threshold**: Avoids regenerating grid on small camera movements

use bevy::math::{Affine3A, Vec3A};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::primitives::{Aabb, Frustum};

/// Component marking a grid entity
#[derive(Component)]
pub struct EditorGrid;

/// Resource for infinite grid configuration
#[derive(Resource, Clone)]
pub struct InfiniteGridConfig {
    /// Grid line spacing (1.0 for voxel alignment)
    pub spacing: f32,

    /// How far from camera to render grid (in world units)
    pub render_distance: f32,

    /// Every Nth line is rendered as a major line (thicker/different color)
    pub major_line_interval: i32,

    /// Grid opacity
    pub opacity: f32,

    /// Regular grid line color
    pub color: Color,

    /// Major grid line color (every Nth line)
    pub major_color: Color,

    /// Last camera position (for regeneration detection)
    pub last_camera_pos: Vec3,

    /// Threshold for camera movement before regenerating grid
    pub regeneration_threshold: f32,
}

impl Default for InfiniteGridConfig {
    fn default() -> Self {
        Self {
            spacing: 1.0,
            render_distance: 100.0, // Base render distance, scales with camera distance
            major_line_interval: 10,
            opacity: 0.3,
            color: Color::srgba(0.5, 0.5, 0.5, 0.3),
            major_color: Color::srgba(0.7, 0.7, 0.7, 0.5),
            last_camera_pos: Vec3::ZERO,
            regeneration_threshold: 2.0,
        }
    }
}

/// Grid bounds for rendering
#[derive(Debug, Clone, Copy)]
struct GridBounds {
    min_x: f32,
    max_x: f32,
    min_z: f32,
    max_z: f32,
}

/// Calculate grid bounds based on camera position
/// Grid lines are offset by 0.5 to align with voxel boundaries
fn calculate_grid_bounds(camera_pos: Vec3, render_distance: f32, spacing: f32) -> GridBounds {
    // Offset by 0.5 to align with voxel boundaries (voxels span from x-0.5 to x+0.5)
    let offset = 0.5;
    let min_x = ((camera_pos.x - render_distance) / spacing).floor() * spacing - offset;
    let max_x = ((camera_pos.x + render_distance) / spacing).ceil() * spacing + offset;
    let min_z = ((camera_pos.z - render_distance) / spacing).floor() * spacing - offset;
    let max_z = ((camera_pos.z + render_distance) / spacing).ceil() * spacing + offset;

    GridBounds {
        min_x,
        max_x,
        min_z,
        max_z,
    }
}

/// Calculate grid bounds with frustum culling
/// Only generates grid within the camera's view frustum
fn calculate_frustum_culled_bounds(
    camera_pos: Vec3,
    render_distance: f32,
    spacing: f32,
    frustum: Option<&Frustum>,
) -> GridBounds {
    // Start with distance-based bounds
    let mut bounds = calculate_grid_bounds(camera_pos, render_distance, spacing);

    // If no frustum available, return distance-based bounds
    let Some(frustum) = frustum else {
        return bounds;
    };

    // Test grid corners against frustum and shrink bounds
    // We test the AABB of each potential grid section against the frustum
    let grid_y = 0.0; // Grid is at Y=0
    let grid_height = 0.1; // Small height for AABB test

    // Find the tightest bounds by testing grid sections
    let section_size = spacing * 10.0; // Test in chunks of 10 grid lines

    let mut visible_min_x = f32::MAX;
    let mut visible_max_x = f32::MIN;
    let mut visible_min_z = f32::MAX;
    let mut visible_max_z = f32::MIN;

    let mut x = bounds.min_x;
    while x < bounds.max_x {
        let mut z = bounds.min_z;
        while z < bounds.max_z {
            // Create AABB for this grid section
            let section_center = Vec3A::new(x + section_size / 2.0, grid_y, z + section_size / 2.0);
            let section_half_extents =
                Vec3A::new(section_size / 2.0, grid_height, section_size / 2.0);

            let section_aabb = Aabb {
                center: section_center,
                half_extents: section_half_extents,
            };

            // Test if this section is visible in the frustum
            if frustum.intersects_obb(&section_aabb, &Affine3A::IDENTITY, true, true) {
                visible_min_x = visible_min_x.min(x);
                visible_max_x = visible_max_x.max(x + section_size);
                visible_min_z = visible_min_z.min(z);
                visible_max_z = visible_max_z.max(z + section_size);
            }

            z += section_size;
        }
        x += section_size;
    }

    // If nothing visible, return empty bounds
    if visible_min_x > visible_max_x {
        return GridBounds {
            min_x: camera_pos.x,
            max_x: camera_pos.x,
            min_z: camera_pos.z,
            max_z: camera_pos.z,
        };
    }

    // Constrain to original distance-based bounds and snap to grid
    let offset = 0.5;
    bounds.min_x = ((visible_min_x.max(bounds.min_x)) / spacing).floor() * spacing - offset;
    bounds.max_x = ((visible_max_x.min(bounds.max_x)) / spacing).ceil() * spacing + offset;
    bounds.min_z = ((visible_min_z.max(bounds.min_z)) / spacing).floor() * spacing - offset;
    bounds.max_z = ((visible_max_z.min(bounds.max_z)) / spacing).ceil() * spacing + offset;

    bounds
}

/// Check if grid should be regenerated based on camera movement
fn should_regenerate_grid(camera_pos: Vec3, last_pos: Vec3, threshold: f32) -> bool {
    let distance = camera_pos.distance(last_pos);
    distance > threshold
}

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

/// System to update infinite grid based on camera movement with frustum culling
pub fn update_infinite_grid(
    mut config: ResMut<InfiniteGridConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    camera_query: Query<
        (&Transform, &Frustum, &crate::editor::camera::EditorCamera),
        With<crate::editor::camera::EditorCamera>,
    >,
    grid_query: Query<(Entity, &Mesh3d), With<EditorGrid>>,
) {
    let Ok((camera_transform, frustum, editor_camera)) = camera_query.get_single() else {
        return;
    };

    let camera_pos = camera_transform.translation;

    // Scale render distance based on camera distance from target (zoom level)
    // The further the camera is zoomed out, the larger the grid render area
    let base_render_distance = config.render_distance;
    let camera_height = camera_pos.y.abs();
    let camera_distance = editor_camera.distance;

    // Use the larger of camera height or camera distance to determine grid extent
    // Multiply by a factor to ensure grid extends beyond visible area
    let dynamic_render_distance =
        (base_render_distance + camera_height * 2.0 + camera_distance * 1.5)
            .max(base_render_distance);

    // Check if we need to regenerate the grid (also regenerate if zoom changed significantly)
    let zoom_changed =
        (camera_distance - config.last_camera_pos.y.abs()).abs() > config.regeneration_threshold;
    if !should_regenerate_grid(
        camera_pos,
        config.last_camera_pos,
        config.regeneration_threshold,
    ) && !zoom_changed
    {
        return;
    }

    // Update last camera position
    config.last_camera_pos = camera_pos;

    // Create a temporary config with dynamic render distance
    let dynamic_config = InfiniteGridConfig {
        render_distance: dynamic_render_distance,
        ..config.clone()
    };

    // Regenerate grid mesh with frustum culling and dynamic render distance
    let new_mesh = create_infinite_grid_mesh(&dynamic_config, camera_pos, Some(frustum));

    // Update existing grid entity
    for (_entity, mesh_handle) in grid_query.iter() {
        if let Some(mesh) = meshes.get_mut(mesh_handle.0.id()) {
            *mesh = new_mesh.clone();
        }
    }
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
    mut cursor_query: Query<&mut Transform, With<CursorIndicator>>,
) {
    for mut transform in cursor_query.iter_mut() {
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
    fn test_calculate_grid_bounds() {
        let camera_pos = Vec3::new(5.0, 0.0, 5.0);
        let bounds = calculate_grid_bounds(camera_pos, 10.0, 1.0);

        assert!(bounds.min_x <= -5.0);
        assert!(bounds.max_x >= 15.0);
        assert!(bounds.min_z <= -5.0);
        assert!(bounds.max_z >= 15.0);
    }

    #[test]
    fn test_should_regenerate_grid() {
        let pos1 = Vec3::new(0.0, 0.0, 0.0);
        let pos2 = Vec3::new(1.0, 0.0, 0.0);
        let pos3 = Vec3::new(3.0, 0.0, 0.0);

        assert!(!should_regenerate_grid(pos1, pos2, 2.0));
        assert!(should_regenerate_grid(pos1, pos3, 2.0));
    }

    #[test]
    fn test_create_infinite_grid_mesh() {
        let config = InfiniteGridConfig::default();
        let camera_pos = Vec3::ZERO;
        let mesh = create_infinite_grid_mesh(&config, camera_pos, None);

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
