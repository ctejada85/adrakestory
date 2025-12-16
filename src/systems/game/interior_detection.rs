//! Interior detection system for room-aware occlusion.
//!
//! This module detects when the player enters an interior space (house, room, cave)
//! by casting rays upward to find ceiling voxels, then flood-filling to find
//! all connected ceiling voxels at the same Y level.
//!
//! Detection works at VOXEL level (1x1x1 units), not sub-voxel level.
//! The detected region bounds are passed to the shader for GPU-based hiding.

use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};

use super::components::{Player, SubVoxel};
use super::resources::SpatialGrid;

/// Maximum number of voxels to include in a flood-fill region (prevents runaway)
const MAX_REGION_SIZE: usize = 1000;

/// Y tolerance for considering voxels at "same" ceiling level (in voxel units)
const CEILING_Y_TOLERANCE: i32 = 0;

/// Minimum distance (in world units) the player must be inside the ceiling region
/// before occlusion is triggered. This prevents flickering at doorways/edges.
const INTERIOR_ENTRY_MARGIN: f32 = 0.4;

/// Represents a detected interior region (ceiling above player).
#[derive(Debug, Clone, Default)]
pub struct InteriorRegion {
    /// Minimum bounds of the ceiling region (in world coordinates)
    pub min: Vec3,
    /// Maximum bounds of the ceiling region (in world coordinates)
    pub max: Vec3,
    /// Y level of the ceiling (voxel Y coordinate)
    pub ceiling_y: i32,
    /// Number of voxels in the region
    pub voxel_count: usize,
}

impl InteriorRegion {
    /// Check if a point is inside this region.
    #[inline]
    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }
}

/// Resource tracking current interior state.
#[derive(Resource, Default)]
pub struct InteriorState {
    /// Currently detected interior region (None if player is outside)
    pub current_region: Option<InteriorRegion>,
    /// Last player position used for detection (avoid recalculating every frame)
    pub last_detection_pos: Vec3,
    /// Frame counter for throttled updates
    pub frames_since_update: u32,
    /// Cached set of occupied voxel positions (rebuilt when map changes)
    pub occupied_voxels_cache: Option<HashSet<IVec3>>,
    /// Number of entities when cache was built (used to detect map changes)
    pub cache_entity_count: usize,
}

/// System to detect if player is inside an interior.
///
/// This system:
/// 1. Casts a ray upward from the player to find ceiling voxels
/// 2. If a ceiling is found within threshold, flood-fills to find the region
/// 3. Updates `InteriorState` with the detected region bounds
pub fn detect_interior_system(
    player_query: Query<&Transform, With<Player>>,
    spatial_grid: Option<Res<SpatialGrid>>,
    sub_voxels: Query<&SubVoxel, Without<Player>>,
    mut interior_state: ResMut<InteriorState>,
    config: Res<super::occlusion::OcclusionConfig>,
) {
    // Only run for region-based or hybrid modes
    if !matches!(
        config.mode,
        super::occlusion::OcclusionMode::RegionBased
            | super::occlusion::OcclusionMode::Hybrid
    ) {
        interior_state.current_region = None;
        return;
    }

    // Throttle updates based on config
    interior_state.frames_since_update += 1;
    if interior_state.frames_since_update < config.region_update_interval {
        return;
    }

    let Some(spatial_grid) = spatial_grid else {
        return;
    };

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation;

    // Skip if player hasn't moved significantly
    if player_pos.distance(interior_state.last_detection_pos) < 0.3 {
        return;
    }

    interior_state.frames_since_update = 0;
    interior_state.last_detection_pos = player_pos;

    // Get player's voxel Y level (floor of player position)
    let player_voxel_y = player_pos.y.floor() as i32;

    // Get or rebuild the occupied voxels cache
    // We detect map changes by counting entities in the spatial grid
    let current_entity_count = spatial_grid.cells.values().map(|v| v.len()).sum();
    
    let occupied_voxels = if interior_state.occupied_voxels_cache.is_some() 
        && interior_state.cache_entity_count == current_entity_count 
    {
        // Use cached version
        interior_state.occupied_voxels_cache.as_ref().unwrap()
    } else {
        // Rebuild cache
        let new_cache = build_occupied_voxel_set(&spatial_grid, &sub_voxels);
        interior_state.cache_entity_count = current_entity_count;
        interior_state.occupied_voxels_cache = Some(new_cache);
        interior_state.occupied_voxels_cache.as_ref().unwrap()
    };

    // Check multiple voxel positions around the player's center to find a ceiling
    // This prevents the region from disappearing when crossing voxel boundaries
    let player_voxel_x_floor = player_pos.x.floor() as i32;
    let player_voxel_z_floor = player_pos.z.floor() as i32;
    
    // Check the 4 voxels the player could be overlapping
    let positions_to_check = [
        (player_voxel_x_floor, player_voxel_z_floor),
        (player_voxel_x_floor + 1, player_voxel_z_floor),
        (player_voxel_x_floor, player_voxel_z_floor + 1),
        (player_voxel_x_floor + 1, player_voxel_z_floor + 1),
    ];

    // Find ceiling from any of these positions
    let mut best_ceiling: Option<(i32, i32, i32, i32)> = None; // (x, z, ceiling_y, voxel_count estimate)
    
    for (vx, vz) in positions_to_check {
        if let Some(ceiling_y) = find_ceiling_voxel_above(
            vx,
            player_voxel_y,
            vz,
            config.interior_height_threshold as i32,
            &occupied_voxels,
        ) {
            // Use this ceiling if we haven't found one yet
            if best_ceiling.is_none() {
                best_ceiling = Some((vx, vz, ceiling_y, 0));
            }
        }
    }

    if let Some((start_x, start_z, ceiling_y, _)) = best_ceiling {
        // Flood-fill to find connected ceiling region at voxel level
        let region = flood_fill_ceiling_region_voxel(
            start_x,
            ceiling_y,
            start_z,
            player_voxel_y,
            occupied_voxels,
        );

        // The key check: is the player still under a ceiling?
        // We check if ANY of the 4 positions the player overlaps has a ceiling above
        // This is more reliable than checking if player is inside the region bounds
        let player_under_ceiling = positions_to_check.iter().any(|(vx, vz)| {
            find_ceiling_voxel_above(
                *vx,
                player_voxel_y,
                *vz,
                config.interior_height_threshold as i32,
                occupied_voxels,
            ).is_some()
        });

        // Hysteresis: once inside, stay inside as long as we're under a ceiling
        let is_currently_inside = interior_state.current_region.is_some();
        
        if player_under_ceiling {
            // Player is under a ceiling - use the detected region
            // But only if we have a meaningful region (more than just a tiny area)
            if region.voxel_count >= 2 {
                interior_state.current_region = Some(region);
            } else if is_currently_inside {
                // Keep the previous region if we're already inside
                // (don't change anything)
            } else {
                interior_state.current_region = None;
            }
        } else if !is_currently_inside {
            // Player not under ceiling and wasn't inside - no region
            interior_state.current_region = None;
        }
        // If player was inside but moved to edge, keep the region active
        // (hysteresis - they need to fully exit)
    } else {
        interior_state.current_region = None;
    }
}

/// Build a set of occupied voxel positions from all sub-voxels.
/// Each voxel is represented by its integer (x, y, z) coordinate.
fn build_occupied_voxel_set(
    spatial_grid: &SpatialGrid,
    sub_voxels: &Query<&SubVoxel, Without<Player>>,
) -> HashSet<IVec3> {
    let mut occupied = HashSet::new();

    // Iterate through all cells in the spatial grid
    for (_, entities) in spatial_grid.cells.iter() {
        for entity in entities {
            if let Ok(sub_voxel) = sub_voxels.get(*entity) {
                let (min, _) = sub_voxel.bounds;
                // Convert sub-voxel position to parent voxel position
                // Sub-voxels within a voxel have positions like (x + 0.0625, y + 0.0625, z + 0.0625)
                // to (x + 0.9375, y + 0.9375, z + 0.9375)
                // The parent voxel is at floor(center)
                let voxel_pos = IVec3::new(
                    min.x.floor() as i32,
                    min.y.floor() as i32,
                    min.z.floor() as i32,
                );
                occupied.insert(voxel_pos);
            }
        }
    }

    occupied
}

/// Find ceiling voxel directly above player within threshold.
/// Returns the Y coordinate of the ceiling voxel, or None if no ceiling found.
/// 
/// A ceiling is defined as a voxel that:
/// 1. Is above the player (Y > player_y + 1)
/// 2. Has empty space below it (not a wall that extends from ground level)
fn find_ceiling_voxel_above(
    x: i32,
    player_y: i32,
    z: i32,
    max_height: i32,
    occupied_voxels: &HashSet<IVec3>,
) -> Option<i32> {
    // Start from 2 voxels above the player to skip the immediate head space
    // This avoids detecting walls that the player is standing next to
    let start_y = player_y + 2;
    
    for y in start_y..=(player_y + max_height) {
        if occupied_voxels.contains(&IVec3::new(x, y, z)) {
            // Found a voxel above - check if it's a ceiling (has empty space below)
            // or a wall (has solid voxels below connecting to ground)
            
            // Check if there's empty space between player and this voxel
            let has_gap_below = !(player_y + 1..y).any(|check_y| {
                occupied_voxels.contains(&IVec3::new(x, check_y, z))
            });
            
            if has_gap_below {
                // This is a ceiling - there's empty space between player and this voxel
                return Some(y);
            }
            // Otherwise it's a wall - continue looking higher
        }
    }
    None
}

/// Flood-fill to find all connected ceiling voxels at same Y level.
/// Works at voxel level (integer coordinates).
/// Searches multiple Y levels to find comprehensive ceiling coverage.
fn flood_fill_ceiling_region_voxel(
    start_x: i32,
    ceiling_y: i32,
    start_z: i32,
    player_y: i32,
    occupied_voxels: &HashSet<IVec3>,
) -> InteriorRegion {
    let mut visited: HashSet<IVec2> = HashSet::new();
    let mut queue: VecDeque<IVec2> = VecDeque::new();
    let mut ceiling_positions: HashSet<IVec2> = HashSet::new();

    let start = IVec2::new(start_x, start_z);
    queue.push_back(start);
    visited.insert(start);

    let mut min_x = start_x;
    let mut max_x = start_x;
    let mut min_z = start_z;
    let mut max_z = start_z;

    // Y range to search for ceiling voxels (ceiling_y +/- tolerance, but above player)
    let min_ceiling_y = (ceiling_y - 1).max(player_y + 2);
    let max_ceiling_y = ceiling_y + 2;

    // BFS flood-fill in XZ plane
    // We continue expanding even through gaps to find the full ceiling extent
    while let Some(current) = queue.pop_front() {
        if ceiling_positions.len() >= MAX_REGION_SIZE {
            break;
        }

        // Check if there's a ceiling voxel at this XZ position at any valid Y level
        let mut found_ceiling = false;
        for y in min_ceiling_y..=max_ceiling_y {
            if occupied_voxels.contains(&IVec3::new(current.x, y, current.y)) {
                found_ceiling = true;
                break;
            }
        }

        if found_ceiling {
            ceiling_positions.insert(current);
            min_x = min_x.min(current.x);
            max_x = max_x.max(current.x);
            min_z = min_z.min(current.y);
            max_z = max_z.max(current.y);
        }

        // Always expand to neighbors within a reasonable distance from start
        // This ensures we don't miss ceiling sections due to small gaps
        let distance_from_start = (current.x - start_x).abs() + (current.y - start_z).abs();
        let max_search_distance = 30; // Maximum manhattan distance to search

        if distance_from_start < max_search_distance {
            // Add neighbors (4-connected in XZ plane)
            let neighbors = [
                IVec2::new(current.x + 1, current.y),
                IVec2::new(current.x - 1, current.y),
                IVec2::new(current.x, current.y + 1),
                IVec2::new(current.x, current.y - 1),
            ];

            for neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    
                    // Only queue if we found ceiling here OR if a neighbor has ceiling
                    // This allows crossing small gaps but not open areas
                    let neighbor_has_ceiling = (min_ceiling_y..=max_ceiling_y)
                        .any(|y| occupied_voxels.contains(&IVec3::new(neighbor.x, y, neighbor.y)));
                    
                    if found_ceiling || neighbor_has_ceiling {
                        queue.push_back(neighbor);
                    }
                }
            }
        }
    }

    // If we found very few ceiling voxels, the region might not be valid
    let voxel_count = ceiling_positions.len();

    // Convert voxel bounds to world bounds
    // IMPORTANT: min.y must be ABOVE player level to avoid hiding fences/walls at player level
    // A voxel at integer Y occupies world space from Y-0.5 to Y+0.5
    // Player at voxel Y=0 means we should only hide voxels at Y >= 2 (two levels above)
    // This ensures voxels at player level (Y=0) and one above (Y=1, where player's head might be) are visible
    let padding = 0.05;
    InteriorRegion {
        min: Vec3::new(
            min_x as f32 - 0.5 + padding,
            (player_y + 2) as f32 - 0.5 + padding, // Start hiding from 2 voxels above player
            min_z as f32 - 0.5 + padding,
        ),
        max: Vec3::new(
            max_x as f32 + 1.5 - padding,
            ceiling_y as f32 + 100.0, // Extend high to hide all above
            max_z as f32 + 1.5 - padding,
        ),
        ceiling_y,
        voxel_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interior_region_contains() {
        let region = InteriorRegion {
            min: Vec3::new(0.0, 0.0, 0.0),
            max: Vec3::new(10.0, 5.0, 10.0),
            ceiling_y: 4,
            voxel_count: 100,
        };

        assert!(region.contains(Vec3::new(5.0, 2.5, 5.0)));
        assert!(region.contains(Vec3::new(0.0, 0.0, 0.0)));
        assert!(region.contains(Vec3::new(10.0, 5.0, 10.0)));
        assert!(!region.contains(Vec3::new(-1.0, 2.5, 5.0)));
        assert!(!region.contains(Vec3::new(5.0, 6.0, 5.0)));
    }

    #[test]
    fn test_interior_region_default() {
        let region = InteriorRegion::default();
        assert_eq!(region.voxel_count, 0);
        assert_eq!(region.ceiling_y, 0);
    }

    #[test]
    fn test_find_ceiling_voxel_above() {
        let mut occupied = HashSet::new();
        // Place a voxel at y=5
        occupied.insert(IVec3::new(0, 5, 0));

        // Player at y=2, should find ceiling at y=5
        assert_eq!(find_ceiling_voxel_above(0, 2, 0, 10, &occupied), Some(5));

        // Player at y=5, should NOT find ceiling (same level)
        assert_eq!(find_ceiling_voxel_above(0, 5, 0, 10, &occupied), None);

        // Player at y=6, no ceiling above
        assert_eq!(find_ceiling_voxel_above(0, 6, 0, 10, &occupied), None);

        // Different XZ position, no ceiling
        assert_eq!(find_ceiling_voxel_above(1, 2, 0, 10, &occupied), None);
    }
}
