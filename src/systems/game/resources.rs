use bevy::prelude::*;

#[derive(Resource)]
pub struct RoomSize {
    pub width: usize,
    pub height: usize,
}

impl Default for RoomSize {
    fn default() -> Self {
        Self {
            width: 4,
            height: 4,
        }
    }
}