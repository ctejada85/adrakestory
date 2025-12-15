//! Hot reload state management and file watching.

use bevy::prelude::*;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Resource to track hot reload state
#[derive(Resource)]
pub struct HotReloadState {
    /// File watcher instance (wrapped for thread safety)
    watcher: Option<Arc<Mutex<RecommendedWatcher>>>,
    /// Channel receiver for file events (wrapped for thread safety)
    receiver: Option<Arc<Mutex<Receiver<Result<Event, notify::Error>>>>>,
    /// Path being watched
    watched_path: Option<PathBuf>,
    /// Last reload time (for debouncing)
    last_reload: Instant,
    /// Whether hot reload is enabled
    pub enabled: bool,
}

impl Default for HotReloadState {
    fn default() -> Self {
        Self {
            watcher: None,
            receiver: None,
            watched_path: None,
            last_reload: Instant::now(),
            enabled: true,
        }
    }
}

impl HotReloadState {
    /// Start watching a map file for changes
    pub fn watch_file(&mut self, path: PathBuf) -> Result<(), String> {
        // Stop any existing watcher
        self.stop_watching();

        let (sender, receiver) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = sender.send(res);
            },
            Config::default().with_poll_interval(Duration::from_millis(500)),
        )
        .map_err(|e| format!("Failed to create file watcher: {}", e))?;

        // Watch the parent directory since some editors do atomic saves
        // (write to temp file, then rename)
        let watch_path = path.parent().unwrap_or(&path).to_path_buf();

        watcher
            .watch(&watch_path, RecursiveMode::NonRecursive)
            .map_err(|e| format!("Failed to watch directory {:?}: {}", watch_path, e))?;

        self.watcher = Some(Arc::new(Mutex::new(watcher)));
        self.receiver = Some(Arc::new(Mutex::new(receiver)));
        self.watched_path = Some(path.clone());

        info!("Hot reload: watching {:?}", path);
        Ok(())
    }

    /// Stop watching for file changes
    pub fn stop_watching(&mut self) {
        if let Some(watcher_arc) = self.watcher.take() {
            if let Ok(mut watcher) = watcher_arc.lock() {
                if let Some(path) = &self.watched_path {
                    let watch_path = path.parent().unwrap_or(path);
                    let _ = watcher.unwatch(watch_path);
                }
            }
        }
        self.receiver = None;
        self.watched_path = None;
        info!("Hot reload: stopped watching");
    }

    /// Check for file changes (called each frame)
    /// Returns the path if a reload should be triggered
    pub fn poll_changes(&mut self) -> Option<PathBuf> {
        const DEBOUNCE_MS: u128 = 200;

        if !self.enabled {
            return None;
        }

        let receiver_arc = self.receiver.as_ref()?;
        let watched_path = self.watched_path.as_ref()?;

        // Try to lock the receiver
        let receiver = match receiver_arc.try_lock() {
            Ok(r) => r,
            Err(_) => return None, // Another thread has the lock, skip this frame
        };

        // Drain all pending events
        let mut should_reload = false;
        while let Ok(event_result) = receiver.try_recv() {
            if let Ok(event) = event_result {
                // Check if this event affects our watched file
                let affects_our_file = event.paths.iter().any(|p| {
                    // Check if the path matches our file (handle both exact match and canonicalized paths)
                    p == watched_path
                        || p.file_name() == watched_path.file_name()
                            && p.parent() == watched_path.parent()
                });

                if affects_our_file {
                    match event.kind {
                        EventKind::Modify(_) | EventKind::Create(_) => {
                            should_reload = true;
                            debug!("Hot reload: detected {:?} on {:?}", event.kind, event.paths);
                        }
                        _ => {}
                    }
                }
            }
        }

        // Drop the lock before potentially returning
        drop(receiver);

        // Debounce: only reload if enough time has passed since last reload
        if should_reload && self.last_reload.elapsed().as_millis() > DEBOUNCE_MS {
            self.last_reload = Instant::now();
            info!("Hot reload: triggering reload for {:?}", watched_path);
            return Some(watched_path.clone());
        }

        None
    }

    /// Get the currently watched path
    pub fn watched_path(&self) -> Option<&PathBuf> {
        self.watched_path.as_ref()
    }

    /// Check if currently watching a file
    #[allow(dead_code)]
    pub fn is_watching(&self) -> bool {
        self.watcher.is_some() && self.watched_path.is_some()
    }
}

#[cfg(test)]
mod tests {
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
}
