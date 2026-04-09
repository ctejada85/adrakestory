use super::*;

// --- Present mode selection ---

#[test]
fn select_present_mode_vsync_off_is_auto_no_vsync() {
    assert_eq!(select_present_mode(false, 1.0), PresentMode::AutoNoVsync);
    assert_eq!(select_present_mode(false, 0.5), PresentMode::AutoNoVsync);
    assert_eq!(select_present_mode(false, 2.0), PresentMode::AutoNoVsync);
}

#[test]
fn select_present_mode_vsync_on_multiplier_one_is_fifo() {
    assert_eq!(select_present_mode(true, 1.0), PresentMode::Fifo);
}

#[test]
fn select_present_mode_vsync_on_sub_one_multiplier_is_fifo() {
    // 0.5× = 60 fps on 120 Hz — Fifo at 120 Hz doesn't interfere with sub-Hz cap.
    assert_eq!(select_present_mode(true, 0.5), PresentMode::Fifo);
    assert_eq!(select_present_mode(true, 0.25), PresentMode::Fifo);
}

#[test]
fn select_present_mode_vsync_on_above_one_multiplier_is_auto_no_vsync() {
    // Fifo would hard-cap at native Hz; use AutoNoVsync so software cap can run above it.
    assert_eq!(select_present_mode(true, 2.0), PresentMode::AutoNoVsync);
    assert_eq!(select_present_mode(true, 16.0), PresentMode::AutoNoVsync);
}

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
fn precise_sleep_sleeps_at_least_the_requested_duration() {
    let duration = Duration::from_millis(5);
    let before = Instant::now();
    precise_sleep(duration);
    let elapsed = before.elapsed();
    assert!(
        elapsed >= duration,
        "precise_sleep undershot: requested {duration:?}, slept {elapsed:?}"
    );
}

#[test]
fn precise_sleep_does_not_overshoot_excessively() {
    // Allow up to 5 ms of overshoot — this is a sanity check, not a precision test.
    let duration = Duration::from_millis(5);
    let max_allowed = duration + Duration::from_millis(5);
    let before = Instant::now();
    precise_sleep(duration);
    let elapsed = before.elapsed();
    assert!(
        elapsed <= max_allowed,
        "precise_sleep overshot excessively: requested {duration:?}, slept {elapsed:?}"
    );
}

#[test]
fn precise_sleep_zero_duration_returns_immediately() {
    let before = Instant::now();
    precise_sleep(Duration::ZERO);
    assert!(before.elapsed() < Duration::from_millis(5));
}

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
