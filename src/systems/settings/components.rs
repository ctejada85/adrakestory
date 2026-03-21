use bevy::prelude::*;

/// Marker for the root node of the settings menu UI.
#[derive(Component)]
pub struct SettingsMenuRoot;

/// Identifies a settings row and its position in the list.
#[derive(Component)]
pub struct SettingRow {
    pub index: usize,
    pub id: SettingId,
}

/// Identifies which config field a settings row controls.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingId {
    // OcclusionConfig fields
    Enabled,
    Technique,
    Mode,
    MinAlpha,
    ShadowQuality,
    ShowDebug,
    OcclusionRadius,
    HeightThreshold,
    FalloffSoftness,
    InteriorHeight,
    RegionUpdateInterval,
    // VsyncConfig fields
    VsyncEnabled,
    VsyncMultiplier,
}

/// Marks the text node that displays the current value of a setting row.
#[derive(Component)]
pub struct SettingValueDisplay;

/// Marker for the Back button.
#[derive(Component)]
pub struct BackButton;
