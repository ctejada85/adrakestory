//! Map loading and management system.
//!
//! This module provides functionality for loading maps from RON files,
//! validating them, and spawning them into the game world with progress tracking.
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use bevy::prelude::*;
//! use crate::systems::game::map::{MapLoader, LoadedMapData, MapLoadProgress};
//!
//! fn load_map(mut commands: Commands, mut progress: ResMut<MapLoadProgress>) {
//!     match MapLoader::load_from_file("assets/maps/level1.ron", &mut progress) {
//!         Ok(map) => {
//!             commands.insert_resource(LoadedMapData { map });
//!         }
//!         Err(e) => {
//!             eprintln!("Failed to load map: {}", e);
//!         }
//!     }
//! }
//! ```

pub mod error;
pub mod format;
pub mod loader;
pub mod spawner;
pub mod validation;

// Re-export commonly used types
pub use error::{MapLoadError, MapResult};
pub use format::{
    CameraData, DirectionalLightData, EntityData, EntityType, LightingData, MapData, MapMetadata,
    SubVoxelPattern, VoxelData, WorldData,
};
pub use loader::{LoadProgress, LoadedMapData, MapLoadProgress, MapLoader};
pub use spawner::spawn_map_system;
pub use validation::validate_map;
