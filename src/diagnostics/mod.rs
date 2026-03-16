//! Frame-level performance profiler.
//!
//! Records per-system timings to a CSV file in `target/benchmarks/`.
//! Only active in **debug builds** (`debug_assertions` on). In release the
//! [`FrameProfilerPlugin`] is a no-op and `Option<Res<FrameProfiler>>` is
//! always `None`, so instrumented systems have zero overhead.
//!
//! ## Usage
//!
//! Add `FrameProfilerPlugin` to the Bevy `App` once (unconditional):
//!
//! ```rust,ignore
//! app.add_plugins(FrameProfilerPlugin);
//! ```
//!
//! In each system to profile, add an `Option<Res<FrameProfiler>>` parameter
//! and call the `profile_scope!` macro at the top of the function body:
//!
//! ```rust,ignore
//! pub fn my_system(
//!     ...
//!     profiler: Option<Res<FrameProfiler>>,
//! ) {
//!     profile_scope!(profiler, "my_system");
//!     // rest of system
//! }
//! ```
//!
//! ## Output format
//!
//! `target/benchmarks/profile_<unix_timestamp>.csv`
//!
//! ```csv
//! frame,label,duration_us
//! 1,move_player,45
//! 1,apply_gravity,3
//! 1,apply_physics,12
//! ```
//!
//! Analyse with any CSV tool, e.g.:
//! ```sh
//! # Average duration per label
//! awk -F',' 'NR>1 {sum[$2]+=$3; cnt[$2]++} END {for(l in sum) print l, sum[l]/cnt[l]}' profile.csv | sort -k2 -n
//! ```

use bevy::prelude::*;
use std::fs;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, SyncSender};
use std::time::Instant;

// ---------------------------------------------------------------------------
// ProfileLine
// ---------------------------------------------------------------------------

/// A single pre-formatted CSV row sent to the background writer thread.
struct ProfileLine(String);

// Safety: String is Send.
unsafe impl Send for ProfileLine {}

// ---------------------------------------------------------------------------
// FrameProfiler resource
// ---------------------------------------------------------------------------

/// Bevy resource that times code scopes and streams results to a CSV file.
///
/// Only inserted into the ECS in debug builds. Instrumented systems use
/// `Option<Res<FrameProfiler>>` so they compile and run without modification
/// in release, where the option is always `None`.
#[derive(Resource)]
pub struct FrameProfiler {
    sender: SyncSender<ProfileLine>,
    frame: AtomicU64,
}

// SyncSender<T>: Send + Sync when T: Send — satisfies Resource bounds.

impl FrameProfiler {
    fn new() -> Self {
        let dir = std::path::Path::new("target/benchmarks");
        fs::create_dir_all(dir).expect("could not create target/benchmarks");

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let path = dir.join(format!("profile_{timestamp}.csv"));

        info!("FrameProfiler → {}", path.display());

        // Bounded channel: 8 192 slots ≈ ~130 frames of headroom at 60 fps with
        // ~10 labels/frame. The background thread should always drain faster than
        // this fills up, but the bound prevents unbounded memory growth.
        let (sender, receiver) = mpsc::sync_channel::<ProfileLine>(8_192);

        std::thread::spawn(move || {
            let file = fs::File::create(&path).expect("could not create profile CSV");
            let mut writer = BufWriter::new(file);
            writeln!(writer, "frame,label,duration_us").unwrap();
            while let Ok(ProfileLine(line)) = receiver.recv() {
                let _ = writer.write_all(line.as_bytes());
            }
            // Channel closed (FrameProfiler dropped) → flush and exit.
            let _ = writer.flush();
        });

        Self {
            sender,
            frame: AtomicU64::new(0),
        }
    }

    /// Advance the frame counter. Called once per frame by [`FrameProfilerPlugin`].
    pub fn advance_frame(&self) {
        self.frame.fetch_add(1, Ordering::Relaxed);
    }

    /// Begin timing a labeled scope. Writes a CSV row when the returned guard drops.
    pub fn scope(&self, label: &'static str) -> ProfileScope {
        ProfileScope {
            start: Instant::now(),
            label,
            frame: self.frame.load(Ordering::Relaxed),
            sender: self.sender.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// ProfileScope RAII guard
// ---------------------------------------------------------------------------

/// Timing guard. Drop at end of scope to emit a CSV row.
pub struct ProfileScope {
    start: Instant,
    label: &'static str,
    frame: u64,
    sender: SyncSender<ProfileLine>,
}

impl Drop for ProfileScope {
    fn drop(&mut self) {
        let duration_us = self.start.elapsed().as_micros() as u64;
        // Non-blocking try_send: if the channel is full (background thread fell
        // behind) the record is silently dropped rather than stalling the game.
        let _ = self
            .sender
            .try_send(ProfileLine(format!("{},{},{}\n", self.frame, self.label, duration_us)));
    }
}

// ---------------------------------------------------------------------------
// Bevy plugin
// ---------------------------------------------------------------------------

/// Adds the [`FrameProfiler`] resource and frame-counter system.
///
/// Register unconditionally — it becomes a no-op in release builds.
pub struct FrameProfilerPlugin;

impl Plugin for FrameProfilerPlugin {
    fn build(&self, app: &mut App) {
        // Only wire up the real profiler in debug builds.
        #[cfg(debug_assertions)]
        app.insert_resource(FrameProfiler::new())
            .add_systems(First, advance_frame_counter);
    }
}

/// Increments the frame counter at the start of every frame so all `Update`
/// timing records carry the correct frame number.
fn advance_frame_counter(profiler: Res<FrameProfiler>) {
    profiler.advance_frame();
}

// ---------------------------------------------------------------------------
// profile_scope! macro
// ---------------------------------------------------------------------------

/// Time the enclosing scope and append a row to the profile CSV.
///
/// Expands to a single let-binding whose drop records elapsed time. Completely
/// removed (zero AST nodes) in release builds via `#[cfg(debug_assertions)]`.
///
/// ```rust,ignore
/// pub fn my_system(profiler: Option<Res<FrameProfiler>>, ...) {
///     profile_scope!(profiler, "my_system");
///     // ...
/// }
/// ```
#[macro_export]
macro_rules! profile_scope {
    ($profiler:expr, $label:literal) => {
        #[cfg(debug_assertions)]
        let _profile_scope = $profiler
            .as_ref()
            .map(|p| p.scope($label));
    };
}
