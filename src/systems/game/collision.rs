//! Collision detection and spatial query utilities for the voxel game.
//!
//! This module provides helper functions for:
//! - Calculating sub-voxel world positions
//! - Getting sub-voxel bounding boxes
//! - Checking collisions between player and sub-voxels

use super::components::{Player, SubVoxel};
use super::resources::SpatialGrid;
use bevy::prelude::*;

const SUB_VOXEL_SIZE: f32 = 1.0 / 8.0;
const STEP_UP_TOLERANCE: f32 = 0.02;

/// Result of a collision check, including step-up information.
#[derive(Debug, Clone, Copy)]
pub struct CollisionResult {
    /// Whether a collision was detected
    pub has_collision: bool,
    /// Whether the player can step up onto this obstacle
    pub can_step_up: bool,
    /// The height to step up (if can_step_up is true)
    pub step_up_height: f32,
    /// The new Y position after step-up (center of player cylinder)
    pub new_y: f32,
}

impl CollisionResult {
    /// No collision detected
    pub fn no_collision() -> Self {
        Self {
            has_collision: false,
            can_step_up: false,
            step_up_height: 0.0,
            new_y: 0.0,
        }
    }

    /// Blocking collision (cannot step up)
    pub fn blocking() -> Self {
        Self {
            has_collision: true,
            can_step_up: false,
            step_up_height: 0.0,
            new_y: 0.0,
        }
    }

    /// Step-up collision (can walk up)
    pub fn step_up(height: f32, new_y: f32) -> Self {
        Self {
            has_collision: true,
            can_step_up: true,
            step_up_height: height,
            new_y,
        }
    }
}

/// Parameters for collision checking, grouping position and cylinder dimensions.
#[derive(Debug, Clone, Copy)]
pub struct CollisionParams {
    /// X position of the cylinder center
    pub x: f32,
    /// Y position of the cylinder center
    pub y: f32,
    /// Z position of the cylinder center
    pub z: f32,
    /// Horizontal radius of the collision cylinder
    pub radius: f32,
    /// Vertical half-height of the collision cylinder
    pub half_height: f32,
    /// Y position of the floor the player is currently standing on
    pub current_floor_y: f32,
}

/// Get the axis-aligned bounding box (AABB) of a sub-voxel.
///
/// This function now returns the cached bounds from the SubVoxel component,
/// eliminating the need to recalculate them every frame.
///
/// # Arguments
/// * `sub_voxel` - The sub-voxel component
///
/// # Returns
/// A tuple of (min, max) corners of the AABB
#[inline]
pub fn get_sub_voxel_bounds(sub_voxel: &SubVoxel) -> (Vec3, Vec3) {
    sub_voxel.bounds
}

/// Check if a player cylinder collides with any sub-voxels at the given position.
///
/// This function uses the spatial grid for efficient collision detection,
/// only checking sub-voxels that are near the player's position.
///
/// The player uses a cylinder collider with:
/// - `radius` for horizontal collision (XZ plane)
/// - `half_height` for vertical extent from center (Y axis)
///
/// # Arguments
/// * `spatial_grid` - The spatial partitioning grid for efficient queries
/// * `sub_voxel_query` - Query to access sub-voxel components
/// * `params` - Collision parameters (position, dimensions, floor height)
///
/// # Returns
/// A `CollisionResult` indicating collision status and step-up information
pub fn check_sub_voxel_collision(
    spatial_grid: &SpatialGrid,
    sub_voxel_query: &Query<&SubVoxel, Without<Player>>,
    params: CollisionParams,
) -> CollisionResult {
    let collision_radius = params.radius;

    // Calculate player's AABB for grid lookup (cylinder bounds)
    let player_min = Vec3::new(
        params.x - collision_radius,
        params.y - params.half_height,
        params.z - collision_radius,
    );
    let player_max = Vec3::new(
        params.x + collision_radius,
        params.y + params.half_height,
        params.z + collision_radius,
    );

    let relevant_sub_voxel_entities = spatial_grid.get_entities_in_aabb(player_min, player_max);

    // Track potential step-up collision
    let mut step_up_candidate: Option<f32> = None;

    // Check all relevant sub-voxels for collision
    for entity in relevant_sub_voxel_entities.iter() {
        if let Ok(sub_voxel) = sub_voxel_query.get(*entity) {
            let (min, max) = get_sub_voxel_bounds(sub_voxel);

            // Skip sub-voxels that are floor/ground (below player's feet)
            // We use a small threshold to avoid blocking movement on flat ground
            let player_bottom = params.y - params.half_height;
            if max.y <= player_bottom + 0.01 {
                continue;
            }

            // Skip if sub-voxel is too far above the player's top
            if min.y > params.y + params.half_height {
                continue;
            }

            // Quick AABB check for horizontal bounds
            if params.x + collision_radius < min.x
                || params.x - collision_radius > max.x
                || params.z + collision_radius < min.z
                || params.z - collision_radius > max.z
            {
                continue;
            }

            // Find closest point on sub-voxel AABB to player center (horizontal only)
            let closest_x = params.x.clamp(min.x, max.x);
            let closest_z = params.z.clamp(min.z, max.z);

            // Check horizontal distance only
            let dx = params.x - closest_x;
            let dz = params.z - closest_z;
            let distance_squared = dx * dx + dz * dz;

            if distance_squared < collision_radius * collision_radius {
                // Collision detected - check if it's a step-up candidate
                let obstacle_height = max.y - params.current_floor_y;

                // Check if obstacle is within valid step-up range
                // Allow stepping up to one sub-voxel height (with tolerance for floating-point errors)
                // The obstacle must be above the current floor but not too tall
                if obstacle_height > 0.0 && obstacle_height <= SUB_VOXEL_SIZE + STEP_UP_TOLERANCE {
                    // This is a step-up candidate - track the highest one
                    step_up_candidate = Some(step_up_candidate.unwrap_or(0.0).max(obstacle_height));
                } else if obstacle_height > SUB_VOXEL_SIZE + STEP_UP_TOLERANCE {
                    // This is a blocking collision (too tall to step up)
                    return CollisionResult::blocking();
                }
                // If obstacle_height <= 0.0, it's below us, so we ignore it
            }
        }
    }

    // If we found a step-up candidate, verify that the stepped-up position is collision-free
    if let Some(height) = step_up_candidate {
        let new_y = params.current_floor_y + height + params.half_height;

        // Check for collisions at the stepped-up position
        // We need to query a larger area that includes the new height
        let stepped_min = Vec3::new(
            params.x - collision_radius,
            new_y - params.half_height,
            params.z - collision_radius,
        );
        let stepped_max = Vec3::new(
            params.x + collision_radius,
            new_y + params.half_height,
            params.z + collision_radius,
        );

        let stepped_entities = spatial_grid.get_entities_in_aabb(stepped_min, stepped_max);

        // Check if player body would collide at the new height
        for entity in stepped_entities {
            if let Ok(sub_voxel) = sub_voxel_query.get(entity) {
                let (min, max) = get_sub_voxel_bounds(sub_voxel);

                // At the new height, check if this sub-voxel would block the player's body
                // The player's new bottom is at the stepped-up floor level
                let new_bottom = new_y - params.half_height;
                let new_top = new_y + params.half_height;

                // Skip sub-voxels that are now below the player's feet (floor we're standing on)
                if max.y <= new_bottom + 0.01 {
                    continue;
                }

                // Skip sub-voxels above the player
                if min.y >= new_top {
                    continue;
                }

                // Check horizontal overlap with cylinder
                let closest_x = params.x.clamp(min.x, max.x);
                let closest_z = params.z.clamp(min.z, max.z);
                let dx = params.x - closest_x;
                let dz = params.z - closest_z;
                let distance_squared = dx * dx + dz * dz;

                if distance_squared < collision_radius * collision_radius {
                    // There's a collision at the stepped-up position - block movement
                    return CollisionResult::blocking();
                }
            }
        }

        // Step-up is valid and the new position is collision-free
        return CollisionResult::step_up(height, new_y);
    }

    // No collision
    CollisionResult::no_collision()
}

/// Check if a cylinder intersects with an AABB (axis-aligned bounding box).
///
/// This is a pure function useful for testing collision logic in isolation.
///
/// # Arguments
/// * `cylinder_center` - Center position of the cylinder (x, y, z)
/// * `cylinder_radius` - Horizontal radius of the cylinder
/// * `cylinder_half_height` - Vertical half-height of the cylinder
/// * `aabb_min` - Minimum corner of the AABB
/// * `aabb_max` - Maximum corner of the AABB
///
/// # Returns
/// `true` if the cylinder intersects the AABB, `false` otherwise
pub fn cylinder_aabb_intersects(
    cylinder_center: Vec3,
    cylinder_radius: f32,
    cylinder_half_height: f32,
    aabb_min: Vec3,
    aabb_max: Vec3,
) -> bool {
    // Check vertical overlap first (cheaper)
    let cylinder_bottom = cylinder_center.y - cylinder_half_height;
    let cylinder_top = cylinder_center.y + cylinder_half_height;

    if cylinder_top < aabb_min.y || cylinder_bottom > aabb_max.y {
        return false;
    }

    // Find closest point on AABB to cylinder center (horizontal only)
    let closest_x = cylinder_center.x.clamp(aabb_min.x, aabb_max.x);
    let closest_z = cylinder_center.z.clamp(aabb_min.z, aabb_max.z);

    // Check horizontal distance
    let dx = cylinder_center.x - closest_x;
    let dz = cylinder_center.z - closest_z;
    let distance_squared = dx * dx + dz * dz;

    distance_squared < cylinder_radius * cylinder_radius
}

#[cfg(test)]
mod tests {
    use super::*;

    // CollisionResult tests
    #[test]
    fn test_collision_result_no_collision() {
        let result = CollisionResult::no_collision();
        assert!(!result.has_collision);
        assert!(!result.can_step_up);
        assert_eq!(result.step_up_height, 0.0);
    }

    #[test]
    fn test_collision_result_blocking() {
        let result = CollisionResult::blocking();
        assert!(result.has_collision);
        assert!(!result.can_step_up);
        assert_eq!(result.step_up_height, 0.0);
    }

    #[test]
    fn test_collision_result_step_up() {
        let result = CollisionResult::step_up(0.125, 1.5);
        assert!(result.has_collision);
        assert!(result.can_step_up);
        assert_eq!(result.step_up_height, 0.125);
        assert_eq!(result.new_y, 1.5);
    }

    // cylinder_aabb_intersects tests
    #[test]
    fn test_cylinder_aabb_no_intersection_above() {
        // Cylinder is above the AABB
        let result = cylinder_aabb_intersects(
            Vec3::new(0.0, 5.0, 0.0),  // cylinder center
            0.2,                        // radius
            0.4,                        // half_height
            Vec3::new(-1.0, 0.0, -1.0), // aabb min
            Vec3::new(1.0, 1.0, 1.0),   // aabb max
        );
        assert!(!result);
    }

    #[test]
    fn test_cylinder_aabb_no_intersection_below() {
        // Cylinder is below the AABB
        let result = cylinder_aabb_intersects(
            Vec3::new(0.0, -5.0, 0.0),  // cylinder center
            0.2,                         // radius
            0.4,                         // half_height
            Vec3::new(-1.0, 0.0, -1.0),  // aabb min
            Vec3::new(1.0, 1.0, 1.0),    // aabb max
        );
        assert!(!result);
    }

    #[test]
    fn test_cylinder_aabb_no_intersection_horizontal() {
        // Cylinder is horizontally far from AABB
        let result = cylinder_aabb_intersects(
            Vec3::new(5.0, 0.5, 0.0),   // cylinder center
            0.2,                         // radius
            0.4,                         // half_height
            Vec3::new(-1.0, 0.0, -1.0), // aabb min
            Vec3::new(1.0, 1.0, 1.0),   // aabb max
        );
        assert!(!result);
    }

    #[test]
    fn test_cylinder_aabb_intersection_center() {
        // Cylinder center is inside AABB
        let result = cylinder_aabb_intersects(
            Vec3::new(0.0, 0.5, 0.0),   // cylinder center
            0.2,                         // radius
            0.4,                         // half_height
            Vec3::new(-1.0, 0.0, -1.0), // aabb min
            Vec3::new(1.0, 1.0, 1.0),   // aabb max
        );
        assert!(result);
    }

    #[test]
    fn test_cylinder_aabb_intersection_edge() {
        // Cylinder edge touches AABB
        let result = cylinder_aabb_intersects(
            Vec3::new(1.1, 0.5, 0.0),   // cylinder center just outside
            0.2,                         // radius extends into AABB
            0.4,                         // half_height
            Vec3::new(-1.0, 0.0, -1.0), // aabb min
            Vec3::new(1.0, 1.0, 1.0),   // aabb max
        );
        assert!(result);
    }

    #[test]
    fn test_cylinder_aabb_intersection_corner() {
        // Test corner case - cylinder near corner of AABB
        let result = cylinder_aabb_intersects(
            Vec3::new(1.1, 0.5, 1.1),   // cylinder center near corner
            0.2,                         // radius
            0.4,                         // half_height
            Vec3::new(0.0, 0.0, 0.0),   // aabb min
            Vec3::new(1.0, 1.0, 1.0),   // aabb max
        );
        // Distance to corner is sqrt(0.1^2 + 0.1^2) = ~0.141, less than radius 0.2
        assert!(result);
    }

    #[test]
    fn test_cylinder_aabb_no_intersection_corner() {
        // Cylinder near corner but not touching
        let result = cylinder_aabb_intersects(
            Vec3::new(1.3, 0.5, 1.3),   // cylinder center further from corner
            0.2,                         // radius
            0.4,                         // half_height
            Vec3::new(0.0, 0.0, 0.0),   // aabb min
            Vec3::new(1.0, 1.0, 1.0),   // aabb max
        );
        // Distance to corner is sqrt(0.3^2 + 0.3^2) = ~0.424, greater than radius 0.2
        assert!(!result);
    }

    #[test]
    fn test_cylinder_aabb_vertical_touch() {
        // Cylinder just barely overlaps vertically
        let result = cylinder_aabb_intersects(
            Vec3::new(0.0, 1.3, 0.0),   // cylinder center
            0.2,                         // radius
            0.4,                         // half_height, bottom at 0.9
            Vec3::new(-1.0, 0.0, -1.0), // aabb min
            Vec3::new(1.0, 1.0, 1.0),   // aabb max
        );
        assert!(result);
    }

    // CollisionParams tests
    #[test]
    fn test_collision_params_creation() {
        let params = CollisionParams {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            radius: 0.2,
            half_height: 0.4,
            current_floor_y: 1.6,
        };
        assert_eq!(params.x, 1.0);
        assert_eq!(params.y, 2.0);
        assert_eq!(params.z, 3.0);
        assert_eq!(params.radius, 0.2);
        assert_eq!(params.half_height, 0.4);
        assert_eq!(params.current_floor_y, 1.6);
    }
}
