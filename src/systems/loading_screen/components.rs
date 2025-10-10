//! Components for the loading screen.

use bevy::prelude::*;

/// Marker component for the loading screen UI root.
#[derive(Component)]
pub struct LoadingScreenUI;

/// Component for the progress bar fill.
#[derive(Component)]
pub struct ProgressBarFill;

/// Component for the loading text.
#[derive(Component)]
pub struct LoadingText;
