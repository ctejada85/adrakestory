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

/// Calculate the world position of a sub-voxel's center.
///
/// # Arguments
/// * `sub_voxel` - The sub-voxel component containing parent and local coordinates
///
/// # Returns
/// The world-space position of the sub-voxel's center
pub fn calculate_sub_voxel_world_pos(sub_voxel: &SubVoxel) -> Vec3 {
    let offset = -0.5 + SUB_VOXEL_SIZE * 0.5;
    Vec3::new(
        sub_voxel.parent_x as f32 + offset + (sub_voxel.sub_x as f32 * SUB_VOXEL_SIZE),
        sub_voxel.parent_y as f32 + offset + (sub_voxel.sub_y as f32 * SUB_VOXEL_SIZE),
        sub_voxel.parent_z as f32 + offset + (sub_voxel.sub_z as f32 * SUB_VOXEL_SIZE),
    )
}

/// Get the axis-aligned bounding box (AABB) of a sub-voxel.
///
/// # Arguments
/// * `sub_voxel` - The sub-voxel component
///
/// # Returns
/// A tuple of (min, max) corners of the AABB
pub fn get_sub_voxel_bounds(sub_voxel: &SubVoxel) -> (Vec3, Vec3) {
    let center = calculate_sub_voxel_world_pos(sub_voxel);
    let half_size = SUB_VOXEL_SIZE / 2.0;
    (
        center - Vec3::splat(half_size),
        center + Vec3::splat(half_size),
    )
}

/// Check if a player sphere collides with any sub-voxels at the given position.
///
/// This function uses the spatial grid for efficient collision detection,
/// only checking sub-voxels that are near the player's position.
///
/// # Arguments
/// * `spatial_grid` - The spatial partitioning grid for efficient queries
/// * `sub_voxel_query` - Query to access sub-voxel components
/// * `x`, `y`, `z` - The position to check for collision
/// * `radius` - The radius of the player's collision sphere
///
/// # Returns
/// `true` if a collision is detected, `false` otherwise
pub fn check_sub_voxel_collision(
    spatial_grid: &SpatialGrid,
    sub_voxel_query: &Query<&SubVoxel, Without<Player>>,
    x: f32,
    y: f32,
    z: f32,
    radius: f32,
) -> bool {
    // Use slightly smaller collision radius for tighter fit
    let collision_radius = radius;

    // Calculate player's AABB for grid lookup
    let player_min = Vec3::new(x - collision_radius, y - radius, z - collision_radius);
    let player_max = Vec3::new(x + collision_radius, y + radius, z + collision_radius);

    let relevant_sub_voxel_entities = spatial_grid.get_entities_in_aabb(player_min, player_max);

    // Check all relevant sub-voxels for collision
    for entity in relevant_sub_voxel_entities {
        if let Ok(sub_voxel) = sub_voxel_query.get(entity) {
            let (min, max) = get_sub_voxel_bounds(sub_voxel);

            // Only check sub-voxels that overlap with player's height
            // Skip sub-voxels that are completely below the player's bottom
            if max.y < y - radius {
                continue;
            }

            // Skip if sub-voxel is too far above the player's top
            if min.y > y + radius {
                continue;
            }

            // Quick AABB check for horizontal bounds
            if x + collision_radius < min.x
                || x - collision_radius > max.x
                || z + collision_radius < min.z
                || z - collision_radius > max.z
            {
                continue;
            }

            // Find closest point on sub-voxel AABB to player center (horizontal only)
            let closest_x = x.clamp(min.x, max.x);
            let closest_z = z.clamp(min.z, max.z);

            // Check horizontal distance only
            let dx = x - closest_x;
            let dz = z - closest_z;
            let distance_squared = dx * dx + dz * dz;

            if distance_squared < collision_radius * collision_radius {
                return true; // Collision detected
            }
        }
    }

    false
}
