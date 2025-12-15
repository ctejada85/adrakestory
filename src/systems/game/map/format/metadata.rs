//! Map metadata structure.

use serde::{Deserialize, Serialize};

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
