# Map Editor Implementation Roadmap

## Overview

This document outlines the step-by-step implementation plan for the A Drake's Story Map Editor. The project is divided into 6 phases, each building upon the previous one.

## Timeline Estimate

- **Phase 1**: 2-3 days (Foundation)
- **Phase 2**: 3-4 days (Basic Editing)
- **Phase 3**: 4-5 days (Advanced Features)
- **Phase 4**: 2-3 days (File Operations)
- **Phase 5**: 3-4 days (Polish & UX)
- **Phase 6**: 2-3 days (Testing & Documentation)

**Total Estimated Time**: 16-22 days

## Phase 1: Foundation (Core Infrastructure)

### Goals
- Set up the basic application structure
- Integrate bevy_egui
- Create minimal working editor window
- Establish core data structures

### Tasks

#### 1.1 Project Setup
- [ ] Add `bevy_egui` dependency to Cargo.toml
- [ ] Add `rfd` (native file dialogs) dependency
- [ ] Create `src/editor/` module structure
- [ ] Create `src/bin/map_editor.rs` entry point

#### 1.2 Basic Window & UI
- [ ] Initialize Bevy app with egui plugin
- [ ] Create main window with basic layout
- [ ] Implement top menu bar (File, Edit, View, Tools, Help)
- [ ] Implement status bar at bottom
- [ ] Set up 3-panel layout (toolbar, viewport, properties)

#### 1.3 Editor State
- [ ] Define `EditorState` resource
- [ ] Define `EditorTool` enum
- [ ] Implement state initialization
- [ ] Add state serialization for settings

#### 1.4 Camera System
- [ ] Create `EditorCamera` component
- [ ] Implement orbit camera controls (mouse drag)
- [ ] Implement pan controls (middle mouse)
- [ ] Implement zoom controls (mouse wheel)
- [ ] Add camera reset functionality

#### 1.5 Grid Visualization
- [ ] Create grid mesh generator
- [ ] Implement grid rendering system
- [ ] Add grid toggle functionality
- [ ] Add grid opacity control
- [ ] Implement grid snapping logic

### Deliverables
- Working editor window with 3D viewport
- Functional camera controls
- Visible grid system
- Basic UI layout

### Success Criteria
- Editor launches without errors
- Camera can orbit, pan, and zoom
- Grid is visible and can be toggled
- UI panels are properly laid out

---

## Phase 2: Basic Editing (Voxel Tools)

### Goals
- Implement core voxel editing functionality
- Enable placing and removing voxels
- Add voxel type and pattern selection

### Tasks

#### 2.1 Voxel Placement Tool
- [ ] Implement cursor positioning in 3D space
- [ ] Add ray casting for mouse-to-world position
- [ ] Implement voxel placement on click
- [ ] Add visual cursor indicator
- [ ] Implement grid snapping for placement

#### 2.2 Voxel Removal Tool
- [ ] Implement voxel selection via ray cast
- [ ] Add voxel removal on click
- [ ] Add visual highlight for selected voxel
- [ ] Implement delete key support

#### 2.3 Voxel Type Selection
- [ ] Create voxel type dropdown UI
- [ ] Add icons/colors for each type (Grass, Dirt, Stone)
- [ ] Implement type switching
- [ ] Update cursor preview with selected type

#### 2.4 Pattern Selection
- [ ] Create pattern dropdown UI
- [ ] Add preview icons for patterns
- [ ] Implement pattern switching
- [ ] Update cursor preview with selected pattern

#### 2.5 Voxel Rendering
- [ ] Integrate existing voxel spawning code
- [ ] Implement real-time voxel updates
- [ ] Add voxel highlighting on hover
- [ ] Optimize rendering for editor use

### Deliverables
- Functional voxel placement tool
- Functional voxel removal tool
- Type and pattern selection UI
- Real-time 3D preview

### Success Criteria
- Can place voxels at any grid position
- Can remove voxels by clicking
- Can switch between voxel types
- Can switch between patterns
- Changes are immediately visible in 3D

---

## Phase 3: Advanced Features

### Goals
- Add entity placement and editing
- Implement metadata editor
- Add lighting and camera configuration
- Enable world dimension editing

### Tasks

#### 3.1 Entity Placement
- [ ] Create entity placement tool
- [ ] Add entity type selection UI
- [ ] Implement entity positioning
- [ ] Add visual indicators for entities
- [ ] Implement entity removal

#### 3.2 Entity Properties
- [ ] Create entity properties panel
- [ ] Add position editing (X, Y, Z inputs)
- [ ] Implement custom properties editor
- [ ] Add entity validation (e.g., require PlayerSpawn)

#### 3.3 Metadata Editor
- [ ] Create metadata panel UI
- [ ] Add text inputs for name, author, description
- [ ] Add version and date fields
- [ ] Implement metadata validation

#### 3.4 Lighting Configuration
- [ ] Create lighting panel UI
- [ ] Add ambient intensity slider
- [ ] Add directional light controls
- [ ] Implement real-time lighting preview
- [ ] Add color picker for light color

#### 3.5 Camera Configuration
- [ ] Create camera config panel
- [ ] Add position and look_at inputs
- [ ] Add rotation offset control
- [ ] Implement camera preset buttons
- [ ] Add "Use Current View" button

#### 3.6 World Dimensions
- [ ] Create dimension editor UI
- [ ] Add width, height, depth inputs
- [ ] Implement dimension validation
- [ ] Add resize confirmation dialog
- [ ] Handle voxel data when resizing

### Deliverables
- Entity placement and editing
- Complete metadata editor
- Lighting configuration UI
- Camera configuration UI
- World dimension editor

### Success Criteria
- Can place and configure entities
- Can edit all metadata fields
- Can configure lighting and see changes
- Can configure camera settings
- Can resize world dimensions safely

---

## Phase 4: File Operations

### Goals
- Implement complete file I/O
- Add file dialogs
- Handle unsaved changes
- Implement recent files

### Tasks

#### 4.1 New Map
- [ ] Implement "New" menu action
- [ ] Create default map template
- [ ] Add unsaved changes warning
- [ ] Clear editor state properly

#### 4.2 Open Map
- [ ] Implement "Open" menu action
- [ ] Integrate native file dialog (rfd)
- [ ] Add .ron file filter
- [ ] Load and parse map file
- [ ] Handle parse errors gracefully
- [ ] Update editor state with loaded map

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

#### 4.5 Recent Files
- [ ] Implement recent files list
- [ ] Add "Open Recent" submenu
- [ ] Persist recent files to config
- [ ] Limit recent files list size

### Deliverables ✅ COMPLETE
- ✅ Complete file operations (New, Open, Save, Save As)
- ✅ Native file dialogs (non-blocking)
- ✅ Unsaved changes protection
- ⏳ Recent files menu (planned)

### Success Criteria ✅ ACHIEVED
- ✅ Can create new maps
- ✅ Can open existing .ron files
- ✅ Can save maps successfully with auto-expand
- ✅ Warns before losing unsaved changes
- ⏳ Recent files work correctly (planned)

---

## Phase 5: Polish & UX

### Goals
- Implement undo/redo system
- Add keyboard shortcuts
- Implement real-time validation
- Improve visual feedback
- Add status bar information

### Tasks

#### 5.1 Undo/Redo System
- [ ] Implement `EditorHistory` resource
- [ ] Define `EditorAction` enum
- [ ] Implement action recording
- [ ] Implement undo functionality
- [ ] Implement redo functionality
- [ ] Add undo/redo buttons
- [ ] Display action history

#### 5.2 Keyboard Shortcuts
- [ ] Implement Ctrl+N (New)
- [ ] Implement Ctrl+O (Open)
- [ ] Implement Ctrl+S (Save)
- [ ] Implement Ctrl+Shift+S (Save As)
- [ ] Implement Ctrl+Z (Undo)
- [ ] Implement Ctrl+Y (Redo)
- [ ] Implement Delete/Backspace (Remove)
- [ ] Implement G (Toggle Grid)
- [ ] Implement tool shortcuts (V, B, E, C)
- [ ] Add keyboard shortcuts help dialog

#### 5.3 Real-time Validation
- [ ] Implement validation system
- [ ] Add validation on every change
- [ ] Display validation errors in UI
- [ ] Add error highlighting
- [ ] Implement warning messages
- [ ] Add "Fix" suggestions for common issues

#### 5.4 Visual Feedback
- [ ] Add hover effects on UI elements
- [ ] Implement cursor preview
- [ ] Add selection highlighting
- [ ] Implement tool indicators
- [ ] Add loading spinners
- [ ] Improve error messages

#### 5.5 Status Bar
- [ ] Display current tool
- [ ] Show voxel count
- [ ] Show entity count
- [ ] Display cursor position
- [ ] Show validation status
- [ ] Add modified indicator

### Deliverables
- Complete undo/redo system
- Full keyboard shortcut support
- Real-time validation with feedback
- Polished visual feedback
- Informative status bar

### Success Criteria
- Undo/redo works for all operations
- All keyboard shortcuts function
- Validation errors are clear and helpful
- UI provides good visual feedback
- Status bar shows relevant information

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
- [ ] Can create new maps from scratch
- [ ] Can open and edit existing maps
- [ ] Can save maps in valid .ron format
- [ ] All voxel types and patterns work
- [ ] Entity placement works correctly
- [ ] Undo/redo works reliably

### Performance Metrics
- [ ] Editor starts in < 2 seconds
- [ ] UI remains responsive with 1000+ voxels
- [ ] File operations complete in < 1 second
- [ ] Undo/redo is instant

### Quality Metrics
- [ ] No crashes during normal use
- [ ] Clear error messages for all failures
- [ ] Intuitive UI that doesn't require documentation
- [ ] Keyboard shortcuts match industry standards

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

**Document Version**: 1.1.0
**Last Updated**: 2025-10-23
**Status**: Phase 4 Complete - File Operations Implemented