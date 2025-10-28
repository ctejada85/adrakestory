use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub velocity: Vec3,
    pub is_grounded: bool,
    pub radius: f32,
}

#[derive(Component)]
pub struct CollisionBox;

/// Marker component for a voxel entity.
/// All voxel data is managed in VoxelWorld; this is used for ECS queries and cleanup.
#[derive(Component)]
pub struct Voxel;

/// Sub-voxel component with cached bounding box for efficient collision detection.
///
/// Previously stored parent and sub-voxel coordinates to calculate bounds on-demand,
/// but now caches the computed bounds at spawn time for better performance.
#[derive(Component)]
pub struct SubVoxel {
    /// Cached bounding box (min, max) to avoid recalculation every frame.
    /// Calculated once at spawn time and reused for all collision checks.
    pub bounds: (Vec3, Vec3),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum VoxelType {
    Air,
    Grass,
    Dirt,
    Stone,
}

#[derive(Component)]
pub struct GameCamera {
    pub original_rotation: Quat,
    pub target_rotation: Quat,
    pub rotation_speed: f32,
}
