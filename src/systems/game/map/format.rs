//! Map data structures and format definitions.

use super::super::components::VoxelType;
use super::geometry::{RotationAxis, SubVoxelGeometry};
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
    /// Optional rotation state for the voxel's geometry
    #[serde(default)]
    pub rotation_state: Option<RotationState>,
}

/// Rotation state for a voxel's geometry.
/// This tracks cumulative rotations applied to the voxel's sub-voxel pattern.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RotationState {
    /// The axis of rotation
    pub axis: RotationAxis,
    /// The rotation angle in 90° increments (0-3 for 0°, 90°, 180°, 270°)
    pub angle: i32,
}

impl RotationState {
    /// Create a new rotation state.
    pub fn new(axis: RotationAxis, angle: i32) -> Self {
        Self {
            axis,
            angle: angle.rem_euclid(4),
        }
    }

    /// Compose this rotation with another rotation.
    /// This handles the case where rotations are applied sequentially.
    pub fn compose(self, axis: RotationAxis, angle: i32) -> Self {
        // If rotating around the same axis, just add the angles
        if self.axis == axis {
            Self::new(axis, self.angle + angle)
        } else {
            // For different axes, we need to apply the new rotation
            // For simplicity, we'll store the most recent rotation
            // A full implementation would compose the rotations properly
            Self::new(axis, angle)
        }
    }
}

/// Sub-voxel patterns for different voxel appearances.
/// Patterns with orientation variants support proper rotation transformations.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SubVoxelPattern {
    /// Full 8x8x8 cube of sub-voxels (symmetric, no orientation)
    #[default]
    Full,

    // Platform variants (thin slabs in different orientations)
    /// Thin 8x1x8 platform on XZ plane (horizontal, default)
    #[serde(alias = "Platform")] // For backward compatibility
    PlatformXZ,
    /// Thin 8x8x1 platform on XY plane (vertical wall facing Z)
    PlatformXY,
    /// Thin 1x8x8 platform on YZ plane (vertical wall facing X)
    PlatformYZ,

    // Staircase variants (progressive height in different directions)
    /// Stairs ascending in +X direction (default)
    #[serde(alias = "Staircase")] // For backward compatibility
    StaircaseX,
    /// Stairs ascending in -X direction
    StaircaseNegX,
    /// Stairs ascending in +Z direction
    StaircaseZ,
    /// Stairs ascending in -Z direction
    StaircaseNegZ,

    /// Small 2x2x2 centered column (symmetric, no orientation)
    Pillar,

    /// Fence pattern along X axis (posts at ends with horizontal rails)
    #[serde(alias = "FenceX")]
    #[serde(alias = "FenceZ")]
    #[serde(alias = "FenceCorner")]
    Fence,
}

impl SubVoxelPattern {
    /// Get the base geometry representation of this pattern without any rotation.
    ///
    /// This converts the pattern enum into actual 3D sub-voxel positions.
    pub fn geometry(&self) -> SubVoxelGeometry {
        match self {
            Self::Full => SubVoxelGeometry::full(),
            Self::PlatformXZ => SubVoxelGeometry::platform_horizontal(),
            Self::PlatformXY => {
                // Horizontal platform rotated 90° around X
                SubVoxelGeometry::platform_horizontal()
                    .rotate(crate::editor::tools::RotationAxis::X, 1)
            }
            Self::PlatformYZ => {
                // Horizontal platform rotated 90° around Z
                SubVoxelGeometry::platform_horizontal()
                    .rotate(crate::editor::tools::RotationAxis::Z, 1)
            }
            Self::StaircaseX => SubVoxelGeometry::staircase_x(),
            Self::StaircaseNegX => {
                // Staircase rotated 180° around Y
                SubVoxelGeometry::staircase_x().rotate(crate::editor::tools::RotationAxis::Y, 2)
            }
            Self::StaircaseZ => {
                // Staircase rotated 90° around Y
                SubVoxelGeometry::staircase_x().rotate(crate::editor::tools::RotationAxis::Y, 1)
            }
            Self::StaircaseNegZ => {
                // Staircase rotated 270° around Y
                SubVoxelGeometry::staircase_x().rotate(crate::editor::tools::RotationAxis::Y, 3)
            }
            Self::Pillar => SubVoxelGeometry::pillar(),
            Self::Fence => SubVoxelGeometry::fence_post(), // Default to just a post
        }
    }

    /// Check if this pattern is a fence that needs neighbor-aware geometry.
    pub fn is_fence(&self) -> bool {
        matches!(self, Self::Fence)
    }

    /// Get geometry for fence patterns based on neighboring fences.
    ///
    /// # Arguments
    /// * `neighbors` - Tuple of (has_neg_x, has_pos_x, has_neg_z, has_pos_z) indicating adjacent fences
    pub fn fence_geometry_with_neighbors(&self, neighbors: (bool, bool, bool, bool)) -> SubVoxelGeometry {
        if !self.is_fence() {
            return self.geometry();
        }
        SubVoxelGeometry::fence_with_connections(neighbors.0, neighbors.1, neighbors.2, neighbors.3)
    }

    /// Get the geometry representation of this pattern with rotation applied.
    ///
    /// This applies the rotation state to the base pattern geometry.
    ///
    /// # Arguments
    /// * `rotation_state` - Optional rotation state to apply
    ///
    /// # Returns
    /// The geometry with rotation applied, or base geometry if no rotation state.
    pub fn geometry_with_rotation(
        &self,
        rotation_state: Option<RotationState>,
    ) -> SubVoxelGeometry {
        let base_geometry = self.geometry();

        if let Some(rotation) = rotation_state {
            base_geometry.rotate(rotation.axis, rotation.angle)
        } else {
            base_geometry
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
    /// NPC spawn point (static, non-moving characters)
    Npc,
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
