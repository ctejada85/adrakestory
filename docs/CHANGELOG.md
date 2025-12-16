# Changelog

All notable changes to A Drake's Story will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Map Editor - Controller Support**: Full Xbox controller support for Minecraft Creative mode-style editing
  - **Flying Camera**: Left stick moves, right stick looks, A/B buttons for vertical movement
  - **Raycast Cursor**: Cursor automatically targets voxel faces you're looking at
  - **Trigger Actions**: RT executes current tool action, LT always removes voxels
  - **Automatic Input Switching**: Seamlessly switch between controller and mouse/keyboard
  - **Tool Integration**: Works with all editor tools (Place, Remove, Entity, Select)

- **In-Game FPS Counter**: Added toggleable FPS counter for performance monitoring
  - Press `F3` to toggle the FPS counter on/off
  - Displays in top-left corner with green text
  - Uses Bevy's built-in `FrameTimeDiagnosticsPlugin` for accurate measurements
  - Only updates when visible to minimize performance impact

- **Map Editor - Voxel Rendering Optimizations**: Implemented Tiers 1-5 rendering optimizations for the map editor viewport
  - **Tier 1: Material Palette** - Shared material palette reduces GPU memory from millions of materials to 64
  - **Tier 2: GPU Instancing** - Sub-voxels with same material are automatically batched
  - **Tier 3: Chunk-Based Meshing** - Voxels grouped into 16³ chunks with merged meshes (99.99% entity reduction)
  - **Tier 4: Hidden Face Culling** - Interior faces between adjacent voxels are culled (60-90% triangle reduction)
  - **Tier 5: Greedy Meshing** - Adjacent coplanar faces merged into larger quads
  - **Note**: Tier 6 (LOD) intentionally disabled for editor - full detail needed when editing

- **Map Editor - Frustum Culling**: Elements outside the camera viewport are no longer rendered
  - Voxel chunks have explicit AABB components for Bevy's automatic frustum culling
  - Entity markers include AABB for frustum culling
  - Grid generation uses frustum bounds testing to only create visible grid lines

- **Map Editor - Dynamic Grid Render Distance**: Grid now scales with camera zoom level
  - When zoomed out, grid extends further to maintain infinite appearance
  - Render distance formula: `base + camera_height * 2 + camera_distance * 1.5`
  - Grid regenerates when zoom changes significantly

- **Map Editor - Increased Camera Zoom Range**: Maximum camera distance increased from 50 to 200 units
  - Allows viewing larger maps from further away
  - Grid automatically extends to match zoom level

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

- **Map Editor - Tool Parameter Memory**: Tools now remember their last-used parameters when switching between them
  - Voxel Place tool remembers selected voxel type and pattern
  - Entity Place tool remembers selected entity type
  - Parameters persist during the editing session
  - Switching back to a tool restores its previous settings

- **Map Editor - Global Keyboard Shortcuts**: Implemented standard keyboard shortcuts for common operations
  - `Ctrl+S` - Save current map
  - `Ctrl+Shift+S` - Save As (new file location)
  - `Ctrl+O` - Open map file
  - `Ctrl+N` - New map
  - `Ctrl+Z` - Undo last action
  - `Ctrl+Y` or `Ctrl+Shift+Z` - Redo last undone action
  - Shortcuts work globally (not just in menus)
  - Menu items now display their keyboard shortcuts

- **Map Editor - Functional Undo/Redo System**: Undo and Redo operations are now fully functional
  - Supports voxel placement and removal
  - Supports entity placement, removal, and modification
  - Supports metadata changes
  - Supports batch operations (multiple actions as one undo step)
  - Works via keyboard shortcuts (`Ctrl+Z`/`Ctrl+Y`) and menu buttons

### Changed
- **Player Collision Shape**: Changed from sphere to cylinder collider
  - `radius` (0.2) controls horizontal collision (XZ plane)
  - `half_height` (0.4) controls vertical extent (total height 0.8)
  - More accurate collision for humanoid characters
  - Debug collision box (toggle with 'C' key) now displays cylinder shape
  - Fixed corner-landing exploit where players could land on voxel corners

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

- **Map Editor - Double Voxel Placement/Removal**: Fixed bug where clicking to place or remove a voxel would sometimes place/remove multiple voxels
  - Issue occurred because after placing/removing a voxel, the cursor raycast would hit a different voxel (adjacent or behind)
  - The drag handler incorrectly interpreted this geometry change as intentional mouse movement
  - Fix: Added screen-space mouse movement threshold (5 pixels) before drag operations activate
  - Single clicks now reliably place/remove exactly one voxel
  - Drag-to-place/remove functionality still works when intentionally dragging

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