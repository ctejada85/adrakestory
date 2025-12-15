//! Map data structures and format definitions.

mod camera;
mod defaults;
mod entities;
mod lighting;
mod metadata;
mod patterns;
mod rotation;
mod world;

pub use camera::CameraData;
pub use entities::{EntityData, EntityType};
pub use lighting::LightingData;
pub use metadata::MapMetadata;
pub use patterns::SubVoxelPattern;
pub use rotation::RotationState;
pub use world::{VoxelData, WorldData};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete map data structure containing all information needed to load a map.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MapData {
    /// Metadata about the map (name, author, etc.)
    pub metadata: MapMetadata,
    /// World voxel data
    pub world: WorldData,
    /// Entity spawn data
    pub entities: Vec<EntityData>,
    /// Lighting configuration
    pub lighting: LightingData,
    /// Camera configuration
    pub camera: CameraData,
    /// Custom properties for extensibility
    #[serde(default)]
    pub custom_properties: HashMap<String, String>,
}

impl MapData {
    /// Create an empty map with minimal dimensions for starting a new map.
    /// This provides a blank canvas for map creation.
    pub fn empty_map() -> Self {
        Self {
            metadata: MapMetadata {
                name: "Untitled Map".to_string(),
                author: "".to_string(),
                description: "".to_string(),
                version: "1.0.0".to_string(),
                created: "".to_string(),
            },
            world: WorldData {
                width: 1,
                height: 1,
                depth: 1,
                voxels: vec![],
            },
            entities: vec![],
            lighting: LightingData::default(),
            camera: CameraData::default(),
            custom_properties: HashMap::new(),
        }
    }
}
