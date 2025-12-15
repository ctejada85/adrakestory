# Map Editor Documentation

Welcome to the comprehensive documentation for the A Drake's Story Map Editor. This directory contains all technical documentation for developers working on or with the map editor.

## Quick Navigation

### 📋 Planning & Status
- **[Implementation Status](implementation-status.md)** - Current progress and completed features
- **[Roadmap](roadmap.md)** - Future features and development plans
- **[Design Document](design.md)** - Feature specifications and requirements

### 🏗️ Architecture & Design
- **[Architecture Overview](architecture.md)** - System design, data flow, and component interactions
- **[Input Handling Guide](input-handling.md)** - Unified input architecture and best practices
- **[Face-Aware Placement System](face-aware-placement.md)** ⭐ - Voxel placement on face detection

### 🧪 Testing
- **[Testing Documentation](testing/)** - Complete testing guides
  - [Move Operations Testing](testing/move-operations.md)
  - [Rotation Operations Testing](testing/rotation-operations.md)

### 📚 Archive
- **[Archived Documentation](archive/)** - Historical documents and resolved issues
  - [Lighting Performance Optimization](archive/lighting-performance-optimization.md) ⭐ - Change detection fix
  - [Input Refactoring Summary](archive/input-refactoring-summary.md) ⭐ - Complete refactoring details
  - [Input Refactoring Plan](archive/input-refactoring-plan.md) - Original design document
  - [Keyboard Input Fix](archive/keyboard-input-fix.md)
  - [UI Input Propagation Fix](archive/ui-input-propagation-fix.md)
  - [Move/Rotate Implementation Plan](archive/move-rotate-plan.md)

## Documentation Structure

```
map-editor/
├── README.md                          # This file - navigation hub
├── architecture.md                    # System architecture and design patterns
├── design.md                          # Feature specifications
├── implementation-status.md           # Current development status
├── roadmap.md                         # Future development plans
├── input-handling.md                  # Unified input architecture guide
├── rotation-system.md                 # Rotation system documentation
├── face-aware-placement.md            # Face-aware voxel placement system ⭐ NEW
├── testing/                           # Testing documentation
│   ├── README.md                      # Testing overview
│   ├── move-operations.md             # Move operation testing guide
│   └── rotation-operations.md         # Rotation operation testing guide
└── archive/                           # Historical documentation
    ├── README.md                      # Archive index
    ├── lighting-performance-optimization.md  # Completed: Lighting performance fix ⭐
    ├── input-refactoring-summary.md   # Completed: Input system refactoring ⭐
    ├── input-refactoring-plan.md      # Completed: Input system design
    ├── keyboard-input-fix.md          # Resolved: Keyboard input issues
    ├── ui-input-propagation-fix.md    # Resolved: UI click propagation
    └── move-rotate-plan.md            # Completed: Move/rotate implementation
```

## Getting Started

### For New Contributors
1. Start with [Architecture Overview](architecture.md) to understand the system design
2. Read [Design Document](design.md) to understand feature specifications
3. Review [Input Handling Guide](input-handling.md) for implementation patterns
4. Check [Implementation Status](implementation-status.md) to see what's complete

### For Testers
1. Review [Testing Documentation](testing/) for comprehensive test guides
2. Follow test procedures in [Move Operations](testing/move-operations.md)
3. Verify rotation features with [Rotation Operations](testing/rotation-operations.md)

### For Users
User-facing documentation is located in:
- [User Guide - Getting Started](../../../user-guide/map-editor/getting-started.md)
- [User Guide - Controls Reference](../../../user-guide/map-editor/controls.md)
- [User Guide - Troubleshooting](../../../user-guide/map-editor/troubleshooting.md)

## Key Features

### ✅ Implemented
- **Core Infrastructure**: Window, UI layout, state management
- **Blank Canvas Start** ⭐: Editor starts with empty map for new creations
  - Empty map with 0 voxels and minimal dimensions (1×1×1)
  - Default lighting configuration preserved
  - Auto-expand dimensions on save
  - `default_map()` retained for testing and fallback
- **3D Viewport**: Real-time rendering with camera controls
- **WYSIWYG Lighting** ⭐: Congruent illumination between editor and game
  - Map-defined directional and ambient lighting
  - Event-driven lighting updates (99.9% performance improvement)
  - Separate cursor state to prevent change detection pollution
  - Identical rendering in editor and gameplay
- **Voxel Editing**: Place, remove, and configure voxels
- **Face-Aware Placement** ⭐: Intelligent voxel placement on target faces
  - Enhanced ray-box intersection with face detection
  - Adjacent placement on voxel faces (±X, ±Y, ±Z)
  - Grid snapping for consistent placement
  - Tool-aware cursor rendering
  - Keyboard mode support
- **Entity Management**: Place and configure entities
- **Selection Tool**: Select, move, and rotate voxels
- **File Operations** ⭐: Complete save/load functionality
  - Open and load .ron map files
  - Save and Save As with RON serialization
  - Auto-expand map dimensions to fit voxels
  - Non-blocking file dialogs
  - Unsaved changes protection
- **History System**: Undo/redo for all operations
- **Unified Input System** ⭐: Event-driven architecture with 72% system reduction
  - Single keyboard input handler
  - Context-aware key mapping
  - Clear separation of concerns
  - Improved maintainability
- **Keyboard Cursor Navigation** ⭐: Vim-like keyboard editing mode
  - I key to enter keyboard mode, Escape to exit
  - Arrow keys for X/Z movement, Space/C for Y-axis
  - Enter key for selection (Select tool)
  - Shift modifier for fast movement (5 units)
  - Smart Escape behavior (clears selections before exiting mode)
  - Visual mode indicator in status bar
  - Mouse override prevention in keyboard mode
- **Tool Switching Hotkeys** ⭐: Quick tool switching with number keys
  - Press 1 for VoxelPlace tool
  - Press 2 for Select tool
  - Preserves voxel type and pattern settings
  - Works from anywhere (except text input)

### 🚧 In Progress
- Additional keyboard shortcuts (Ctrl+N, Ctrl+O, etc.)
- Enhanced validation display
- Rotation operations

### 📋 Planned
See [Roadmap](roadmap.md) for detailed future plans.

## Development Guidelines

### Adding New Features
1. Update [Design Document](design.md) with specifications
2. Implement following [Architecture](architecture.md) patterns
3. Follow [Input Handling](input-handling.md) best practices
4. Create test documentation in [testing/](testing/)
5. Update [Implementation Status](implementation-status.md)

### Code Organization
```
src/editor/
├── mod.rs              # Module exports
├── state.rs            # Editor state management
├── history.rs          # Undo/redo system
├── camera.rs           # Camera controls
├── grid/               # Grid visualization (refactored to module)
│   ├── mod.rs
│   ├── bounds.rs
│   ├── mesh.rs
│   ├── systems.rs
│   └── cursor_indicator.rs
├── cursor/             # 3D cursor ray casting + keyboard navigation (refactored to module)
│   ├── mod.rs
│   ├── state.rs
│   ├── raycasting.rs
│   ├── keyboard.rs
│   └── systems.rs
├── renderer.rs         # Map rendering
├── tools/              # Editing tools
│   ├── input/          # Unified input handling (refactored to module)
│   │   ├── mod.rs
│   │   ├── events.rs
│   │   ├── handlers.rs
│   │   └── operations.rs
│   ├── voxel_tool.rs   # Voxel placement/removal
│   ├── entity_tool.rs  # Entity placement
│   └── selection_tool/ # Selection and transformation (refactored to module)
│       ├── mod.rs
│       ├── selection.rs
│       ├── move_operation.rs
│       ├── rotation.rs
│       └── preview.rs
└── ui/                 # UI components
    ├── toolbar/        # Top toolbar (refactored to module)
    │   ├── mod.rs
    │   ├── file_menu.rs
    │   ├── edit_menu.rs
    │   └── view_menu.rs
    ├── properties/     # Properties panel (refactored to module)
    │   ├── mod.rs
    │   ├── voxel_panel.rs
    │   ├── entity_panel.rs
    │   └── selection_panel.rs
    ├── viewport.rs     # Viewport controls
    └── dialogs.rs      # File dialogs
```

### Best Practices
1. **Input Handling**: Always check `wants_pointer_input()` and `wants_keyboard_input()`
2. **History Integration**: All editing operations should support undo/redo
3. **State Management**: Use resources for global state, components for entity data
4. **Documentation**: Update docs when adding features
5. **Testing**: Create test documentation for new features

## Common Tasks

### Implementing a New Tool
1. Create tool module in `src/editor/tools/`
2. Add tool variant to `EditorTool` enum in `state.rs`
3. Implement input handlers with proper UI checks
4. Add UI controls in `properties.rs`
5. Integrate with history system
6. Document in design.md and create test guide

### Adding a Keyboard Shortcut
1. Add key mapping to `handle_keyboard_input()` in `input/handlers.rs`
2. Add corresponding `EditorInputEvent` variant in `input/events.rs` if needed
3. Handle event in `handle_transformation_operations()` in `input/operations.rs` or create new handler
4. Update [Controls Reference](../../../user-guide/map-editor/controls.md)
5. Add to keyboard shortcuts help dialog

### Fixing Input Issues
1. Review [Input Handling Guide](input-handling.md)
2. Ensure `EguiContexts` parameter is present
3. Add appropriate `wants_*_input()` checks
4. Test with UI interactions

## Troubleshooting

### Common Issues
- **Input not working**: Check [Input Handling Guide](input-handling.md)
- **UI clicks trigger canvas**: Add `wants_pointer_input()` check
- **Keyboard shortcuts not working**: Add `wants_keyboard_input()` check
- **Undo/redo not working**: Ensure history integration

### Getting Help
- Check [Archived Documentation](archive/) for resolved issues
- Review [Testing Documentation](testing/) for known limitations
- Consult [Architecture](architecture.md) for system design questions

## Related Documentation

### Developer Documentation
- [Main Architecture](../../architecture.md) - Overall game architecture
- [Map Loader System](../map-loader.md) - Map loading internals
- [Contributing Guidelines](../../contributing.md) - How to contribute
- [Input Refactoring Summary](archive/input-refactoring-summary.md) ⭐ - Input system details (archived)

### User Documentation
- [Map Editor - Getting Started](../../../user-guide/map-editor/getting-started.md)
- [Map Editor - Controls](../../../user-guide/map-editor/controls.md)
- [Map Editor - Troubleshooting](../../../user-guide/map-editor/troubleshooting.md)

### API Documentation
- [Map Format Specification](../../../api/map-format-spec.md)

## Version History

| Version | Date | Description |
|---------|------|-------------|
| 2.8.0 | 2025-11-03 | Blank canvas start - editor opens with empty map ⭐ |
| 2.7.0 | 2025-10-30 | Face-aware voxel placement system ⭐ |
| 2.6.0 | 2025-10-30 | Lighting performance optimization - event-based updates (99.9% improvement) ⭐ |
| 2.5.0 | 2025-10-23 | WYSIWYG lighting - congruent illumination between editor and game ⭐ |
| 2.4.0 | 2025-10-23 | Tool switching hotkeys (1 for Place, 2 for Select) ⭐ |
| 2.3.0 | 2025-10-23 | Keyboard cursor navigation and vim-like editing mode ⭐ |
| 2.2.0 | 2025-10-23 | Save functionality complete with auto-expand feature ⭐ |
| 2.1.0 | 2025-10-22 | Input system refactoring complete (72% system reduction) |
| 2.0.0 | 2025-10-22 | Documentation reorganization and consolidation |
| 1.1.0 | 2025-01-15 | File operations and rendering complete |
| 1.0.0 | 2025-01-10 | Initial map editor implementation |

---

**Last Updated**: 2025-11-03
**Maintainer**: Development Team
**Status**: Active Development - Blank Canvas Start Implemented ⭐