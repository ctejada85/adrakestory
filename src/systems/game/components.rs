use bevy::prelude::*;

/// Marker component for the player's flashlight.
/// Used to identify the flashlight entity for toggling and rotation.
#[derive(Component)]
pub struct PlayerFlashlight;

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub velocity: Vec3,
    pub is_grounded: bool,
    /// Horizontal collision radius (cylinder radius)
    pub radius: f32,
    /// Half height of the collision cylinder (from center to top/bottom)
    pub half_height: f32,
    /// Target rotation angle in radians (Y-axis rotation)
    pub target_rotation: f32,
    /// Current rotation angle in radians (Y-axis rotation)
    pub current_rotation: f32,
    /// Rotation angle when the current rotation started (for easing)
    pub start_rotation: f32,
    /// Time elapsed since rotation started (for easing)
    pub rotation_elapsed: f32,
    /// Fixed duration for all rotations in seconds
    pub rotation_duration: f32,
}

#[derive(Component)]
pub struct CollisionBox;

/// Sub-voxel component with cached bounding box for efficient collision detection.
///
/// Previously stored parent and sub-voxel coordinates to calculate bounds on-demand,
/// but now caches the computed bounds at spawn time for better performance.
#[derive(Component)]
pub struct SubVoxel {
    /// Cached bounding box (min, max) to avoid recalculation every frame.
    /// Calculated once at spawn time and reused for all collision checks.
    pub bounds: (Vec3, Vec3),
}

pub use crate::systems::game::map::format::VoxelType;

#[derive(Component)]
pub struct GameCamera {
    pub original_rotation: Quat,
    pub target_rotation: Quat,
    pub rotation_speed: f32,
    /// Offset from the player in local camera space (before rotation is applied)
    pub follow_offset: Vec3,
    /// Speed at which the camera follows the player (higher = more responsive)
    pub follow_speed: f32,
    /// Current target position the camera is following (typically the player's position)
    pub target_position: Vec3,
}

/// Component for NPC entities.
/// NPCs are static characters that the player can interact with.
#[derive(Component)]
pub struct Npc {
    /// Display name of the NPC — rendered as a world-space label above the entity.
    pub name: String,
    /// Collision radius for player collision
    pub radius: f32,
}

/// Component on the screen-space UI text label associated with an [`Npc`] entity.
///
/// Spawned as a **root-level UI node** (not a child of the NPC entity) by
/// `spawn_npc_label`. The `npc_entity` field links the label back to its NPC
/// so that `update_npc_label_visibility` can project the NPC's world position
/// and `despawn_removed_npc_labels` can detect when the NPC is gone.
#[derive(Component)]
pub struct NpcLabel {
    /// The NPC entity this label belongs to.
    pub npc_entity: Entity,
}

impl Default for Npc {
    fn default() -> Self {
        Self {
            name: "NPC".to_string(),
            radius: 0.3,
        }
    }
}

/// Component for light source entities.
/// Light sources emit light in all directions (point light / omnidirectional).
#[derive(Component)]
pub struct LightSource {
    /// Light color (RGB, 0.0-1.0)
    pub color: Color,
    /// Light intensity in lumens (typical range: 100-100000)
    pub intensity: f32,
    /// Maximum range/radius of the light in world units
    pub range: f32,
    /// Whether this light casts shadows
    pub shadows_enabled: bool,
}

impl Default for LightSource {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 10000.0, // 10,000 lumens - bright enough for interior lighting
            range: 10.0,
            shadows_enabled: false, // Disabled by default for performance
        }
    }
}

/// Opt-in component that makes a [`LightSource`] flicker over time.
///
/// Any entity that has both `LightSource` and `FlickerLight` will have its
/// intensity oscillated by `flicker_lights` each frame using a sine wave:
///
/// ```text
/// intensity = base_intensity + amplitude * sin(elapsed_secs * speed)
/// ```
///
/// Removing this component from an entity stops the flicker effect without
/// touching the underlying `LightSource` or the `sync_light_sources` pipeline.
#[derive(Component)]
pub struct FlickerLight {
    /// Centre intensity around which the light flickers (lumens).
    pub base_intensity: f32,
    /// Peak deviation from `base_intensity` in either direction (lumens).
    pub amplitude: f32,
    /// Oscillation frequency in radians per second.
    pub speed: f32,
}

impl Default for FlickerLight {
    fn default() -> Self {
        Self {
            base_intensity: 10_000.0,
            amplitude: 3_000.0,
            speed: 4.0,
        }
    }
}
