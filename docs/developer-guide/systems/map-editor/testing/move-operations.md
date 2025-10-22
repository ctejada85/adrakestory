# Move Operation Testing Guide

## Overview
This guide provides step-by-step instructions for testing the newly implemented move operation in the map editor.

## Prerequisites
- Map editor compiled successfully (`cargo build --bin map_editor`)
- Basic understanding of map editor controls

## Test Scenarios

### Test 1: Basic Single Voxel Move

**Steps:**
1. Launch the map editor: `cargo run --bin map_editor`
2. Place a voxel using the Voxel tool (V key)
3. Switch to Select tool (S key)
4. Click on the voxel to select it (should see yellow highlight)
5. Press **G** key to enter move mode
6. Verify UI shows "Transform Mode: Move" in properties panel
7. Press **Arrow Up** to move +Z
8. Press **Arrow Down** to move -Z
9. Press **Arrow Left** to move -X
10. Press **Arrow Right** to move +X
11. Hold **Shift + Arrow Up** to move +Y (up)
12. Hold **Shift + Arrow Down** to move -Y (down)
13. Observe ghost preview at new position (semi-transparent)
14. Press **Enter** to confirm move
15. Verify voxel moved to new position
16. Press **Ctrl+Z** to undo - voxel should return to original position
17. Press **Ctrl+Y** to redo - voxel should move back

**Expected Results:**
- ✓ Yellow selection highlight visible
- ✓ Ghost preview shows at offset position
- ✓ Arrow keys move preview in correct directions
- ✓ Shift modifier enables Y-axis movement
- ✓ Enter confirms and applies move
- ✓ Undo/redo works correctly

### Test 2: Multiple Voxel Move

**Steps:**
1. Place 3-4 voxels in different positions
2. Select all voxels (click each while holding Ctrl, or use box select if implemented)
3. Press **G** to enter move mode
4. Move using arrow keys
5. Verify all ghost previews move together
6. Press **Enter** to confirm
7. Verify all voxels moved by same offset

**Expected Results:**
- ✓ All selected voxels show ghost previews
- ✓ All previews move together
- ✓ All voxels move by same offset on confirm

### Test 3: Move Cancellation

**Steps:**
1. Select a voxel
2. Press **G** to enter move mode
3. Move using arrow keys
4. Press **Escape** to cancel
5. Verify voxel remains at original position
6. Verify no ghost preview visible
7. Verify UI shows normal select mode

**Expected Results:**
- ✓ Escape cancels operation
- ✓ Voxel stays at original position
- ✓ Transform mode cleared

### Test 4: Collision Detection

**Steps:**
1. Place two voxels next to each other at (0,0,0) and (1,0,0)
2. Select the voxel at (0,0,0)
3. Press **G** to enter move mode
4. Press **Arrow Right** to move +X (would collide with voxel at 1,0,0)
5. Observe ghost preview color (should be red/pink indicating collision)
6. Try to confirm with **Enter**
7. Verify move is blocked or warning shown

**Expected Results:**
- ✓ Ghost preview shows collision state (different color)
- ✓ Cannot move into occupied space
- ✓ Clear visual feedback about collision

### Test 5: UI Button Controls

**Steps:**
1. Select a voxel
2. Click "Move" button in properties panel
3. Verify move mode activated
4. Move using arrow keys
5. Click "Confirm" button in properties panel
6. Verify move applied

**Alternative:**
1. Select a voxel
2. Click "Move" button
3. Move using arrow keys
4. Click "Cancel" button
5. Verify move cancelled

**Expected Results:**
- ✓ "Move" button activates move mode
- ✓ "Confirm" button applies move
- ✓ "Cancel" button cancels move
- ✓ Buttons only visible when appropriate

### Test 6: Edge Cases

**Test 6a: Move with No Selection**
1. Ensure no voxels selected
2. Press **G** key
3. Verify nothing happens or appropriate message shown

**Test 6b: Move Out of Bounds**
1. Select a voxel near map edge
2. Press **G** to enter move mode
3. Try to move beyond map boundaries
4. Verify move is constrained or blocked

**Test 6c: Rapid Mode Switching**
1. Select a voxel
2. Press **G** to enter move mode
3. Press **Escape** to cancel
4. Immediately press **G** again
5. Verify mode switches correctly

**Expected Results:**
- ✓ Graceful handling of edge cases
- ✓ No crashes or undefined behavior
- ✓ Clear feedback for invalid operations

## Keyboard Shortcuts Reference

| Key | Action |
|-----|--------|
| **G** | Enter move mode (with selection) |
| **Arrow Keys** | Move in X/Z plane |
| **Shift + Arrow Up/Down** | Move in Y axis (up/down) |
| **Enter** | Confirm transformation |
| **Escape** | Cancel transformation |
| **Ctrl+Z** | Undo last action |
| **Ctrl+Y** | Redo last action |

## Known Limitations (Current Implementation)

1. **No Box Selection**: Must select voxels individually
2. **No Rotation**: Only move operation implemented (rotation coming in Step 2)
3. **Basic Collision**: Simple overlap detection, no complex collision handling
4. **No Snapping**: Moves by 1 unit increments only
5. **No Multi-Axis**: Can only move one axis at a time

## Troubleshooting

### Ghost Preview Not Visible
- Check that voxel is selected (yellow highlight)
- Verify move mode is active (check properties panel)
- Try moving with arrow keys to update preview

### Move Not Applying
- Ensure you pressed Enter to confirm
- Check for collision with existing voxels
- Verify selection is still active

### Undo/Redo Not Working
- Check that move was confirmed (not cancelled)
- Verify history system is enabled
- Try other undo/redo operations to test history

### UI Buttons Not Responding
- Ensure voxel is selected
- Check that properties panel is visible
- Verify correct tool is active (Select tool)

## Reporting Issues

When reporting issues, please include:
1. Steps to reproduce
2. Expected behavior
3. Actual behavior
4. Screenshots if applicable
5. Console output/error messages

## Next Steps

After testing move operation:
1. Document any bugs or issues found
2. Proceed to Step 2: Rotation implementation
3. Test rotation operation similarly
4. Integration testing with both move and rotate

## Success Criteria

Move operation is considered complete when:
- ✓ All test scenarios pass
- ✓ No crashes or errors
- ✓ Undo/redo works correctly
- ✓ UI provides clear feedback
- ✓ Collision detection works
- ✓ Performance is acceptable (no lag)