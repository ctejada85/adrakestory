//! Infinite grid visualization for the map editor.
//!
//! This module provides an infinite grid that spans in all directions,
//! dynamically regenerating based on camera position for efficient rendering.
//!
//! ## Optimizations
//! - **Distance culling**: Only renders grid within render_distance of camera
//! - **Frustum culling**: Only generates grid lines visible in the camera's view frustum
//! - **Regeneration threshold**: Avoids regenerating grid on small camera movements

mod bounds;
mod cursor_indicator;
mod mesh;
mod systems;

pub use cursor_indicator::{spawn_cursor_indicator, update_cursor_indicator, CursorIndicator};
pub use mesh::{create_infinite_grid_mesh, spawn_infinite_grid};
pub use systems::{update_grid_visibility, update_infinite_grid};

use bevy::prelude::*;

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
