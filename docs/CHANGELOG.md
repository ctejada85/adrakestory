# Changelog

All notable changes to A Drake's Story will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
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