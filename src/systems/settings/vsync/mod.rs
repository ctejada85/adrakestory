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
    /// Absolute time the current frame should start by.
    ///
    /// Advanced by `target_frame_time` from the **previous deadline** (not from the
    /// actual wakeup time), giving self-correcting frame pacing: a 1 ms overshoot in
    /// frame N automatically shortens frame N+1's sleep by 1 ms.
    pub next_frame_deadline: Option<Instant>,
    pub target_frame_time: Option<Duration>,
}

/// Sleeps for approximately `duration` with sub-millisecond precision.
///
/// Uses OS sleep for the bulk of the wait, then busy-spins for the final
/// `SPIN_THRESHOLD` to avoid scheduler wakeup latency on non-RT systems like macOS
/// where `std::thread::sleep` can overshoot short durations by 1–2 ms.
fn precise_sleep(duration: Duration) {
    const SPIN_THRESHOLD: Duration = Duration::from_millis(2);
    // Fix the deadline before the OS sleep so overshoot is automatically absorbed.
    let deadline = Instant::now() + duration;
    if duration > SPIN_THRESHOLD {
        std::thread::sleep(duration - SPIN_THRESHOLD);
    }
    while Instant::now() < deadline {
        std::hint::spin_loop();
    }
}

/// Computes the target frame time for a given refresh rate and multiplier.
///
/// Returns `None` when VSync is disabled (no software cap applied).
/// For any multiplier when VSync is enabled, the cap is `refresh_hz × multiplier`:
/// - `< 1.0` reduces fps below refresh rate (e.g., 0.5× = 30 fps on 60 Hz)
/// - `= 1.0` caps at native refresh rate
/// - `> 1.0` caps above refresh rate (e.g., 2× = 120 fps on 60 Hz)
pub fn target_frame_time(
    refresh_hz: f32,
    vsync_enabled: bool,
    multiplier: f32,
) -> Option<Duration> {
    if vsync_enabled && multiplier > 0.0 {
        let target_fps = refresh_hz * multiplier;
        Some(Duration::from_secs_f32(1.0 / target_fps))
    } else {
        None
    }
}

/// Selects the GPU present mode for the given VSync configuration.
///
/// - VSync enabled, multiplier ≤ 1.0 → `Fifo` (GPU-synced, tear-free at native Hz)
/// - VSync enabled, multiplier > 1.0 → `AutoNoVsync` (software cap only; `Fifo` would
///   hard-clamp at the native refresh rate regardless of the multiplier)
/// - VSync disabled → `AutoNoVsync`
pub fn select_present_mode(vsync_enabled: bool, multiplier: f32) -> PresentMode {
    if vsync_enabled && multiplier <= 1.0 {
        PresentMode::Fifo
    } else {
        PresentMode::AutoNoVsync
    }
}

/// Clamps `vsync_multiplier` to `[0.25, 16.0]`, logging a warning if clamped.
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
/// Registered in the `First` schedule so the sleep happens **before** any frame work.
/// Uses self-correcting absolute deadlines: the next deadline is always the previous
/// deadline + target, so sleep overshoots in one frame are automatically compensated
/// by a shorter sleep in the next. `precise_sleep` adds spin-waiting for the final
/// 2 ms to avoid macOS scheduler wakeup latency.
/// Gated by `VsyncConfig.dirty` to avoid mutating `Window` every frame.
pub fn apply_vsync_system(
    mut vsync_config: ResMut<VsyncConfig>,
    monitor_info: Res<MonitorInfo>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    mut limiter: Local<FrameLimiterState>,
) {
    // Defensively clear the limiter whenever VSync is disabled, regardless of the
    // dirty flag. This prevents any residual Some(target)/Some(deadline) state from
    // surviving a settings change and capping fps when VSync is off.
    if !vsync_config.vsync_enabled {
        limiter.target_frame_time = None;
        limiter.next_frame_deadline = None;
    }

    // Software frame cap with self-correcting absolute deadlines.
    // Advancing the deadline from the PREVIOUS deadline (not from actual wakeup)
    // means any sleep overshoot is compensated: the next sleep is automatically
    // shorter by the same amount. Clamp to now to avoid burst catch-up after a
    // long stall (e.g., window minimized, breakpoint).
    if let (Some(target), Some(deadline)) = (limiter.target_frame_time, limiter.next_frame_deadline)
    {
        let now = Instant::now();
        if now < deadline {
            precise_sleep(deadline - now);
        }
        let next = deadline + target;
        limiter.next_frame_deadline = Some(next.max(Instant::now()));
    }

    if !vsync_config.dirty {
        return;
    }

    // Clamp multiplier before applying.
    vsync_config.vsync_multiplier = clamp_multiplier(vsync_config.vsync_multiplier);

    // Update Window present mode.
    // multiplier > 1.0 uses AutoNoVsync: Fifo would hard-cap at the native refresh
    // rate, preventing any frame rate above the monitor Hz regardless of multiplier.
    let present_mode =
        select_present_mode(vsync_config.vsync_enabled, vsync_config.vsync_multiplier);
    if window.present_mode != present_mode {
        window.present_mode = present_mode;
    }

    // Configure software frame cap.
    limiter.target_frame_time = target_frame_time(
        monitor_info.refresh_hz,
        vsync_config.vsync_enabled,
        vsync_config.vsync_multiplier,
    );

    // Reset to a fresh deadline so stale past/future deadlines don't cause
    // an immediate burst or a long wait after a settings change.
    limiter.next_frame_deadline = limiter.target_frame_time.map(|t| Instant::now() + t);

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
mod tests;
