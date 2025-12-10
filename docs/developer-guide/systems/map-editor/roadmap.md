# Map Editor Implementation Roadmap

## Overview

This document outlines the step-by-step implementation plan for the A Drake's Story Map Editor. The project is divided into 6 phases, each building upon the previous one.

## Timeline Estimate

- **Phase 1**: ✅ COMPLETE (Foundation)
- **Phase 2**: ✅ COMPLETE (Basic Editing)
- **Phase 3**: ✅ COMPLETE (Advanced Features)
- **Phase 4**: ✅ COMPLETE (File Operations)
- **Phase 5**: ✅ COMPLETE (Polish & UX)
- **Phase 6**: ⏳ PENDING (Testing & Documentation)

**Status**: All core features complete. Map editor is fully functional with undo/redo, selection transforms, keyboard navigation, and outliner panel.

## Phase 1: Foundation (Core Infrastructure)

### Goals
- Set up the basic application structure
- Integrate bevy_egui
- Create minimal working editor window
- Establish core data structures

### Tasks

#### 1.1 Project Setup
- [x] Add `bevy_egui` dependency to Cargo.toml
- [x] Add `rfd` (native file dialogs) dependency
- [x] Create `src/editor/` module structure
- [x] Create `src/bin/map_editor.rs` entry point

#### 1.2 Basic Window & UI
- [x] Initialize Bevy app with egui plugin
- [x] Create main window with basic layout
- [x] Implement top menu bar (File, Edit, View, Tools, Help)
- [x] Implement status bar at bottom
- [x] Set up 3-panel layout (toolbar, viewport, properties)

#### 1.3 Editor State
- [x] Define `EditorState` resource
- [x] Define `EditorTool` enum
- [x] Implement state initialization
- [x] Add state serialization for settings

#### 1.4 Camera System
- [x] Create `EditorCamera` component
- [x] Implement orbit camera controls (mouse drag)
- [x] Implement pan controls (middle mouse)
- [x] Implement zoom controls (mouse wheel)
- [x] Add camera reset functionality

#### 1.5 Grid Visualization
- [x] Create grid mesh generator
- [x] Implement grid rendering system
- [x] Add grid toggle functionality
- [x] Add grid opacity control
- [x] Implement grid snapping logic

### Deliverables ✅ COMPLETE
- ✅ Working editor window with 3D viewport
- ✅ Functional camera controls
- ✅ Visible grid system
- ✅ Basic UI layout

### Success Criteria ✅ ACHIEVED
- ✅ Editor launches without errors
- ✅ Camera can orbit, pan, and zoom
- ✅ Grid is visible and can be toggled
- ✅ UI panels are properly laid out

---

## Phase 2: Basic Editing (Voxel Tools)

### Goals
- Implement core voxel editing functionality
- Enable placing and removing voxels
- Add voxel type and pattern selection

### Tasks

#### 2.1 Voxel Placement Tool ✅ COMPLETE
- [x] Implement cursor positioning in 3D space
- [x] Add ray casting for mouse-to-world position
- [x] Implement voxel placement on click
- [x] Add visual cursor indicator
- [x] Implement grid snapping for placement
- [x] **Face-aware placement** (2025-10-30) ⭐ NEW
  - [x] Enhanced ray-box intersection with face detection
  - [x] Adjacent placement on voxel faces
  - [x] Grid snapping for placement preview
  - [x] Tool-aware cursor rendering

#### 2.2 Voxel Removal Tool ✅ COMPLETE
- [x] Implement voxel selection via ray cast
- [x] Add voxel removal on click
- [x] Add visual highlight for selected voxel
- [x] Implement delete key support

#### 2.3 Voxel Type Selection ✅ COMPLETE
- [x] Create voxel type dropdown UI
- [x] Add icons/colors for each type (Grass, Dirt, Stone)
- [x] Implement type switching
- [x] Update cursor preview with selected type

#### 2.4 Pattern Selection ✅ COMPLETE
- [x] Create pattern dropdown UI
- [x] Add preview icons for patterns
- [x] Implement pattern switching
- [x] Update cursor preview with selected pattern

#### 2.5 Voxel Rendering ✅ COMPLETE
- [x] Integrate existing voxel spawning code
- [x] Implement real-time voxel updates
- [x] Add voxel highlighting on hover
- [x] Optimize rendering for editor use

### Deliverables ✅ COMPLETE
- ✅ Functional voxel placement tool with face-aware placement
- ✅ Functional voxel removal tool
- ✅ Type and pattern selection UI
- ✅ Real-time 3D preview

### Success Criteria ✅ ACHIEVED
- ✅ Can place voxels at any grid position
- ✅ Can remove voxels by clicking
- ✅ Can switch between voxel types
- ✅ Can switch between patterns
- ✅ Changes are immediately visible in 3D
- ✅ Voxels place on the face of target voxels (face-aware)

---

## Phase 3: Advanced Features

### Goals
- Add entity placement and editing
- Implement metadata editor
- Add lighting and camera configuration
- Enable world dimension editing

### Tasks

#### 3.1 Entity Placement ✅ COMPLETE
- [x] Create entity placement tool
- [x] Add entity type selection UI
- [x] Implement entity positioning
- [x] Add visual indicators for entities
- [x] Implement entity removal

#### 3.2 Entity Properties ✅ COMPLETE
- [x] Create entity properties panel
- [x] Add position editing (X, Y, Z inputs)
- [x] Implement custom properties editor
- [x] Add entity validation (e.g., require PlayerSpawn)

#### 3.3 Metadata Editor ✅ COMPLETE
- [x] Create metadata panel UI
- [x] Add text inputs for name, author, description
- [x] Add version and date fields
- [x] Implement metadata validation

#### 3.4 Lighting Configuration ✅ COMPLETE
- [x] Create lighting panel UI
- [x] Add ambient intensity slider
- [x] Add directional light controls
- [x] Implement real-time lighting preview
- [x] Add color picker for light color

#### 3.5 Camera Configuration ✅ COMPLETE
- [x] Create camera config panel
- [x] Add position and look_at inputs
- [x] Add rotation offset control
- [x] Implement camera preset buttons
- [x] Add "Use Current View" button

#### 3.6 World Dimensions ✅ COMPLETE
- [x] Create dimension editor UI
- [x] Add width, height, depth inputs
- [x] Implement dimension validation
- [x] Add resize confirmation dialog
- [x] Handle voxel data when resizing

### Deliverables ✅ COMPLETE
- ✅ Entity placement and editing
- ✅ Complete metadata editor
- ✅ Lighting configuration UI
- ✅ Camera configuration UI
- ✅ World dimension editor

### Success Criteria ✅ ACHIEVED
- ✅ Can place and configure entities
- ✅ Can edit all metadata fields
- ✅ Can configure lighting and see changes
- ✅ Can configure camera settings
- ✅ Can resize world dimensions safely

---

## Phase 4: File Operations

### Goals
- Implement complete file I/O
- Add file dialogs
- Handle unsaved changes
- Implement recent files

### Tasks

#### 4.1 New Map ✅ COMPLETE
- [x] Implement "New" menu action
- [x] Create default map template
- [x] Add unsaved changes warning
- [x] Clear editor state properly

#### 4.2 Open Map ✅ COMPLETE
- [x] Implement "Open" menu action
- [x] Integrate native file dialog (rfd)
- [x] Add .ron file filter
- [x] Load and parse map file
- [x] Handle parse errors gracefully
- [x] Update editor state with loaded map

#### 4.3 Save Operations ✅ COMPLETE
- [x] Implement "Save" menu action
- [x] Implement "Save As" menu action
- [x] Validate map before saving
- [x] Serialize map to RON format
- [x] Write file atomically
- [x] Update file path and clear dirty flag
- [x] Auto-expand map dimensions to fit all voxels

#### 4.4 File State Management ✅ COMPLETE
- [x] Track current file path
- [x] Implement dirty flag (modified indicator)
- [x] Add unsaved changes dialog
- [x] Update window title with filename
- [x] Add asterisk (*) for modified files

#### 4.5 Recent Files ✅ COMPLETE
- [x] Implement recent files list
- [x] Add "Open Recent" submenu
- [x] Persist recent files to config
- [x] Limit recent files list size

### Deliverables ✅ COMPLETE
- ✅ Complete file operations (New, Open, Save, Save As)
- ✅ Native file dialogs (non-blocking)
- ✅ Unsaved changes protection
- ✅ Recent files menu

### Success Criteria ✅ ACHIEVED
- ✅ Can create new maps
- ✅ Can open existing .ron files
- ✅ Can save maps successfully with auto-expand
- ✅ Warns before losing unsaved changes
- ✅ Recent files work correctly

---

## Phase 5: Polish & UX

### Goals
- Implement undo/redo system
- Add keyboard shortcuts
- Implement real-time validation
- Improve visual feedback
- Add status bar information

### Tasks

#### 5.1 Undo/Redo System ✅ COMPLETE
- [x] Implement `EditorHistory` resource
- [x] Define `EditorAction` enum
- [x] Implement action recording
- [x] Implement undo functionality
- [x] Implement redo functionality
- [x] Add undo/redo buttons
- [x] Display action history

#### 5.2 Keyboard Shortcuts ✅ COMPLETE
- [x] Implement Ctrl+N (New)
- [x] Implement Ctrl+O (Open)
- [x] Implement Ctrl+S (Save)
- [x] Implement Ctrl+Shift+S (Save As)
- [x] Implement Ctrl+Z (Undo)
- [x] Implement Ctrl+Y (Redo)
- [x] Implement Delete/Backspace (Remove)
- [x] Implement G (Toggle Grid)
- [x] Implement tool shortcuts (V, B, X, E, C)
- [x] Add keyboard shortcuts help dialog

#### 5.3 Real-time Validation ✅ COMPLETE
- [x] Implement validation system
- [x] Add validation on every change
- [x] Display validation errors in UI
- [x] Add error highlighting
- [x] Implement warning messages
- [x] Add "Fix" suggestions for common issues

#### 5.4 Visual Feedback ✅ COMPLETE
- [x] Add hover effects on UI elements
- [x] Implement cursor preview
- [x] Add selection highlighting
- [x] Implement tool indicators
- [x] Add loading spinners
- [x] Improve error messages

#### 5.5 Status Bar ✅ COMPLETE
- [x] Display current tool
- [x] Show voxel count
- [x] Show entity count
- [x] Display cursor position
- [x] Show validation status
- [x] Add modified indicator

#### 5.6 Selection & Transform Tools ✅ COMPLETE (2025-12)
- [x] Implement drag-to-select for voxels
- [x] Implement multi-selection with Shift
- [x] Add Move operation (G key) with arrow key controls
- [x] Add Rotate operation (R key) with axis selection (X/Y/Z)
- [x] Implement collision detection during transforms
- [x] Add transform preview visualization
- [x] Support entity selection and movement

#### 5.7 Keyboard Edit Mode ✅ COMPLETE (2025-12)
- [x] Implement keyboard-only cursor navigation (I to enable)
- [x] Arrow keys move cursor on X/Z plane
- [x] Space/C keys move cursor on Y axis
- [x] Enter key toggles selection in keyboard mode
- [x] Escape exits keyboard mode

#### 5.8 Outliner Panel ✅ COMPLETE (2025-12)
- [x] Create outliner panel (left side)
- [x] Display voxels grouped by type
- [x] Display entities with icons
- [x] Click-to-select from outliner
- [x] Search/filter functionality
- [x] Context menu for entity operations

### Deliverables ✅ COMPLETE
- ✅ Complete undo/redo system
- ✅ Full keyboard shortcut support
- ✅ Real-time validation with feedback
- ✅ Polished visual feedback
- ✅ Informative status bar
- ✅ Selection and transform tools
- ✅ Keyboard edit mode
- ✅ Outliner panel

### Success Criteria ✅ ACHIEVED
- ✅ Undo/redo works for all operations
- ✅ All keyboard shortcuts function
- ✅ Validation errors are clear and helpful
- ✅ UI provides good visual feedback
- ✅ Status bar shows relevant information
- ✅ Can move and rotate selected voxels
- ✅ Can navigate and edit entirely with keyboard

---

## Phase 6: Testing & Documentation

### Goals
- Comprehensive testing
- Performance optimization
- User documentation
- Developer documentation

### Tasks

#### 6.1 Unit Testing
- [ ] Test voxel placement logic
- [ ] Test voxel removal logic
- [ ] Test entity placement logic
- [ ] Test history system
- [ ] Test validation logic
- [ ] Test file I/O operations

#### 6.2 Integration Testing
- [ ] Test complete editing workflows
- [ ] Test save/load cycles
- [ ] Test undo/redo chains
- [ ] Test error recovery
- [ ] Test UI interactions

#### 6.3 Edge Case Testing
- [ ] Test with empty maps
- [ ] Test with large maps (100x100x100)
- [ ] Test with invalid files
- [ ] Test with corrupted data
- [ ] Test boundary conditions

#### 6.4 Performance Optimization
- [ ] Profile rendering performance
- [ ] Optimize voxel spawning
- [ ] Optimize UI updates
- [ ] Reduce memory usage
- [ ] Implement LOD if needed

#### 6.5 User Documentation
- [ ] Write user guide
- [ ] Create tutorial with screenshots
- [ ] Document keyboard shortcuts
- [ ] Add tooltips to UI elements
- [ ] Create video tutorial (optional)

#### 6.6 Developer Documentation
- [ ] Document code architecture
- [ ] Add inline code comments
- [ ] Create API documentation
- [ ] Write contribution guide
- [ ] Document build process

### Deliverables
- Comprehensive test suite
- Optimized performance
- Complete user documentation
- Complete developer documentation

### Success Criteria
- All tests pass
- Performance is acceptable for large maps
- Documentation is clear and complete
- New contributors can understand the code

---

## Dependencies & Prerequisites

### Required Dependencies
```toml
bevy = "0.15"
bevy_egui = "0.30"
serde = { version = "1.0", features = ["derive"] }
ron = "0.8"
thiserror = "1.0"
rfd = "0.15"  # Native file dialogs
```

### Optional Dependencies
```toml
# For better error reporting
anyhow = "1.0"

# For configuration persistence
serde_json = "1.0"

# For logging
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Development Tools
- Rust 1.70+ (latest stable)
- Cargo
- IDE with Rust support (VSCode, RustRover, etc.)
- Git for version control

---

## Risk Assessment

### High Risk Items
1. **Performance with Large Maps**
   - Mitigation: Implement LOD, culling, lazy rendering
   
2. **Undo/Redo Complexity**
   - Mitigation: Start simple, iterate based on needs
   
3. **File Format Changes**
   - Mitigation: Version checking, migration support

### Medium Risk Items
1. **UI Responsiveness**
   - Mitigation: Async operations, progress indicators
   
2. **Cross-platform File Dialogs**
   - Mitigation: Use well-tested rfd library

### Low Risk Items
1. **Basic Voxel Editing**
   - Already have working game code to reference
   
2. **Map Validation**
   - Already implemented in game

---

## Success Metrics

### Functional Metrics
- [x] Can create new maps from scratch
- [x] Can open and edit existing maps
- [x] Can save maps in valid .ron format
- [x] All voxel types and patterns work
- [x] Entity placement works correctly
- [x] Undo/redo works reliably

### Performance Metrics
- [x] Editor starts in < 2 seconds
- [x] UI remains responsive with 1000+ voxels
- [x] File operations complete in < 1 second
- [x] Undo/redo is instant

### Quality Metrics
- [x] No crashes during normal use
- [x] Clear error messages for all failures
- [x] Intuitive UI that doesn't require documentation
- [x] Keyboard shortcuts match industry standards

---

## Next Steps

After completing all phases:

1. **Beta Testing**
   - Release to small group of users
   - Gather feedback
   - Fix critical issues

2. **Public Release**
   - Announce on project channels
   - Create release notes
   - Provide download instructions

3. **Future Enhancements**
   - Terrain generation tools
   - Prefab system
   - Collaborative editing
   - Plugin system

---

**Document Version**: 2.0.0
**Last Updated**: 2025-12-10
**Status**: Phases 1-5 Complete - Full-featured map editor with undo/redo, transforms, keyboard mode, and outliner