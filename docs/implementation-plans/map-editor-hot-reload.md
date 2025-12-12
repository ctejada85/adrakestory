# Map Editor Hot Reload Implementation Plan

## Implementation Status

| Phase | Description | Status |
|-------|-------------|--------|
| Phase 1 | File Watching Infrastructure | ✅ Complete |
| Phase 2 | Map Reload System | ✅ Complete |
| Phase 3 | Editor Integration | ✅ Complete |
| Phase 4 | Visual Feedback | ✅ Complete |
| Phase 5 | Manual Reload Hotkey | 🔲 Not Started |
| Phase 6 | Settings & Polish | 🔲 Not Started |

## Overview

Implement hot reload functionality that allows the running game to automatically detect and reload map changes made in the editor. This significantly improves the map development workflow by eliminating the need to manually stop and restart the game after each edit.

## Requirements

### Functional Requirements
1. When the editor saves a map file while the game is running, the game automatically detects the change
2. Game reloads the map without returning to loading screen (seamless transition)
3. Player position is preserved when possible (if spawn point hasn't moved)
4. Visual feedback in both editor and game when hot reload occurs
5. Option to disable hot reload if desired
6. Manual reload hotkey in game (e.g., F5 or Ctrl+R)

### Non-Functional Requirements
1. File change detection must be efficient (not polling every frame)
2. Reload should complete in < 500ms for typical maps
3. No memory leaks from repeated reloads
4. Graceful handling of invalid/corrupt map files during reload
5. Cross-platform support (Windows primary)

## Architecture

### Communication Strategy Options

#### Option A: File System Watching (Recommended)
```
┌─────────────────┐                    ┌─────────────────┐
│   MAP EDITOR    │                    │      GAME       │
│                 │                    │                 │
│  Save Map ──────┼────> map.ron ─────>│ File Watcher    │
│                 │      (file)        │       │         │
│                 │                    │       ▼         │
│                 │                    │ Detect Change   │
│                 │                    │       │         │
│                 │                    │       ▼         │
│                 │                    │ Reload Map      │
└─────────────────┘                    └─────────────────┘
```

**Pros:**
- Simple, no IPC needed
- Works with any editor (not just ours)
- Reliable file system notifications

**Cons:**
- Slight delay for FS notification
- Need to handle rapid successive saves

#### Option B: Named Pipe / IPC
```
┌─────────────────┐    Named Pipe     ┌─────────────────┐
│   MAP EDITOR    │◄──────────────────►│      GAME       │
│                 │   "reload" msg     │                 │
└─────────────────┘                    └─────────────────┘
```

**Pros:**
- Instant notification
- Can send additional data (e.g., what changed)

**Cons:**
- More complex implementation
- Platform-specific code
- Need to manage connection lifecycle

#### Option C: Shared Memory + Semaphore
- Most complex, not recommended for this use case

**Decision: Option A (File System Watching)**
- Simpler implementation
- More robust (works even if editor crashes)
- Can be extended later if needed

### Data Flow
```
1. Editor: User presses Ctrl+S or Save button
2. Editor: Save map to file (existing functionality)
3. Editor: Optionally write timestamp to sidecar file (.ron.timestamp)
4. Game: File watcher detects change to map file
5. Game: Debounce (wait 100ms for additional changes)
6. Game: Read and validate new map data
7. Game: If valid, trigger reload event
8. Game: Despawn existing map entities
9. Game: Store player position
10. Game: Spawn new map
11. Game: Restore player position (clamped to valid location)
12. Game: Show brief "Map Reloaded" notification
```

## Implementation Steps

### Phase 1: File Watching Infrastructure (Priority: High)

#### Step 1.1: Add `notify` Dependency

**File:** `Cargo.toml`

```toml
[dependencies]
notify = "6.1"
```

#### Step 1.2: Create Hot Reload Module

**File:** `src/systems/game/hot_reload.rs` (new file)

```rust
//! Hot reload functionality for runtime map reloading.

use bevy::prelude::*;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};

/// Resource to track hot reload state
#[derive(Resource)]
pub struct HotReloadState {
    /// File watcher instance
    watcher: Option<RecommendedWatcher>,
    /// Channel receiver for file events
    receiver: Option<Receiver<Result<Event, notify::Error>>>,
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

/// Event sent when a map reload is triggered
#[derive(Event)]
pub struct MapReloadEvent {
    pub path: PathBuf,
}

/// Event sent when map reload completes
#[derive(Event)]
pub struct MapReloadedEvent {
    pub success: bool,
    pub message: String,
}
```

#### Step 1.3: Implement File Watcher Setup

```rust
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
            Config::default().with_poll_interval(Duration::from_millis(100)),
        )
        .map_err(|e| format!("Failed to create file watcher: {}", e))?;

        watcher
            .watch(&path, RecursiveMode::NonRecursive)
            .map_err(|e| format!("Failed to watch file: {}", e))?;

        self.watcher = Some(watcher);
        self.receiver = Some(receiver);
        self.watched_path = Some(path.clone());

        info!("Hot reload: watching {:?}", path);
        Ok(())
    }

    /// Stop watching for file changes
    pub fn stop_watching(&mut self) {
        if let (Some(mut watcher), Some(path)) = (self.watcher.take(), self.watched_path.take()) {
            let _ = watcher.unwatch(&path);
        }
        self.receiver = None;
    }

    /// Check for file changes (called each frame)
    pub fn poll_changes(&mut self) -> Option<PathBuf> {
        const DEBOUNCE_MS: u128 = 200;

        if !self.enabled {
            return None;
        }

        let receiver = self.receiver.as_ref()?;

        // Drain all pending events
        let mut should_reload = false;
        while let Ok(event) = receiver.try_recv() {
            if let Ok(event) = event {
                if event.kind.is_modify() {
                    should_reload = true;
                }
            }
        }

        // Debounce: only reload if enough time has passed
        if should_reload && self.last_reload.elapsed().as_millis() > DEBOUNCE_MS {
            self.last_reload = Instant::now();
            return self.watched_path.clone();
        }

        None
    }
}
```

---

### Phase 2: Map Reload System (Priority: High)

#### Step 2.1: Create Reload Event Handler

**File:** `src/systems/game/hot_reload.rs`

```rust
/// System to poll for file changes and trigger reload events
pub fn poll_hot_reload(
    mut hot_reload: ResMut<HotReloadState>,
    mut reload_events: EventWriter<MapReloadEvent>,
) {
    if let Some(path) = hot_reload.poll_changes() {
        info!("Hot reload: detected change in {:?}", path);
        reload_events.send(MapReloadEvent { path });
    }
}

/// System to handle map reload events
pub fn handle_map_reload(
    mut commands: Commands,
    mut reload_events: EventReader<MapReloadEvent>,
    mut reloaded_events: EventWriter<MapReloadedEvent>,
    mut progress: ResMut<MapLoadProgress>,
    player_query: Query<&Transform, With<Player>>,
    // Entities to despawn
    chunks_query: Query<Entity, With<VoxelChunk>>,
    entities_query: Query<Entity, Or<(With<Player>, With<Npc>)>>,
    collision_query: Query<Entity, With<SubVoxel>>,
    lights_query: Query<Entity, Or<(With<DirectionalLight>, With<AmbientLight>)>>,
) {
    for event in reload_events.read() {
        info!("Hot reload: reloading map from {:?}", event.path);

        // Store player position before despawning
        let player_pos = player_query.get_single().ok().map(|t| t.translation);

        // Try to load the new map
        progress.clear();
        let map_result = MapLoader::load_from_file(
            event.path.to_string_lossy().as_ref(),
            &mut progress,
        );

        match map_result {
            Ok(map) => {
                // Despawn all existing map entities
                for entity in chunks_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                for entity in entities_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                for entity in collision_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                for entity in lights_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }

                // Insert new map data (spawn_map_system will handle spawning)
                commands.insert_resource(LoadedMapData { map });
                commands.insert_resource(PendingPlayerPosition(player_pos));

                reloaded_events.send(MapReloadedEvent {
                    success: true,
                    message: "Map reloaded successfully".to_string(),
                });
            }
            Err(e) => {
                warn!("Hot reload failed: {}", e);
                reloaded_events.send(MapReloadedEvent {
                    success: false,
                    message: format!("Reload failed: {}", e),
                });
            }
        }
    }
}

/// Resource to store player position during reload
#[derive(Resource)]
pub struct PendingPlayerPosition(pub Option<Vec3>);
```

#### Step 2.2: Modify Spawn System for Reload Support

**File:** `src/systems/game/map/spawner.rs`

Add support for restoring player position after reload:

```rust
/// System to spawn the map (modified for hot reload support)
pub fn spawn_map_system(
    // ... existing parameters ...
    pending_pos: Option<Res<PendingPlayerPosition>>,
) {
    // ... existing spawn logic ...

    // After spawning player, restore position if available
    if let Some(pending) = pending_pos {
        if let Some(saved_pos) = pending.0 {
            // Validate position is within map bounds
            let clamped_pos = clamp_to_map_bounds(saved_pos, &map_data.map);
            // Update player transform
            // ...
        }
    }

    // Clean up pending position resource
    commands.remove_resource::<PendingPlayerPosition>();
}
```

---

### Phase 3: Game Integration (Priority: High)

#### Step 3.1: Register Systems in Main

**File:** `src/main.rs`

```rust
use systems::game::hot_reload::{
    HotReloadState, MapReloadEvent, MapReloadedEvent,
    poll_hot_reload, handle_map_reload,
};

fn main() {
    App::new()
        // ... existing setup ...
        .init_resource::<HotReloadState>()
        .add_event::<MapReloadEvent>()
        .add_event::<MapReloadedEvent>()
        .add_systems(
            Update,
            poll_hot_reload.run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            Update,
            handle_map_reload
                .run_if(in_state(GameState::InGame))
                .after(poll_hot_reload),
        )
        // ...
}
```

#### Step 3.2: Setup Watcher When Entering InGame

**File:** `src/main.rs` or new system

```rust
/// System to setup hot reload watcher when entering InGame state
fn setup_hot_reload(
    mut hot_reload: ResMut<HotReloadState>,
    cli_map_path: Res<CommandLineMapPath>,
) {
    if let Some(path) = &cli_map_path.path {
        if let Err(e) = hot_reload.watch_file(path.clone()) {
            warn!("Failed to setup hot reload: {}", e);
        }
    }
}
```

---

### Phase 4: Visual Feedback (Priority: Medium)

#### Step 4.1: In-Game Reload Notification

**File:** `src/systems/game/hot_reload.rs`

```rust
/// Component for reload notification UI
#[derive(Component)]
pub struct ReloadNotification {
    pub spawn_time: f32,
    pub duration: f32,
}

/// System to show reload notification
pub fn show_reload_notification(
    mut commands: Commands,
    mut reloaded_events: EventReader<MapReloadedEvent>,
    time: Res<Time>,
) {
    for event in reloaded_events.read() {
        let color = if event.success {
            Color::srgba(0.2, 0.8, 0.2, 0.9)
        } else {
            Color::srgba(0.8, 0.2, 0.2, 0.9)
        };

        // Spawn notification text
        commands.spawn((
            Text::new(&event.message),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(color),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(50.0),
                left: Val::Percent(50.0),
                ..default()
            },
            ReloadNotification {
                spawn_time: time.elapsed_secs(),
                duration: 2.0,
            },
        ));
    }
}

/// System to fade out and despawn notifications
pub fn update_reload_notifications(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &ReloadNotification, &mut TextColor)>,
) {
    for (entity, notification, mut color) in query.iter_mut() {
        let elapsed = time.elapsed_secs() - notification.spawn_time;
        if elapsed > notification.duration {
            commands.entity(entity).despawn();
        } else if elapsed > notification.duration * 0.7 {
            // Fade out in last 30%
            let fade = 1.0 - (elapsed - notification.duration * 0.7) / (notification.duration * 0.3);
            color.0 = color.0.with_alpha(fade);
        }
    }
}
```

#### Step 4.2: Editor Status Indicator

**File:** `src/editor/ui/toolbar.rs`

Add indicator showing hot reload connection status:

```rust
// In toolbar, show hot reload status when game is running
if play_state.is_running {
    ui.separator();
    ui.label(
        egui::RichText::new("🔄 Hot Reload Active")
            .color(egui::Color32::from_rgb(100, 180, 100))
            .small(),
    );
}
```

---

### Phase 5: Manual Reload & Settings (Priority: Low)

#### Step 5.1: Manual Reload Hotkey

**File:** `src/systems/game/hot_reload.rs`

```rust
/// System to handle manual reload hotkey (F5 in game)
pub fn handle_reload_hotkey(
    keyboard: Res<ButtonInput<KeyCode>>,
    hot_reload: Res<HotReloadState>,
    mut reload_events: EventWriter<MapReloadEvent>,
) {
    if keyboard.just_pressed(KeyCode::F5) {
        if let Some(path) = &hot_reload.watched_path {
            info!("Manual reload triggered via F5");
            reload_events.send(MapReloadEvent { path: path.clone() });
        }
    }
}
```

#### Step 5.2: Toggle Hot Reload Setting

Add to pause menu or settings:

```rust
// In pause menu
if ui.checkbox(&mut hot_reload.enabled, "Hot Reload").changed() {
    info!("Hot reload {}", if hot_reload.enabled { "enabled" } else { "disabled" });
}
```

---

## File Changes Summary

### New Files
| File | Purpose |
|------|---------|
| `src/systems/game/hot_reload.rs` | Hot reload module with file watching and reload systems |

### Modified Files
| File | Changes |
|------|---------|
| `Cargo.toml` | Add `notify` dependency |
| `src/main.rs` | Register hot reload systems and resources |
| `src/systems/game/mod.rs` | Export hot_reload module |
| `src/systems/game/map/spawner.rs` | Support position restoration after reload |
| `src/editor/ui/toolbar.rs` | Show hot reload status indicator |

### Dependencies
| Crate | Version | Purpose |
|-------|---------|---------|
| `notify` | 6.1 | Cross-platform file system notifications |

---

## Testing Plan

### Unit Tests
1. `HotReloadState::watch_file` creates watcher successfully
2. `HotReloadState::poll_changes` detects file modifications
3. Debouncing prevents multiple rapid reloads
4. `stop_watching` properly cleans up resources

### Integration Tests
1. Modify map file → game detects change
2. Invalid map file → shows error, keeps running
3. Rapid saves → only one reload occurs (debounce)
4. Player position preserved after reload
5. Hot reload disabled → no reload occurs

### Manual Testing
1. Edit map in editor → game reloads automatically
2. Add/remove voxels → changes appear in game
3. Move spawn point → player moves to new position
4. Break map validation → error shown, game continues
5. F5 in game → manual reload works
6. Close editor → game continues running

---

## Future Enhancements

1. **Partial Reload**: Only reload changed chunks instead of entire map
2. **Entity Preservation**: Keep NPC states/positions when possible
3. **Undo in Game**: Allow undoing last reload
4. **Diff Preview**: Show what changed before applying reload
5. **Networked Reload**: Support hot reload for multiplayer servers
6. **Asset Hot Reload**: Extend to textures, models, sounds

---

## Implementation Order

1. ⏳ Phase 1: File watching infrastructure
2. ⏳ Phase 2: Map reload system
3. ⏳ Phase 3: Game integration
4. ⏳ Phase 4: Visual feedback
5. ⏳ Phase 5: Manual reload & settings

**Estimated Total Time**: 6-8 hours

---

## Appendix: Sequence Diagram

```
┌──────────┐         ┌──────────┐         ┌──────────┐         ┌──────────┐
│  Editor  │         │   File   │         │ Watcher  │         │   Game   │
│          │         │  System  │         │          │         │          │
└────┬─────┘         └────┬─────┘         └────┬─────┘         └────┬─────┘
     │                    │                    │                    │
     │  Save map.ron      │                    │                    │
     │───────────────────>│                    │                    │
     │                    │                    │                    │
     │                    │  File modified     │                    │
     │                    │───────────────────>│                    │
     │                    │                    │                    │
     │                    │                    │  MapReloadEvent    │
     │                    │                    │───────────────────>│
     │                    │                    │                    │
     │                    │                    │                    │ Despawn old
     │                    │                    │                    │────┐
     │                    │                    │                    │    │
     │                    │                    │                    │<───┘
     │                    │                    │                    │
     │                    │       Read file    │                    │
     │                    │<───────────────────────────────────────│
     │                    │                    │                    │
     │                    │       File data    │                    │
     │                    │────────────────────────────────────────>│
     │                    │                    │                    │
     │                    │                    │                    │ Spawn new
     │                    │                    │                    │────┐
     │                    │                    │                    │    │
     │                    │                    │                    │<───┘
     │                    │                    │                    │
     │                    │                    │                    │ Show "Reloaded"
     │                    │                    │                    │────┐
     │                    │                    │                    │    │
     │                    │                    │                    │<───┘
```
