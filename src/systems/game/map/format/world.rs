//! World and voxel data structures.

use super::patterns::SubVoxelPattern;
use super::rotation::RotationState;
use crate::systems::game::components::VoxelType;
use serde::{Deserialize, Serialize};

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
