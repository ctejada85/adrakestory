use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub velocity: Vec3,
    pub is_grounded: bool,
    pub radius: f32,
}

#[derive(Component)]
pub struct CollisionBox;

#[derive(Component)]
pub struct Voxel {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub voxel_type: VoxelType,
}

#[derive(Component)]
pub struct SubVoxel {
    pub parent_x: i32,
    pub parent_y: i32,
    pub parent_z: i32,
    pub sub_x: i32,
    pub sub_y: i32,
    pub sub_z: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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