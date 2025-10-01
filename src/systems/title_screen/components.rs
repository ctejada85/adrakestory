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