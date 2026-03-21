use super::components::{BackButton, SettingId, SettingRow, SettingValueDisplay, SettingsMenuRoot};
use super::resources::{SelectedSettingsIndex, SettingsOrigin};
use super::vsync::VsyncConfig;
use crate::states::GameState;
use crate::systems::game::gamepad::{get_menu_gamepad_input, ActiveGamepad, GamepadSettings};
use crate::systems::game::occlusion::{OcclusionConfig, OcclusionMode, ShadowQuality, TransparencyTechnique};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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
    (SettingId::ShadowQuality, "Shadow Quality"),
    (SettingId::ShowDebug, "Debug Visualization"),
    (SettingId::OcclusionRadius, "Occlusion Radius"),
    (SettingId::HeightThreshold, "Height Threshold"),
    (SettingId::FalloffSoftness, "Falloff Softness"),
    (SettingId::InteriorHeight, "Interior Height"),
    (SettingId::RegionUpdateInterval, "Region Update Rate"),
    // Display settings
    (SettingId::VsyncEnabled, "VSync"),
    (SettingId::VsyncMultiplier, "VSync Multiplier"),
];

fn format_value(id: SettingId, config: &OcclusionConfig, vsync: &VsyncConfig) -> String {
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
        SettingId::ShadowQuality => match config.shadow_quality {
            ShadowQuality::None => "Off".to_string(),
            ShadowQuality::CharactersOnly => "Characters".to_string(),
            ShadowQuality::Low => "Low".to_string(),
            ShadowQuality::High => "High".to_string(),
        },
        SettingId::ShowDebug => bool_label(config.show_debug),
        SettingId::OcclusionRadius => format!("{:.2}", config.occlusion_radius),
        SettingId::HeightThreshold => format!("{:.2}", config.height_threshold),
        SettingId::FalloffSoftness => format!("{:.2}", config.falloff_softness),
        SettingId::InteriorHeight => format!("{:.1}", config.interior_height_threshold),
        SettingId::RegionUpdateInterval => format!("{}", config.region_update_interval),
        SettingId::VsyncEnabled => bool_label(vsync.vsync_enabled),
        SettingId::VsyncMultiplier => {
            // Show whole-number multipliers without a decimal (e.g. "2×" not "2.00×").
            if vsync.vsync_multiplier.fract() == 0.0 {
                format!("{}×", vsync.vsync_multiplier as u32)
            } else {
                format!("{:.2}×", vsync.vsync_multiplier)
            }
        }
    }
}

fn bool_label(v: bool) -> String {
    if v {
        "On".to_string()
    } else {
        "Off".to_string()
    }
}

/// Adjust OcclusionConfig or VsyncConfig field by delta (-1 = previous/decrease, +1 = next/increase).
fn adjust_value(id: SettingId, config: &mut OcclusionConfig, vsync: &mut VsyncConfig, delta: i32) {
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
        SettingId::ShadowQuality => {
            let variants = [
                ShadowQuality::None,
                ShadowQuality::CharactersOnly,
                ShadowQuality::Low,
                ShadowQuality::High,
            ];
            let cur = variants
                .iter()
                .position(|v| *v == config.shadow_quality)
                .unwrap_or(2);
            config.shadow_quality =
                variants[(cur as i32 + delta).rem_euclid(variants.len() as i32) as usize];
        }
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
        SettingId::VsyncEnabled => {
            vsync.vsync_enabled = !vsync.vsync_enabled;
            vsync.dirty = true;
        }
        SettingId::VsyncMultiplier => {
            // Cycle through discrete steps: 0.25 → 0.5 → 1.0 → 2 → 3 → … → 16.
            const STEPS: &[f32] = &[
                0.25, 0.5, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0,
                9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
            ];
            let cur = STEPS
                .iter()
                .position(|&s| (s - vsync.vsync_multiplier).abs() < f32::EPSILON)
                .unwrap_or(STEPS.len() - 1);
            vsync.vsync_multiplier =
                STEPS[(cur as i32 + delta).rem_euclid(STEPS.len() as i32) as usize];
            vsync.dirty = true;
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
pub fn setup_settings_menu(mut commands: Commands, config: Res<OcclusionConfig>, vsync: Res<VsyncConfig>) {
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
                        let value_text = format_value(id, &config, &vsync);
                        spawn_setting_row(parent, i, id, label, &value_text);
                    }

                    // Back button (index = ALL_SETTINGS.len())
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
    parent: &mut ChildSpawnerCommands<'_>,
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
        commands.entity(entity).despawn();
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
    mut vsync: ResMut<VsyncConfig>,
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
            adjust_value(id, &mut config, &mut vsync, -1);
        }
        if keyboard.just_pressed(KeyCode::ArrowRight) {
            adjust_value(id, &mut config, &mut vsync, 1);
        }
    }

    // Enter / A button on Back (last index) or on a toggle (cycle it)
    if keyboard.just_pressed(KeyCode::Enter) || gp_select {
        if selected.index == ALL_SETTINGS.len() {
            go_back(&origin, &mut next_state);
        } else {
            let (id, _) = ALL_SETTINGS[selected.index];
            adjust_value(id, &mut config, &mut vsync, 1);
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
    vsync: Res<VsyncConfig>,
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
    if config.is_changed() || vsync.is_changed() || selected.is_changed() {
        for (row, mut text) in &mut value_query {
            **text = format_value(row.id, &config, &vsync);
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

/// Combined serialization struct for `settings.ron`.
///
/// Uses `#[serde(flatten)]` so `OcclusionConfig` fields remain at the top level,
/// preserving backward compatibility with existing save files.
#[derive(Serialize, Deserialize, Default)]
struct AppSettings {
    #[serde(flatten)]
    occlusion: OcclusionConfig,
    #[serde(default)]
    vsync_enabled: bool,
    #[serde(default = "default_vsync_multiplier_for_settings")]
    vsync_multiplier: f32,
}

fn default_vsync_multiplier_for_settings() -> f32 {
    1.0
}

/// Loads `OcclusionConfig` and `VsyncConfig` from `settings.ron` on startup.
pub fn load_settings(mut config: ResMut<OcclusionConfig>, mut vsync: ResMut<VsyncConfig>) {
    match std::fs::read_to_string("settings.ron") {
        Ok(contents) => match ron::from_str::<AppSettings>(&contents) {
            Ok(loaded) => {
                *config = loaded.occlusion;
                vsync.vsync_enabled = loaded.vsync_enabled;
                vsync.vsync_multiplier = loaded.vsync_multiplier;
                vsync.dirty = true; // Apply loaded values on first frame.
                info!("[Settings] Loaded settings from settings.ron");
            }
            Err(e) => warn!("[Settings] Failed to parse settings.ron: {e}; using defaults"),
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // File not found is expected on first run -- use defaults silently.
        }
        Err(e) => warn!("[Settings] Could not read settings.ron: {e}"),
    }
}

/// Saves `OcclusionConfig` and `VsyncConfig` to `settings.ron` when leaving the settings screen.
pub fn save_settings(config: Res<OcclusionConfig>, vsync: Res<VsyncConfig>) {
    let all = AppSettings {
        occlusion: config.clone(),
        vsync_enabled: vsync.vsync_enabled,
        vsync_multiplier: vsync.vsync_multiplier,
    };
    match ron::to_string(&all) {
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
