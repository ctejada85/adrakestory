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
