# Changelog

All notable changes to A Drake's Story will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Map Editor - Drag-to-Place Voxels**: Hold left-click and drag with Voxel Place tool to place multiple voxels
  - Voxels placed in direction of cursor movement
  - Extends from the last placed voxel position
  - Makes drawing lines and walls much faster

- **Map Editor - Drag-to-Remove Voxels**: Hold left-click and drag with Voxel Remove tool to remove multiple voxels
  - Removes each voxel the cursor passes over
  - Quickly clear areas by dragging across them

- **Map Editor - Drag-to-Select**: Hold left-click and drag with Select tool to select multiple voxels
  - Selects each voxel the cursor passes over
  - Click on already-selected voxel (without dragging) to deselect

- **Map Editor - Recent Files**: Added Recent Files feature to File menu
  - Tracks last 10 opened map files
  - Persists across editor sessions (stored in user config directory)
  - Quick access to frequently used maps
  - Automatically removes non-existent files from list

- **Map Editor - Tool Keyboard Shortcuts**: Added missing tool selection shortcuts
  - `V` - Select tool
  - `B` - Voxel Place tool
  - `X` - Voxel Remove tool
  - `E` - Entity Place tool
  - `C` - Camera tool
  - `1` and `2` shortcuts retained for backward compatibility

### Fixed
- **Map Editor - Panel Overlay Positioning**: Floating overlays (camera controls, tool options) now position relative to side panels instead of screen edges
  - Overlays dynamically adjust when panels are resized
  - Status bar height is properly accounted for bottom margins

- **Map Editor - Resize Bar Click-Through**: Fixed issue where clicking on panel resize bars would trigger tool actions
  - Added `is_using_pointer()` check alongside `is_pointer_over_area()`
  - Prevents voxel/entity placement while dragging resize bars

- **Map Editor - Entity Grid Alignment**: Entity placement now snaps to grid like voxels
  - New entities placed at integer grid positions
  - Entity rendering uses `.round()` for position snapping
  - Legacy entities with float positions are displayed correctly

- **Map Editor - Entity Movement Responsiveness**: Fixed entity movement lag when using arrow keys
  - Added proper system ordering with `.after()` constraints
  - Transformation operations now run after keyboard input handling

- **Map Editor - Tool Shortcuts Not Working**: Fixed keyboard shortcuts not switching tools
  - Added system ordering to ensure keyboard handling runs after UI rendering
  - Prevents egui from consuming keyboard events meant for tool switching

- **Map Editor Save Function**: Fixed critical bug where maps with negative voxel coordinates would save with incorrect dimensions, causing "Invalid voxel position" errors on load. The save function now automatically normalizes all coordinates to start at (0, 0, 0) by:
  - Calculating the bounding box of all voxels
  - Determining the offset needed to shift minimum coordinates to origin
  - Applying the offset to all voxels, entities, and camera positions
  - Setting dimensions based on actual voxel span rather than just maximum values
  - This ensures all saved maps are valid and maintain proper spatial relationships
  - Backward compatible: maps already starting at origin are unchanged

### Changed
- **Map Editor**: Coordinate normalization is now performed automatically during save operations
- **Documentation**: Updated architecture, API specification, and user guides to reflect coordinate normalization behavior

## [0.1.0] - 2025-01-10

### Added
- Initial release
- Basic voxel-based world with sub-voxel patterns
- Character movement and physics
- Map loading system with RON format
- Map editor with voxel placement tools
- Camera follow system
- Collision detection
- Title screen and loading screen
- Pause menu functionality

### Known Issues
- Map editor may allow placement of voxels at negative coordinates during editing (fixed in unreleased)