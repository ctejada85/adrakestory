//! NPC name label systems.
//!
//! Each named NPC gets a screen-space UI text label that is positioned by
//! projecting the NPC's world position through the active 3D camera each
//! frame. Labels fade in/out over [`FADE_DURATION_SECS`] when the player
//! enters or leaves [`INTERACTION_RANGE`] (horizontal XZ distance).
//!
//! Both systems run in `GameSystemSet::Visual` under
//! `GameState::InGame | GameState::Paused`.
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

/// Duration of the fade-in and fade-out transition in seconds (50 ms).
pub const FADE_DURATION_SECS: f32 = 0.05;

/// Per-label fade state.
///
/// Tracks the current opacity (`alpha`) and the desired opacity (`target`).
/// `alpha` moves toward `target` at a rate of `1.0 / FADE_DURATION_SECS`
/// units per second, clamped to `[0.0, 1.0]`.
#[derive(Component)]
pub struct NpcLabelFade {
    /// Current opacity in `[0.0, 1.0]`.
    pub alpha: f32,
    /// Desired opacity — either `0.0` (hidden) or `1.0` (fully visible).
    pub target: f32,
}

impl NpcLabelFade {
    fn new() -> Self {
        Self {
            alpha: 0.0,
            target: 0.0,
        }
    }
}

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
            // Translate by (-50%, -50%) of the node's own size so the centre of
            // the text box lands on the projected screen point, not the top-left.
            UiTransform::from_translation(Val2::percent(-50.0, -50.0)),
            Text::new(npc.name.clone()),
            TextFont {
                font_size: LABEL_FONT_SIZE,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
            Visibility::Hidden,
            NpcLabel { npc_entity },
            NpcLabelFade::new(),
        ));
    }
}

/// Sets the fade target for each NPC label based on player proximity and camera
/// visibility, and keeps the screen position up to date.
///
/// - Sets `fade.target = 1.0` when the player is within [`INTERACTION_RANGE`]
///   and the NPC is in front of the camera.
/// - Sets `fade.target = 0.0` otherwise.
///
/// Screen position (`node.left` / `node.top`) is updated every frame while the
/// label is visible or fading in, so it tracks the NPC smoothly.
pub fn update_npc_label_visibility(
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    player_query: Query<&Transform, With<Player>>,
    npc_query: Query<&GlobalTransform, With<Npc>>,
    mut label_query: Query<(&mut Node, &NpcLabel, &mut NpcLabelFade)>,
) {
    let Ok((camera, camera_global_transform)) = camera_query.single() else {
        return;
    };
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation;

    for (mut node, label, mut fade) in &mut label_query {
        let Ok(npc_global_transform) = npc_query.get(label.npc_entity) else {
            fade.target = 0.0;
            continue;
        };

        let npc_pos = npc_global_transform.translation();
        let dx = player_pos.x - npc_pos.x;
        let dz = player_pos.z - npc_pos.z;
        let horizontal_distance = (dx * dx + dz * dz).sqrt();

        if horizontal_distance >= INTERACTION_RANGE {
            fade.target = 0.0;
            continue;
        }

        let label_world_pos = npc_pos + Vec3::Y * LABEL_Y_OFFSET;
        match camera.world_to_viewport(camera_global_transform, label_world_pos) {
            Ok(screen_pos) => {
                node.left = Val::Px(screen_pos.x);
                node.top = Val::Px(screen_pos.y);
                fade.target = 1.0;
            }
            Err(_) => {
                // NPC is behind the camera or outside the viewport.
                fade.target = 0.0;
            }
        }
    }
}

/// Advances each label's `alpha` toward its `target` and writes the result to
/// [`TextColor`] and [`Visibility`].
///
/// Runs after [`update_npc_label_visibility`] in the same system set so that
/// `fade.target` is already set for this frame.
pub fn tick_npc_label_fade(
    time: Res<Time>,
    mut label_query: Query<(&mut Visibility, &mut TextColor, &mut NpcLabelFade)>,
) {
    let dt = time.delta_secs();
    let step = dt / FADE_DURATION_SECS;

    for (mut visibility, mut color, mut fade) in &mut label_query {
        // Move alpha toward target.
        if fade.target > fade.alpha {
            fade.alpha = (fade.alpha + step).min(1.0);
        } else if fade.target < fade.alpha {
            fade.alpha = (fade.alpha - step).max(0.0);
        }

        // Apply alpha to the text colour.
        color.0 = Color::srgba(1.0, 1.0, 1.0, fade.alpha);

        // Only hide the node when fully transparent to avoid layout flicker.
        *visibility = if fade.alpha <= 0.0 {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
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

    /// Newly spawned label starts fully transparent and hidden.
    #[test]
    fn new_label_starts_transparent() {
        let mut app = app_with_spawn_system();
        app.world_mut().spawn(Npc {
            name: "Sage".to_string(),
            radius: 0.3,
        });
        app.update();

        let mut q = app.world_mut().query::<(&TextColor, &NpcLabelFade)>();
        let Ok((color, fade)) = q.single(app.world()) else {
            panic!("expected exactly one label entity");
        };
        assert_eq!(fade.alpha, 0.0);
        assert_eq!(fade.target, 0.0);
        // Alpha channel of the initial colour must be 0.
        let Srgba { alpha, .. } = color.0.to_srgba();
        assert_eq!(alpha, 0.0);
    }

    /// `tick_npc_label_fade` advances alpha toward 1.0 when target is 1.0.
    #[test]
    fn fade_in_advances_alpha() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, tick_npc_label_fade);

        let entity = app
            .world_mut()
            .spawn((
                Visibility::Hidden,
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
                NpcLabelFade {
                    alpha: 0.0,
                    target: 1.0,
                },
            ))
            .id();

        // Run enough frames to fully fade in (50 ms total; each MinimalPlugins
        // frame advances time by a small fixed delta).
        for _ in 0..10 {
            app.update();
        }

        let fade = app.world().get::<NpcLabelFade>(entity).unwrap();
        assert!(
            fade.alpha > 0.0,
            "alpha should have advanced above 0.0, got {}",
            fade.alpha
        );
    }

    /// When target is 0.0 and alpha is 1.0, `tick_npc_label_fade` decreases alpha.
    #[test]
    fn fade_out_decreases_alpha() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, tick_npc_label_fade);

        let entity = app
            .world_mut()
            .spawn((
                Visibility::Visible,
                TextColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
                NpcLabelFade {
                    alpha: 1.0,
                    target: 0.0,
                },
            ))
            .id();

        for _ in 0..10 {
            app.update();
        }

        let fade = app.world().get::<NpcLabelFade>(entity).unwrap();
        assert!(
            fade.alpha < 1.0,
            "alpha should have decreased below 1.0, got {}",
            fade.alpha
        );
    }

    /// When fully faded out, Visibility must be Hidden.
    #[test]
    fn fully_faded_out_is_hidden() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, tick_npc_label_fade);

        let entity = app
            .world_mut()
            .spawn((
                Visibility::Visible,
                TextColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
                NpcLabelFade {
                    alpha: 0.0,
                    target: 0.0,
                },
            ))
            .id();

        app.update();

        let vis = app.world().get::<Visibility>(entity).unwrap();
        assert_eq!(*vis, Visibility::Hidden);
    }

    /// When partially faded in, Visibility must be Visible.
    #[test]
    fn partially_faded_in_is_visible() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, tick_npc_label_fade);

        let entity = app
            .world_mut()
            .spawn((
                Visibility::Hidden,
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
                NpcLabelFade {
                    alpha: 0.5,
                    target: 1.0,
                },
            ))
            .id();

        app.update();

        let vis = app.world().get::<Visibility>(entity).unwrap();
        assert_eq!(*vis, Visibility::Visible);
    }
}
