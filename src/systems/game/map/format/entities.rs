//! Entity data structures.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    /// Point light source (omnidirectional)
    LightSource,
}
