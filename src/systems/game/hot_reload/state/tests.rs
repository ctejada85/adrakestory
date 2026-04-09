use super::*;

#[test]
fn test_hot_reload_state_default() {
    let state = HotReloadState::default();
    assert!(state.enabled);
    assert!(!state.is_watching());
    assert!(state.watched_path().is_none());
}

#[test]
fn test_debounce_timing() {
    let mut state = HotReloadState {
        last_reload: Instant::now(),
        ..Default::default()
    };

    // Immediate poll should not trigger (debounce)
    assert!(state.poll_changes().is_none());
}
