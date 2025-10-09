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

/// Result of a collision check, including step-up information.
#[derive(Debug, Clone, Copy)]
pub struct CollisionResult {
    /// Whether a collision was detected
    pub has_collision: bool,
    /// Whether the player can step up onto this obstacle
    pub can_step_up: bool,
    /// The height to step up (if can_step_up is true)
    pub step_up_height: f32,
}

impl CollisionResult {
    /// No collision detected
    pub fn no_collision() -> Self {
        Self {
            has_collision: false,
            can_step_up: false,
            step_up_height: 0.0,
        }
    }

    /// Blocking collision (cannot step up)
    pub fn blocking() -> Self {
        Self {
            has_collision: true,
            can_step_up: false,
            step_up_height: 0.0,
        }
    }

    /// Step-up collision (can walk up)
    pub fn step_up(height: f32) -> Self {
        Self {
            has_collision: true,
            can_step_up: true,
            step_up_height: height,
        }
    }
}

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
/// * `current_floor_y` - The Y position of the floor the player is currently standing on
///
/// # Returns
/// A `CollisionResult` indicating collision status and step-up information
pub fn check_sub_voxel_collision(
    spatial_grid: &SpatialGrid,
    sub_voxel_query: &Query<&SubVoxel, Without<Player>>,
    x: f32,
    y: f32,
    z: f32,
    radius: f32,
    current_floor_y: f32,
) -> CollisionResult {
    // Use slightly smaller collision radius for tighter fit
    let collision_radius = radius;

    // Calculate player's AABB for grid lookup
    let player_min = Vec3::new(x - collision_radius, y - radius, z - collision_radius);
    let player_max = Vec3::new(x + collision_radius, y + radius, z + collision_radius);

    let relevant_sub_voxel_entities = spatial_grid.get_entities_in_aabb(player_min, player_max);

    // Track potential step-up collision
    let mut step_up_candidate: Option<f32> = None;

    // Check all relevant sub-voxels for collision
    for entity in relevant_sub_voxel_entities {
        if let Ok(sub_voxel) = sub_voxel_query.get(entity) {
            let (min, max) = get_sub_voxel_bounds(sub_voxel);

            // Skip sub-voxels that are floor/ground (below player's feet)
            // We use a small threshold to avoid blocking movement on flat ground
            let player_bottom = y - radius;
            if max.y <= player_bottom + 0.01 {
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
                // Collision detected - check if it's a step-up candidate
                let obstacle_height = max.y - current_floor_y;
                let height_diff = (obstacle_height - SUB_VOXEL_SIZE).abs();

                // Check if obstacle is exactly one sub-voxel height above current floor
                // Use small epsilon for floating-point comparison
                if height_diff < 0.01 && obstacle_height > 0.0 {
                    // This is a step-up candidate
                    step_up_candidate = Some(obstacle_height);
                } else {
                    // This is a blocking collision (too tall or wrong height)
                    return CollisionResult::blocking();
                }
            }
        }
    }

    // If we found a step-up candidate, return it
    if let Some(height) = step_up_candidate {
        return CollisionResult::step_up(height);
    }

    // No collision
    CollisionResult::no_collision()
}
