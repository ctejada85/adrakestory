//! Map loading functionality with progress tracking.

use super::error::{MapLoadError, MapResult};
use super::format::MapData;
use super::validation::validate_map;
use bevy::prelude::*;
use std::fs;
use std::path::Path;

/// Progress stages during map loading.
#[derive(Clone, Debug, PartialEq)]
pub enum LoadProgress {
    /// Loading has started
    Started,
    /// Reading file from disk (0-20%)
    LoadingFile(f32),
    /// Parsing RON data (20-40%)
    ParsingData(f32),
    /// Validating map data (40-60%)
    ValidatingMap(f32),
    /// Spawning voxels (60-90%)
    SpawningVoxels(f32),
    /// Spawning entities (90-95%)
    SpawningEntities(f32),
    /// Finalizing setup (95-100%)
    Finalizing(f32),
    /// Loading complete
    Complete,
    /// Error occurred
    Error(String),
}

impl LoadProgress {
    /// Get the overall percentage (0.0 to 1.0) for this progress stage.
    pub fn percentage(&self) -> f32 {
        match self {
            Self::Started => 0.0,
            Self::LoadingFile(p) => p * 0.2,
            Self::ParsingData(p) => 0.2 + (p * 0.2),
            Self::ValidatingMap(p) => 0.4 + (p * 0.2),
            Self::SpawningVoxels(p) => 0.6 + (p * 0.3),
            Self::SpawningEntities(p) => 0.9 + (p * 0.05),
            Self::Finalizing(p) => 0.95 + (p * 0.05),
            Self::Complete => 1.0,
            Self::Error(_) => 0.0,
        }
    }

    /// Get a human-readable description of the current progress.
    pub fn description(&self) -> String {
        match self {
            Self::Started => "Starting map load...".to_string(),
            Self::LoadingFile(p) => format!("Loading file... {:.0}%", p * 100.0),
            Self::ParsingData(p) => format!("Parsing map data... {:.0}%", p * 100.0),
            Self::ValidatingMap(p) => format!("Validating map... {:.0}%", p * 100.0),
            Self::SpawningVoxels(p) => format!("Spawning voxels... {:.0}%", p * 100.0),
            Self::SpawningEntities(p) => format!("Spawning entities... {:.0}%", p * 100.0),
            Self::Finalizing(p) => format!("Finalizing... {:.0}%", p * 100.0),
            Self::Complete => "Complete!".to_string(),
            Self::Error(msg) => format!("Error: {}", msg),
        }
    }
}

/// Resource to track map loading progress.
#[derive(Resource, Default)]
pub struct MapLoadProgress {
    /// Current progress stage
    pub current: Option<LoadProgress>,
    /// History of progress events
    pub events: Vec<LoadProgress>,
}

impl MapLoadProgress {
    /// Create a new progress tracker.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            current: None,
            events: Vec::new(),
        }
    }

    /// Update the current progress.
    pub fn update(&mut self, progress: LoadProgress) {
        self.current = Some(progress.clone());
        self.events.push(progress);
    }

    /// Get the current percentage (0.0 to 1.0).
    pub fn percentage(&self) -> f32 {
        self.current.as_ref().map(|p| p.percentage()).unwrap_or(0.0)
    }

    /// Check if loading is complete.
    pub fn is_complete(&self) -> bool {
        matches!(self.current, Some(LoadProgress::Complete))
    }

    /// Check if an error occurred.
    #[allow(dead_code)]
    pub fn has_error(&self) -> bool {
        matches!(self.current, Some(LoadProgress::Error(_)))
    }

    /// Clear the progress tracker.
    pub fn clear(&mut self) {
        self.current = None;
        self.events.clear();
    }
}

/// Resource containing the loaded map data.
#[derive(Resource)]
pub struct LoadedMapData {
    pub map: MapData,
}

/// Map loader with progress tracking.
pub struct MapLoader;

impl MapLoader {
    /// Load a map from a file path with progress tracking.
    ///
    /// This function loads the map synchronously and updates the progress
    /// resource as it goes through each stage.
    pub fn load_from_file(
        path: impl AsRef<Path>,
        progress: &mut MapLoadProgress,
    ) -> MapResult<MapData> {
        // Start loading
        progress.update(LoadProgress::Started);

        // Stage 1: Load file (0-20%)
        progress.update(LoadProgress::LoadingFile(0.0));
        let content = fs::read_to_string(path.as_ref())?;
        progress.update(LoadProgress::LoadingFile(1.0));

        // Stage 2: Parse RON data (20-40%)
        progress.update(LoadProgress::ParsingData(0.0));
        let map: MapData = ron::from_str(&content)?;
        progress.update(LoadProgress::ParsingData(1.0));

        // Stage 3: Validate map (40-60%)
        progress.update(LoadProgress::ValidatingMap(0.0));
        validate_map(&map)?;
        progress.update(LoadProgress::ValidatingMap(1.0));

        Ok(map)
    }

    /// Load a map from a file path without progress tracking.
    ///
    /// This is a simpler version for cases where progress tracking is not needed.
    #[allow(dead_code)]
    pub fn load_simple(path: impl AsRef<Path>) -> MapResult<MapData> {
        let content = fs::read_to_string(path.as_ref())?;
        let map: MapData = ron::from_str(&content)?;
        validate_map(&map)?;
        Ok(map)
    }

    /// Save a map to a file.
    ///
    /// This can be used by a map editor to save maps.
    #[allow(dead_code)]
    pub fn save_to_file(map: &MapData, path: impl AsRef<Path>) -> MapResult<()> {
        let ron_string = ron::ser::to_string_pretty(map, ron::ser::PrettyConfig::default())
            .map_err(|e| {
                MapLoadError::ValidationError(format!("Failed to serialize map: {}", e))
            })?;

        fs::write(path.as_ref(), ron_string)?;
        Ok(())
    }

    /// Load the default map (fallback when no file is specified).
    pub fn load_default() -> MapData {
        MapData::default_map()
    }
}

/// System to load a map from a file path.
///
/// This system should be run once when entering the LoadingMap state.
#[allow(dead_code)]
pub fn load_map_system(
    mut commands: Commands,
    mut progress: ResMut<MapLoadProgress>,
    // In a real implementation, you'd get the path from a resource or event
) {
    // For now, try to load from a default path, or use the default map
    let map = match MapLoader::load_from_file("assets/maps/default.ron", &mut progress) {
        Ok(map) => map,
        Err(e) => {
            warn!("Failed to load map: {}. Using default map.", e);
            progress.update(LoadProgress::Error(e.to_string()));
            MapLoader::load_default()
        }
    };

    commands.insert_resource(LoadedMapData { map });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_progress_percentage() {
        assert_eq!(LoadProgress::Started.percentage(), 0.0);
        assert_eq!(LoadProgress::LoadingFile(0.5).percentage(), 0.1);
        assert_eq!(LoadProgress::ParsingData(0.5).percentage(), 0.3);
        assert_eq!(LoadProgress::ValidatingMap(0.5).percentage(), 0.5);
        assert_eq!(LoadProgress::SpawningVoxels(0.5).percentage(), 0.75);
        assert_eq!(LoadProgress::SpawningEntities(0.5).percentage(), 0.925);
        assert_eq!(LoadProgress::Finalizing(0.5).percentage(), 0.975);
        assert_eq!(LoadProgress::Complete.percentage(), 1.0);
    }

    #[test]
    fn test_map_load_progress() {
        let mut progress = MapLoadProgress::new();
        assert!(!progress.is_complete());
        assert!(!progress.has_error());

        progress.update(LoadProgress::Started);
        assert_eq!(progress.percentage(), 0.0);

        progress.update(LoadProgress::Complete);
        assert!(progress.is_complete());
        assert_eq!(progress.percentage(), 1.0);
    }

    #[test]
    fn test_load_default_map() {
        let map = MapLoader::load_default();
        assert_eq!(map.metadata.name, "Default Map");
        assert!(validate_map(&map).is_ok());
    }
}
