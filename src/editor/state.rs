//! Editor state management.

use crate::systems::game::components::VoxelType;
use crate::systems::game::map::format::{EntityType, MapData, SubVoxelPattern};
use bevy::prelude::*;
use std::collections::HashSet;
use std::path::PathBuf;

/// Stores the last-used parameters for each tool type.
/// This allows tools to remember their settings when switching between them.
#[derive(Resource)]
pub struct ToolMemory {
    /// Last-used voxel type for VoxelPlace tool
    pub voxel_type: VoxelType,
    /// Last-used pattern for VoxelPlace tool
    pub voxel_pattern: SubVoxelPattern,
    /// Last-used entity type for EntityPlace tool
    pub entity_type: EntityType,
}

impl Default for ToolMemory {
    fn default() -> Self {
        Self {
            voxel_type: VoxelType::Grass,
            voxel_pattern: SubVoxelPattern::Full,
            entity_type: EntityType::PlayerSpawn,
        }
    }
}

/// Main editor state resource.
#[derive(Resource)]
pub struct EditorState {
    /// The map currently being edited
    pub current_map: MapData,

    /// File path of the current map (None for new unsaved maps)
    pub file_path: Option<PathBuf>,

    /// Whether the map has unsaved changes
    pub is_modified: bool,

    /// Currently active tool
    pub active_tool: EditorTool,

    /// Set of selected voxel positions
    pub selected_voxels: HashSet<(i32, i32, i32)>,

    /// Set of selected entity indices
    pub selected_entities: HashSet<usize>,

    /// Whether to show the grid
    pub show_grid: bool,

    /// Grid opacity (0.0 to 1.0)
    pub grid_opacity: f32,

    /// Whether to snap cursor to grid
    pub snap_to_grid: bool,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            current_map: MapData::empty_map(),
            file_path: None,
            is_modified: false,
            active_tool: EditorTool::VoxelPlace {
                voxel_type: VoxelType::Grass,
                pattern: SubVoxelPattern::Full,
            },
            selected_voxels: HashSet::new(),
            selected_entities: HashSet::new(),
            show_grid: true,
            grid_opacity: 0.3,
            snap_to_grid: true,
        }
    }
}

impl EditorState {
    /// Create a new editor state with a default map
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new editor state with a specific map
    pub fn with_map(map: MapData) -> Self {
        Self {
            current_map: map,
            ..Default::default()
        }
    }

    /// Mark the map as modified
    pub fn mark_modified(&mut self) {
        self.is_modified = true;
    }

    /// Clear the modified flag (after saving)
    pub fn clear_modified(&mut self) {
        self.is_modified = false;
    }

    /// Get the display name for the current file
    pub fn get_display_name(&self) -> String {
        if let Some(path) = &self.file_path {
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Untitled")
                .to_string()
        } else {
            "Untitled".to_string()
        }
    }

    /// Get the window title
    pub fn get_window_title(&self) -> String {
        let name = self.get_display_name();
        let modified = if self.is_modified { "*" } else { "" };
        format!("{}{} - Map Editor", name, modified)
    }

    /// Clear all selections
    pub fn clear_selections(&mut self) {
        self.selected_voxels.clear();
        self.selected_entities.clear();
    }
}

/// Editor tools available for map editing.
#[derive(Debug, Clone, PartialEq)]
pub enum EditorTool {
    /// Place voxels with specified type and pattern
    VoxelPlace {
        voxel_type: VoxelType,
        pattern: SubVoxelPattern,
    },

    /// Remove voxels
    VoxelRemove,

    /// Place entities
    EntityPlace { entity_type: EntityType },

    /// Select and manipulate objects
    Select,

    /// Camera control tool
    Camera,
}

impl EditorTool {
    /// Get a human-readable name for the tool
    pub fn name(&self) -> &str {
        match self {
            Self::VoxelPlace { .. } => "Voxel Place",
            Self::VoxelRemove => "Voxel Remove",
            Self::EntityPlace { .. } => "Entity Place",
            Self::Select => "Select",
            Self::Camera => "Camera",
        }
    }

    /// Get a short description of the tool
    pub fn description(&self) -> &str {
        match self {
            Self::VoxelPlace { .. } => "Click to place voxels",
            Self::VoxelRemove => "Click to remove voxels",
            Self::EntityPlace { .. } => "Click to place entities",
            Self::Select => "Click to select objects",
            Self::Camera => "Drag to move camera",
        }
    }
}

/// UI state for managing dialog visibility and temporary data
#[derive(Resource, Default)]
pub struct EditorUIState {
    /// Whether the file dialog is open
    pub file_dialog_open: bool,

    /// Whether the new map dialog is open
    pub new_map_dialog_open: bool,

    /// Whether the unsaved changes dialog is open
    pub unsaved_changes_dialog_open: bool,

    /// Pending action after unsaved changes dialog
    pub pending_action: Option<PendingAction>,

    /// Whether the about dialog is open
    pub about_dialog_open: bool,

    /// Whether the keyboard shortcuts help is open
    pub shortcuts_help_open: bool,

    /// Whether the error dialog is open
    pub error_dialog_open: bool,

    /// Error message to display in the error dialog
    pub error_message: String,
}

/// Actions that can be pending after user confirmation
#[derive(Debug, Clone)]
pub enum PendingAction {
    NewMap,
    OpenMap,
    OpenRecentFile(PathBuf),
    Quit,
}

/// Resource to track keyboard editing mode (like vim's insert mode)
/// When enabled, keyboard controls the cursor instead of mouse
#[derive(Resource, Default)]
pub struct KeyboardEditMode {
    /// Whether keyboard edit mode is active
    pub enabled: bool,
}

impl KeyboardEditMode {
    /// Create a new keyboard edit mode (disabled by default)
    pub fn new() -> Self {
        Self { enabled: false }
    }

    /// Enable keyboard edit mode
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable keyboard edit mode
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Toggle keyboard edit mode
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}
