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
pub mod geometry;
pub mod loader;
pub mod spawner;
pub mod validation;

pub use loader::{LoadProgress, LoadedMapData, MapLoadProgress, MapLoader};
// Exported for external use (game spawning, editor rendering, chunk management, LOD, material access)
#[allow(unused_imports)]
pub use spawner::{
    spawn_map_system, update_chunk_lods, ChunkLOD, ChunkMeshBuilder, Face, GreedyMesher,
    OccupancyGrid, VoxelChunk, VoxelMaterialPalette, CHUNK_SIZE, LOD_DISTANCES, LOD_LEVELS,
    SUB_VOXEL_COUNT, SUB_VOXEL_SIZE,
};
