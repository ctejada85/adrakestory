//! Default map generation for testing and fallback.

use super::{
    CameraData, EntityData, EntityType, LightingData, MapData, MapMetadata, SubVoxelPattern,
    VoxelData, WorldData,
};
use crate::systems::game::components::VoxelType;
use std::collections::HashMap;

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
                voxels: create_default_voxels(),
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

/// Create the default voxel layout for the default map.
fn create_default_voxels() -> Vec<VoxelData> {
    vec![
        // Floor layer - row 0
        VoxelData {
            pos: (0, 0, 0),
            voxel_type: VoxelType::Grass,
            pattern: Some(SubVoxelPattern::Full),
            rotation_state: None,
        },
        VoxelData {
            pos: (1, 0, 0),
            voxel_type: VoxelType::Grass,
            pattern: Some(SubVoxelPattern::Full),
            rotation_state: None,
        },
        VoxelData {
            pos: (2, 0, 0),
            voxel_type: VoxelType::Grass,
            pattern: Some(SubVoxelPattern::Full),
            rotation_state: None,
        },
        VoxelData {
            pos: (3, 0, 0),
            voxel_type: VoxelType::Grass,
            pattern: Some(SubVoxelPattern::Full),
            rotation_state: None,
        },
        // Floor layer - row 1
        VoxelData {
            pos: (0, 0, 1),
            voxel_type: VoxelType::Grass,
            pattern: Some(SubVoxelPattern::Full),
            rotation_state: None,
        },
        VoxelData {
            pos: (1, 0, 1),
            voxel_type: VoxelType::Grass,
            pattern: Some(SubVoxelPattern::Full),
            rotation_state: None,
        },
        VoxelData {
            pos: (2, 0, 1),
            voxel_type: VoxelType::Grass,
            pattern: Some(SubVoxelPattern::Full),
            rotation_state: None,
        },
        VoxelData {
            pos: (3, 0, 1),
            voxel_type: VoxelType::Grass,
            pattern: Some(SubVoxelPattern::Full),
            rotation_state: None,
        },
        // Floor layer - row 2
        VoxelData {
            pos: (0, 0, 2),
            voxel_type: VoxelType::Grass,
            pattern: Some(SubVoxelPattern::Full),
            rotation_state: None,
        },
        VoxelData {
            pos: (1, 0, 2),
            voxel_type: VoxelType::Grass,
            pattern: Some(SubVoxelPattern::Full),
            rotation_state: None,
        },
        VoxelData {
            pos: (2, 0, 2),
            voxel_type: VoxelType::Grass,
            pattern: Some(SubVoxelPattern::Full),
            rotation_state: None,
        },
        VoxelData {
            pos: (3, 0, 2),
            voxel_type: VoxelType::Grass,
            pattern: Some(SubVoxelPattern::Full),
            rotation_state: None,
        },
        // Floor layer - row 3
        VoxelData {
            pos: (0, 0, 3),
            voxel_type: VoxelType::Grass,
            pattern: Some(SubVoxelPattern::Full),
            rotation_state: None,
        },
        VoxelData {
            pos: (1, 0, 3),
            voxel_type: VoxelType::Grass,
            pattern: Some(SubVoxelPattern::Full),
            rotation_state: None,
        },
        VoxelData {
            pos: (2, 0, 3),
            voxel_type: VoxelType::Grass,
            pattern: Some(SubVoxelPattern::Full),
            rotation_state: None,
        },
        VoxelData {
            pos: (3, 0, 3),
            voxel_type: VoxelType::Grass,
            pattern: Some(SubVoxelPattern::Full),
            rotation_state: None,
        },
        // Corner pillars
        VoxelData {
            pos: (0, 1, 0),
            voxel_type: VoxelType::Stone,
            pattern: Some(SubVoxelPattern::Pillar),
            rotation_state: None,
        },
        VoxelData {
            pos: (0, 1, 3),
            voxel_type: VoxelType::Stone,
            pattern: Some(SubVoxelPattern::Pillar),
            rotation_state: None,
        },
        VoxelData {
            pos: (3, 1, 0),
            voxel_type: VoxelType::Stone,
            pattern: Some(SubVoxelPattern::Pillar),
            rotation_state: None,
        },
        VoxelData {
            pos: (3, 1, 3),
            voxel_type: VoxelType::Stone,
            pattern: Some(SubVoxelPattern::Pillar),
            rotation_state: None,
        },
        // Platforms
        VoxelData {
            pos: (1, 1, 1),
            voxel_type: VoxelType::Dirt,
            pattern: Some(SubVoxelPattern::PlatformXZ),
            rotation_state: None,
        },
        VoxelData {
            pos: (2, 1, 2),
            voxel_type: VoxelType::Dirt,
            pattern: Some(SubVoxelPattern::PlatformXZ),
            rotation_state: None,
        },
        // Staircase
        VoxelData {
            pos: (2, 1, 1),
            voxel_type: VoxelType::Stone,
            pattern: Some(SubVoxelPattern::StaircaseX),
            rotation_state: None,
        },
    ]
}
