//! Undo/redo history system for the map editor.

use crate::systems::game::map::format::{EntityData, MapMetadata, VoxelData};
use bevy::prelude::*;

/// Maximum number of actions to keep in history
const MAX_HISTORY_SIZE: usize = 100;

/// Resource managing undo/redo history
#[derive(Resource)]
pub struct EditorHistory {
    /// Stack of actions that can be undone
    undo_stack: Vec<EditorAction>,

    /// Stack of actions that can be redone
    redo_stack: Vec<EditorAction>,

    /// Maximum history size
    max_history: usize,
}

impl Default for EditorHistory {
    fn default() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: MAX_HISTORY_SIZE,
        }
    }
}

impl EditorHistory {
    /// Create a new history manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new history manager with custom max size
    pub fn with_max_size(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history,
        }
    }

    /// Push a new action onto the undo stack
    pub fn push(&mut self, action: EditorAction) {
        // Clear redo stack when new action is performed
        self.redo_stack.clear();

        // Add to undo stack
        self.undo_stack.push(action);

        // Limit stack size
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
    }

    /// Pop an action from the undo stack
    pub fn undo(&mut self) -> Option<EditorAction> {
        if let Some(action) = self.undo_stack.pop() {
            self.redo_stack.push(action.clone());
            Some(action)
        } else {
            None
        }
    }

    /// Pop an action from the redo stack
    pub fn redo(&mut self) -> Option<EditorAction> {
        if let Some(action) = self.redo_stack.pop() {
            self.undo_stack.push(action.clone());
            Some(action)
        } else {
            None
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the number of actions in the undo stack
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of actions in the redo stack
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Get a description of the last action that can be undone
    pub fn undo_description(&self) -> Option<String> {
        self.undo_stack.last().map(|a| a.description())
    }

    /// Get a description of the last action that can be redone
    pub fn redo_description(&self) -> Option<String> {
        self.redo_stack.last().map(|a| a.description())
    }
}

/// Actions that can be undone/redone
#[derive(Debug, Clone)]
pub enum EditorAction {
    /// Place a voxel
    PlaceVoxel {
        pos: (i32, i32, i32),
        data: VoxelData,
    },

    /// Remove a voxel
    RemoveVoxel {
        pos: (i32, i32, i32),
        data: VoxelData,
    },

    /// Place an entity
    PlaceEntity { index: usize, data: EntityData },

    /// Remove an entity
    RemoveEntity { index: usize, data: EntityData },

    /// Modify entity
    ModifyEntity {
        index: usize,
        old_data: EntityData,
        new_data: EntityData,
    },

    /// Modify metadata
    ModifyMetadata { old: MapMetadata, new: MapMetadata },

    /// Batch of multiple actions
    Batch {
        description: String,
        actions: Vec<EditorAction>,
    },
}

impl EditorAction {
    /// Get a human-readable description of the action
    pub fn description(&self) -> String {
        match self {
            Self::PlaceVoxel { pos, data } => {
                format!("Place {:?} at {:?}", data.voxel_type, pos)
            }
            Self::RemoveVoxel { pos, .. } => {
                format!("Remove voxel at {:?}", pos)
            }
            Self::PlaceEntity { data, .. } => {
                format!("Place {:?}", data.entity_type)
            }
            Self::RemoveEntity { data, .. } => {
                format!("Remove {:?}", data.entity_type)
            }
            Self::ModifyEntity { .. } => "Modify entity".to_string(),
            Self::ModifyMetadata { .. } => "Modify metadata".to_string(),
            Self::Batch {
                description,
                actions,
            } => {
                format!("{} ({} actions)", description, actions.len())
            }
        }
    }

    /// Get the inverse action (for undo)
    pub fn inverse(&self) -> Self {
        match self {
            Self::PlaceVoxel { pos, data } => Self::RemoveVoxel {
                pos: *pos,
                data: data.clone(),
            },
            Self::RemoveVoxel { pos, data } => Self::PlaceVoxel {
                pos: *pos,
                data: data.clone(),
            },
            Self::PlaceEntity { index, data } => Self::RemoveEntity {
                index: *index,
                data: data.clone(),
            },
            Self::RemoveEntity { index, data } => Self::PlaceEntity {
                index: *index,
                data: data.clone(),
            },
            Self::ModifyEntity {
                index,
                old_data,
                new_data,
            } => Self::ModifyEntity {
                index: *index,
                old_data: new_data.clone(),
                new_data: old_data.clone(),
            },
            Self::ModifyMetadata { old, new } => Self::ModifyMetadata {
                old: new.clone(),
                new: old.clone(),
            },
            Self::Batch {
                description,
                actions,
            } => Self::Batch {
                description: format!("Undo {}", description),
                actions: actions.iter().rev().map(|a| a.inverse()).collect(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::game::components::VoxelType;
    use crate::systems::game::map::format::SubVoxelPattern;

    #[test]
    fn test_history_push_and_undo() {
        let mut history = EditorHistory::new();

        let action = EditorAction::PlaceVoxel {
            pos: (0, 0, 0),
            data: VoxelData {
                pos: (0, 0, 0),
                voxel_type: VoxelType::Grass,
                pattern: Some(SubVoxelPattern::Full),
                rotation_state: None,
            },
        };

        history.push(action.clone());
        assert!(history.can_undo());
        assert!(!history.can_redo());

        let undone = history.undo();
        assert!(undone.is_some());
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn test_history_redo() {
        let mut history = EditorHistory::new();

        let action = EditorAction::PlaceVoxel {
            pos: (0, 0, 0),
            data: VoxelData {
                pos: (0, 0, 0),
                voxel_type: VoxelType::Grass,
                pattern: Some(SubVoxelPattern::Full),
                rotation_state: None,
            },
        };

        history.push(action.clone());
        history.undo();

        let redone = history.redo();
        assert!(redone.is_some());
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_history_clear_redo_on_new_action() {
        let mut history = EditorHistory::new();

        let action1 = EditorAction::PlaceVoxel {
            pos: (0, 0, 0),
            data: VoxelData {
                pos: (0, 0, 0),
                voxel_type: VoxelType::Grass,
                pattern: Some(SubVoxelPattern::Full),
                rotation_state: None,
            },
        };

        let action2 = EditorAction::PlaceVoxel {
            pos: (1, 0, 0),
            data: VoxelData {
                pos: (1, 0, 0),
                voxel_type: VoxelType::Dirt,
                pattern: Some(SubVoxelPattern::Full),
                rotation_state: None,
            },
        };

        history.push(action1);
        history.undo();
        assert!(history.can_redo());

        history.push(action2);
        assert!(!history.can_redo());
    }
}
