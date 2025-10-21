//! Infinite grid visualization for the map editor.
//!
//! This module provides an infinite grid that spans in all directions,
//! dynamically regenerating based on camera position for efficient rendering.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

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
            render_distance: 50.0,
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

/// Check if grid should be regenerated based on camera movement
fn should_regenerate_grid(camera_pos: Vec3, last_pos: Vec3, threshold: f32) -> bool {
    let distance = camera_pos.distance(last_pos);
    distance > threshold
}

/// Create an infinite grid mesh based on camera position
pub fn create_infinite_grid_mesh(config: &InfiniteGridConfig, camera_pos: Vec3) -> Mesh {
    let bounds = calculate_grid_bounds(camera_pos, config.render_distance, config.spacing);
    
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
) -> Entity {
    let mesh = create_infinite_grid_mesh(config, camera_pos);
    
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

/// System to update infinite grid based on camera movement
pub fn update_infinite_grid(
    mut config: ResMut<InfiniteGridConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    camera_query: Query<&Transform, With<crate::editor::camera::EditorCamera>>,
    grid_query: Query<(Entity, &Mesh3d), With<EditorGrid>>,
) {
    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };
    
    let camera_pos = camera_transform.translation;
    
    // Check if we need to regenerate the grid
    if !should_regenerate_grid(camera_pos, config.last_camera_pos, config.regeneration_threshold) {
        return;
    }
    
    // Update last camera position
    config.last_camera_pos = camera_pos;
    
    // Regenerate grid mesh
    let new_mesh = create_infinite_grid_mesh(&config, camera_pos);
    
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
    editor_state: Res<crate::editor::state::EditorState>,
    mut cursor_query: Query<&mut Transform, With<CursorIndicator>>,
) {
    for mut transform in cursor_query.iter_mut() {
        if let Some(grid_pos) = editor_state.cursor_grid_pos {
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
        let mesh = create_infinite_grid_mesh(&config, camera_pos);
        
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
