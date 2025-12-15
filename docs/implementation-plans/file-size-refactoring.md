# File Size Refactoring Plan

## Overview

This plan addresses files that exceed the file size guidelines (~200-400 lines target, ~300 lines as split threshold). The goal is to improve code navigability, maintainability, and review efficiency by splitting large files into focused modules.

**Status**: 📋 Planned  
**Priority**: Medium  
**Last Updated**: 2025-12-15

---

## Files Requiring Refactoring

Files sorted by line count, with those exceeding ~400 lines flagged for splitting:

| File | Lines | Priority | Status |
|------|-------|----------|--------|
| `src/systems/game/map/spawner/` | 1384 → 4 files | 🔴 High | ✅ Done |
| `src/editor/ui/properties/` | 778 → 5 files | 🔴 High | ✅ Done |
| `src/editor/tools/input/` | 768 → 4 files | 🔴 High | ✅ Done |
| `src/editor/ui/toolbar/` | 701 → 5 files | 🔴 High | ✅ Done |
| `src/editor/tools/selection_tool/` | 559 → 4 files | 🟡 Medium | ✅ Done |
| `src/bin/map_editor/` | 553 → 6 files | 🟡 Medium | ✅ Done |
| `src/systems/game/map/geometry/` | 546 → 5 files | 🟡 Medium | ✅ Done |
| `src/editor/cursor/` | 505 → 5 files | 🟡 Medium | ✅ Done |
| `src/systems/game/hot_reload.rs` | 494 | 🟡 Medium | Pending |
| `src/systems/game/map/format.rs` | 471 | 🟡 Medium | Pending |
| `src/editor/grid.rs` | 447 | 🟡 Medium | Pending |
| `src/editor/tools/voxel_tool.rs` | 416 | 🟢 Low | Pending |

---

## Refactoring 1: spawner.rs (1384 lines → ~4 files) ✅ COMPLETED

### Status: ✅ IMPLEMENTED (2025-12-15)

### Problem

The spawner module handles too many responsibilities: chunk creation, mesh generation, LOD system, entity spawning, and sub-voxel processing.

### Proposed Split

```
src/systems/game/map/
├── spawner/
│   ├── mod.rs          (~100 lines) - Public API, re-exports
│   ├── chunks.rs       (~350 lines) - Chunk creation, ChunkLOD
│   ├── meshing.rs      (~400 lines) - Greedy meshing algorithm
│   ├── entities.rs     (~250 lines) - Entity spawning logic
│   └── subvoxels.rs    (~300 lines) - Sub-voxel processing, SpatialGrid insertion
└── spawner.rs          (deleted, replaced by spawner/)
```

### Key Extractions

| New File | Content to Extract |
|----------|-------------------|
| `chunks.rs` | `spawn_voxels_chunked`, `ChunkLOD` struct, chunk iteration logic |
| `meshing.rs` | Greedy meshing functions, face merging, quad generation |
| `entities.rs` | `spawn_entity`, `spawn_player`, entity type handlers |
| `subvoxels.rs` | Sub-voxel bounds calculation, SpatialGrid insertion, pattern application |

### Migration Steps

1. Create `src/systems/game/map/spawner/` directory
2. Extract `meshing.rs` first (most self-contained)
3. Extract `chunks.rs` (depends on meshing)
4. Extract `entities.rs` and `subvoxels.rs`
5. Create `mod.rs` with public re-exports
6. Update imports in dependent files
7. Delete original `spawner.rs`
8. Run `cargo test` and `cargo clippy`

---

## Refactoring 2: properties.rs (778 lines → ~5 files) ✅ COMPLETED

### Status: ✅ IMPLEMENTED (2025-12-15)

### Problem

The properties panel handles UI for multiple entity types and property categories in one file.

### Proposed Split

```
src/editor/ui/
├── properties/
│   ├── mod.rs              (~100 lines) - Main panel, re-exports
│   ├── voxel_props.rs      (~250 lines) - Voxel/sub-voxel property editing
│   ├── entity_props.rs     (~250 lines) - Entity property editing
│   └── transform_props.rs  (~180 lines) - Position, rotation, scale widgets
└── properties.rs           (deleted, replaced by properties/)
```

### Migration Steps

1. Create `src/editor/ui/properties/` directory
2. Extract common transform editing into `transform_props.rs`
3. Extract voxel-specific UI into `voxel_props.rs`
4. Extract entity-specific UI into `entity_props.rs`
5. Create `mod.rs` with unified `PropertiesPanel` component
6. Update `src/editor/ui/mod.rs` imports
7. Run `cargo build` to verify

---

## Refactoring 3: input.rs (768 lines → ~4 files) ✅ COMPLETED

### Status: ✅ IMPLEMENTED (2025-12-15)

### Problem

Tool input handling mixes mouse input, keyboard shortcuts, and tool-specific logic.

### Proposed Split

```
src/editor/tools/
├── input/
│   ├── mod.rs          (~100 lines) - Input system coordination
│   ├── mouse.rs        (~300 lines) - Mouse input, raycasting, drag handling
│   ├── keyboard.rs     (~200 lines) - Keyboard shortcuts, modifiers
│   └── actions.rs      (~180 lines) - Action dispatch, tool mode switching
└── input.rs            (deleted, replaced by input/)
```

### Migration Steps

1. Create `src/editor/tools/input/` directory
2. Extract mouse handling (click, drag, raycast) into `mouse.rs`
3. Extract keyboard handling into `keyboard.rs`
4. Extract action dispatch logic into `actions.rs`
5. Create `mod.rs` coordinating input systems
6. Update `src/editor/tools/mod.rs` imports
7. Run tests

---

## Refactoring 4: toolbar.rs (701 lines → ~5 files) ✅ COMPLETED

### Status: ✅ IMPLEMENTED (2025-12-15)

### Problem

Toolbar contains multiple tool groups, menus, and icon rendering logic.

### Proposed Split

```
src/editor/ui/
├── toolbar/
│   ├── mod.rs          (~100 lines) - Main toolbar layout
│   ├── tool_buttons.rs (~250 lines) - Tool selection buttons, icons
│   ├── menus.rs        (~200 lines) - File, Edit, View menus
│   └── status_bar.rs   (~150 lines) - Status bar, tooltips
└── toolbar.rs          (deleted, replaced by toolbar/)
```

---

## Refactoring 5: selection_tool.rs (559 lines → ~4 files) ✅ COMPLETED

### Status: ✅ IMPLEMENTED (2025-12-15)

### Problem

Selection tool handles both single and multi-selection, plus selection visualization.

### Proposed Split

```
src/editor/tools/
├── selection_tool/
│   ├── mod.rs          (~150 lines) - Tool interface, mode switching
│   ├── operations.rs   (~250 lines) - Selection operations (add, remove, toggle)
│   └── rendering.rs    (~160 lines) - Selection highlight rendering
└── selection_tool.rs   (deleted)
```

---

## Refactoring 6: map_editor.rs (553 lines → ~6 files) ✅ COMPLETED

### Status: ✅ IMPLEMENTED (2025-12-15)

### Problem

Editor entry point mixes app configuration, plugin setup, and initialization.

### Proposed Split

```
src/bin/
├── map_editor.rs       (~150 lines) - Entry point, minimal main()
src/editor/
├── app.rs              (~250 lines) - App builder, plugin registration
└── init.rs             (~150 lines) - Initialization systems, startup
```

---

## Refactoring 7: geometry.rs (546 lines → ~5 files) ✅ COMPLETED

### Status: ✅ IMPLEMENTED (2025-12-15)

### Problem

Geometry file contains both pattern definitions and rotation logic.

### Proposed Split

```
src/systems/game/map/
├── geometry/
│   ├── mod.rs          (~100 lines) - Re-exports, SubVoxelGeometry struct
│   ├── patterns.rs     (~300 lines) - Pattern definitions (Full, Staircase, etc.)
│   └── rotation.rs     (~150 lines) - Rotation math, geometry_with_rotation()
└── geometry.rs         (deleted)
```

---

## Refactoring 8: cursor.rs (505 lines → ~5 files) ✅ COMPLETED

### Status: ✅ IMPLEMENTED (2025-12-15)

### Problem

Cursor module handles both cursor state and 3D raycasting.

### Proposed Split

```
src/editor/
├── cursor/
│   ├── mod.rs          (~150 lines) - Cursor state, position tracking
│   ├── raycast.rs      (~200 lines) - 3D raycasting, voxel intersection
│   └── preview.rs      (~160 lines) - Preview rendering, ghost voxels
└── cursor.rs           (deleted)
```

---

## Refactoring 9: hot_reload.rs (494 lines → ~2 files)

### Problem

Hot reload handles file watching, debouncing, and map reloading.

### Proposed Split

```
src/systems/game/
├── hot_reload/
│   ├── mod.rs          (~150 lines) - Plugin, public API
│   ├── watcher.rs      (~200 lines) - File system watching, debounce
│   └── reload.rs       (~150 lines) - Map reload logic, state preservation
└── hot_reload.rs       (deleted)
```

---

## Refactoring 10: format.rs (471 lines → ~2 files)

### Problem

Format file contains both data structures and serialization logic.

### Proposed Split

```
src/systems/game/map/
├── format/
│   ├── mod.rs          (~150 lines) - MapData, VoxelData structs
│   ├── types.rs        (~200 lines) - EntityType, SubVoxelPattern enums
│   └── serde.rs        (~120 lines) - Custom serialization/deserialization
└── format.rs           (deleted)
```

---

## Refactoring 11: grid.rs (447 lines → ~2 files)

### Problem

Grid handles both 3D grid rendering and snapping logic.

### Proposed Split

```
src/editor/
├── grid/
│   ├── mod.rs          (~150 lines) - Grid configuration, public API
│   ├── rendering.rs    (~200 lines) - 3D grid mesh generation
│   └── snapping.rs     (~100 lines) - Snap-to-grid calculations
└── grid.rs             (deleted)
```

---

## Implementation Order

Prioritized by impact and risk:

| Phase | Files | Rationale |
|-------|-------|-----------|
| 1 | `spawner.rs` | Largest file, core game code, high impact |
| 2 | `geometry.rs`, `format.rs` | Dependencies of spawner, should be done together |
| 3 | `properties.rs`, `toolbar.rs` | Editor UI, can be done in parallel |
| 4 | `input.rs`, `selection_tool.rs`, `voxel_tool.rs` | Editor tools, related changes |
| 5 | `cursor.rs`, `grid.rs` | Editor utilities |
| 6 | `hot_reload.rs`, `map_editor.rs` | Lower priority, less frequently modified |

---

## Testing Strategy

### Per-Refactoring Tests

1. **Before refactoring**: Run `cargo test` and `cargo clippy`, note baseline
2. **After each file split**: 
   - `cargo build` - verify compilation
   - `cargo test` - verify behavior unchanged
   - `cargo clippy` - check for new warnings
3. **Integration**: Run game and editor manually to verify no regressions

### Regression Checklist

- [ ] Game loads maps correctly
- [ ] Sub-voxel collision works
- [ ] LOD system functions
- [ ] Editor tools work (voxel, selection, entity)
- [ ] Undo/redo works
- [ ] Hot reload functions
- [ ] Save/load maps works

---

## Success Criteria

| Metric | Before | Target |
|--------|--------|--------|
| Files > 500 lines | 8 | 0 |
| Files > 400 lines | 12 | 0 |
| Average file size | ~200 lines | ~150 lines |
| Max file size | 1384 lines | < 400 lines |

---

## Rollback Plan

Each refactoring should be a single commit. If issues arise:

1. `git revert <commit>` to undo the specific refactoring
2. Document the issue in this plan
3. Re-evaluate the split strategy before retrying

---

## References

- [AGENTS.md - File Size Guidelines](../../AGENTS.md)
- [Architecture Documentation](../developer-guide/architecture.md)
- [Sub-voxel System](../developer-guide/systems/)
