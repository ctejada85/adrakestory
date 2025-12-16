# File Size Refactoring Plan - Phase 2

## Overview

This plan identifies additional files that could benefit from refactoring based on the file size guidelines (~200-400 lines target, ~300 lines as split threshold). These are lower priority than Phase 1 but would improve code organization.

**Status**: 📋 Planned  
**Priority**: Low  
**Last Updated**: 2025-12-15

---

## Files Analysis

Files sorted by line count that exceed or approach the guidelines:

| File | Lines | Priority | Recommendation |
|------|-------|----------|----------------|
| `src/systems/game/map/spawner/meshing/` | 649→5 files | ✅ Done | Split into modules |
| `src/editor/tools/voxel_tool.rs` | 416 | 🟡 Medium | Split into modules |
| `src/editor/ui/dialogs.rs` | 368 | 🟢 Low | Consider splitting |
| `src/systems/game/occlusion.rs` | 341 | 🟢 Low | Acceptable - single responsibility |
| `src/systems/game/gamepad.rs` | 339 | 🟢 Low | Acceptable - single responsibility |
| `src/editor/ui/outliner.rs` | 330 | 🟢 Low | Acceptable - single responsibility |
| `src/editor/renderer.rs` | 325 | 🟢 Low | Acceptable - single responsibility |
| `src/editor/ui/viewport.rs` | 321 | 🟢 Low | Acceptable - single responsibility |
| `src/main.rs` | 318 | 🟢 Low | Acceptable - entry point |
| `src/systems/game/map/spawner/mod.rs` | 316 | 🟢 Low | Acceptable - module root |

---

## Refactoring 1: meshing.rs (649 lines → 5 files)

### Status: ✅ Completed (2025-12-15)

### Problem

The meshing module is large and handles multiple distinct responsibilities:
- Occupancy grid for neighbor lookups
- Greedy meshing algorithm
- Mesh building (quad generation, vertex data)
- LOD mesh generation

### Proposed Structure

```
src/systems/game/map/spawner/meshing/
├── mod.rs              # Re-exports, Face enum (~50 lines)
├── occupancy.rs        # OccupancyGrid (~60 lines)
├── greedy_mesher.rs    # GreedyMesher algorithm (~250 lines)
├── mesh_builder.rs     # ChunkMeshBuilder, quad generation (~250 lines)
└── palette.rs          # VoxelMaterialPalette (~90 lines)
```

### Benefits
- Separates algorithm from data structures
- Easier to test greedy meshing independently
- Clear ownership of mesh building logic

### Migration Steps
1. Create `meshing/` directory
2. Extract `OccupancyGrid` to `occupancy.rs`
3. Extract `GreedyMesher` to `greedy_mesher.rs`
4. Extract `ChunkMeshBuilder` to `mesh_builder.rs`
5. Extract `VoxelMaterialPalette` to `palette.rs`
6. Update imports in `spawner/mod.rs` and `chunks.rs`
7. Verify build and tests pass

---

## Refactoring 2: voxel_tool.rs (416 lines → ~3 files)

### Status: 📋 Planned

### Problem

The voxel tool handles both placement and removal with duplicate drag state logic. The file contains:
- Placement logic with drag-to-place
- Removal logic with drag-to-remove
- Shared helper functions
- Two separate drag state resources

### Proposed Structure

```
src/editor/tools/voxel_tool/
├── mod.rs              # Re-exports, shared types (~40 lines)
├── placement.rs        # VoxelDragState, handle_voxel_placement, handle_voxel_drag_placement (~180 lines)
├── removal.rs          # VoxelRemoveDragState, handle_voxel_removal, handle_voxel_drag_removal (~150 lines)
└── helpers.rs          # try_place_voxel, try_remove_voxel, shared utilities (~50 lines)
```

### Benefits
- Clear separation between place and remove operations
- Drag state resources co-located with their handlers
- Easier to add new voxel operations (e.g., paint, fill)

### Migration Steps
1. Create `voxel_tool/` directory
2. Extract placement systems to `placement.rs`
3. Extract removal systems to `removal.rs`
4. Extract shared helpers to `helpers.rs`
5. Update `tools/mod.rs` exports
6. Verify build passes

---

## Refactoring 3: dialogs.rs (368 lines → ~4 files)

### Status: 📋 Planned (Lower Priority)

### Problem

The dialogs module contains multiple independent dialog windows:
- Unsaved changes dialog
- New map dialog
- About dialog
- Shortcuts help dialog
- Error dialog
- File dialog handling (open file)

### Proposed Structure

```
src/editor/ui/dialogs/
├── mod.rs              # render_dialogs dispatcher, events (~60 lines)
├── file_dialogs.rs     # FileDialogReceiver, open/save dialogs, file handling (~120 lines)
├── confirmation.rs     # Unsaved changes, new map dialogs (~80 lines)
├── help.rs             # About dialog, shortcuts help (~80 lines)
└── error.rs            # Error dialog (~30 lines)
```

### Benefits
- Each dialog type in its own file
- Easier to add new dialogs
- File dialog logic separated from UI dialogs

### Migration Steps
1. Create `dialogs/` directory
2. Extract file dialog systems to `file_dialogs.rs`
3. Extract confirmation dialogs to `confirmation.rs`
4. Extract help dialogs to `help.rs`
5. Extract error dialog to `error.rs`
6. Update `ui/mod.rs` exports
7. Verify build passes

---

## Files Not Recommended for Splitting

The following files exceed 300 lines but have strong cohesion and single responsibility:

### occlusion.rs (341 lines)
- Single responsibility: voxel occlusion transparency system
- Contains shader uniforms, material extension, plugin, and update system
- Splitting would scatter related GPU/shader code

### gamepad.rs (339 lines)
- Single responsibility: gamepad input handling
- Contains settings, input state, connection handling, and input gathering
- Coherent module for controller support

### outliner.rs (330 lines)
- Single responsibility: outliner panel UI
- Contains selection UI, hierarchy display, context menu
- UI code that flows together

### renderer.rs (325 lines)
- Single responsibility: map rendering
- Contains render state, mesh generation, material handling
- Tightly coupled rendering logic

### viewport.rs (321 lines)
- Single responsibility: 3D viewport controls
- Contains camera controls, zoom, pan, rotation
- Coherent viewport interaction code

### main.rs (318 lines)
- Entry point with plugin setup
- Acceptable size for app initialization

### spawner/mod.rs (316 lines)
- Module root with constants, types, and main systems
- Acceptable for a module coordinator

---

## Priority Order

1. **meshing.rs** - High value due to size (649 lines) and clear separation points
2. **voxel_tool.rs** - Medium value, reduces duplicate pattern for drag state
3. **dialogs.rs** - Lower value, mostly UI code that reads sequentially

---

## Success Criteria

- [ ] All refactored modules compile without errors
- [ ] No changes to public API (imports may change)
- [ ] Tests continue to pass
- [ ] Map editor remains fully functional
- [ ] Game builds and runs correctly
- [ ] No performance regression

---

## Related Documents

- [File Size Refactoring - Phase 1](./file-size-refactoring.md) - Completed refactorings
- [AGENTS.md](../../AGENTS.md) - File size guidelines
