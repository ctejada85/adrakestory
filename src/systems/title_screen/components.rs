use bevy::prelude::*;

#[derive(Component)]
pub struct TitleScreenUI;

#[derive(Component)]
pub struct TitleScreenBackground;

#[derive(Component)]
pub enum MenuButton {
    NewGame,
    Continue,
    Settings,
    Exit,
}

#[derive(Component)]
pub struct ScalableText {
    pub base_size: f32,
    pub scale_factor: f32,
}

impl ScalableText {
    pub fn new(base_size: f32, scale_factor: f32) -> Self {
        Self {
            base_size,
            scale_factor,
        }
    }
}