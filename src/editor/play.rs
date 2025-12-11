//! Play/test functionality for the map editor.
//!
//! This module provides the ability to launch the game directly from the editor
//! to test the map being edited.

use crate::editor::file_io::save_map_to_file;
use crate::editor::state::{EditorState, EditorUIState};
use crate::systems::game::map::format::MapData;
use bevy::prelude::*;
use std::path::PathBuf;
use std::process::Child;
use std::sync::{Arc, Mutex};

/// Event sent when user wants to play/test the map
#[derive(Event)]
pub struct PlayMapEvent;

/// Event sent when user wants to stop the running game
#[derive(Event)]
pub struct StopGameEvent;

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
        let process_exited = if let Some(process_arc) = &self.game_process {
            if let Ok(mut process) = process_arc.lock() {
                match process.try_wait() {
                    Ok(Some(_status)) => {
                        // Process has exited
                        true
                    }
                    Ok(None) => {
                        // Still running
                        return true;
                    }
                    Err(e) => {
                        warn!("Error polling game process: {}", e);
                        true
                    }
                }
            } else {
                false
            }
        } else {
            false
        };

        if process_exited {
            self.is_running = false;
            self.cleanup_temp_file();
        }
        false
    }

    /// Stop the running game process
    pub fn stop_game(&mut self) {
        if let Some(process_arc) = self.game_process.take() {
            if let Ok(mut process) = process_arc.lock() {
                if let Err(e) = process.kill() {
                    warn!("Failed to kill game process: {}", e);
                }
                if let Err(e) = process.wait() {
                    warn!("Failed to wait for game process: {}", e);
                }
            }
        }
        self.is_running = false;
        self.cleanup_temp_file();
    }

    /// Clean up temporary map file
    fn cleanup_temp_file(&mut self) {
        if let Some(path) = self.temp_map_path.take() {
            if let Err(e) = std::fs::remove_file(&path) {
                // Only warn if file exists but couldn't be deleted
                if path.exists() {
                    warn!("Failed to clean up temp map file {:?}: {}", path, e);
                }
            }
        }
    }
}

/// Save map to a temporary file for play testing
pub fn save_to_temp(map: &MapData) -> Result<PathBuf, String> {
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join("adrakestory_editor_playtest.ron");

    save_map_to_file(map, &temp_path)?;
    info!("Saved temp map for play testing: {:?}", temp_path);
    Ok(temp_path)
}

/// Get the path to the game executable
pub fn get_game_executable_path() -> PathBuf {
    // Get the current executable's directory
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            // Look for adrakestory.exe in same directory
            #[cfg(windows)]
            {
                let game_exe = parent.join("adrakestory.exe");
                if game_exe.exists() {
                    return game_exe;
                }
            }

            // Also check without .exe for non-Windows
            #[cfg(not(windows))]
            {
                let game_exe = parent.join("adrakestory");
                if game_exe.exists() {
                    return game_exe;
                }
            }
        }
    }

    // Fallback: assume it's in PATH or current directory
    #[cfg(windows)]
    {
        PathBuf::from("adrakestory.exe")
    }
    #[cfg(not(windows))]
    {
        PathBuf::from("adrakestory")
    }
}

/// System to handle the PlayMapEvent
pub fn handle_play_map(
    mut play_events: EventReader<PlayMapEvent>,
    editor_state: Res<EditorState>,
    mut play_state: ResMut<PlayTestState>,
    mut ui_state: ResMut<EditorUIState>,
) {
    use std::process::Command;

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
        match Command::new(&exe_path).arg("--map").arg(&map_path).spawn() {
            Ok(child) => {
                info!("Started game process with map: {:?}", map_path);
                play_state.game_process = Some(Arc::new(Mutex::new(child)));
                play_state.is_running = true;
            }
            Err(e) => {
                error!("Failed to start game: {}", e);
                ui_state.error_message =
                    format!("Failed to start game:\n{}\n\nExecutable: {:?}", e, exe_path);
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
