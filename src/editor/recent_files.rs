//! Recent files management for the map editor.
//!
//! Tracks recently opened files and persists them to disk.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Maximum number of recent files to track
const MAX_RECENT_FILES: usize = 10;

/// Name of the recent files config file
const RECENT_FILES_FILENAME: &str = "recent_files.ron";

/// Resource tracking recently opened files
#[derive(Resource, Default, Serialize, Deserialize)]
pub struct RecentFiles {
    /// List of recent file paths, most recent first
    pub files: Vec<PathBuf>,
}

impl RecentFiles {
    /// Create a new empty recent files list
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Load recent files from the config directory
    pub fn load() -> Self {
        let config_path = get_config_path();
        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(contents) => {
                    match ron::from_str::<RecentFiles>(&contents) {
                        Ok(mut recent) => {
                            // Filter out files that no longer exist
                            recent.files.retain(|p| p.exists());
                            info!("Loaded {} recent files from {:?}", recent.files.len(), config_path);
                            return recent;
                        }
                        Err(e) => {
                            warn!("Failed to parse recent files config: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read recent files config: {}", e);
                }
            }
        }
        Self::new()
    }

    /// Save recent files to the config directory
    pub fn save(&self) {
        let config_path = get_config_path();
        
        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                error!("Failed to create config directory: {}", e);
                return;
            }
        }

        // Serialize and write
        match ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()) {
            Ok(contents) => {
                if let Err(e) = fs::write(&config_path, contents) {
                    error!("Failed to write recent files config: {}", e);
                } else {
                    info!("Saved {} recent files to {:?}", self.files.len(), config_path);
                }
            }
            Err(e) => {
                error!("Failed to serialize recent files: {}", e);
            }
        }
    }

    /// Add a file to the recent files list
    /// Moves it to the front if already present
    pub fn add(&mut self, path: PathBuf) {
        // Canonicalize the path if possible for consistent comparison
        let path = path.canonicalize().unwrap_or(path);

        // Remove if already in list (will re-add at front)
        self.files.retain(|p| {
            p.canonicalize().unwrap_or_else(|_| p.clone()) != path
        });

        // Add to front
        self.files.insert(0, path);

        // Trim to max size
        if self.files.len() > MAX_RECENT_FILES {
            self.files.truncate(MAX_RECENT_FILES);
        }

        // Save to disk
        self.save();
    }

    /// Clear all recent files
    pub fn clear(&mut self) {
        self.files.clear();
        self.save();
    }

    /// Check if there are any recent files
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Get the number of recent files
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Get a display name for a file path (filename only)
    pub fn get_display_name(path: &PathBuf) -> String {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string()
    }

    /// Get a shortened path for display (last 2-3 components)
    pub fn get_short_path(path: &PathBuf) -> String {
        let components: Vec<_> = path.components().collect();
        if components.len() <= 3 {
            path.display().to_string()
        } else {
            // Show last 3 path components
            let start = components.len() - 3;
            let short: PathBuf = components[start..].iter().collect();
            format!(".../{}", short.display())
        }
    }
}

/// Event to open a recent file
#[derive(Event)]
pub struct OpenRecentFileEvent {
    pub path: PathBuf,
}

/// Get the path to the config file
fn get_config_path() -> PathBuf {
    // Try to use a standard config directory
    if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("adrakestory").join(RECENT_FILES_FILENAME)
    } else {
        // Fallback to current directory
        PathBuf::from(RECENT_FILES_FILENAME)
    }
}

/// System to update recent files when a file is saved
pub fn update_recent_on_save(
    mut events: EventReader<super::file_io::FileSavedEvent>,
    mut recent_files: ResMut<RecentFiles>,
) {
    for event in events.read() {
        recent_files.add(event.path.clone());
    }
}
