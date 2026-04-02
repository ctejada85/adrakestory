//! NPC name label systems.
//!
//! Each named NPC gets a screen-space UI text label that is positioned by
//! projecting the NPC's world position through the active 3D camera each
//! frame. Labels are shown only when the player is within [`INTERACTION_RANGE`]
//! (horizontal XZ distance). Both systems run in `GameSystemSet::Visual`
//! under `GameState::InGame | GameState::Paused`.
//!
//! Labels are standalone UI nodes (not children of the NPC entity). They are
//! cleaned up in two ways:
//! - [`cleanup_npc_labels`] — registered on `OnExit(GameState::InGame)`.
//! - [`despawn_removed_npc_labels`] — despawns labels whose NPC was removed
//!   (e.g. during hot-reload).

use bevy::prelude::*;

use super::components::{Npc, NpcLabel, Player};

/// Default name assigned to NPCs without a custom name in the map file.
/// Labels are suppressed for this value (and the empty string).
const DEFAULT_NPC_NAME: &str = "NPC";

/// Distance (horizontal, XZ-plane) within which an NPC's label becomes visible.
pub const INTERACTION_RANGE: f32 = 3.0;

/// Y-offset above the NPC world origin used as the label anchor point before
/// projection. Initial value; tuned after in-game review.
const LABEL_Y_OFFSET: f32 = 1.2;

/// Label font size in logical pixels.
const LABEL_FONT_SIZE: f32 = 24.0;

/// Spawns a UI text label node for each newly-added [`Npc`] that has a
/// non-default, non-empty name. The label is a root-level absolutely-positioned
/// [`Node`] entity — it is **not** a child of the NPC entity.
///
/// Runs once per NPC via the `Added<Npc>` query filter.
pub fn spawn_npc_label(mut commands: Commands, query: Query<(Entity, &Npc), Added<Npc>>) {
    for (npc_entity, npc) in &query {
        if npc.name.is_empty() || npc.name == DEFAULT_NPC_NAME {
            continue;
        }

        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            Text::new(npc.name.clone()),
            TextFont {
                font_size: LABEL_FONT_SIZE,
                ..default()
            },
            TextColor(Color::WHITE),
            Visibility::Hidden,
            NpcLabel { npc_entity },
        ));
    }
}

/// Projects each NPC's world position through the 3D camera to obtain a screen
/// coordinate, then repositions and shows/hides the corresponding UI label.
///
/// A label is shown when:
/// 1. The player is within [`INTERACTION_RANGE`] (XZ distance).
/// 2. The NPC is in front of the camera (projection succeeds).
///
/// The distance check uses only the XZ plane so vertical height differences
/// between the player and the NPC do not affect label visibility.
pub fn update_npc_label_visibility(
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    player_query: Query<&Transform, With<Player>>,
    npc_query: Query<&GlobalTransform, With<Npc>>,
    mut label_query: Query<(&mut Visibility, &mut Node, &NpcLabel)>,
) {
    // Need the 3D camera to project world positions to screen space.
    let Ok((camera, camera_global_transform)) = camera_query.single() else {
        return;
    };
    // Player position is needed for the proximity check.
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation;

    for (mut visibility, mut node, label) in &mut label_query {
        let Ok(npc_global_transform) = npc_query.get(label.npc_entity) else {
            // NPC entity is gone — hide the label (it will be despawned by
            // despawn_removed_npc_labels on the next frame).
            *visibility = Visibility::Hidden;
            continue;
        };

        let npc_pos = npc_global_transform.translation();
        let dx = player_pos.x - npc_pos.x;
        let dz = player_pos.z - npc_pos.z;
        let horizontal_distance = (dx * dx + dz * dz).sqrt();

        if horizontal_distance >= INTERACTION_RANGE {
            *visibility = Visibility::Hidden;
            continue;
        }

        // Project the point above the NPC into screen space.
        let label_world_pos = npc_pos + Vec3::Y * LABEL_Y_OFFSET;
        match camera.world_to_viewport(camera_global_transform, label_world_pos) {
            Ok(screen_pos) => {
                node.left = Val::Px(screen_pos.x);
                node.top = Val::Px(screen_pos.y);
                *visibility = Visibility::Visible;
            }
            Err(_) => {
                // NPC is behind the camera or outside the viewport.
                *visibility = Visibility::Hidden;
            }
        }
    }
}

/// Despawns label entities whose associated NPC entity no longer exists.
///
/// Runs every frame in `GameSystemSet::Visual` alongside the visibility system.
/// Handles hot-reload and any other scenario where NPC entities are removed
/// without an `OnExit(InGame)` transition.
pub fn despawn_removed_npc_labels(
    mut commands: Commands,
    label_query: Query<(Entity, &NpcLabel)>,
    npc_query: Query<(), With<Npc>>,
) {
    for (label_entity, label) in &label_query {
        if !npc_query.contains(label.npc_entity) {
            commands.entity(label_entity).despawn();
        }
    }
}

/// Despawns all NPC label UI nodes when leaving the `InGame` state.
///
/// Because labels are **not** children of NPC entities, they are not
/// automatically removed when the map is unloaded. This system cleans them up
/// on `OnExit(GameState::InGame)`.
pub fn cleanup_npc_labels(mut commands: Commands, label_query: Query<Entity, With<NpcLabel>>) {
    for entity in &label_query {
        commands.entity(entity).despawn();
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

    /// A named NPC must produce exactly one NpcLabel entity.
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

    /// `cleanup_npc_labels` must despawn all NpcLabel entities.
    #[test]
    fn cleanup_removes_all_labels() {
        let mut app = App::new();
        app.add_systems(Update, (spawn_npc_label, cleanup_npc_labels).chain());

        // Spawn in frame 1, then clean up in frame 2.
        app.world_mut().spawn(Npc {
            name: "Wizard".to_string(),
            radius: 0.3,
        });
        app.update(); // spawn
        app.update(); // cleanup runs

        let label_count = app
            .world_mut()
            .query::<&NpcLabel>()
            .iter(app.world())
            .count();
        assert_eq!(label_count, 0);
    }
}
