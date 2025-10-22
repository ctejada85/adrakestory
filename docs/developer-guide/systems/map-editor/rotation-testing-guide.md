# Rotation Testing Guide

## Overview
This guide provides comprehensive testing procedures for the subvoxel pattern rotation feature in the map editor.

## Fixed Issues
- **X and Z axis rotations**: Previously, staircase patterns were not transforming correctly when rotated around X or Z axes. This has been fixed by implementing proper transformation logic that flips staircase directions at 180° rotations.

## Pattern Rotation Behavior

### Platform Patterns

#### PlatformXZ (Horizontal - 8×1×8)
- **Y-axis rotation**: Stays as PlatformXZ (horizontal orientation preserved)
- **X-axis rotation**: 
  - 90°/270°: Transforms to PlatformYZ (vertical wall facing X)
  - 180°: Stays as PlatformXZ
- **Z-axis rotation**:
  - 90°/270°: Transforms to PlatformXY (vertical wall facing Z)
  - 180°: Stays as PlatformXZ

#### PlatformXY (Vertical wall facing Z - 8×8×1)
- **Y-axis rotation**: Stays as PlatformXY (vertical orientation preserved)
- **X-axis rotation**:
  - 90°/270°: Transforms to PlatformXZ (horizontal)
  - 180°: Stays as PlatformXY
- **Z-axis rotation**:
  - 90°/270°: Transforms to PlatformYZ (vertical wall facing X)
  - 180°: Stays as PlatformXY

#### PlatformYZ (Vertical wall facing X - 1×8×8)
- **Y-axis rotation**: Stays as PlatformYZ (vertical orientation preserved)
- **X-axis rotation**:
  - 90°/270°: Transforms to PlatformXZ (horizontal)
  - 180°: Stays as PlatformYZ
- **Z-axis rotation**:
  - 90°/270°: Transforms to PlatformXY (vertical wall facing Z)
  - 180°: Stays as PlatformYZ

### Staircase Patterns

#### StaircaseX (Ascending in +X direction)
- **Y-axis rotation**:
  - 90°: Transforms to StaircaseZ
  - 180°: Transforms to StaircaseNegX
  - 270°: Transforms to StaircaseNegZ
- **X-axis rotation**: Stays as StaircaseX (parallel to rotation axis)
- **Z-axis rotation**:
  - 180°: Transforms to StaircaseNegX (flips direction)
  - 90°/270°: Stays as StaircaseX

#### StaircaseNegX (Ascending in -X direction)
- **Y-axis rotation**:
  - 90°: Transforms to StaircaseNegZ
  - 180°: Transforms to StaircaseX
  - 270°: Transforms to StaircaseZ
- **X-axis rotation**: Stays as StaircaseNegX (parallel to rotation axis)
- **Z-axis rotation**:
  - 180°: Transforms to StaircaseX (flips direction)
  - 90°/270°: Stays as StaircaseNegX

#### StaircaseZ (Ascending in +Z direction)
- **Y-axis rotation**:
  - 90°: Transforms to StaircaseNegX
  - 180°: Transforms to StaircaseNegZ
  - 270°: Transforms to StaircaseX
- **X-axis rotation**:
  - 180°: Transforms to StaircaseNegZ (flips direction)
  - 90°/270°: Stays as StaircaseZ
- **Z-axis rotation**: Stays as StaircaseZ (parallel to rotation axis)

#### StaircaseNegZ (Ascending in -Z direction)
- **Y-axis rotation**:
  - 90°: Transforms to StaircaseX
  - 180°: Transforms to StaircaseZ
  - 270°: Transforms to StaircaseNegX
- **X-axis rotation**:
  - 180°: Transforms to StaircaseZ (flips direction)
  - 90°/270°: Stays as StaircaseNegZ
- **Z-axis rotation**: Stays as StaircaseNegZ (parallel to rotation axis)

### Other Patterns
- **Full**: No transformation (always stays Full)
- **Pillar**: No transformation (always stays Pillar)

## Testing Procedure

### Setup
1. Launch the map editor: `cargo run --bin map_editor`
2. Create test voxels with different subvoxel patterns
3. Use the Properties panel to set specific patterns

### Test Cases

#### Test 1: Platform Y-Axis Rotation
1. Create a PlatformXZ voxel
2. Select it with the selection tool
3. Press `R` to enter rotation mode
4. Press `Y` to select Y-axis
5. Press `→` (right arrow) to rotate 90°
6. Press `Enter` to confirm
7. **Expected**: Platform should remain PlatformXZ (horizontal)

#### Test 2: Platform X-Axis Rotation
1. Create a PlatformXZ voxel
2. Select and enter rotation mode (`R`)
3. Press `X` to select X-axis
4. Press `→` to rotate 90°
5. Press `Enter` to confirm
6. **Expected**: Platform should transform to PlatformYZ (vertical wall facing X)

#### Test 3: Platform Z-Axis Rotation
1. Create a PlatformXZ voxel
2. Select and enter rotation mode (`R`)
3. Press `Z` to select Z-axis
4. Press `→` to rotate 90°
5. Press `Enter` to confirm
6. **Expected**: Platform should transform to PlatformXY (vertical wall facing Z)

#### Test 4: Staircase Y-Axis Rotation (Known Working)
1. Create a StaircaseX voxel
2. Select and enter rotation mode (`R`)
3. Press `Y` to select Y-axis
4. Press `→` to rotate 90°
5. Press `Enter` to confirm
6. **Expected**: Staircase should transform to StaircaseZ

#### Test 5: Staircase X-Axis Rotation (FIXED)
1. Create a StaircaseZ voxel
2. Select and enter rotation mode (`R`)
3. Press `X` to select X-axis
4. Press `→` twice to rotate 180°
5. Press `Enter` to confirm
6. **Expected**: Staircase should transform to StaircaseNegZ (flipped direction)

#### Test 6: Staircase Z-Axis Rotation (FIXED)
1. Create a StaircaseX voxel
2. Select and enter rotation mode (`R`)
3. Press `Z` to select Z-axis
4. Press `→` twice to rotate 180°
5. Press `Enter` to confirm
6. **Expected**: Staircase should transform to StaircaseNegX (flipped direction)

#### Test 7: Multiple Rotations
1. Create a StaircaseX voxel
2. Rotate 90° around Y-axis → Should become StaircaseZ
3. Rotate 90° around X-axis → Should stay StaircaseZ
4. Rotate 180° around X-axis → Should become StaircaseNegZ
5. Rotate 90° around Y-axis → Should become StaircaseX
6. **Expected**: Should return to original StaircaseX orientation

#### Test 8: Undo/Redo with Pattern Rotation
1. Create a StaircaseX voxel
2. Rotate it to StaircaseZ (Y-axis 90°)
3. Press `Ctrl+Z` to undo
4. **Expected**: Should revert to StaircaseX
5. Press `Ctrl+Y` to redo
6. **Expected**: Should return to StaircaseZ

### Visual Verification

For each test, verify:
1. **Preview rendering**: Blue ghost blocks show correct orientation during rotation
2. **Final rendering**: Confirmed voxel renders with correct pattern orientation
3. **Properties panel**: Shows correct pattern type after rotation
4. **Collision**: Pattern collision should match visual representation (test in game mode)

## Common Issues

### Issue: Pattern doesn't change after rotation
- **Cause**: Rotation axis is parallel to pattern direction
- **Example**: Rotating StaircaseX around X-axis (except 180°)
- **Expected**: This is correct behavior

### Issue: Pattern flips unexpectedly
- **Cause**: 180° rotation flips direction for perpendicular axes
- **Example**: StaircaseX rotated 180° around Z-axis becomes StaircaseNegX
- **Expected**: This is correct behavior

### Issue: Preview shows wrong orientation
- **Cause**: Renderer not updated for new pattern variants
- **Solution**: Check [`spawn_platform_voxel()`](../../../src/editor/renderer.rs:116) and [`spawn_staircase_voxel()`](../../../src/editor/renderer.rs:145)

## Implementation Details

### Key Files
- [`src/systems/game/map/format.rs`](../../../src/systems/game/map/format.rs:102) - Pattern rotation logic
- [`src/editor/tools/selection_tool.rs`](../../../src/editor/tools/selection_tool.rs:986) - Rotation confirmation
- [`src/editor/renderer.rs`](../../../src/editor/renderer.rs:79) - Pattern rendering
- [`src/systems/game/map/spawner.rs`](../../../src/systems/game/map/spawner.rs) - Game spawning

### Rotation Mathematics
- Rotations are in 90° increments (angle 0-3 for 0°/90°/180°/270°)
- X-axis rotation affects Y and Z coordinates (pitch)
- Y-axis rotation affects X and Z coordinates (yaw)
- Z-axis rotation affects X and Y coordinates (roll)

## Next Steps
After testing confirms all rotations work correctly:
1. Update main implementation status document
2. Add rotation feature to user guide
3. Consider adding rotation angle indicators in UI
4. Consider adding rotation gizmo visualization