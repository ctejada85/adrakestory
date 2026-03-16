use super::components::{BackButton, SettingId, SettingRow, SettingValueDisplay, SettingsMenuRoot};
use super::resources::{SelectedSettingsIndex, SettingsOrigin};
use crate::states::GameState;
use crate::systems::game::gamepad::{get_menu_gamepad_input, ActiveGamepad, GamepadSettings};
use crate::systems::game::occlusion::{OcclusionConfig, OcclusionMode, TransparencyTechnique};
use bevy::prelude::*;

const NORMAL_ROW: Color = Color::srgba(0.15, 0.15, 0.15, 0.0);
const SELECTED_ROW: Color = Color::srgba(1.0, 0.8, 0.2, 0.2);
const LABEL_COLOR: Color = Color::srgba(0.9, 0.9, 0.9, 1.0);
const VALUE_COLOR: Color = Color::srgba(1.0, 0.8, 0.2, 1.0);
const BACK_NORMAL: Color = Color::srgba(0.15, 0.15, 0.15, 0.0);
const BACK_HOVERED: Color = Color::srgba(1.0, 0.8, 0.2, 0.3);

const ALL_SETTINGS: &[(SettingId, &str)] = &[
    (SettingId::Enabled, "Enable Occlusion"),
    (SettingId::Technique, "Transparency"),
    (SettingId::Mode, "Occlusion Mode"),
    (SettingId::MinAlpha, "Min Alpha"),
    (SettingId::HideShadows, "Hide Shadows"),
    (SettingId::ShowDebug, "Debug Visualization"),
    (SettingId::OcclusionRadius, "Occlusion Radius"),
    (SettingId::HeightThreshold, "Height Threshold"),
    (SettingId::FalloffSoftness, "Falloff Softness"),
    (SettingId::InteriorHeight, "Interior Height"),
    (SettingId::RegionUpdateInterval, "Region Update Rate"),
];

fn format_value(id: SettingId, config: &OcclusionConfig) -> String {
    match id {
        SettingId::Enabled => bool_label(config.enabled),
        SettingId::Technique => match config.technique {
            TransparencyTechnique::Dithered => "Dithered".to_string(),
            TransparencyTechnique::AlphaBlend => "Smooth".to_string(),
        },
        SettingId::Mode => match config.mode {
            OcclusionMode::None => "None".to_string(),
            OcclusionMode::ShaderBased => "Shader".to_string(),
            OcclusionMode::RegionBased => "Region".to_string(),
            OcclusionMode::Hybrid => "Hybrid".to_string(),
        },
        SettingId::MinAlpha => format!("{:.2}", config.min_alpha),
        SettingId::HideShadows => bool_label(config.hide_shadows),
        SettingId::ShowDebug => bool_label(config.show_debug),
        SettingId::OcclusionRadius => format!("{:.2}", config.occlusion_radius),
        SettingId::HeightThreshold => format!("{:.2}", config.height_threshold),
        SettingId::FalloffSoftness => format!("{:.2}", config.falloff_softness),
        SettingId::InteriorHeight => format!("{:.1}", config.interior_height_threshold),
        SettingId::RegionUpdateInterval => format!("{}", config.region_update_interval),
    }
}

fn bool_label(v: bool) -> String {
    if v {
        "On".to_string()
    } else {
        "Off".to_string()
    }
}

/// Adjust OcclusionConfig field by delta (-1 = previous/decrease, +1 = next/increase).
fn adjust_value(id: SettingId, config: &mut OcclusionConfig, delta: i32) {
    match id {
        SettingId::Enabled => config.enabled = !config.enabled,
        SettingId::Technique => {
            let variants = [TransparencyTechnique::Dithered, TransparencyTechnique::AlphaBlend];
            let cur = variants.iter().position(|v| *v == config.technique).unwrap_or(0);
            config.technique =
                variants[(cur as i32 + delta).rem_euclid(variants.len() as i32) as usize];
        }
        SettingId::Mode => {
            let variants = [
                OcclusionMode::None,
                OcclusionMode::ShaderBased,
                OcclusionMode::RegionBased,
                OcclusionMode::Hybrid,
            ];
            let cur = variants.iter().position(|v| *v == config.mode).unwrap_or(0);
            config.mode =
                variants[(cur as i32 + delta).rem_euclid(variants.len() as i32) as usize];
        }
        SettingId::MinAlpha => {
            config.min_alpha =
                round2((config.min_alpha + delta as f32 * 0.05).clamp(0.0, 1.0));
        }
        SettingId::HideShadows => config.hide_shadows = !config.hide_shadows,
        SettingId::ShowDebug => config.show_debug = !config.show_debug,
        SettingId::OcclusionRadius => {
            config.occlusion_radius =
                round2((config.occlusion_radius + delta as f32 * 0.25).clamp(0.5, 5.0));
        }
        SettingId::HeightThreshold => {
            config.height_threshold =
                round2((config.height_threshold + delta as f32 * 0.25).clamp(0.0, 5.0));
        }
        SettingId::FalloffSoftness => {
            config.falloff_softness =
                round2((config.falloff_softness + delta as f32 * 0.1).clamp(0.0, 2.0));
        }
        SettingId::InteriorHeight => {
            config.interior_height_threshold =
                round1((config.interior_height_threshold + delta as f32).clamp(1.0, 20.0));
        }
        SettingId::RegionUpdateInterval => {
            let new_val =
                (config.region_update_interval as i32 + delta * 10).clamp(10, 120) as u32;
            config.region_update_interval = new_val;
        }
    }
}

fn round2(v: f32) -> f32 {
    (v * 100.0).round() / 100.0
}

fn round1(v: f32) -> f32 {
    (v * 10.0).round() / 10.0
}

/// Spawns the settings screen UI.
pub fn setup_settings_menu(mut commands: Commands, config: Res<OcclusionConfig>) {
    commands.insert_resource(SelectedSettingsIndex::default());

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
            SettingsMenuRoot,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Settings"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::srgba(0.9, 0.9, 0.9, 1.0)),
                Node {
                    margin: UiRect::bottom(Val::Vh(2.0)),
                    ..default()
                },
            ));

            // Settings rows container
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        width: Val::Vw(50.0),
                        row_gap: Val::Vh(0.5),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|parent| {
                    for (i, &(id, label)) in ALL_SETTINGS.iter().enumerate() {
                        let value_text = format_value(id, &config);
                        spawn_setting_row(parent, i, id, label, &value_text);
                    }

                    // Back button (index 11)
                    parent
                        .spawn((
                            Button,
                            Node {
                                height: Val::Vh(5.5),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::top(Val::Vh(1.5)),
                                ..default()
                            },
                            BackgroundColor(BACK_NORMAL),
                            BackButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Back"),
                                TextFont {
                                    font_size: 26.0,
                                    ..default()
                                },
                                TextColor(LABEL_COLOR),
                            ));
                        });
                });
        });
}

fn spawn_setting_row(
    parent: &mut ChildBuilder,
    index: usize,
    id: SettingId,
    label: &str,
    value: &str,
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                height: Val::Vh(5.5),
                padding: UiRect::horizontal(Val::Vw(1.0)),
                ..default()
            },
            BackgroundColor(NORMAL_ROW),
            SettingRow { index, id },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(LABEL_COLOR),
            ));

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Vw(0.5),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("◄ "),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.6, 0.6, 0.6, 1.0)),
                    ));
                    parent.spawn((
                        Text::new(value.to_string()),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(VALUE_COLOR),
                        SettingValueDisplay,
                        SettingRow { index, id },
                    ));
                    parent.spawn((
                        Text::new(" ►"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.6, 0.6, 0.6, 1.0)),
                    ));
                });
        });
}

/// Cleans up settings screen entities and resources.
pub fn cleanup_settings_menu(
    mut commands: Commands,
    root_query: Query<Entity, With<SettingsMenuRoot>>,
) {
    for entity in &root_query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<SelectedSettingsIndex>();
}

/// Handles keyboard and gamepad input for the settings screen.
pub fn settings_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    active_gamepad: Res<ActiveGamepad>,
    gamepad_query: Query<&Gamepad>,
    gamepad_settings: Res<GamepadSettings>,
    origin: Res<SettingsOrigin>,
    mut selected: ResMut<SelectedSettingsIndex>,
    mut config: ResMut<OcclusionConfig>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let (gp_up, gp_down, gp_select, gp_back) =
        get_menu_gamepad_input(&active_gamepad, &gamepad_query, &gamepad_settings);

    // Up / Down navigation
    if (keyboard.just_pressed(KeyCode::ArrowUp) || gp_up) && selected.index > 0 {
        selected.index -= 1;
    }
    if (keyboard.just_pressed(KeyCode::ArrowDown) || gp_down)
        && selected.index < selected.total - 1
    {
        selected.index += 1;
    }

    // Left / Right value adjustment (only for settings rows, not Back)
    if selected.index < ALL_SETTINGS.len() {
        let (id, _) = ALL_SETTINGS[selected.index];
        if keyboard.just_pressed(KeyCode::ArrowLeft) {
            adjust_value(id, &mut config, -1);
        }
        if keyboard.just_pressed(KeyCode::ArrowRight) {
            adjust_value(id, &mut config, 1);
        }
    }

    // Enter / A button on Back (index 11) or on a toggle (cycle it)
    if keyboard.just_pressed(KeyCode::Enter) || gp_select {
        if selected.index == ALL_SETTINGS.len() {
            go_back(&origin, &mut next_state);
        } else {
            let (id, _) = ALL_SETTINGS[selected.index];
            adjust_value(id, &mut config, 1);
        }
    }

    // Escape / Back button -> save then go back
    if keyboard.just_pressed(KeyCode::Escape) || gp_back {
        go_back(&origin, &mut next_state);
    }
}

fn go_back(origin: &SettingsOrigin, next_state: &mut NextState<GameState>) {
    match origin {
        SettingsOrigin::TitleScreen => next_state.set(GameState::TitleScreen),
        SettingsOrigin::Paused => next_state.set(GameState::Paused),
    }
}

/// Updates the visual appearance of rows (highlight selected) and value display text.
pub fn update_settings_visual(
    selected: Res<SelectedSettingsIndex>,
    config: Res<OcclusionConfig>,
    mut row_query: Query<(&SettingRow, &mut BackgroundColor), Without<SettingValueDisplay>>,
    mut value_query: Query<(&SettingRow, &mut Text), With<SettingValueDisplay>>,
    mut back_query: Query<
        (&Interaction, &mut BackgroundColor),
        (With<BackButton>, Without<SettingRow>),
    >,
) {
    // Update row backgrounds
    for (row, mut bg) in &mut row_query {
        if row.index == selected.index {
            *bg = SELECTED_ROW.into();
        } else {
            *bg = NORMAL_ROW.into();
        }
    }

    // Update value display text if config changed
    if config.is_changed() || selected.is_changed() {
        for (row, mut text) in &mut value_query {
            **text = format_value(row.id, &config);
        }
    }

    // Back button hover
    for (interaction, mut bg) in &mut back_query {
        let is_selected = selected.index == ALL_SETTINGS.len();
        match interaction {
            Interaction::Hovered | Interaction::Pressed => *bg = BACK_HOVERED.into(),
            Interaction::None => {
                *bg = if is_selected {
                    SELECTED_ROW.into()
                } else {
                    BACK_NORMAL.into()
                };
            }
        }
    }
}

/// Handles Back button mouse interaction.
pub fn settings_back_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<BackButton>)>,
    origin: Res<SettingsOrigin>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            go_back(&origin, &mut next_state);
        }
    }
}

/// Loads OcclusionConfig from settings.ron on startup.
pub fn load_settings(mut config: ResMut<OcclusionConfig>) {
    match std::fs::read_to_string("settings.ron") {
        Ok(contents) => match ron::from_str::<OcclusionConfig>(&contents) {
            Ok(loaded) => {
                *config = loaded;
                info!("[Settings] Loaded settings from settings.ron");
            }
            Err(e) => warn!("[Settings] Failed to parse settings.ron: {e}; using defaults"),
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // File not found is expected on first run -- use defaults silently
        }
        Err(e) => warn!("[Settings] Could not read settings.ron: {e}"),
    }
}

/// Saves OcclusionConfig to settings.ron when leaving the settings screen.
pub fn save_settings(config: Res<OcclusionConfig>) {
    match ron::to_string(config.as_ref()) {
        Ok(contents) => {
            if let Err(e) = std::fs::write("settings.ron", contents) {
                warn!("[Settings] Failed to write settings.ron: {e}");
            } else {
                info!("[Settings] Saved settings to settings.ron");
            }
        }
        Err(e) => warn!("[Settings] Failed to serialize settings: {e}"),
    }
}
