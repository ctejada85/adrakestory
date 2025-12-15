//! Sub-voxel geometry representation and transformation.
//!
//! This module provides a generic geometry system for representing and transforming
//! sub-voxel patterns within voxels. Instead of hardcoding rotation logic for each
//! pattern type, we store the actual 3D geometry and apply mathematical transformations.

mod patterns;
mod rotation;
mod sub_voxel_geometry;
#[cfg(test)]
mod tests;

pub use sub_voxel_geometry::SubVoxelGeometry;

use serde::{Deserialize, Serialize};

/// Rotation axis for 3D transformations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RotationAxis {
    /// Rotation around the X axis
    X,
    /// Rotation around the Y axis (default)
    #[default]
    Y,
    /// Rotation around the Z axis
    Z,
}
