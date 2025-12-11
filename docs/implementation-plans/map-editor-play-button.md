# Map Editor Play Button Implementation Plan

## Overview

Add a "Play" button to the map editor toolbar that allows developers to quickly test the map they're currently editing by launching the game with that specific map loaded. This will significantly improve the map development workflow by eliminating the need to manually save, switch to a terminal, and run the game with command-line arguments.

## Requirements

### Functional Requirements
1. Play button visible in the map editor toolbar
2. Clicking play saves the current map (if modified) to a temporary location
3. Launches the main game executable with the map loaded
4. Option to stop/kill the running game from the editor
5. Keyboard shortcut support (F5 for Play)
6. Visual indicator when game is running

### Non-Functional Requirements
1. Game launches as a separate process (not blocking the editor)
2. Editor remains fully functional while game is running
3. Works on Windows (primary), with cross-platform support design
4. Minimal latency between pressing Play and game appearing

## Architecture

### Process Flow
```
┌─────────────────────────────────────────────────────────────────────┐
│                        MAP EDITOR                                    │
│                                                                      │
│  1. User clicks Play (▶) or presses F5                              │
│                          │                                           │
│                          ▼                                           │
│  2. Check if map modified ──────────────────┐                       │
│          │                                   │                       │
│          ▼ (yes)                            ▼ (no)                  │
│  3. Auto-save to temp file         4. Use current file path         │
│          │                                   │                       │
│          └──────────────┬───────────────────┘                       │
│                         ▼                                            │
│  5. Spawn game process with --map <path> argument                   │
│                         │                                            │
│                         ▼                                            │
│  6. Store process handle, update UI state                           │
│                         │                                            │
│                         ▼                                            │
│  7. Show "Stop" button (■), play button becomes disabled            │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      GAME PROCESS                                    │
│                                                                      │
│  - Parses --map argument                                            │
│  - Skips title screen, loads directly into map                      │
│  - Full game functionality for testing                              │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Data Flow
1. Editor detects Play action (button click or F5)
2. If map modified, saves to temporary file in system temp directory
3. Spawns `adrakestory.exe` with `--map <path>` argument
4. Stores `Child` process handle in resource
5. UI polls process status to detect when game closes
6. When game closes or Stop pressed, cleanup and re-enable Play

## Implementation Steps

### Phase 1: Command-Line Map Loading in Main Game (Priority: High)

#### Step 1.1: Add CLI Argument Parsing

**File:** `src/main.rs`

Add `clap` dependency for argument parsing, or use simple `std::env::args()`:

```rust
use std::env;

/// Command-line arguments for the game
struct GameArgs {
    /// Path to map file to load directly (skips title screen)
    map_path: Option<String>,
}

fn parse_args() -> GameArgs {
    let args: Vec<String> = env::args().collect();
    let mut map_path = None;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--map" | "-m" => {
                if i + 1 < args.len() {
                    map_path = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    
    GameArgs { map_path }
}
```

#### Step 1.2: Add Resource for Direct Map Loading

**File:** `src/systems/game/resources.rs` (or new file)

```rust
/// Resource to hold command-line specified map path
#[derive(Resource, Default)]
pub struct CommandLineMapPath {
    pub path: Option<String>,
}
```

#### Step 1.3: Modify Game Startup Flow

**File:** `src/main.rs`

```rust
fn main() {
    let args = parse_args();
    
    App::new()
        // ... existing plugins ...
        .insert_resource(CommandLineMapPath { path: args.map_path.clone() })
        // If map specified, start in LoadingMap state instead of IntroAnimation
        .init_state_with(if args.map_path.is_some() {
            GameState::LoadingMap
        } else {
            GameState::IntroAnimation
        })
        // ... rest of app setup ...
}
```

#### Step 1.4: Modify Map Loading to Use CLI Path

**File:** `src/systems/game/map/mod.rs` or loading system

Modify `load_map_on_enter` to check for CLI path:

```rust
pub fn load_map_on_enter(
    mut commands: Commands,
    cli_map: Res<CommandLineMapPath>,
    // ... other params ...
) {
    let map_path = if let Some(path) = &cli_map.path {
        PathBuf::from(path)
    } else {
        // Default map path
        PathBuf::from("assets/maps/default.ron")
    };
    
    // Load the specified map...
}
```

---

### Phase 2: Editor Play Button Infrastructure (Priority: High)

#### Step 2.1: Add Play State Resource

**File:** `src/editor/state.rs`

```rust
use std::process::Child;
use std::sync::{Arc, Mutex};

/// State for the play/test functionality
#[derive(Resource, Default)]
pub struct PlayTestState {
    /// Handle to the running game process
    pub game_process: Option<Arc<Mutex<Child>>>,
    /// Whether the game is currently running
    pub is_running: bool,
    /// Path to the temporary map file (if created)
    pub temp_map_path: Option<PathBuf>,
}

impl PlayTestState {
    /// Check if the game process is still running
    pub fn poll_process(&mut self) -> bool {
        if let Some(process_arc) = &self.game_process {
            if let Ok(mut process) = process_arc.lock() {
                match process.try_wait() {
                    Ok(Some(_status)) => {
                        // Process has exited
                        self.is_running = false;
                        self.cleanup_temp_file();
                        return false;
                    }
                    Ok(None) => {
                        // Still running
                        return true;
                    }
                    Err(_) => {
                        self.is_running = false;
                        return false;
                    }
                }
            }
        }
        false
    }
    
    /// Stop the running game process
    pub fn stop_game(&mut self) {
        if let Some(process_arc) = self.game_process.take() {
            if let Ok(mut process) = process_arc.lock() {
                let _ = process.kill();
                let _ = process.wait();
            }
        }
        self.is_running = false;
        self.cleanup_temp_file();
    }
    
    /// Clean up temporary map file
    fn cleanup_temp_file(&mut self) {
        if let Some(path) = self.temp_map_path.take() {
            let _ = std::fs::remove_file(path);
        }
    }
}
```

#### Step 2.2: Add Play Events

**File:** `src/editor/mod.rs` or new `src/editor/play.rs`

```rust
/// Event sent when user wants to play/test the map
#[derive(Event)]
pub struct PlayMapEvent;

/// Event sent when user wants to stop the running game
#[derive(Event)]
pub struct StopGameEvent;
```

---

### Phase 3: Play Button UI (Priority: High)

#### Step 3.1: Add Play Button to Toolbar

**File:** `src/editor/ui/toolbar.rs`

Add to the `render_toolbar` function signature:

```rust
pub fn render_toolbar(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    ui_state: &mut EditorUIState,
    tool_memory: &mut ToolMemory,
    history: &EditorHistory,
    recent_files: &mut RecentFiles,
    play_state: &mut PlayTestState,  // NEW
    save_events: &mut EventWriter<SaveMapEvent>,
    save_as_events: &mut EventWriter<SaveMapAsEvent>,
    open_recent_events: &mut EventWriter<OpenRecentFileEvent>,
    play_events: &mut EventWriter<PlayMapEvent>,  // NEW
    stop_events: &mut EventWriter<StopGameEvent>,  // NEW
)
```

Add play/stop buttons in toolbar horizontal section:

```rust
// In the horizontal toolbar section, after tool buttons
ui.separator();

// === Play/Test Section ===
render_play_controls(ui, play_state, play_events, stop_events);
```

New function for play controls:

```rust
/// Render play/stop buttons for map testing
fn render_play_controls(
    ui: &mut egui::Ui,
    play_state: &mut PlayTestState,
    play_events: &mut EventWriter<PlayMapEvent>,
    stop_events: &mut EventWriter<StopGameEvent>,
) {
    if play_state.is_running {
        // Show Stop button when game is running
        let stop_button = egui::Button::new("⏹ Stop")
            .fill(egui::Color32::from_rgb(180, 60, 60))
            .min_size(egui::vec2(60.0, 24.0));
        
        if ui.add(stop_button)
            .on_hover_text("Stop the running game (Shift+F5)")
            .clicked() 
        {
            stop_events.send(StopGameEvent);
        }
        
        // Running indicator
        ui.label(egui::RichText::new("● Running")
            .color(egui::Color32::from_rgb(100, 200, 100)));
    } else {
        // Show Play button when game is not running
        let play_button = egui::Button::new("▶ Play")
            .fill(egui::Color32::from_rgb(60, 140, 60))
            .min_size(egui::vec2(60.0, 24.0));
        
        if ui.add(play_button)
            .on_hover_text("Test map in game (F5)")
            .clicked() 
        {
            play_events.send(PlayMapEvent);
        }
    }
}
```

#### Step 3.2: Add Play Menu Item

Add to the existing menu bar in `render_file_menu` or create new `render_run_menu`:

```rust
fn render_run_menu(
    ui: &mut egui::Ui,
    play_state: &PlayTestState,
    play_events: &mut EventWriter<PlayMapEvent>,
    stop_events: &mut EventWriter<StopGameEvent>,
) {
    ui.menu_button("Run", |ui| {
        if play_state.is_running {
            if ui.button("⏹ Stop Game      Shift+F5").clicked() {
                stop_events.send(StopGameEvent);
                ui.close_menu();
            }
        } else {
            if ui.button("▶ Play Map           F5").clicked() {
                play_events.send(PlayMapEvent);
                ui.close_menu();
            }
        }
        
        ui.separator();
        
        ui.add_enabled(false, egui::Button::new("⚙ Run Settings..."));
    });
}
```

---

### Phase 4: Play System Implementation (Priority: High)

#### Step 4.1: Handle Play Event

**File:** `src/editor/play.rs` (new file)

```rust
//! Play/test functionality for the map editor.

use crate::editor::state::{EditorState, EditorUIState, PlayTestState};
use crate::editor::file_io::save_map_to_file;
use bevy::prelude::*;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

/// Event sent when user wants to play/test the map
#[derive(Event)]
pub struct PlayMapEvent;

/// Event sent when user wants to stop the running game
#[derive(Event)]
pub struct StopGameEvent;

/// System to handle the PlayMapEvent
pub fn handle_play_map(
    mut play_events: EventReader<PlayMapEvent>,
    editor_state: Res<EditorState>,
    mut play_state: ResMut<PlayTestState>,
    mut ui_state: ResMut<EditorUIState>,
) {
    for _event in play_events.read() {
        if play_state.is_running {
            warn!("Game is already running");
            return;
        }
        
        // Determine map path to use
        let map_path = if let Some(path) = &editor_state.file_path {
            if editor_state.is_modified {
                // Save to temp file if modified
                match save_to_temp(&editor_state.current_map) {
                    Ok(temp_path) => {
                        play_state.temp_map_path = Some(temp_path.clone());
                        temp_path
                    }
                    Err(e) => {
                        ui_state.error_message = format!("Failed to save temp map: {}", e);
                        ui_state.error_dialog_open = true;
                        return;
                    }
                }
            } else {
                path.clone()
            }
        } else {
            // No file path - save to temp
            match save_to_temp(&editor_state.current_map) {
                Ok(temp_path) => {
                    play_state.temp_map_path = Some(temp_path.clone());
                    temp_path
                }
                Err(e) => {
                    ui_state.error_message = format!("Failed to save temp map: {}", e);
                    ui_state.error_dialog_open = true;
                    return;
                }
            }
        };
        
        // Get path to game executable
        let exe_path = get_game_executable_path();
        
        // Spawn game process
        match Command::new(&exe_path)
            .arg("--map")
            .arg(&map_path)
            .spawn()
        {
            Ok(child) => {
                info!("Started game process with map: {:?}", map_path);
                play_state.game_process = Some(Arc::new(Mutex::new(child)));
                play_state.is_running = true;
            }
            Err(e) => {
                error!("Failed to start game: {}", e);
                ui_state.error_message = format!(
                    "Failed to start game:\n{}\n\nExecutable: {:?}",
                    e, exe_path
                );
                ui_state.error_dialog_open = true;
            }
        }
    }
}

/// System to handle the StopGameEvent
pub fn handle_stop_game(
    mut stop_events: EventReader<StopGameEvent>,
    mut play_state: ResMut<PlayTestState>,
) {
    for _event in stop_events.read() {
        play_state.stop_game();
        info!("Game stopped by user");
    }
}

/// System to poll game process status
pub fn poll_game_process(mut play_state: ResMut<PlayTestState>) {
    if play_state.is_running {
        let still_running = play_state.poll_process();
        if !still_running {
            info!("Game process has exited");
            play_state.game_process = None;
        }
    }
}

/// Save map to a temporary file
fn save_to_temp(map: &crate::systems::game::map::format::MapData) -> Result<PathBuf, String> {
    let temp_dir = env::temp_dir();
    let temp_path = temp_dir.join("adrakestory_editor_playtest.ron");
    
    save_map_to_file(map, &temp_path)?;
    Ok(temp_path)
}

/// Get the path to the game executable
fn get_game_executable_path() -> PathBuf {
    // Get the current executable's directory
    if let Ok(current_exe) = env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            // Look for adrakestory.exe in same directory
            let game_exe = parent.join("adrakestory.exe");
            if game_exe.exists() {
                return game_exe;
            }
            
            // Also check without .exe for non-Windows
            let game_exe_no_ext = parent.join("adrakestory");
            if game_exe_no_ext.exists() {
                return game_exe_no_ext;
            }
        }
    }
    
    // Fallback: assume it's in PATH or current directory
    #[cfg(windows)]
    { PathBuf::from("adrakestory.exe") }
    #[cfg(not(windows))]
    { PathBuf::from("adrakestory") }
}
```

#### Step 4.2: Register Systems in map_editor.rs

**File:** `src/bin/map_editor.rs`

```rust
use adrakestory::editor::play::{
    PlayMapEvent, StopGameEvent, PlayTestState,
    handle_play_map, handle_stop_game, poll_game_process,
};

// In main():
    .init_resource::<PlayTestState>()
    .add_event::<PlayMapEvent>()
    .add_event::<StopGameEvent>()
    .add_systems(Update, handle_play_map)
    .add_systems(Update, handle_stop_game)
    .add_systems(Update, poll_game_process)
```

---

### Phase 5: Keyboard Shortcuts (Priority: Medium)

#### Step 5.1: Add F5 Shortcut

**File:** `src/editor/cursor.rs` or `src/editor/tools/input.rs`

Add to keyboard handling:

```rust
pub fn handle_play_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    play_state: Res<PlayTestState>,
    mut play_events: EventWriter<PlayMapEvent>,
    mut stop_events: EventWriter<StopGameEvent>,
    egui_contexts: Query<&bevy_egui::EguiContext>,
) {
    // Don't handle shortcuts if egui has focus
    let ctx = egui_contexts.single();
    if ctx.get().wants_keyboard_input() {
        return;
    }
    
    // F5 to Play
    if keyboard.just_pressed(KeyCode::F5) {
        if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
            // Shift+F5 to Stop
            if play_state.is_running {
                stop_events.send(StopGameEvent);
            }
        } else {
            // F5 to Play
            if !play_state.is_running {
                play_events.send(PlayMapEvent);
            }
        }
    }
}
```

---

### Phase 6: Polish and Edge Cases (Priority: Low)

#### Step 6.1: Handle Editor Close While Game Running

When the editor is closed, should we also close the running game?

```rust
// In handle_app_exit or window close handler:
if play_state.is_running {
    play_state.stop_game();
}
```

#### Step 6.2: Auto-Refresh After Game Closes (Optional)

Could detect when game closes and offer to reload if the map file was modified externally:

```rust
pub fn check_external_map_changes(
    play_state: Res<PlayTestState>,
    editor_state: Res<EditorState>,
    // ...
) {
    // After game exits, check if the map file was modified
    // Could offer to reload
}
```

#### Step 6.3: Multiple Instance Prevention (Optional)

Prevent launching multiple game instances:

```rust
if play_state.is_running {
    ui_state.warning_message = "Game is already running. Stop it first.".to_string();
    return;
}
```

---

## File Changes Summary

### New Files
| File | Purpose |
|------|---------|
| `src/editor/play.rs` | Play/test functionality module |

### Modified Files
| File | Changes |
|------|---------|
| `src/main.rs` | Add CLI argument parsing, conditional initial state |
| `src/editor/mod.rs` | Export play module |
| `src/editor/state.rs` | Add `PlayTestState` resource |
| `src/editor/ui/toolbar.rs` | Add play/stop buttons, Run menu |
| `src/bin/map_editor.rs` | Register play systems and resources |
| `src/editor/cursor.rs` | Add F5 keyboard shortcut |

### Dependencies
No new dependencies required. Uses `std::process::Command` for spawning.

---

## Testing Plan

### Unit Tests
1. `parse_args()` correctly handles `--map` argument
2. `save_to_temp()` creates valid temp file
3. `PlayTestState::poll_process()` correctly detects exit

### Integration Tests
1. Editor can launch game with existing saved map
2. Editor can launch game with unsaved (temp) map
3. Stop button terminates game process
4. F5 shortcut works when editor viewport focused
5. Game loads specified map and skips title screen

### Manual Testing
1. Play button appears in toolbar
2. Click play → game launches → Stop button appears
3. Game uses current map (verify visually)
4. Stop button kills game
5. Game exit (via menu/X) clears Running state
6. Modified unsaved map works (temp file)
7. Multiple clicks don't spawn multiple games

---

## Future Enhancements

1. **Hot Reload**: Detect map changes in editor and offer to reload in running game
2. **Debug Mode**: Option to launch game with debug overlays enabled
3. **Spawn Point Selection**: Choose which spawn point to use
4. **Play Settings Dialog**: Configure resolution, windowed mode, etc.
5. **Console Output**: Show game stdout/stderr in editor panel
6. **Breakpoints**: Pause game at specific trigger entities

---

## Implementation Order

1. ✅ Phase 1: CLI map loading (game side) - enables the feature
2. ✅ Phase 2: Play state resource (editor infrastructure)
3. ✅ Phase 3: UI buttons (user interface)
4. ⏳ Phase 4: Play system (core logic)
5. ⏳ Phase 5: Keyboard shortcuts (polish)
6. ⏳ Phase 6: Edge cases and polish (optional)

**Estimated Total Time**: 4-6 hours

---

## Appendix: UI Mockup

### Toolbar with Play Button
```
┌──────────────────────────────────────────────────────────────────────────────┐
│ File  Edit  View  Run  Tools  Help                              [village*] ▼│
├──────────────────────────────────────────────────────────────────────────────┤
│ [🔲][✏️][🗑️][📍][📷] │ Grass ▼ │ Full ▼ │ ▶ Play │ Grid │ Snap │ [village*] │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Toolbar When Game Running
```
┌──────────────────────────────────────────────────────────────────────────────┐
│ File  Edit  View  Run  Tools  Help                              [village*] ▼│
├──────────────────────────────────────────────────────────────────────────────┤
│ [🔲][✏️][🗑️][📍][📷] │ Grass ▼ │ Full ▼ │ ⏹ Stop │ ● Running │ Grid │ Snap │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Run Menu
```
┌─────────────────────┐
│ ▶ Play Map      F5  │
│ ⏹ Stop     Shift+F5 │
│ ─────────────────── │
│ ⚙ Run Settings...   │
└─────────────────────┘
```
