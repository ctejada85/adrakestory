# Voxel Rotation System

## Overview

The map editor implements a generic, geometry-based rotation system for voxel sub-voxel patterns. This system uses mathematical transformations on 3D geometry rather than pattern-specific rotation logic, making it flexible and maintainable.

## Architecture

### Core Components

1. **SubVoxelGeometry** ([`src/systems/game/map/geometry.rs`](../../../../src/systems/game/map/geometry.rs))
   - Represents 8×8×8 sub-voxel grid using bit arrays
   - Implements generic rotation transformations
   - Handles rotation around X, Y, and Z axes in 90° increments

2. **RotationState** ([`src/systems/game/map/format.rs`](../../../../src/systems/game/map/format.rs))
   - Stores cumulative rotation applied to a voxel
   - Tracks rotation axis and angle (0-3 for 0°/90°/180°/270°)
   - Serializable for map persistence

3. **SubVoxelPattern** ([`src/systems/game/map/format.rs`](../../../../src/systems/game/map/format.rs))
   - Enum defining pattern types (Full, Platform, Staircase, Pillar)
   - Provides base geometry for each pattern
   - Applies rotation state when rendering

## How It Works

### Data Flow

```
User rotates voxel (R key + axis selection + arrow keys)
  ↓
ActiveTransform tracks rotation parameters
  ↓
Preview shows rotated geometry in real-time
  ↓
User confirms (Enter) or cancels (Escape)
  ↓
RotationState is created/updated in VoxelData
  ↓
Renderer/Spawner applies rotation when displaying voxel
```

### Rotation Application

When a voxel needs to be rendered:

1. Get the base pattern geometry: `pattern.geometry()`
2. Apply rotation state: `geometry_with_rotation(rotation_state)`
3. Spawn sub-voxels at rotated positions

The `geometry_with_rotation()` method is a convenience function that:
- Returns base geometry if no rotation state exists
- Applies rotation transformation if rotation state is present

This happens in:
- **Editor Renderer** ([`src/editor/renderer.rs`](../../../../src/editor/renderer.rs)) - For editor viewport
- **Game Spawner** ([`src/systems/game/map/spawner.rs`](../../../../src/systems/game/map/spawner.rs)) - For game mode
- **Rotation Preview** ([`src/editor/tools/selection_tool.rs`](../../../../src/editor/tools/selection_tool.rs)) - For preview during rotation

### Rotation Mathematics

The system uses standard 3D rotation matrices for 90° increments:

- **X-axis rotation** (pitch): Affects Y and Z coordinates
- **Y-axis rotation** (yaw): Affects X and Z coordinates  
- **Z-axis rotation** (roll): Affects X and Y coordinates

Coordinates are centered around (3.5, 3.5, 3.5) for rotation, ensuring the voxel rotates around its center.

## Usage

### Rotating Voxels in the Editor

1. Select voxel(s) with the Selection tool
2. Press `R` to enter rotation mode
3. Press `X`, `Y`, or `Z` to select rotation axis
4. Press `←` or `→` to rotate in 90° increments
5. Press `Enter` to confirm or `Escape` to cancel

### Rotation State Composition

When rotating a voxel multiple times:

- **Same axis**: Angles are added (e.g., 90° + 90° = 180°)
- **Different axis**: New rotation replaces old (simplified approach)

For complex multi-axis rotations, the system stores the most recent rotation state.

## Implementation Details

### VoxelData Structure

```rust
pub struct VoxelData {
    pub pos: (i32, i32, i32),
    pub voxel_type: VoxelType,
    pub pattern: Option<SubVoxelPattern>,
    pub rotation_state: Option<RotationState>,  // NEW
}
```

### RotationState Structure

```rust
pub struct RotationState {
    pub axis: RotationAxis,
    pub angle: i32,  // 0-3 for 0°, 90°, 180°, 270°
}
```

### Geometry-Based Rendering

```rust
// Get geometry with rotation applied (if any)
let geometry = pattern.geometry_with_rotation(voxel_data.rotation_state);

// Spawn sub-voxels from rotated geometry
for (sub_x, sub_y, sub_z) in geometry.occupied_positions() {
    spawn_sub_voxel(x, y, z, sub_x, sub_y, sub_z);
}
```

The `geometry_with_rotation()` method simplifies the rendering code by handling the rotation logic internally:

```rust
impl SubVoxelPattern {
    pub fn geometry_with_rotation(&self, rotation_state: Option<RotationState>) -> SubVoxelGeometry {
        let base_geometry = self.geometry();
        if let Some(rotation) = rotation_state {
            base_geometry.rotate(rotation.axis, rotation.angle)
        } else {
            base_geometry
        }
    }
}
```

## Benefits

### Generic System
- No pattern-specific rotation logic needed
- All patterns rotate correctly automatically
- Easy to add new patterns

### Mathematically Correct
- Uses standard 3D rotation matrices
- Preserves geometry properties
- Predictable behavior

### Backward Compatible
- `rotation_state` is optional with `#[serde(default)]`
- Old maps load without rotation state
- New rotations create rotation state automatically

### Efficient
- Rotation computed on-demand during rendering
- Bit-array representation is memory efficient
- No need to store rotated geometry

## Testing

See [`rotation-operations.md`](./testing/rotation-operations.md) for comprehensive testing procedures.

### Quick Test

1. Launch editor: `cargo run --bin map_editor`
2. Place a staircase voxel (StaircaseX pattern)
3. Select it and press `R` to rotate
4. Press `Y` then `→` to rotate 90° around Y-axis
5. Press `Enter` to confirm
6. Verify the staircase now faces a different direction

### Debugging

The rotation system includes debug logging that can be enabled by running in debug mode:

```bash
RUST_LOG=debug cargo run --bin map_editor
```

Debug logs include:
- Rotation confirmation triggers
- UI input state checks
- Voxel rotation state updates
- Renderer rotation state application

These logs use `debug!()` macro and only appear when debug logging is enabled.

## Troubleshooting

### Rotation Not Visible

If rotation appears to not work:

1. **Check voxel pattern**: Symmetrical patterns (like Full cubes) won't show visible rotation
2. **Use asymmetrical patterns**: Test with StaircaseX, StaircaseZ, or Platform patterns
3. **Check camera angle**: Rotation might be subtle from certain viewpoints
4. **Enable debug logging**: Use `RUST_LOG=debug` to verify rotation state is being saved and applied

### Common Issues

- **Rotation state not persisting**: Ensure you press Enter to confirm the rotation
- **Preview shows rotation but final doesn't**: This was a bug that has been fixed - rotation state is now properly saved to all voxels
- **Multiple rotations not composing**: The system currently replaces rotation on axis change; same-axis rotations do compose correctly

## Future Enhancements

Potential improvements:

1. **Full rotation composition**: Store complete rotation matrix instead of single axis/angle
2. **Rotation gizmo**: Visual indicator showing rotation axis and angle
3. **Snap angles**: Support for 45° or custom angle increments
4. **Rotation undo/redo**: Already supported through history system
5. **Batch rotation**: Rotate multiple voxels with different patterns simultaneously

## Related Files

- [`src/systems/game/map/geometry.rs`](../../../../src/systems/game/map/geometry.rs) - Geometry and rotation logic
- [`src/systems/game/map/format.rs`](../../../../src/systems/game/map/format.rs) - Data structures
- [`src/editor/tools/selection_tool.rs`](../../../../src/editor/tools/selection_tool.rs) - Rotation UI and controls
- [`src/editor/renderer.rs`](../../../../src/editor/renderer.rs) - Editor rendering
- [`src/systems/game/map/spawner.rs`](../../../../src/systems/game/map/spawner.rs) - Game spawning