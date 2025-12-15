# AGENTS.md - Code Mode

This file provides guidance for code mode when working with this repository.

## Critical Coding Rules

- **Cylinder collider math**: Player collision uses cylinder (radius: 0.2, half_height: 0.4). Horizontal checks use radius, vertical uses half_height. See [`check_sub_voxel_collision()`](../../src/systems/game/collision.rs:103-189)
- **SubVoxel bounds are cached**: Never recalculate bounds - use `sub_voxel.bounds` directly. Bounds are set at spawn time in [`spawn_voxels_chunked()`](../../src/systems/game/map/spawner.rs:1003-1224)
- **SpatialGrid required for collision**: Always use `spatial_grid.get_entities_in_aabb()` for collision queries. Direct iteration over all SubVoxels is O(n²) and will cause performance issues
- **Delta time clamping**: Physics systems clamp `time.delta_secs().min(0.1)` to prevent physics explosions after window focus loss. See [`apply_gravity()`](../../src/systems/game/physics.rs:22-28)
- **Editor history required**: All map modifications in editor MUST go through `EditorHistory` for undo/redo support. Direct map mutation breaks undo stack

## System Set Ordering

Game systems run in strict order via `GameSystemSet`:
1. `Input` - gather input
2. `Movement` - process player movement  
3. `Physics` - apply gravity, collisions
4. `Visual` - update visual elements
5. `Camera` - update camera last

Adding systems to wrong set causes race conditions.

## Sub-Voxel Geometry

- Geometry stored as bit arrays in [`SubVoxelGeometry`](../../src/systems/game/map/geometry.rs:30-35) - 8 u64 layers
- Rotation uses integer math centered at (3.5, 3.5, 3.5) - see [`rotate_point()`](../../src/systems/game/map/geometry.rs:143-173)
- Pattern variants like `StaircaseX`, `StaircaseNegX`, `StaircaseZ` are pre-rotated versions, not runtime rotations

## Chunk Meshing

- Greedy meshing merges adjacent same-color faces into larger quads
- LOD meshes are built at spawn time, not runtime - see [`ChunkLOD`](../../src/systems/game/map/spawner.rs:55-61)
- Chunk material can be either `OcclusionMaterial` (custom shader) or `StandardMaterial` (PBR) based on config