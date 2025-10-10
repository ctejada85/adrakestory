//! Map data structures and format definitions.

use super::super::components::VoxelType;
use bevy::prelude::*;
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

/// Metadata about the map.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MapMetadata {
    /// Display name of the map
    pub name: String,
    /// Author/creator of the map
    pub author: String,
    /// Description of the map
    pub description: String,
    /// Map format version
    pub version: String,
    /// Creation date (ISO 8601 format recommended)
    pub created: String,
}

/// World voxel data.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorldData {
    /// Width of the world in voxels
    pub width: i32,
    /// Height of the world in voxels
    pub height: i32,
    /// Depth of the world in voxels
    pub depth: i32,
    /// List of non-air voxels with their positions and types
    pub voxels: Vec<VoxelData>,
}

/// Individual voxel data.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VoxelData {
    /// Position in the world grid (x, y, z)
    pub pos: (i32, i32, i32),
    /// Type of voxel
    pub voxel_type: VoxelType,
    /// Optional sub-voxel pattern
    #[serde(default)]
    pub pattern: Option<SubVoxelPattern>,
}

/// Sub-voxel patterns for different voxel appearances.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubVoxelPattern {
    /// Full 8x8x8 cube of sub-voxels
    Full,
    /// Thin 8x1x8 platform
    Platform,
    /// Progressive height increase (stairs)
    Staircase,
    /// Small 2x2x2 centered column
    Pillar,
}

impl Default for SubVoxelPattern {
    fn default() -> Self {
        Self::Full
    }
}

/// Entity spawn data.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EntityData {
    /// Type of entity to spawn
    pub entity_type: EntityType,
    /// World position (x, y, z)
    pub position: (f32, f32, f32),
    /// Custom properties for this entity
    #[serde(default)]
    pub properties: HashMap<String, String>,
}

/// Types of entities that can be spawned.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityType {
    /// Player spawn point
    PlayerSpawn,
    /// Enemy spawn point
    Enemy,
    /// Item spawn point
    Item,
    /// Trigger volume
    Trigger,
}

/// Lighting configuration for the map.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LightingData {
    /// Ambient light intensity (0.0 to 1.0)
    pub ambient_intensity: f32,
    /// Optional directional light
    pub directional_light: Option<DirectionalLightData>,
}

impl Default for LightingData {
    fn default() -> Self {
        Self {
            ambient_intensity: 0.3,
            directional_light: Some(DirectionalLightData::default()),
        }
    }
}

/// Directional light configuration.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DirectionalLightData {
    /// Light direction (x, y, z) - will be normalized
    pub direction: (f32, f32, f32),
    /// Light intensity in lux
    pub illuminance: f32,
    /// Light color (r, g, b) in 0.0-1.0 range
    pub color: (f32, f32, f32),
}

impl Default for DirectionalLightData {
    fn default() -> Self {
        Self {
            direction: (-0.5, -1.0, -0.5),
            illuminance: 10000.0,
            color: (1.0, 1.0, 1.0),
        }
    }
}

/// Camera configuration for the map.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CameraData {
    /// Camera position (x, y, z)
    pub position: (f32, f32, f32),
    /// Point the camera looks at (x, y, z)
    pub look_at: (f32, f32, f32),
    /// Additional rotation offset in radians
    pub rotation_offset: f32,
}

impl Default for CameraData {
    fn default() -> Self {
        Self {
            position: (1.5, 8.0, 5.5),
            look_at: (1.5, 0.0, 1.5),
            rotation_offset: -std::f32::consts::FRAC_PI_2,
        }
    }
}

impl MapData {
    /// Create a default map data for testing or fallback.
    pub fn default_map() -> Self {
        Self {
            metadata: MapMetadata {
                name: "Default Map".to_string(),
                author: "System".to_string(),
                description: "Default procedurally generated map".to_string(),
                version: "1.0.0".to_string(),
                created: "2025-01-10".to_string(),
            },
            world: WorldData {
                width: 4,
                height: 3,
                depth: 4,
                voxels: vec![
                    // Floor layer
                    VoxelData {
                        pos: (0, 0, 0),
                        voxel_type: VoxelType::Grass,
                        pattern: Some(SubVoxelPattern::Full),
                    },
                    VoxelData {
                        pos: (1, 0, 0),
                        voxel_type: VoxelType::Grass,
                        pattern: Some(SubVoxelPattern::Full),
                    },
                    VoxelData {
                        pos: (2, 0, 0),
                        voxel_type: VoxelType::Grass,
                        pattern: Some(SubVoxelPattern::Full),
                    },
                    VoxelData {
                        pos: (3, 0, 0),
                        voxel_type: VoxelType::Grass,
                        pattern: Some(SubVoxelPattern::Full),
                    },
                    VoxelData {
                        pos: (0, 0, 1),
                        voxel_type: VoxelType::Grass,
                        pattern: Some(SubVoxelPattern::Full),
                    },
                    VoxelData {
                        pos: (1, 0, 1),
                        voxel_type: VoxelType::Grass,
                        pattern: Some(SubVoxelPattern::Full),
                    },
                    VoxelData {
                        pos: (2, 0, 1),
                        voxel_type: VoxelType::Grass,
                        pattern: Some(SubVoxelPattern::Full),
                    },
                    VoxelData {
                        pos: (3, 0, 1),
                        voxel_type: VoxelType::Grass,
                        pattern: Some(SubVoxelPattern::Full),
                    },
                    VoxelData {
                        pos: (0, 0, 2),
                        voxel_type: VoxelType::Grass,
                        pattern: Some(SubVoxelPattern::Full),
                    },
                    VoxelData {
                        pos: (1, 0, 2),
                        voxel_type: VoxelType::Grass,
                        pattern: Some(SubVoxelPattern::Full),
                    },
                    VoxelData {
                        pos: (2, 0, 2),
                        voxel_type: VoxelType::Grass,
                        pattern: Some(SubVoxelPattern::Full),
                    },
                    VoxelData {
                        pos: (3, 0, 2),
                        voxel_type: VoxelType::Grass,
                        pattern: Some(SubVoxelPattern::Full),
                    },
                    VoxelData {
                        pos: (0, 0, 3),
                        voxel_type: VoxelType::Grass,
                        pattern: Some(SubVoxelPattern::Full),
                    },
                    VoxelData {
                        pos: (1, 0, 3),
                        voxel_type: VoxelType::Grass,
                        pattern: Some(SubVoxelPattern::Full),
                    },
                    VoxelData {
                        pos: (2, 0, 3),
                        voxel_type: VoxelType::Grass,
                        pattern: Some(SubVoxelPattern::Full),
                    },
                    VoxelData {
                        pos: (3, 0, 3),
                        voxel_type: VoxelType::Grass,
                        pattern: Some(SubVoxelPattern::Full),
                    },
                    // Corner pillars
                    VoxelData {
                        pos: (0, 1, 0),
                        voxel_type: VoxelType::Stone,
                        pattern: Some(SubVoxelPattern::Pillar),
                    },
                    VoxelData {
                        pos: (0, 1, 3),
                        voxel_type: VoxelType::Stone,
                        pattern: Some(SubVoxelPattern::Pillar),
                    },
                    VoxelData {
                        pos: (3, 1, 0),
                        voxel_type: VoxelType::Stone,
                        pattern: Some(SubVoxelPattern::Pillar),
                    },
                    VoxelData {
                        pos: (3, 1, 3),
                        voxel_type: VoxelType::Stone,
                        pattern: Some(SubVoxelPattern::Pillar),
                    },
                    // Platforms
                    VoxelData {
                        pos: (1, 1, 1),
                        voxel_type: VoxelType::Dirt,
                        pattern: Some(SubVoxelPattern::Platform),
                    },
                    VoxelData {
                        pos: (2, 1, 2),
                        voxel_type: VoxelType::Dirt,
                        pattern: Some(SubVoxelPattern::Platform),
                    },
                    // Staircase
                    VoxelData {
                        pos: (2, 1, 1),
                        voxel_type: VoxelType::Stone,
                        pattern: Some(SubVoxelPattern::Staircase),
                    },
                ],
            },
            entities: vec![EntityData {
                entity_type: EntityType::PlayerSpawn,
                position: (1.5, 0.5, 1.5),
                properties: HashMap::new(),
            }],
            lighting: LightingData::default(),
            camera: CameraData::default(),
            custom_properties: HashMap::new(),
        }
    }
}
