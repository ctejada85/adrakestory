# AGENTS.md - Architect Mode

This file provides guidance for architect mode when working with this repository.

## Architecture Constraints

- **ECS-only**: All game logic must use Bevy's Entity Component System. No OOP patterns with methods on components
- **State-gated systems**: Systems must be gated to appropriate `GameState`. Adding systems without state conditions causes them to run in all states
- **System set ordering**: Physics-related systems MUST respect `GameSystemSet` ordering or race conditions occur
- **Single camera per state**: 2D camera for UI states, 3D camera for InGame. Never have both active simultaneously

## Performance-Critical Patterns

- **SpatialGrid for collision**: O(n) spatial partitioning is mandatory. Direct SubVoxel iteration is O(n²) and unacceptable for large maps
- **Bounds caching**: SubVoxel bounds are computed once at spawn. Any system that needs bounds must use cached values, not recompute
- **Chunk-based rendering**: Sub-voxels are merged into chunk meshes via greedy meshing. Individual sub-voxel entities are collision-only (no mesh)
- **LOD system**: 4 LOD levels per chunk, switched based on camera distance. LOD meshes built at spawn time

## Extension Points

- **New entity types**: Add to `EntityType` enum in [`format.rs`](../../src/systems/game/map/format.rs:229-243), spawn logic in [`spawner.rs`](../../src/systems/game/map/spawner.rs:1227-1264)
- **New sub-voxel patterns**: Add to `SubVoxelPattern` enum, implement geometry in [`geometry.rs`](../../src/systems/game/map/geometry.rs)
- **New game states**: Add to `GameState` enum in [`states.rs`](../../src/states.rs), create state module in `src/systems/`

## Hidden Coupling

- `LoadedMapData` resource must exist before `spawn_map_system` runs
- `GameInitialized` resource prevents duplicate spawning - must be reset for hot reload
- `OcclusionConfig` affects material type selection at spawn time - cannot be changed after spawn
- Editor's `EditorHistory` and `EditorState` are tightly coupled - modifications must update both

## Map Editor Architecture

- Editor is standalone binary but shares game code via `src/lib.rs`
- Uses `bevy_egui` for immediate-mode UI
- Tool system: `EditorTool` enum with tool-specific state in `ToolMemory`
- All map changes go through `EditorHistory` for undo/redo
- Renderer detects map changes via `MapRenderState` dirty flag