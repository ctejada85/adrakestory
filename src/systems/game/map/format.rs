//! Map data structures and format definitions.

use super::super::components::VoxelType;
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
/// Patterns with orientation variants support proper rotation transformations.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubVoxelPattern {
    /// Full 8x8x8 cube of sub-voxels (symmetric, no orientation)
    Full,
    
    // Platform variants (thin slabs in different orientations)
    /// Thin 8x1x8 platform on XZ plane (horizontal, default)
    #[serde(alias = "Platform")]  // For backward compatibility
    PlatformXZ,
    /// Thin 8x8x1 platform on XY plane (vertical wall facing Z)
    PlatformXY,
    /// Thin 1x8x8 platform on YZ plane (vertical wall facing X)
    PlatformYZ,
    
    // Staircase variants (progressive height in different directions)
    /// Stairs ascending in +X direction (default)
    #[serde(alias = "Staircase")]  // For backward compatibility
    StaircaseX,
    /// Stairs ascending in -X direction
    StaircaseNegX,
    /// Stairs ascending in +Z direction
    StaircaseZ,
    /// Stairs ascending in -Z direction
    StaircaseNegZ,
    
    /// Small 2x2x2 centered column (symmetric, no orientation)
    Pillar,
}

impl Default for SubVoxelPattern {
    fn default() -> Self {
        Self::Full
    }
}

impl SubVoxelPattern {
    /// Rotate this pattern around the given axis by the specified angle (0-3 for 0°, 90°, 180°, 270°).
    /// Returns the transformed pattern after rotation.
    pub fn rotate(self, axis: crate::editor::tools::RotationAxis, angle: i32) -> Self {
        use crate::editor::tools::RotationAxis;
        
        // Normalize angle to 0-3 range
        let angle = angle.rem_euclid(4);
        
        // No rotation needed
        if angle == 0 {
            return self;
        }
        
        match self {
            // Symmetric patterns don't change
            Self::Full | Self::Pillar => self,
            
            // Platform rotations
            Self::PlatformXZ => match axis {
                RotationAxis::X => match angle {
                    1 => Self::PlatformXY,  // 90° CW around X: XZ -> XY
                    2 => Self::PlatformXZ,  // 180°: stays horizontal
                    3 => Self::PlatformXY,  // 270° CW around X: XZ -> XY
                    _ => self,
                },
                RotationAxis::Y => Self::PlatformXZ,  // Rotation around Y doesn't change horizontal platform
                RotationAxis::Z => match angle {
                    1 => Self::PlatformYZ,  // 90° CW around Z: XZ -> YZ
                    2 => Self::PlatformXZ,  // 180°: stays horizontal
                    3 => Self::PlatformYZ,  // 270° CW around Z: XZ -> YZ
                    _ => self,
                },
            },
            Self::PlatformXY => match axis {
                RotationAxis::X => match angle {
                    1 => Self::PlatformXZ,  // 90° CW around X: XY -> XZ
                    2 => Self::PlatformXY,  // 180°: stays vertical
                    3 => Self::PlatformXZ,  // 270° CW around X: XY -> XZ
                    _ => self,
                },
                RotationAxis::Y => match angle {
                    1 => Self::PlatformYZ,  // 90° CW around Y: XY -> YZ
                    2 => Self::PlatformXY,  // 180°: stays vertical
                    3 => Self::PlatformYZ,  // 270° CW around Y: XY -> YZ
                    _ => self,
                },
                RotationAxis::Z => Self::PlatformXY,  // Rotation around Z doesn't change XY plane
            },
            Self::PlatformYZ => match axis {
                RotationAxis::X => Self::PlatformYZ,  // Rotation around X doesn't change YZ plane
                RotationAxis::Y => match angle {
                    1 => Self::PlatformXY,  // 90° CW around Y: YZ -> XY
                    2 => Self::PlatformYZ,  // 180°: stays vertical
                    3 => Self::PlatformXY,  // 270° CW around Y: YZ -> XY
                    _ => self,
                },
                RotationAxis::Z => match angle {
                    1 => Self::PlatformXZ,  // 90° CW around Z: YZ -> XZ
                    2 => Self::PlatformYZ,  // 180°: stays vertical
                    3 => Self::PlatformXZ,  // 270° CW around Z: YZ -> XZ
                    _ => self,
                },
            },
            
            // Staircase rotations
            Self::StaircaseX => match axis {
                RotationAxis::X => Self::StaircaseX,  // Rotation around X doesn't change X-direction stairs
                RotationAxis::Y => match angle {
                    1 => Self::StaircaseZ,     // 90° CW around Y: +X -> +Z
                    2 => Self::StaircaseNegX,  // 180°: +X -> -X
                    3 => Self::StaircaseNegZ,  // 270° CW around Y: +X -> -Z
                    _ => self,
                },
                RotationAxis::Z => Self::StaircaseX,  // Rotation around Z doesn't change X-direction stairs
            },
            Self::StaircaseNegX => match axis {
                RotationAxis::X => Self::StaircaseNegX,
                RotationAxis::Y => match angle {
                    1 => Self::StaircaseNegZ,  // 90° CW around Y: -X -> -Z
                    2 => Self::StaircaseX,     // 180°: -X -> +X
                    3 => Self::StaircaseZ,     // 270° CW around Y: -X -> +Z
                    _ => self,
                },
                RotationAxis::Z => Self::StaircaseNegX,
            },
            Self::StaircaseZ => match axis {
                RotationAxis::X => Self::StaircaseZ,
                RotationAxis::Y => match angle {
                    1 => Self::StaircaseNegX,  // 90° CW around Y: +Z -> -X
                    2 => Self::StaircaseNegZ,  // 180°: +Z -> -Z
                    3 => Self::StaircaseX,     // 270° CW around Y: +Z -> +X
                    _ => self,
                },
                RotationAxis::Z => Self::StaircaseZ,
            },
            Self::StaircaseNegZ => match axis {
                RotationAxis::X => Self::StaircaseNegZ,
                RotationAxis::Y => match angle {
                    1 => Self::StaircaseX,     // 90° CW around Y: -Z -> +X
                    2 => Self::StaircaseZ,     // 180°: -Z -> +Z
                    3 => Self::StaircaseNegX,  // 270° CW around Y: -Z -> -X
                    _ => self,
                },
                RotationAxis::Z => Self::StaircaseNegZ,
            },
        }
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
                        pattern: Some(SubVoxelPattern::PlatformXZ),
                    },
                    VoxelData {
                        pos: (2, 1, 2),
                        voxel_type: VoxelType::Dirt,
                        pattern: Some(SubVoxelPattern::PlatformXZ),
                    },
                    // Staircase
                    VoxelData {
                        pos: (2, 1, 1),
                        voxel_type: VoxelType::Stone,
                        pattern: Some(SubVoxelPattern::StaircaseX),
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
