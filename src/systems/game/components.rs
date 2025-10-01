use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub speed: f32,
}

#[derive(Component)]
pub struct Voxel {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub voxel_type: VoxelType,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VoxelType {
    Air,
    Grass,
    Dirt,
    Stone,
}

#[derive(Component)]
pub struct GameCamera;