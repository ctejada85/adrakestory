//! Character system for managing player character models and visuals.
//!
//! This module handles:
//! - Loading and managing character 3D models (GLB/GLTF)
//! - Character model components and resources
//! - Visual representation separate from physics collision cylinder

use bevy::prelude::*;

/// Component that tracks a character's 3D model.
#[allow(dead_code)]
///
/// The character model is loaded as a GLB/GLTF scene and spawned as a child
/// entity of the player. This separates the visual representation from the
/// physics collision cylinder, allowing for flexible model swapping and animations.
#[derive(Component)]
pub struct CharacterModel {
    /// Handle to the loaded GLB/GLTF scene
    pub scene_handle: Handle<Scene>,
    /// Scale factor for the model (default: 1.0)
    pub scale: f32,
    /// Position offset from the parent entity (default: Vec3::ZERO)
    pub offset: Vec3,
}

impl Default for CharacterModel {
    fn default() -> Self {
        Self {
            scene_handle: Handle::default(),
            scale: 1.0,
            offset: Vec3::ZERO,
        }
    }
}

#[allow(dead_code)]
impl CharacterModel {
    /// Create a new character model with the given scene handle.
    pub fn new(scene_handle: Handle<Scene>) -> Self {
        Self {
            scene_handle,
            scale: 1.0,
            offset: Vec3::ZERO,
        }
    }

    /// Create a new character model with custom scale.
    pub fn with_scale(scene_handle: Handle<Scene>, scale: f32) -> Self {
        Self {
            scene_handle,
            scale,
            offset: Vec3::ZERO,
        }
    }

    /// Create a new character model with custom offset.
    pub fn with_offset(scene_handle: Handle<Scene>, offset: Vec3) -> Self {
        Self {
            scene_handle,
            scale: 1.0,
            offset,
        }
    }

    /// Create a new character model with custom scale and offset.
    pub fn with_scale_and_offset(scene_handle: Handle<Scene>, scale: f32, offset: Vec3) -> Self {
        Self {
            scene_handle,
            scale,
            offset,
        }
    }
}
