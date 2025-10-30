# Map Editor Archive

This directory contains historical documentation that has been superseded by consolidated or updated versions. These files are preserved for reference and historical context.

## Archived Documents

### [input-refactoring-summary.md](input-refactoring-summary.md) ⭐
**Archived**: 2025-10-22
**Reason**: Implementation completed, refactoring document moved to archive for historical reference

**Original Purpose**: Comprehensive documentation of the input system refactoring that unified keyboard input handling.

**Historical Context**: In October 2025, the map editor had 15 scattered input handling systems across multiple files, making the codebase difficult to maintain and extend. A major refactoring consolidated these into 2 unified systems:
- Single keyboard input handler (`handle_keyboard_input`)
- Dedicated transformation operations handler (`handle_transformation_operations`)

This resulted in a 72% reduction in input systems (from 15 to 2) and ~500 lines of code removed.

**Key Achievements**:
- Event-driven architecture with `EditorInputEvent` enum
- Context-aware key mapping (tool-specific shortcuts)
- Clear separation of concerns (input reading vs. action execution)
- Improved maintainability and extensibility

**Current Documentation**:
- Architecture: [architecture.md](../architecture.md)
- Input handling guide: [input-handling.md](../input-handling.md)
- Implementation status: [implementation-status.md](../implementation-status.md)

### [input-refactoring-plan.md](input-refactoring-plan.md)
**Archived**: 2025-10-22
**Reason**: Implementation completed, planning document no longer needed for active development

**Original Purpose**: Detailed implementation plan for refactoring the input system into a unified architecture.

**Historical Context**: This 638-line document outlined the complete refactoring strategy, including:
- Current state analysis (15 scattered systems)
- Proposed architecture (event-driven design)
- Step-by-step implementation plan
- Technical considerations and best practices
- Success criteria and validation steps

**Status**: Implementation completed successfully. All planned features implemented and tested.

**Current Documentation**:
- Summary of results: [input-refactoring-summary.md](input-refactoring-summary.md)
- Architecture: [architecture.md](../architecture.md)
- Input handling guide: [input-handling.md](../input-handling.md)

### [keyboard-input-fix.md](keyboard-input-fix.md)
**Archived**: 2025-10-22  
**Reason**: Consolidated into [input-handling.md](../input-handling.md)

**Original Purpose**: Documented the fix for keyboard shortcuts not working when voxels were selected.

**Historical Context**: In January 2025, keyboard shortcuts (G key, arrow keys, Delete, etc.) were not working in the map editor. The root cause was egui consuming keyboard events before they could reach game systems. The solution was to add `wants_keyboard_input()` checks before processing keyboard input.

**Current Location**: This information is now part of the comprehensive [Input Handling Guide](../input-handling.md) in the "Historical Context" section.

### [ui-input-propagation-fix.md](ui-input-propagation-fix.md)
**Archived**: 2025-10-22  
**Reason**: Consolidated into [input-handling.md](../input-handling.md)

**Original Purpose**: Documented the fix for UI clicks triggering canvas operations.

**Historical Context**: When clicking on UI controls (menu items, toolbar buttons, properties panel), the canvas was also processing these mouse events, causing unintended actions like placing voxels when clicking toolbar buttons. The solution was to add `wants_pointer_input()` checks to all mouse input handlers.

**Current Location**: This information is now part of the comprehensive [Input Handling Guide](../input-handling.md) in the "Historical Context" section.

### [move-rotate-plan.md](move-rotate-plan.md)
**Archived**: 2025-10-22  
**Reason**: Implementation completed, planning document no longer needed for active development

**Original Purpose**: Detailed implementation plan for adding move and rotate operations to selected voxels.

**Historical Context**: This 539-line document outlined the complete implementation strategy for Phase 2 of the Selection Tool, including:
- Architecture design (components, events, resources)
- Step-by-step implementation plan (7 steps over 3-4 days)
- Technical considerations (rotation mathematics, collision detection)
- Keyboard shortcuts and UI mockups
- Success criteria and testing plan

**Status**: Implementation completed. Move and rotate operations are now fully functional.

**Current Documentation**:
- Implementation status: [implementation-status.md](../implementation-status.md)
- Testing guides: [testing/move-operations.md](../testing/move-operations.md) and [testing/rotation-operations.md](../testing/rotation-operations.md)
- Architecture: [architecture.md](../architecture.md)

### [lighting-performance-optimization.md](lighting-performance-optimization.md)
**Archived**: 2025-10-30
**Reason**: Implementation completed, optimization document moved to archive for historical reference

**Original Purpose**: Comprehensive documentation of the lighting system performance optimization that eliminated unnecessary per-frame updates.

**Historical Context**: In October 2025, the map editor was updating ambient and directional lighting every frame due to Bevy's change detection being triggered by UI interactions. The lighting system checked `editor_state.is_changed()`, but UI systems needed mutable access to `EditorState` for checkboxes, text fields, and tool selections, causing the lighting to update continuously even when the map data hadn't changed.

**Key Achievements**:
- 99.9% reduction in lighting updates (from every frame to only on actual map changes)
- Event-driven architecture with `MapDataChangedEvent`
- Separated `CursorState` resource to prevent cursor movement from polluting change detection
- Improved performance and reduced log spam

**Solution Implemented**: Option 3 - Cursor State Separation + Event-Based Architecture
- Created dedicated `CursorState` resource for high-frequency cursor updates
- Implemented `MapDataChangedEvent` for explicit map change signaling
- Updated lighting system to respond to events instead of change detection
- Maintained clean separation of concerns

**Current Documentation**:
- Architecture: [architecture.md](../architecture.md) - Updated with CursorState and change detection patterns
- Implementation status: [implementation-status.md](../implementation-status.md)

## Why Archive Instead of Delete?

These documents are archived rather than deleted for several reasons:

1. **Historical Reference**: Provides context for why certain design decisions were made
2. **Learning Resource**: Helps new contributors understand the evolution of the codebase
3. **Problem-Solving Patterns**: Documents successful approaches to common issues
4. **Audit Trail**: Maintains a record of significant changes and fixes
5. **Rollback Information**: Preserves details that might be needed if reverting changes

## Using Archived Documentation

### When to Reference
- Understanding why a particular pattern was chosen
- Investigating similar issues in other parts of the codebase
- Learning about the project's development history
- Researching resolved issues before reporting new ones

### When NOT to Reference
- For current implementation details (use main documentation)
- For active development work (use current design docs)
- For user-facing information (use user guides)

## Document Lifecycle

```
Active Documentation
        ↓
    (Superseded by consolidation or completion)
        ↓
Archived Documentation
        ↓
    (After 1+ year with no references)
        ↓
    (Optional) Permanent Deletion
```

## Related Documentation

### Current Documentation
- [Input Handling Guide](../input-handling.md) - Consolidated input handling patterns
- [Implementation Status](../implementation-status.md) - Current development status
- [Testing Documentation](../testing/) - Active testing guides

### Other Archives
- [Main Archive](../../../../archive/) - Project-wide archived documentation

---

**Archive Created**: 2025-10-22
**Last Updated**: 2025-10-30 (Added lighting performance optimization)
**Maintainer**: Development Team