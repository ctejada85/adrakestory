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

#[derive(Component)]
pub struct SubVoxel {
    pub parent_x: i32,
    pub parent_y: i32,
    pub parent_z: i32,
    pub sub_x: i32,
    pub sub_y: i32,
    pub sub_z: i32,
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
