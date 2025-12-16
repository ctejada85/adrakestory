//! Greedy meshing algorithm for chunk-based voxel rendering.
//!
//! This module implements greedy meshing which merges adjacent coplanar faces
//! of the same color into larger quads, dramatically reducing polygon count.

mod greedy_mesher;
mod mesh_builder;
mod occupancy;
mod palette;

// Re-export parent module constants needed by submodules
pub(crate) use super::{Face, SUB_VOXEL_COUNT, SUB_VOXEL_SIZE};

// Public exports
pub use greedy_mesher::GreedyMesher;
pub use mesh_builder::ChunkMeshBuilder;
pub use occupancy::OccupancyGrid;
pub use palette::VoxelMaterialPalette;
