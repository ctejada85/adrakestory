# Map Editor Testing Documentation

This directory contains comprehensive testing guides for the map editor's various features and operations.

## Testing Guides

### [Move Operations Testing](move-operations.md)
Complete testing guide for the move operation feature, including:
- Basic single voxel movement
- Multiple voxel movement
- Move cancellation
- Collision detection
- UI button controls
- Edge cases and troubleshooting

**Key Features Tested:**
- G key activation
- Arrow key movement (X/Z plane)
- Shift + Arrow keys (Y axis)
- Ghost preview rendering
- Collision detection
- Undo/redo integration

### [Rotation Operations Testing](rotation-operations.md)
Comprehensive testing procedures for subvoxel pattern rotation, including:
- Platform pattern rotations (all axes)
- Staircase pattern rotations (all axes)
- Multiple rotation sequences
- Undo/redo with pattern rotation
- Visual verification

**Key Features Tested:**
- R key activation
- X/Y/Z axis selection
- 90° rotation increments
- Pattern transformation logic
- Ghost preview rendering
- Undo/redo integration

## Testing Workflow

### 1. Prerequisites
- Map editor compiled successfully: `cargo build --bin map_editor`
- Basic understanding of map editor controls
- Familiarity with keyboard shortcuts

### 2. Test Execution Order
1. **Move Operations** - Test basic transformation capabilities
2. **Rotation Operations** - Test pattern-specific transformations
3. **Integration Testing** - Test combined move + rotate workflows

### 3. Reporting Issues
When reporting issues, please include:
- Steps to reproduce
- Expected behavior
- Actual behavior
- Screenshots if applicable
- Console output/error messages

## Success Criteria

All tests are considered passing when:
- ✓ All test scenarios complete without errors
- ✓ No crashes or undefined behavior
- ✓ Undo/redo works correctly for all operations
- ✓ UI provides clear feedback
- ✓ Collision detection works as expected
- ✓ Performance is acceptable (no lag)

## Quick Reference

### Move Operation Shortcuts
| Key | Action |
|-----|--------|
| `G` | Enter move mode |
| `Arrow Keys` | Move in X/Z plane |
| `Shift + Arrow Up/Down` | Move in Y axis |
| `Enter` | Confirm transformation |
| `Escape` | Cancel transformation |

### Rotation Operation Shortcuts
| Key | Action |
|-----|--------|
| `R` | Enter rotation mode |
| `X` | Select X-axis |
| `Y` | Select Y-axis |
| `Z` | Select Z-axis |
| `→` | Rotate 90° clockwise |
| `←` | Rotate 90° counter-clockwise |
| `Enter` | Confirm rotation |
| `Escape` | Cancel rotation |

## Related Documentation

- [Map Editor Architecture](../architecture.md)
- [Input Handling Guide](../input-handling.md)
- [Implementation Status](../implementation-status.md)
- [User Controls Reference](../../../../user-guide/map-editor/controls.md)

## Contributing

When adding new features to the map editor:
1. Create corresponding test documentation
2. Follow the existing test guide format
3. Include success criteria
4. Document all keyboard shortcuts
5. Provide troubleshooting section

---

**Last Updated**: 2025-10-22  
**Status**: Active Testing Documentation