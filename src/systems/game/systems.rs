//! Game systems module - Re-exports for backward compatibility.
//!
//! This module has been refactored into smaller, focused modules:
//! - `map` - Map loading and spawning (replaces world_generation)
//! - `collision` - Collision detection helpers
//! - `player_movement` - Player input and movement
//! - `physics` - Gravity and physics simulation
//! - `camera` - Camera control
//! - `input` - General input handling
//!
//! All public functions are re-exported here to maintain backward compatibility
//! with existing code that imports from `systems::game::systems`.

// Re-export player movement
pub use super::player_movement::move_player;

// Re-export physics systems
pub use super::physics::{apply_gravity, apply_physics};

// Re-export camera control
pub use super::camera::rotate_camera;

// Re-export input handling
pub use super::input::{handle_escape_key, toggle_collision_box, update_collision_box};
