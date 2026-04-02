//! NPC name label systems.
//!
//! Spawns a world-space `Text2d` label above each NPC and shows/hides it
//! based on player proximity. Both systems run in `GameSystemSet::Visual`
//! under `GameState::InGame | GameState::Paused`.

use bevy::prelude::*;

use super::components::{Npc, NpcLabel, Player};

/// Default name assigned to NPCs without a custom name in the map file.
/// Labels are suppressed for this value (and the empty string).
const DEFAULT_NPC_NAME: &str = "NPC";

/// Distance (horizontal, XZ-plane) within which an NPC's label becomes visible.
pub const INTERACTION_RANGE: f32 = 3.0;

/// Y-offset above the NPC origin where the label is positioned.
/// Initial best-guess value; will be tuned after in-game review.
const LABEL_Y_OFFSET: f32 = 1.2;

/// Label font size in logical pixels.
const LABEL_FONT_SIZE: f32 = 24.0;

/// Spawns a `Text2d` label child entity for each newly-added `Npc` that has a
/// non-default, non-empty name. Labels for default/empty names are skipped
/// entirely — no entity is created.
///
/// Runs once per NPC via the `Added<Npc>` query filter.
pub fn spawn_npc_label(mut commands: Commands, query: Query<(Entity, &Npc), Added<Npc>>) {
    for (npc_entity, npc) in &query {
        if npc.name.is_empty() || npc.name == DEFAULT_NPC_NAME {
            continue;
        }

        commands.spawn((
            Text2d::new(npc.name.clone()),
            TextFont {
                font_size: LABEL_FONT_SIZE,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_translation(Vec3::new(0.0, LABEL_Y_OFFSET, 0.0)),
            Visibility::Hidden,
            NpcLabel,
            ChildOf(npc_entity),
        ));
    }
}

/// Shows or hides each NPC label based on the player's horizontal distance.
///
/// The distance check uses only the XZ plane (same convention as
/// `apply_npc_collision` in `physics.rs`) so height differences between
/// the player and the NPC do not affect visibility.
pub fn update_npc_label_visibility(
    player_query: Query<&Transform, With<Player>>,
    npc_query: Query<&Transform, With<Npc>>,
    mut label_query: Query<(&mut Visibility, &ChildOf), With<NpcLabel>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation;

    for (mut visibility, parent) in &mut label_query {
        let Ok(npc_transform) = npc_query.get(parent.0) else {
            continue;
        };
        let npc_pos = npc_transform.translation;
        let dx = player_pos.x - npc_pos.x;
        let dz = player_pos.z - npc_pos.z;
        let horizontal_distance = (dx * dx + dz * dz).sqrt();

        *visibility = if horizontal_distance < INTERACTION_RANGE {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_with_spawn_system() -> App {
        let mut app = App::new();
        app.add_systems(Update, spawn_npc_label);
        app
    }

    /// A named NPC must produce exactly one NpcLabel child entity.
    #[test]
    fn label_spawned_for_named_npc() {
        let mut app = app_with_spawn_system();
        app.world_mut().spawn(Npc {
            name: "Village Elder".to_string(),
            radius: 0.3,
        });
        app.update();

        let label_count = app
            .world_mut()
            .query::<&NpcLabel>()
            .iter(app.world())
            .count();
        assert_eq!(label_count, 1);
    }

    /// The default NPC name ("NPC") must not produce a label entity.
    #[test]
    fn no_label_for_default_npc_name() {
        let mut app = app_with_spawn_system();
        app.world_mut().spawn(Npc::default()); // name: "NPC"
        app.update();

        let label_count = app
            .world_mut()
            .query::<&NpcLabel>()
            .iter(app.world())
            .count();
        assert_eq!(label_count, 0);
    }

    /// An empty NPC name must not produce a label entity.
    #[test]
    fn no_label_for_empty_npc_name() {
        let mut app = app_with_spawn_system();
        app.world_mut().spawn(Npc {
            name: "".to_string(),
            radius: 0.3,
        });
        app.update();

        let label_count = app
            .world_mut()
            .query::<&NpcLabel>()
            .iter(app.world())
            .count();
        assert_eq!(label_count, 0);
    }

    /// Two named NPCs produce two independent label entities.
    #[test]
    fn two_named_npcs_produce_two_labels() {
        let mut app = app_with_spawn_system();
        app.world_mut().spawn(Npc {
            name: "Guard".to_string(),
            radius: 0.3,
        });
        app.world_mut().spawn(Npc {
            name: "Merchant".to_string(),
            radius: 0.3,
        });
        app.update();

        let label_count = app
            .world_mut()
            .query::<&NpcLabel>()
            .iter(app.world())
            .count();
        assert_eq!(label_count, 2);
    }

    /// A mix of named and default NPCs — only named ones get labels.
    #[test]
    fn mixed_npcs_only_named_get_labels() {
        let mut app = app_with_spawn_system();
        app.world_mut().spawn(Npc {
            name: "Elder".to_string(),
            radius: 0.3,
        });
        app.world_mut().spawn(Npc::default()); // name: "NPC" — no label
        app.world_mut().spawn(Npc {
            name: "".to_string(),
            radius: 0.3,
        }); // empty — no label
        app.update();

        let label_count = app
            .world_mut()
            .query::<&NpcLabel>()
            .iter(app.world())
            .count();
        assert_eq!(label_count, 1);
    }

    /// Filter is case-sensitive: "npc" (lowercase) is a valid custom name and gets a label.
    #[test]
    fn lowercase_npc_name_gets_label() {
        let mut app = app_with_spawn_system();
        app.world_mut().spawn(Npc {
            name: "npc".to_string(), // lowercase — NOT suppressed
            radius: 0.3,
        });
        app.update();

        let label_count = app
            .world_mut()
            .query::<&NpcLabel>()
            .iter(app.world())
            .count();
        assert_eq!(label_count, 1);
    }

    /// `Added<Npc>` filter — a second `app.update()` must not spawn a duplicate label.
    #[test]
    fn spawn_system_is_idempotent_across_frames() {
        let mut app = app_with_spawn_system();
        app.world_mut().spawn(Npc {
            name: "Blacksmith".to_string(),
            radius: 0.3,
        });
        app.update(); // frame 1 — label spawned
        app.update(); // frame 2 — Added<Npc> no longer fires

        let label_count = app
            .world_mut()
            .query::<&NpcLabel>()
            .iter(app.world())
            .count();
        assert_eq!(label_count, 1);
    }
}
