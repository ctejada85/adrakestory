//! Error types for map loading and validation.

use thiserror::Error;

/// Errors that can occur during map loading and processing.
#[derive(Debug, Error)]
pub enum MapLoadError {
    /// Failed to read the map file from disk.
    #[error("Failed to read map file: {0}")]
    FileReadError(#[from] std::io::Error),

    /// Failed to parse the RON data.
    #[error("Failed to parse map data: {0}")]
    ParseError(#[from] ron::error::SpannedError),

    /// Map validation failed.
    #[error("Map validation failed: {0}")]
    ValidationError(String),

    /// Invalid voxel position outside world bounds.
    #[error("Invalid voxel position: ({0}, {1}, {2})")]
    InvalidVoxelPosition(i32, i32, i32),

    /// Invalid entity type encountered.
    #[allow(dead_code)]
    #[error("Invalid entity type: {0}")]
    InvalidEntityType(String),

    /// World dimensions are invalid (must be positive).
    #[error("Invalid world dimensions: width={0}, height={1}, depth={2}")]
    InvalidWorldDimensions(i32, i32, i32),

    /// Missing required field in map data.
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Unsupported map version.
    #[error("Unsupported map version: {0}")]
    UnsupportedVersion(String),
}

/// Result type for map operations.
pub type MapResult<T> = Result<T, MapLoadError>;
