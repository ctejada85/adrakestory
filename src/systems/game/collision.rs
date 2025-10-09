//! Collision detection and spatial query utilities for the voxel game.
//!
//! This module provides helper functions for:
//! - Calculating sub-voxel world positions
//! - Getting sub-voxel bounding boxes
//! - Checking collisions between player and sub-voxels
//! - Determining step-up heights for player movement

use super::components::{Player, SubVoxel};
use super::resources::SpatialGrid;
use bevy::prelude::*;

const SUB_VOXEL_SIZE: f32 = 1.0 / 8.0;

/// Struct to group player collision parameters for step-up checks.
pub struct PlayerCollision {
    pub pos: Vec3,
    pub radius: f32,
    pub current_y: f32,
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

            // Only check sub-voxels that overlap with player's height, but not the floor
            // Skip sub-voxels that are below player's center (these are floor/ground)
            if max.y <= y - radius * 0.5 {
                continue;
            }

            // Skip if sub-voxel is too far above
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

/// Determine the height the player should step up to when encountering a step.
///
/// This function checks if the player can step up onto a platform by analyzing
/// the sub-voxels in the player's path. It only allows stepping up by a maximum
/// of one sub-voxel height.
///
/// # Arguments
/// * `spatial_grid` - The spatial partitioning grid for efficient queries
/// * `sub_voxel_query` - Query to access sub-voxel components
/// * `player` - The player's collision parameters
/// * `max_step_height` - Maximum height the player can step up
///
/// # Returns
/// `Some(height)` if a valid step-up is possible, `None` otherwise
pub fn get_step_up_height(
    spatial_grid: &SpatialGrid,
    sub_voxel_query: &Query<&SubVoxel, Without<Player>>,
    player: &PlayerCollision,
    max_step_height: f32,
) -> Option<f32> {
    let collision_radius = player.radius * 0.8;
    let current_bottom = player.current_y - player.radius;

    // Calculate player's AABB for grid lookup
    let player_min = Vec3::new(
        player.pos.x - collision_radius,
        player.pos.y - player.radius,
        player.pos.z - collision_radius,
    );
    let player_max = Vec3::new(
        player.pos.x + collision_radius,
        player.pos.y + player.radius,
        player.pos.z + collision_radius,
    );

    let relevant_sub_voxel_entities = spatial_grid.get_entities_in_aabb(player_min, player_max);

    let mut all_voxels = Vec::new();

    for entity in relevant_sub_voxel_entities {
        if let Ok(sub_voxel) = sub_voxel_query.get(entity) {
            let (min, max) = get_sub_voxel_bounds(sub_voxel);

            // Check horizontal overlap with target position
            let closest_x = player.pos.x.clamp(min.x, max.x);
            let closest_z = player.pos.z.clamp(min.z, max.z);
            let dx = player.pos.x - closest_x;
            let dz = player.pos.z - closest_z;
            let distance_squared = dx * dx + dz * dz;

            if distance_squared < collision_radius * collision_radius {
                all_voxels.push(max.y);
            }
        }
    }

    if all_voxels.is_empty() {
        return None;
    }

    // Remove duplicates and sort by height
    all_voxels.sort_by(|a, b| a.partial_cmp(b).unwrap());
    all_voxels.dedup_by(|a, b| (*a - *b).abs() < 0.001);

    // Find voxels above the current bottom (excluding floor)
    let above_floor: Vec<f32> = all_voxels
        .iter()
        .filter(|&&h| h > current_bottom + 0.01)
        .copied()
        .collect();

    // Only step up if there's exactly one level above current position
    if above_floor.len() != 1 {
        return None;
    }

    let step_height = above_floor[0];

    // Check step distance from current bottom
    let step_distance = step_height - current_bottom;

    // Allow step up if within max step height
    if step_distance > 0.005 && step_distance <= max_step_height + 0.005 {
        return Some(step_height + player.radius);
    }

    None
}
