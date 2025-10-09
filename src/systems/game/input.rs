//! General input handling systems for the game.
//!
//! This module handles:
//! - Escape key for pausing the game
//! - Collision box visibility toggle
//! - Collision box position synchronization

use super::components::{CollisionBox, Player};
use bevy::prelude::*;

/// System that handles the Escape key to pause the game.
///
/// When the Escape key is pressed, the game transitions to the Paused state,
/// which displays the pause menu.
pub fn handle_escape_key(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<crate::states::GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(crate::states::GameState::Paused);
    }
}

/// System that toggles the visibility of the collision box.
///
/// When the 'C' key is pressed, the collision box visibility is toggled
/// between visible and hidden. This is useful for debugging collision detection.
pub fn toggle_collision_box(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut collision_box_query: Query<&mut Visibility, With<CollisionBox>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        for mut visibility in &mut collision_box_query {
            *visibility = match *visibility {
                Visibility::Hidden => Visibility::Visible,
                _ => Visibility::Hidden,
            };
        }
    }
}

/// System that updates the collision box position to match the player.
///
/// This system ensures the collision box visualization stays synchronized
/// with the player's actual position, making it easier to debug collision issues.
pub fn update_collision_box(
    player_query: Query<&Transform, With<Player>>,
    mut collision_box_query: Query<&mut Transform, (With<CollisionBox>, Without<Player>)>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for mut box_transform in &mut collision_box_query {
            box_transform.translation = player_transform.translation;
        }
    }
}
