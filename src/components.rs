use bevy::prelude::*;

#[derive(Component)]
pub struct IntroUI;

#[derive(Component)]
pub struct IntroText;

#[derive(Component)]
pub struct TitleScreenUI;

#[derive(Component)]
pub enum MenuButton {
    NewGame,
    Continue,
    Settings,
    Exit,
}