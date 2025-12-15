//! Grid bounds calculation with frustum culling.

use bevy::math::{Affine3A, Vec3A};
use bevy::prelude::*;
use bevy::render::primitives::{Aabb, Frustum};

/// Grid bounds for rendering
#[derive(Debug, Clone, Copy)]
pub struct GridBounds {
    pub min_x: f32,
    pub max_x: f32,
    pub min_z: f32,
    pub max_z: f32,
}

/// Calculate grid bounds based on camera position
/// Grid lines are offset by 0.5 to align with voxel boundaries
pub fn calculate_grid_bounds(camera_pos: Vec3, render_distance: f32, spacing: f32) -> GridBounds {
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
pub fn calculate_frustum_culled_bounds(
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

    // Use a taller AABB to ensure intersection even when camera is close
    // This helps detect the grid plane from various angles
    let grid_height = camera_pos.y.abs().max(10.0);

    // Find the tightest bounds by testing grid sections
    // Use smaller sections when camera is close for better accuracy
    let base_section_size = spacing * 10.0;
    let section_size = if camera_pos.y.abs() < 5.0 {
        spacing * 2.0 // Smaller sections when close
    } else {
        base_section_size
    };

    let mut visible_min_x = f32::MAX;
    let mut visible_max_x = f32::MIN;
    let mut visible_min_z = f32::MAX;
    let mut visible_max_z = f32::MIN;

    let mut x = bounds.min_x;
    while x < bounds.max_x {
        let mut z = bounds.min_z;
        while z < bounds.max_z {
            // Create AABB for this grid section
            // Center the AABB vertically to span from below to above the grid plane
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

    // If nothing visible through frustum culling, fall back to distance-based bounds
    // This ensures the grid is always visible when the camera should see it
    if visible_min_x > visible_max_x {
        // Grid is always visible if camera can see ground plane
        // Return a minimum grid area around where the camera is looking
        return bounds;
    }

    // Constrain to original distance-based bounds and snap to grid
    let offset = 0.5;
    bounds.min_x = ((visible_min_x.max(bounds.min_x)) / spacing).floor() * spacing - offset;
    bounds.max_x = ((visible_max_x.min(bounds.max_x)) / spacing).ceil() * spacing + offset;
    bounds.min_z = ((visible_min_z.max(bounds.min_z)) / spacing).floor() * spacing - offset;
    bounds.max_z = ((visible_max_z.min(bounds.max_z)) / spacing).ceil() * spacing + offset;

    bounds
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
}
