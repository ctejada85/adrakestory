//! Voxel material type used in the map format.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum VoxelType {
    Air,
    Grass,
    Dirt,
    Stone,
}
