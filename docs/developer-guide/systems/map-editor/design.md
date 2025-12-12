# Map Editor Design Document

## Overview

A standalone GUI application for creating and editing A Drake's Story map files (.ron format). Built using bevy_egui for UI and leveraging existing game rendering code for 3D preview.

## Architecture

### Application Structure

```
adrakestory/
├── src/
│   ├── bin/
│   │   └── map_editor.rs          # Map editor entry point
│   └── editor/
│       ├── mod.rs                  # Editor module exports
│       ├── state.rs                # Editor state management
│       ├── cursor.rs               # Cursor ray casting and keyboard navigation
│       ├── file_io.rs              # Save/load file operations
│       ├── shortcuts.rs            # Global keyboard shortcuts (Ctrl+S, Ctrl+Z, etc.)
│       ├── play.rs                 # Play/test map functionality
│       ├── recent_files.rs         # Recent files tracking
│       ├── ui/
│       │   ├── mod.rs
│       │   ├── toolbar.rs          # Top toolbar UI
│       │   ├── properties.rs       # Right properties panel
│       │   ├── viewport.rs         # 3D viewport controls
│       │   ├── outliner.rs         # Left outliner panel
│       │   └── dialogs.rs          # File dialogs, confirmations
│       ├── tools/
│       │   ├── mod.rs
│       │   ├── input.rs            # Unified keyboard input handling
│       │   ├── voxel_tool.rs       # Voxel placement/removal
│       │   ├── entity_tool.rs      # Entity placement
│       │   └── selection_tool.rs   # Selection and manipulation
│       ├── camera.rs               # Editor camera controls
│       ├── grid.rs                 # Grid visualization
│       ├── history.rs              # Undo/redo history tracking
│       └── renderer.rs             # Map rendering with optimizations
```

### Technology Stack

- **GUI Framework**: bevy_egui (v0.30+)
- **Rendering**: Bevy 0.15 (reuse game rendering)
- **File I/O**: Existing map loader/saver
- **State Management**: Bevy ECS resources

## Core Components

### 1. Editor State

```rust
#[derive(Resource)]
pub struct EditorState {
    // Current map being edited
    pub current_map: MapData,
    
    // File path (None for new unsaved maps)
    pub file_path: Option<PathBuf>,
    
    // Dirty flag for unsaved changes
    pub is_modified: bool,
    
    // Current tool
    pub active_tool: EditorTool,
    
    // Selection state
    pub selected_voxels: HashSet<(i32, i32, i32)>,
    pub selected_entities: HashSet<usize>,
    
    // UI state
    pub show_grid: bool,
    pub grid_opacity: f32,
    pub snap_to_grid: bool,
}
```

### 2. Editor Tools

```rust
pub enum EditorTool {
    VoxelPlace {
        voxel_type: VoxelType,
        pattern: SubVoxelPattern,
    },
    VoxelRemove,
    EntityPlace {
        entity_type: EntityType,
    },
    Select,
    Camera,
}
```

### 3. History System

```rust
pub struct EditorHistory {
    undo_stack: Vec<EditorAction>,
    redo_stack: Vec<EditorAction>,
    max_history: usize,
}

pub enum EditorAction {
    PlaceVoxel { pos: (i32, i32, i32), data: VoxelData },
    RemoveVoxel { pos: (i32, i32, i32), data: VoxelData },
    PlaceEntity { index: usize, data: EntityData },
    RemoveEntity { index: usize, data: EntityData },
    ModifyMetadata { old: MapMetadata, new: MapMetadata },
    // ... more actions
}
```

## UI Layout

```
┌─────────────────────────────────────────────────────────────┐
│ File  Edit  View  Tools  Help                    [X]        │ Toolbar
├─────────────────────────────────────────────────────────────┤
│ [New] [Open] [Save] | [Undo] [Redo] | [Grid] [Snap]        │ Quick Actions
├──────────────────────────────────────┬──────────────────────┤
│                                      │  Properties          │
│                                      │  ┌────────────────┐  │
│                                      │  │ Voxel Type:    │  │
│         3D Viewport                  │  │ [Grass ▼]      │  │
│                                      │  │                │  │
│    (Interactive 3D view with         │  │ Pattern:       │  │
│     camera controls and grid)        │  │ [Full ▼]       │  │
│                                      │  │                │  │
│                                      │  │ Position:      │  │
│                                      │  │ X: [  ] Y: [  ]│  │
│                                      │  │ Z: [  ]        │  │
│                                      │  └────────────────┘  │
│                                      │                      │
│                                      │  Map Info            │
│                                      │  ┌────────────────┐  │
│                                      │  │ Name: [      ] │  │
│                                      │  │ Author: [    ] │  │
│                                      │  │ Size: 4x3x4    │  │
│                                      │  └────────────────┘  │
├──────────────────────────────────────┴──────────────────────┤
│ Status: Ready | Voxels: 23 | Entities: 1 | Modified: *     │ Status Bar
└─────────────────────────────────────────────────────────────┘
```

## Key Features

### 1. Lighting System (WYSIWYG)
- **Map-Defined Lighting**: Uses lighting configuration from the current map
- **Directional Light**: Reads illuminance, color, and direction from map data
- **Ambient Light**: Converts map's ambient_intensity (0.0-1.0) to Bevy brightness (×1000)
- **Dynamic Updates**: Automatically updates lighting when loading new maps
- **Congruent Rendering**: Identical illumination in editor and game mode
- **Real-time Preview**: What you see in the editor is what you get in-game

### 2. Voxel Editing
- **Placement**: Click to place voxels at grid positions
- **Removal**: Right-click or delete key to remove voxels
- **Type Selection**: Dropdown for Grass, Dirt, Stone, Air
- **Pattern Selection**: Dropdown for Full, Platform, Staircase, Pillar
- **Multi-select**: Shift+click for multiple voxels
- **Copy/Paste**: Duplicate voxel configurations

### 3. Entity Management
- **Placement**: Click to place entities (PlayerSpawn, etc.)
- **Properties**: Edit entity position and custom properties
- **Validation**: Ensure at least one PlayerSpawn exists
- **Visual Indicators**: Different colors/icons for entity types

### 4. Camera Controls
- **Orbit**: Right-click and drag to rotate around center
- **Pan**: Multiple options for accessibility:
  - Middle-click and drag (traditional)
  - Shift + Right-click and drag
  - Space + Left-click and drag (trackpad-friendly)
  - Cmd/Ctrl + Left-click and drag (Mac trackpad-friendly)
- **Zoom**: Mouse wheel to zoom in/out (reduced sensitivity for smoother control)
- **Reset**: Button or Home key to reset to default view
- **Presets**: Top, Front, Side, Isometric views

### 5. Grid System
- **Visualization**: Render grid lines at voxel boundaries
- **Snapping**: Snap cursor to grid intersections
- **Opacity**: Adjustable grid transparency
- **Toggle**: Show/hide grid with keyboard shortcut

### 6. File Operations
- **New**: Create new map with default settings
- **Open**: Load existing .ron file with file picker
- **Save**: Save to current file path
- **Save As**: Save to new file path
- **Auto-save**: Optional periodic auto-save
- **Recent Files**: Quick access to recently edited maps

### 7. Undo/Redo
- **Action History**: Track all editing operations via `EditorHistory` resource
- **Undo**: `Ctrl+Z` to undo last action (also available in Edit menu)
- **Redo**: `Ctrl+Y` or `Ctrl+Shift+Z` to redo (also available in Edit menu)
- **Supported Actions**:
  - Voxel placement and removal
  - Entity placement, removal, and modification
  - Metadata changes
  - Batch operations (multiple actions as one undo step)
- **History Limit**: Configurable max history size (default: 100)
- **Implementation**: `shortcuts.rs` handles keyboard shortcuts, applies inverse actions via `apply_action()` and `apply_action_inverse()`

### 8. Global Keyboard Shortcuts
- **File Operations**:
  - `Ctrl+N` - New map
  - `Ctrl+O` - Open map
  - `Ctrl+S` - Save map
  - `Ctrl+Shift+S` - Save As
- **Edit Operations**:
  - `Ctrl+Z` - Undo
  - `Ctrl+Y` / `Ctrl+Shift+Z` - Redo
- **Implementation**: `shortcuts.rs` module with `handle_global_shortcuts` system
- **UI Integration**: Menu items display shortcut hints

### 9. Validation
- **Real-time**: Validate as user edits
- **Visual Feedback**: Highlight errors in red
- **Error Panel**: List all validation errors
- **Warnings**: Show warnings for best practices
- **Fix Suggestions**: Suggest fixes for common issues

## Implementation Phases

### Phase 1: Foundation (Core Infrastructure)
1. Set up bevy_egui integration
2. Create basic window and UI layout
3. Implement editor state management
4. Set up 3D viewport with camera controls
5. Implement grid visualization

### Phase 2: Basic Editing (Voxel Tools)
1. Implement voxel placement tool
2. Implement voxel removal tool
3. Add voxel type selection UI
4. Add pattern selection UI
5. Implement cursor positioning and snapping

### Phase 3: Advanced Features
1. Implement entity placement tools
2. Add metadata editor panel
3. Implement lighting configuration
4. Implement camera configuration
5. Add world dimension editor

### Phase 4: File Operations
1. Implement New map functionality
2. Implement Open file dialog
3. Implement Save/Save As
4. Add file path tracking
5. Implement unsaved changes warning

### Phase 5: Polish & UX
1. Implement undo/redo system
2. Add keyboard shortcuts
3. Implement real-time validation
4. Add status bar with info
5. Improve visual feedback

### Phase 6: Testing & Documentation
1. Test with various map configurations
2. Test edge cases and error handling
3. Write user documentation
4. Create tutorial/examples
5. Performance optimization

## Technical Considerations

### Rendering Strategy
- Reuse existing voxel spawning code from game
- Render map in real-time as user edits
- Use separate camera for editor viewport
- Optimize for large maps (LOD, culling)

### Performance
- Lazy rendering: Only update when changed
- Efficient voxel storage: HashMap for sparse data
- Batch operations: Group multiple edits
- Background validation: Don't block UI

### User Experience
- Intuitive controls matching industry standards
- Clear visual feedback for all actions
- Helpful error messages with solutions
- Keyboard shortcuts for power users
- Responsive UI even with large maps

### Data Integrity
- Validate before saving
- Backup before overwriting
- Atomic file writes
- Handle corrupted files gracefully

## Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| New Map | Ctrl+N |
| Open Map | Ctrl+O |
| Save | Ctrl+S |
| Save As | Ctrl+Shift+S |
| Undo | Ctrl+Z |
| Redo | Ctrl+Y |
| Delete | Delete/Backspace |
| Toggle Grid | G |
| Toggle Snap | Shift+G |
| Select Tool | V |
| Voxel Tool | B |
| Entity Tool | E |
| Camera Tool | C |
| Reset Camera | Home |

## Dependencies

```toml
[dependencies]
bevy = "0.15"
bevy_egui = "0.30"
serde = { version = "1.0", features = ["derive"] }
ron = "0.8"
thiserror = "1.0"
rfd = "0.15"  # Native file dialogs
```

## Future Enhancements

- **Terrain Generation**: Procedural terrain tools
- **Prefabs**: Save/load voxel groups as prefabs
- **Layers**: Organize voxels in layers
- **Scripting**: Lua/Rhai for custom logic
- **Multiplayer**: Collaborative editing
- **Version Control**: Git integration
- **Asset Browser**: Visual asset picker
- **Animation**: Animate entities and voxels

## References

- [bevy_egui Documentation](https://docs.rs/bevy_egui/)
- [egui Documentation](https://docs.rs/egui/)
- [Map Format Specification](../api/map-format-spec.md)
- [Map Loader System](systems/map-loader.md)

---

**Document Version**: 1.1.0
**Last Updated**: 2025-10-23
**Status**: Active Development - WYSIWYG Lighting Implemented