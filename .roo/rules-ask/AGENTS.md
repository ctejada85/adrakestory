# AGENTS.md - Ask Mode

This file provides guidance for ask mode when working with this repository.

## Project Overview

A Drake's Story is a 3D voxel game with sub-voxel rendering (8×8×8 per voxel) built with Rust and Bevy 0.15.

## Key Documentation Locations

- [`docs/developer-guide/architecture.md`](../../docs/developer-guide/architecture.md) - System architecture and ECS patterns
- [`docs/developer-guide/systems/`](../../docs/developer-guide/systems/) - Individual system documentation
- [`docs/api/map-format-spec.md`](../../docs/api/map-format-spec.md) - RON map format specification
- [`docs/user-guide/map-editor/`](../../docs/user-guide/map-editor/) - Map editor user guide

## Non-Obvious Code Organization

- `src/lib.rs` exports shared code for both binaries (game and editor)
- `src/systems/game/` contains core gameplay - NOT in a separate crate
- `src/editor/` is editor-specific code but lives in main crate
- `src/bin/map_editor.rs` is just the entry point - actual editor code in `src/editor/`

## Terminology

- **Voxel**: 1×1×1 world unit containing 8×8×8 sub-voxels
- **Sub-voxel**: 0.125×0.125×0.125 unit, smallest renderable element
- **Chunk**: 16×16×16 voxels grouped for meshing/LOD
- **Pattern**: Sub-voxel arrangement (Full, Staircase, Platform, Pillar, Fence)
- **Greedy meshing**: Optimization that merges adjacent same-color faces

## State Machine

```
IntroAnimation → TitleScreen → LoadingMap → InGame ⇄ Paused
```

States defined in [`src/states.rs`](../../src/states.rs). Systems are state-gated via `.run_if(in_state(...))`.

## Map Format Quick Reference

Maps are RON files with structure:
- `metadata`: name, author, version
- `world`: dimensions + voxel list with pos, type, pattern, rotation
- `entities`: spawn points, NPCs, lights
- `lighting`: ambient + directional
- `camera`: initial position and look_at