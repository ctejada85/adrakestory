//! Grid update systems.

use super::mesh::create_infinite_grid_mesh;
use super::{EditorGrid, InfiniteGridConfig};
use bevy::prelude::*;

/// Check if grid should be regenerated based on camera movement
pub fn should_regenerate_grid(camera_pos: Vec3, last_pos: Vec3, threshold: f32) -> bool {
    let distance = camera_pos.distance(last_pos);
    distance > threshold
}

/// System to update infinite grid based on camera movement with frustum culling
pub fn update_infinite_grid(
    mut config: ResMut<InfiniteGridConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    camera_query: Query<
        (
            &Transform,
            &bevy::render::primitives::Frustum,
            &crate::editor::camera::EditorCamera,
        ),
        With<crate::editor::camera::EditorCamera>,
    >,
    grid_query: Query<(Entity, &Mesh3d), With<EditorGrid>>,
) {
    let Ok((camera_transform, frustum, _editor_camera)) = camera_query.get_single() else {
        return;
    };

    let camera_pos = camera_transform.translation;

    // Scale render distance based on camera height
    // The further the camera is from ground, the larger the grid render area
    let base_render_distance = config.render_distance;
    let camera_height = camera_pos.y.abs();

    // Use camera height to determine grid extent
    // Multiply by a factor to ensure grid extends beyond visible area
    let dynamic_render_distance =
        (base_render_distance + camera_height * 3.0)
            .max(base_render_distance);

    // Check if we need to regenerate the grid
    let height_changed =
        (camera_height - config.last_camera_pos.y.abs()).abs() > config.regeneration_threshold;
    if !should_regenerate_grid(
        camera_pos,
        config.last_camera_pos,
        config.regeneration_threshold,
    ) && !height_changed
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_regenerate_grid() {
        let pos1 = Vec3::new(0.0, 0.0, 0.0);
        let pos2 = Vec3::new(1.0, 0.0, 0.0);
        let pos3 = Vec3::new(3.0, 0.0, 0.0);

        assert!(!should_regenerate_grid(pos1, pos2, 2.0));
        assert!(should_regenerate_grid(pos1, pos3, 2.0));
    }
}
