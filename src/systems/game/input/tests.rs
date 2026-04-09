use super::*;

// ── sync_light_sources ────────────────────────────────────────────────────

/// All four fields are propagated from `LightSource` to `PointLight`.
#[test]
fn sync_propagates_all_four_fields() {
    let mut app = App::new();
    app.add_systems(Update, sync_light_sources);

    let entity = app
        .world_mut()
        .spawn((
            LightSource {
                color: Color::srgb(1.0, 0.5, 0.0),
                intensity: 5_000.0,
                range: 8.0,
                shadows_enabled: true,
            },
            PointLight::default(),
        ))
        .id();

    app.update();

    let pl = app.world().get::<PointLight>(entity).unwrap();
    let ls = app.world().get::<LightSource>(entity).unwrap();
    assert_eq!(pl.color, ls.color, "color must match");
    assert_eq!(pl.intensity, 5_000.0, "intensity must match");
    assert_eq!(pl.range, 8.0, "range must match");
    assert!(pl.shadows_enabled, "shadows_enabled must match");
}

/// A `PointLight` without a `LightSource` is never touched by the system.
#[test]
fn sync_skips_entity_without_light_source() {
    let mut app = App::new();
    app.add_systems(Update, sync_light_sources);

    let entity = app
        .world_mut()
        .spawn(PointLight {
            intensity: 42.0,
            ..default()
        })
        .id();

    app.update();

    let pl = app.world().get::<PointLight>(entity).unwrap();
    assert_eq!(pl.intensity, 42.0, "untouched entity must keep its value");
}

/// Mutating `LightSource` after the first frame causes `sync_light_sources`
/// to pick up the change on the next update.
#[test]
fn sync_picks_up_runtime_mutation() {
    let mut app = App::new();
    app.add_systems(Update, sync_light_sources);

    let entity = app
        .world_mut()
        .spawn((LightSource::default(), PointLight::default()))
        .id();

    app.update(); // frame 1 — initial sync

    app.world_mut()
        .get_mut::<LightSource>(entity)
        .unwrap()
        .intensity = 99_999.0;

    app.update(); // frame 2 — Changed<LightSource> fires

    let pl = app.world().get::<PointLight>(entity).unwrap();
    assert_eq!(pl.intensity, 99_999.0);
}

/// `shadows_enabled: false` on `LightSource` overrides a `PointLight` that
/// had shadows enabled.
#[test]
fn sync_shadows_disabled_overrides_point_light() {
    let mut app = App::new();
    app.add_systems(Update, sync_light_sources);

    let entity = app
        .world_mut()
        .spawn((
            LightSource {
                shadows_enabled: false,
                ..LightSource::default()
            },
            PointLight {
                shadows_enabled: true,
                ..default()
            },
        ))
        .id();

    app.update();

    let pl = app.world().get::<PointLight>(entity).unwrap();
    assert!(!pl.shadows_enabled);
}

/// `color` field is correctly reflected: a red `LightSource` produces a
/// red `PointLight`.
#[test]
fn sync_color_field_is_reflected() {
    let mut app = App::new();
    app.add_systems(Update, sync_light_sources);

    let red = Color::srgb(1.0, 0.0, 0.0);
    let entity = app
        .world_mut()
        .spawn((
            LightSource {
                color: red,
                ..LightSource::default()
            },
            PointLight::default(),
        ))
        .id();

    app.update();

    let pl = app.world().get::<PointLight>(entity).unwrap();
    assert_eq!(pl.color, red);
}

// ── flicker_lights ────────────────────────────────────────────────────────

/// After running `flicker_lights`, intensity stays within
/// `[base - amplitude, base + amplitude]` on every frame.
#[test]
fn flicker_intensity_stays_within_bounds() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, flicker_lights);

    let flicker = FlickerLight::default();
    let lo = flicker.base_intensity - flicker.amplitude;
    let hi = flicker.base_intensity + flicker.amplitude;

    let entity = app
        .world_mut()
        .spawn((LightSource::default(), flicker))
        .id();

    for _ in 0..30 {
        app.update();
        let intensity = app.world().get::<LightSource>(entity).unwrap().intensity;
        assert!(
            intensity >= lo && intensity <= hi,
            "intensity {intensity} out of [{lo}, {hi}]"
        );
    }
}

/// A `LightSource` without a `FlickerLight` component is not touched by
/// `flicker_lights`.
#[test]
fn flicker_skips_entity_without_flicker_component() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, flicker_lights);

    let entity = app
        .world_mut()
        .spawn(LightSource {
            intensity: 1_234.0,
            ..LightSource::default()
        })
        .id();

    for _ in 0..5 {
        app.update();
    }

    let intensity = app.world().get::<LightSource>(entity).unwrap().intensity;
    assert_eq!(intensity, 1_234.0, "intensity must be unchanged");
}

/// `flicker_lights` followed by `sync_light_sources` in the same frame
/// causes the `PointLight` to reflect the flickered intensity.
#[test]
fn flicker_then_sync_round_trip() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // flicker_lights must run before sync_light_sources so the mutation is
    // propagated within the same frame.
    app.add_systems(Update, (flicker_lights, sync_light_sources).chain());

    let entity = app
        .world_mut()
        .spawn((
            LightSource::default(),
            FlickerLight::default(),
            PointLight::default(),
        ))
        .id();

    for _ in 0..10 {
        app.update();
        let ls_intensity = app.world().get::<LightSource>(entity).unwrap().intensity;
        let pl_intensity = app.world().get::<PointLight>(entity).unwrap().intensity;
        assert_eq!(
            pl_intensity, ls_intensity,
            "PointLight must mirror LightSource after flicker+sync"
        );
    }
}
