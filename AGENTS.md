# AGENTS.md

This file provides guidance to agents when working with code in this repository.

## Build Commands

```bash
cargo build                          # Debug build (game)
cargo build --release                # Release build (recommended for testing)
cargo build --bin map_editor         # Build map editor only
cargo run --release                  # Run game
cargo run --bin map_editor --release # Run map editor
cargo run -- --map path/to/map.ron   # Load specific map directly (skips title screen)
```

## Testing

```bash
cargo test                           # Run all tests
cargo test test_name                 # Run single test by name
cargo test -- --nocapture            # Run with stdout output
```

## Linting

```bash
cargo clippy                         # Lint check (runs on save via rust-analyzer)
cargo fmt                            # Format code (runs on save)
```

## Non-Obvious Architecture

- **Two binaries**: `adrakestory` (game) and `map_editor` (standalone editor) - both share code via `src/lib.rs`
- **Sub-voxel system**: Each voxel contains 8×8×8 sub-voxels for high-detail rendering. Constants in [`src/systems/game/map/spawner/`](src/systems/game/map/spawner/)
- **Chunk-based meshing**: Voxels are grouped into 16×16×16 chunks with greedy meshing and 4 LOD levels
- **Cylinder collider**: Player uses cylinder collision (radius: 0.2, half_height: 0.4), NOT a box. See [`Player`](src/systems/game/components.rs:4-23)
- **SubVoxel bounds caching**: Collision bounds are pre-calculated at spawn time in [`SubVoxel.bounds`](src/systems/game/components.rs:37-42), not computed per-frame
- **SpatialGrid**: Collision detection uses spatial partitioning via [`SpatialGrid`](src/systems/game/resources.rs) - always use it for O(n) instead of O(n²)
- **Camera separation**: 2D camera for UI states, 3D camera for gameplay - they are mutually exclusive, see [`cleanup_2d_camera()`](src/main.rs:282-287)

## Map Format

- Maps use RON format in `assets/maps/`
- Map data structures in [`src/systems/game/map/format/`](src/systems/game/map/format/)
- Patterns like `SubVoxelPattern::Full`, `StaircaseX`, `PlatformXZ` define sub-voxel geometry
- Rotation state is stored separately from pattern and applied via [`geometry_with_rotation()`](src/systems/game/map/format/rotation.rs)

## Editor-Specific

- Editor uses `bevy_egui` for UI
- History/undo system in [`src/editor/history.rs`](src/editor/history.rs) - all map modifications must go through `EditorHistory`
- Hot reload via F5 or Ctrl+R when running game from editor

## Code Style

- Systems follow Bevy ECS patterns: `fn system_name(query: Query<...>, res: Res<...>)`
- Components are pure data structs with `#[derive(Component)]`
- Use `info!()`, `warn!()`, `error!()` for logging (Bevy's tracing)
- Tests are inline with `#[cfg(test)]` modules at bottom of files

## File Size Guidelines

- **Keep files small and focused**: Each file should have a single, clear responsibility
- **Target ~200-400 lines per file**: Split larger files into focused modules
- **Extract when complexity grows**: If a system or component grows beyond ~300 lines, consider splitting into submodules
- **Prefer many small files over few large files**: Easier to navigate, review, and maintain
- **Module organization**: Group related functionality in directories with `mod.rs` re-exporting public items
- **Example splits**:
  - `systems.rs` (500+ lines) → `camera.rs`, `physics.rs`, `collision.rs`, `input.rs`
  - `components.rs` (300+ lines) → `player.rs`, `voxel.rs`, `camera.rs`
- **Signs a file needs splitting**: Multiple unrelated `impl` blocks, many `#[cfg(test)]` sections, scrolling to find functions

---

## Critical Coding Rules

- **Cylinder collider math**: Player collision uses cylinder (radius: 0.2, half_height: 0.4). Horizontal checks use radius, vertical uses half_height. See [`check_sub_voxel_collision()`](src/systems/game/collision.rs:103-189)
- **SubVoxel bounds are cached**: Never recalculate bounds - use `sub_voxel.bounds` directly. Bounds are set at spawn time in [`spawn_voxels_chunked()`](src/systems/game/map/spawner/chunks.rs)
- **SpatialGrid required for collision**: Always use `spatial_grid.get_entities_in_aabb()` for collision queries. Direct iteration over all SubVoxels is O(n²) and will cause performance issues
- **Delta time clamping**: Physics systems clamp `time.delta_secs().min(0.1)` to prevent physics explosions after window focus loss. See [`apply_gravity()`](src/systems/game/physics.rs:22-28)
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

- Geometry stored as bit arrays in [`SubVoxelGeometry`](src/systems/game/map/geometry/types.rs) - 8 u64 layers
- Rotation uses integer math centered at (3.5, 3.5, 3.5) - see [`rotate_point()`](src/systems/game/map/geometry/rotation.rs)
- Pattern variants like `StaircaseX`, `StaircaseNegX`, `StaircaseZ` are pre-rotated versions, not runtime rotations

## Chunk Meshing

- Greedy meshing merges adjacent same-color faces into larger quads
- LOD meshes are built at spawn time, not runtime - see [`ChunkLOD`](src/systems/game/map/spawner/meshing.rs)
- Chunk material can be either `OcclusionMaterial` (custom shader) or `StandardMaterial` (PBR) based on config

---

## Debugging

### In-Game Debug Keys

- `C` - Toggle collision box visualization (shows cylinder collider)
- `F3` - Toggle FPS counter overlay
- `ESC` - Pause menu

### VSCode Debugging

- Use "Debug (CodeLLDB, Debug Build)" launch config for breakpoints
- LLDB configured to hide disassembly - see [`.vscode/settings.json`](.vscode/settings.json:11)
- Debug builds have `opt-level = 1` for faster iteration while dependencies use `opt-level = 3`

### Common Issues

- **Physics explosion after alt-tab**: Delta time is clamped to 0.1s max in physics systems. If you see teleporting, check delta clamping
- **Camera order ambiguity warning**: 2D and 3D cameras must not coexist. Check [`cleanup_2d_camera()`](src/main.rs:282-287) runs on state transition
- **Collision not working**: Verify SubVoxel entities are in SpatialGrid. Check [`spawn_voxels_chunked()`](src/systems/game/map/spawner/chunks.rs) for grid insertion
- **Map not loading**: Check RON syntax. Validation errors logged via `warn!()`. See [`validation.rs`](src/systems/game/map/validation.rs)

### Hot Reload

- F5 or Ctrl+R reloads map file when running from editor
- Ctrl+H toggles hot reload on/off
- Player position is preserved across reloads via [`restore_player_position()`](src/systems/game/hot_reload/)
- Debounce prevents rapid reloads - 500ms minimum between reloads

### Logging

- Use `RUST_LOG=info cargo run` for detailed logs
- Map loading progress logged at each stage
- Chunk/quad counts logged after spawn: "Spawned X chunks with Y quads"

---

## Documentation & Code Organization

### Key Documentation Locations

- [`docs/developer-guide/architecture.md`](docs/developer-guide/architecture.md) - System architecture and ECS patterns
- [`docs/developer-guide/systems/`](docs/developer-guide/systems/) - Individual system documentation
- [`docs/api/map-format-spec.md`](docs/api/map-format-spec.md) - RON map format specification
- [`docs/user-guide/map-editor/`](docs/user-guide/map-editor/) - Map editor user guide

### Non-Obvious Code Organization

- `src/lib.rs` exports shared code for both binaries (game and editor)
- `src/systems/game/` contains core gameplay - NOT in a separate crate
- `src/editor/` is editor-specific code but lives in main crate
- `src/bin/map_editor.rs` is just the entry point - actual editor code in `src/editor/`

### Terminology

- **Voxel**: 1×1×1 world unit containing 8×8×8 sub-voxels
- **Sub-voxel**: 0.125×0.125×0.125 unit, smallest renderable element
- **Chunk**: 16×16×16 voxels grouped for meshing/LOD
- **Pattern**: Sub-voxel arrangement (Full, Staircase, Platform, Pillar, Fence)
- **Greedy meshing**: Optimization that merges adjacent same-color faces

### State Machine

```
IntroAnimation → TitleScreen → LoadingMap → InGame ⇄ Paused
```

States defined in [`src/states.rs`](src/states.rs). Systems are state-gated via `.run_if(in_state(...))`.

---

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

- **New entity types**: Add to `EntityType` enum in [`format/entities.rs`](src/systems/game/map/format/entities.rs), spawn logic in [`spawner/entities.rs`](src/systems/game/map/spawner/entities.rs)
- **New sub-voxel patterns**: Add to `SubVoxelPattern` enum in [`format/patterns.rs`](src/systems/game/map/format/patterns.rs), implement geometry in [`geometry/`](src/systems/game/map/geometry/)
- **New game states**: Add to `GameState` enum in [`states.rs`](src/states.rs), create state module in `src/systems/`

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