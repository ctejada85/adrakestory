//! VSync configuration and software frame pacing.
//!
//! Provides `VsyncConfig` (serializable resource), `MonitorInfo` (runtime-only),
//! and systems to detect the monitor refresh rate and apply VSync + frame cap changes.

use bevy::prelude::*;
use bevy::window::{Monitor, PresentMode, PrimaryWindow};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Configuration for vertical synchronization and frame pacing.
///
/// Serialized to `settings.ron` alongside `OcclusionConfig`. Fields use
/// `#[serde(default)]` so existing save files without VSync fields load correctly.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct VsyncConfig {
    /// Whether VSync is enabled.
    /// `true` → `PresentMode::Fifo` (tear-free, capped at refresh rate).
    /// `false` → `PresentMode::AutoNoVsync` (unlimited, current default behavior).
    #[serde(default)]
    pub vsync_enabled: bool,

    /// Target frame rate as a multiple of the monitor's refresh rate.
    /// `1.0` = native refresh rate (e.g., 60 fps on 60 Hz).
    /// `0.5` = half rate (e.g., 30 fps on 60 Hz).
    /// Clamped to `[0.25, 4.0]`. Software cap only applied when
    /// `vsync_enabled = true` and value `< 1.0`.
    #[serde(default = "default_vsync_multiplier")]
    pub vsync_multiplier: f32,

    /// Triggers `apply_vsync_system` to re-apply settings on the next frame.
    /// Excluded from serialization.
    #[serde(skip)]
    pub dirty: bool,
}

fn default_vsync_multiplier() -> f32 {
    1.0
}

impl Default for VsyncConfig {
    fn default() -> Self {
        Self {
            vsync_enabled: false,
            vsync_multiplier: 1.0,
            // Apply on first frame to initialize window + limiter state.
            dirty: true,
        }
    }
}

/// Cached monitor refresh rate, populated at runtime.
///
/// Not serialized. Defaults to 60 Hz if detection fails.
#[derive(Resource)]
pub struct MonitorInfo {
    /// Refresh rate of the primary monitor in Hz.
    pub refresh_hz: f32,
}

impl Default for MonitorInfo {
    fn default() -> Self {
        Self { refresh_hz: 60.0 }
    }
}

/// Per-system state for the software frame limiter.
#[derive(Default)]
pub struct FrameLimiterState {
    pub last_frame_end: Option<Instant>,
    pub target_frame_time: Option<Duration>,
}

/// Computes the target frame time for a given refresh rate and multiplier.
///
/// Returns `None` when VSync is disabled (no software cap applied).
/// For any multiplier when VSync is enabled, the cap is `refresh_hz × multiplier`:
/// - `< 1.0` reduces fps below refresh rate (e.g., 0.5× = 30 fps on 60 Hz)
/// - `= 1.0` caps at native refresh rate
/// - `> 1.0` caps above refresh rate (e.g., 2× = 120 fps on 60 Hz)
pub fn target_frame_time(refresh_hz: f32, vsync_enabled: bool, multiplier: f32) -> Option<Duration> {
    if vsync_enabled && multiplier > 0.0 {
        let target_fps = refresh_hz * multiplier;
        Some(Duration::from_secs_f32(1.0 / target_fps))
    } else {
        None
    }
}

/// Clamps `vsync_multiplier` to `[0.25, 4.0]`, logging a warning if clamped.
pub fn clamp_multiplier(value: f32) -> f32 {
    const MIN: f32 = 0.25;
    const MAX: f32 = 16.0;
    let clamped = value.clamp(MIN, MAX);
    if (clamped - value).abs() > f32::EPSILON {
        warn!(
            "[VSync] Multiplier {:.2} is out of range [{MIN}, {MAX}]; clamped to {:.2}",
            value, clamped
        );
    }
    clamped
}

/// Reads the primary monitor's refresh rate and stores it in `MonitorInfo`.
///
/// Runs each frame until the refresh rate is successfully detected, then stops.
/// Falls back to 60 Hz if no monitor data is available.
pub fn detect_monitor_refresh_system(
    mut monitor_info: ResMut<MonitorInfo>,
    mut vsync_config: ResMut<VsyncConfig>,
    mut detected: Local<bool>,
    monitors: Query<&Monitor>,
) {
    if *detected {
        return;
    }

    for monitor in monitors.iter() {
        if let Some(millihertz) = monitor.refresh_rate_millihertz {
            let hz = millihertz as f32 / 1000.0;
            if hz > 0.0 {
                let changed = (monitor_info.refresh_hz - hz).abs() > f32::EPSILON;
                monitor_info.refresh_hz = hz;
                *detected = true;
                info!("[VSync] Detected monitor refresh rate: {:.1} Hz", hz);
                // Re-apply frame cap with the correct Hz if VSync is active.
                if changed {
                    vsync_config.dirty = true;
                }
                return;
            }
        }
    }
    // Monitor entities not yet spawned — will retry next frame.
}

/// Applies VSync configuration changes to the window and manages software frame pacing.
///
/// Registered in the `Last` schedule so pacing sleeps happen after all game logic.
/// Gated by `VsyncConfig.dirty` to avoid mutating `Window` every frame.
pub fn apply_vsync_system(
    mut vsync_config: ResMut<VsyncConfig>,
    monitor_info: Res<MonitorInfo>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    mut limiter: Local<FrameLimiterState>,
) {
    // Software frame cap: sleep if the current frame finished early.
    if let (Some(target), Some(last)) = (limiter.target_frame_time, limiter.last_frame_end) {
        let elapsed = last.elapsed();
        if elapsed < target {
            std::thread::sleep(target - elapsed);
        }
    }
    limiter.last_frame_end = Some(Instant::now());

    if !vsync_config.dirty {
        return;
    }

    // Clamp multiplier before applying.
    vsync_config.vsync_multiplier = clamp_multiplier(vsync_config.vsync_multiplier);

    // Update Window present mode.
    if let Ok(mut window) = window_query.single_mut() {
        let present_mode = if vsync_config.vsync_enabled {
            PresentMode::Fifo
        } else {
            PresentMode::AutoNoVsync
        };
        if window.present_mode != present_mode {
            window.present_mode = present_mode;
        }
    }

    // Configure software frame cap.
    limiter.target_frame_time =
        target_frame_time(monitor_info.refresh_hz, vsync_config.vsync_enabled, vsync_config.vsync_multiplier);

    if let Some(ft) = limiter.target_frame_time {
        let target_fps = 1.0 / ft.as_secs_f32();
        info!(
            "[VSync] Frame cap: {:.1} fps ({:.2}× of {:.1} Hz)",
            target_fps, vsync_config.vsync_multiplier, monitor_info.refresh_hz
        );
    }

    vsync_config.dirty = false;
    info!(
        "[VSync] Applied — enabled={}, multiplier={:.2}",
        vsync_config.vsync_enabled, vsync_config.vsync_multiplier
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- VsyncConfig defaults ---

    #[test]
    fn vsync_config_default_vsync_disabled() {
        let config = VsyncConfig::default();
        assert!(!config.vsync_enabled);
    }

    #[test]
    fn vsync_config_default_multiplier_is_one() {
        let config = VsyncConfig::default();
        assert!((config.vsync_multiplier - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn vsync_config_default_is_dirty() {
        // dirty=true on default triggers apply_vsync_system on the first frame.
        let config = VsyncConfig::default();
        assert!(config.dirty);
    }

    // --- Multiplier clamping ---

    #[test]
    fn clamp_multiplier_below_min_snaps_to_025() {
        assert!((clamp_multiplier(0.1) - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn clamp_multiplier_above_max_snaps_to_16() {
        assert!((clamp_multiplier(20.0) - 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn clamp_multiplier_in_range_unchanged() {
        assert!((clamp_multiplier(0.5) - 0.5).abs() < f32::EPSILON);
        assert!((clamp_multiplier(1.0) - 1.0).abs() < f32::EPSILON);
        assert!((clamp_multiplier(8.0) - 8.0).abs() < f32::EPSILON);
        assert!((clamp_multiplier(16.0) - 16.0).abs() < f32::EPSILON);
    }

    // --- Target frame time calculation ---

    #[test]
    fn target_frame_time_half_rate_on_60hz() {
        let ft = target_frame_time(60.0, true, 0.5).expect("should return Some");
        let expected = Duration::from_secs_f32(1.0 / 30.0);
        assert!((ft.as_secs_f32() - expected.as_secs_f32()).abs() < 0.0001);
    }

    #[test]
    fn target_frame_time_quarter_rate_on_60hz() {
        let ft = target_frame_time(60.0, true, 0.25).expect("should return Some");
        let expected = Duration::from_secs_f32(1.0 / 15.0);
        assert!((ft.as_secs_f32() - expected.as_secs_f32()).abs() < 0.0001);
    }

    #[test]
    fn target_frame_time_native_rate_returns_some() {
        // multiplier = 1.0 → cap at refresh rate.
        let ft = target_frame_time(60.0, true, 1.0).expect("should return Some");
        let expected = Duration::from_secs_f32(1.0 / 60.0);
        assert!((ft.as_secs_f32() - expected.as_secs_f32()).abs() < 0.0001);
    }

    #[test]
    fn target_frame_time_double_rate_on_60hz() {
        // multiplier = 2.0 → cap at 120 fps.
        let ft = target_frame_time(60.0, true, 2.0).expect("should return Some");
        let expected = Duration::from_secs_f32(1.0 / 120.0);
        assert!((ft.as_secs_f32() - expected.as_secs_f32()).abs() < 0.0001);
    }

    #[test]
    fn target_frame_time_16x_on_60hz() {
        // multiplier = 16.0 → cap at 960 fps.
        let ft = target_frame_time(60.0, true, 16.0).expect("should return Some");
        let expected = Duration::from_secs_f32(1.0 / 960.0);
        assert!((ft.as_secs_f32() - expected.as_secs_f32()).abs() < 0.0001);
    }

    #[test]
    fn target_frame_time_vsync_off_returns_none() {
        // No software cap when VSync is disabled.
        assert!(target_frame_time(60.0, false, 0.5).is_none());
        assert!(target_frame_time(60.0, false, 1.0).is_none());
        assert!(target_frame_time(60.0, false, 2.0).is_none());
    }

    #[test]
    fn target_frame_time_on_144hz_monitor() {
        let ft = target_frame_time(144.0, true, 0.5).expect("should return Some");
        let expected = Duration::from_secs_f32(1.0 / 72.0);
        assert!((ft.as_secs_f32() - expected.as_secs_f32()).abs() < 0.0001);
    }

    // --- RON round-trip serialization ---

    #[test]
    fn vsync_config_ron_roundtrip_preserves_values() {
        let original = VsyncConfig {
            vsync_enabled: true,
            vsync_multiplier: 0.5,
            dirty: false,
        };
        let ron_str = ron::to_string(&original).expect("serialization failed");
        let loaded: VsyncConfig = ron::from_str(&ron_str).expect("deserialization failed");
        assert_eq!(loaded.vsync_enabled, original.vsync_enabled);
        assert!((loaded.vsync_multiplier - original.vsync_multiplier).abs() < f32::EPSILON);
        // dirty is #[serde(skip)] so it resets to false on deserialization.
        assert!(!loaded.dirty);
    }

    #[test]
    fn vsync_config_ron_missing_fields_use_defaults() {
        // Simulate an old settings.ron that has no VSync fields at all.
        let ron_str = "()";
        let loaded: VsyncConfig = ron::from_str(ron_str).expect("deserialization failed");
        assert!(!loaded.vsync_enabled);
        assert!((loaded.vsync_multiplier - 1.0).abs() < f32::EPSILON);
    }
}
