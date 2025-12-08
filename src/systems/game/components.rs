use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub velocity: Vec3,
    pub is_grounded: bool,
    pub radius: f32,
    /// Target rotation angle in radians (Y-axis rotation)
    pub target_rotation: f32,
    /// Current rotation angle in radians (Y-axis rotation)
    pub current_rotation: f32,
    /// Rotation angle when the current rotation started (for easing)
    pub start_rotation: f32,
    /// Time elapsed since rotation started (for easing)
    pub rotation_elapsed: f32,
    /// Fixed duration for all rotations in seconds
    pub rotation_duration: f32,
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
    /// Offset from the player in local camera space (before rotation is applied)
    pub follow_offset: Vec3,
    /// Speed at which the camera follows the player (higher = more responsive)
    pub follow_speed: f32,
    /// Current target position the camera is following (typically the player's position)
    pub target_position: Vec3,
}

/// Component for NPC entities.
/// NPCs are static characters that the player can interact with.
#[derive(Component)]
pub struct Npc {
    /// Display name of the NPC
    pub name: String,
    /// Collision radius for player collision
    pub radius: f32,
}

impl Default for Npc {
    fn default() -> Self {
        Self {
            name: "NPC".to_string(),
            radius: 0.3,
        }
    }
}
