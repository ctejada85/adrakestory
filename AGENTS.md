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
- **Sub-voxel system**: Each voxel contains 8×8×8 sub-voxels for high-detail rendering. Constants in [`src/systems/game/map/spawner.rs`](src/systems/game/map/spawner.rs:30-32)
- **Chunk-based meshing**: Voxels are grouped into 16×16×16 chunks with greedy meshing and 4 LOD levels
- **Cylinder collider**: Player uses cylinder collision (radius: 0.2, half_height: 0.4), NOT a box. See [`Player`](src/systems/game/components.rs:4-23)
- **SubVoxel bounds caching**: Collision bounds are pre-calculated at spawn time in [`SubVoxel.bounds`](src/systems/game/components.rs:37-42), not computed per-frame
- **SpatialGrid**: Collision detection uses spatial partitioning via [`SpatialGrid`](src/systems/game/resources.rs) - always use it for O(n) instead of O(n²)
- **Camera separation**: 2D camera for UI states, 3D camera for gameplay - they are mutually exclusive, see [`cleanup_2d_camera()`](src/main.rs:282-287)

## Map Format

- Maps use RON format in `assets/maps/`
- Map data structures in [`src/systems/game/map/format.rs`](src/systems/game/map/format.rs)
- Patterns like `SubVoxelPattern::Full`, `StaircaseX`, `PlatformXZ` define sub-voxel geometry
- Rotation state is stored separately from pattern and applied via [`geometry_with_rotation()`](src/systems/game/map/format.rs:202-213)

## Editor-Specific

- Editor uses `bevy_egui` for UI
- History/undo system in [`src/editor/history.rs`](src/editor/history.rs) - all map modifications must go through `EditorHistory`
- Hot reload via F5 or Ctrl+R when running game from editor

## Code Style

- Systems follow Bevy ECS patterns: `fn system_name(query: Query<...>, res: Res<...>)`
- Components are pure data structs with `#[derive(Component)]`
- Use `info!()`, `warn!()`, `error!()` for logging (Bevy's tracing)
- Tests are inline with `#[cfg(test)]` modules at bottom of files