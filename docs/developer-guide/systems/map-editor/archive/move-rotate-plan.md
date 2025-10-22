# Move and Rotate Operations - Implementation Plan

## Overview

This document outlines the implementation plan for adding move and rotate operations to selected voxels in the map editor. This is Phase 2 of the Selection Tool implementation.

**Status**: Planning  
**Priority**: High  
**Estimated Time**: 3-4 days

## Current State Analysis

### What We Have
- ✅ Selection system with visual highlighting
- ✅ Single-click selection with toggle
- ✅ Delete operation with history integration
- ✅ 3D ray-voxel intersection for cursor positioning
- ✅ Properties panel showing selection info
- ✅ History system supporting batch operations

### What We Need
- ⏳ Move selected voxels to new positions
- ⏳ Rotate selected voxels around a pivot point
- ⏳ Visual preview during move/rotate operations
- ⏳ Keyboard shortcuts for move/rotate
- ⏳ UI controls for move/rotate
- ⏳ History integration for undo/redo

## Design Decisions

### Move Operation

**Interaction Model**: Gizmo-based movement
- **Activation**: Press `G` key or click "Move" button when voxels are selected
- **Control**: 
  - Mouse movement translates selection along grid
  - Arrow keys for precise 1-unit movement
  - Shift + Arrow keys for 5-unit movement
  - Confirm with Left-click or Enter
  - Cancel with Right-click or Escape

**Visual Feedback**:
- Ghost preview of voxels at new positions (semi-transparent)
- Original positions remain visible until confirmed
- Grid snapping indicator
- Collision detection (highlight conflicts in red)

### Rotate Operation

**Interaction Model**: Keyboard-based rotation
- **Activation**: Press `R` key or click "Rotate" button when voxels are selected
- **Control**:
  - `X` key: Rotate 90° around X-axis
  - `Y` key: Rotate 90° around Y-axis  
  - `Z` key: Rotate 90° around Z-axis
  - Shift + key: Rotate -90° (counter-clockwise)
  - Confirm with Enter or Left-click
  - Cancel with Escape or Right-click

**Rotation Pivot**:
- Default: Center of selection bounding box
- Alternative: First selected voxel (toggle with `P` key)

**Visual Feedback**:
- Ghost preview of rotated voxels
- Rotation axis indicator (colored line)
- Pivot point indicator (small sphere)

## Architecture

### New Components

```rust
/// Component marking voxels in move/rotate preview
#[derive(Component)]
pub struct TransformPreview {
    pub original_pos: (i32, i32, i32),
    pub preview_pos: (i32, i32, i32),
    pub is_valid: bool, // false if collision detected
}

/// Resource tracking active transformation
#[derive(Resource)]
pub struct ActiveTransform {
    pub mode: TransformMode,
    pub selected_voxels: Vec<VoxelData>,
    pub pivot: Vec3,
    pub current_offset: IVec3,
    pub current_rotation: Quat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransformMode {
    None,
    Move,
    Rotate { axis: RotationAxis },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RotationAxis {
    X,
    Y,
    Z,
}
```

### New Events

```rust
/// Event to start move operation
#[derive(Event)]
pub struct StartMoveOperation;

/// Event to start rotate operation
#[derive(Event)]
pub struct StartRotateOperation;

/// Event to confirm transformation
#[derive(Event)]
pub struct ConfirmTransform;

/// Event to cancel transformation
#[derive(Event)]
pub struct CancelTransform;

/// Event to update transform preview
#[derive(Event)]
pub struct UpdateTransformPreview {
    pub offset: IVec3,
    pub rotation: Option<Quat>,
}
```

### New History Actions

```rust
pub enum EditorAction {
    // ... existing actions ...
    
    /// Move multiple voxels
    MoveVoxels {
        voxels: Vec<(i32, i32, i32)>, // original positions
        offset: (i32, i32, i32),
    },
    
    /// Rotate multiple voxels
    RotateVoxels {
        voxels: Vec<VoxelData>, // original data
        pivot: (f32, f32, f32),
        axis: RotationAxis,
        angle: f32, // in radians
        new_positions: Vec<(i32, i32, i32)>,
    },
}
```

## Implementation Steps

### Step 1: Move Operation Foundation (Day 1)

#### 1.1 Add Transform State
- [ ] Create `ActiveTransform` resource
- [ ] Add `TransformMode` enum to `EditorState`
- [ ] Create `TransformPreview` component

#### 1.2 Implement Move Activation
- [ ] Add `G` key handler to start move mode
- [ ] Create `start_move_operation()` system
- [ ] Store selected voxels in `ActiveTransform`
- [ ] Calculate selection bounding box center

#### 1.3 Implement Move Preview
- [ ] Create `update_move_preview()` system
- [ ] Track mouse movement delta
- [ ] Calculate new positions with grid snapping
- [ ] Spawn ghost preview meshes
- [ ] Detect collisions with existing voxels

#### 1.4 Implement Move Confirmation
- [ ] Add Enter/Left-click handler for confirmation
- [ ] Create `confirm_move()` system
- [ ] Update voxel positions in map data
- [ ] Create `MoveVoxels` history action
- [ ] Clear preview and reset state

#### 1.5 Implement Move Cancellation
- [ ] Add Escape/Right-click handler
- [ ] Create `cancel_transform()` system
- [ ] Despawn preview meshes
- [ ] Reset transform state

### Step 2: Arrow Key Movement (Day 1-2)

#### 2.1 Keyboard Movement
- [ ] Add arrow key handlers
- [ ] Implement 1-unit movement per key press
- [ ] Add Shift modifier for 5-unit movement
- [ ] Update preview in real-time

#### 2.2 Collision Detection
- [ ] Create `check_voxel_collision()` function
- [ ] Highlight conflicting voxels in red
- [ ] Prevent confirmation if collisions exist
- [ ] Show collision count in UI

### Step 3: Rotate Operation (Day 2-3)

#### 3.1 Rotation Foundation
- [ ] Add `R` key handler to start rotate mode
- [ ] Create `start_rotate_operation()` system
- [ ] Calculate rotation pivot point
- [ ] Add pivot visualization

#### 3.2 Rotation Controls
- [ ] Add X/Y/Z key handlers for axis selection
- [ ] Implement 90° rotation calculation
- [ ] Add Shift modifier for -90° rotation
- [ ] Update rotation quaternion

#### 3.3 Rotation Preview
- [ ] Create `update_rotate_preview()` system
- [ ] Calculate rotated positions around pivot
- [ ] Round to nearest grid position
- [ ] Spawn ghost preview meshes
- [ ] Detect collisions

#### 3.4 Rotation Confirmation
- [ ] Create `confirm_rotate()` system
- [ ] Apply rotation to voxel positions
- [ ] Create `RotateVoxels` history action
- [ ] Update map data
- [ ] Clear preview

### Step 4: Visual Feedback (Day 3)

#### 4.1 Ghost Preview Material
- [ ] Create semi-transparent material
- [ ] Use green for valid positions
- [ ] Use red for collision positions
- [ ] Add subtle emission

#### 4.2 Axis Indicators
- [ ] Create colored line meshes (X=red, Y=green, Z=blue)
- [ ] Show active rotation axis
- [ ] Add pivot point sphere

#### 4.3 UI Feedback
- [ ] Show current transform mode in status bar
- [ ] Display offset/rotation values
- [ ] Show collision warnings
- [ ] Add instruction text

### Step 5: UI Integration (Day 3-4)

#### 5.1 Properties Panel Updates
- [ ] Add "Move Selected" button
- [ ] Add "Rotate Selected" button
- [ ] Show transform mode status
- [ ] Display current offset/rotation
- [ ] Add "Confirm" and "Cancel" buttons during transform

#### 5.2 Keyboard Shortcuts Help
- [ ] Update shortcuts dialog with move/rotate keys
- [ ] Add transform mode instructions
- [ ] Document all keyboard controls

### Step 6: History Integration (Day 4)

#### 6.1 Move History
- [ ] Implement `MoveVoxels` action
- [ ] Implement inverse (move back)
- [ ] Test undo/redo for moves
- [ ] Handle batch moves

#### 6.2 Rotate History
- [ ] Implement `RotateVoxels` action
- [ ] Implement inverse (rotate back)
- [ ] Test undo/redo for rotations
- [ ] Handle complex rotation chains

### Step 7: Testing & Polish (Day 4)

#### 7.1 Edge Cases
- [ ] Test move with single voxel
- [ ] Test move with large selection
- [ ] Test rotation with asymmetric shapes
- [ ] Test collision detection accuracy
- [ ] Test undo/redo chains

#### 7.2 Performance
- [ ] Profile preview rendering
- [ ] Optimize collision detection
- [ ] Test with 100+ selected voxels

#### 7.3 UX Polish
- [ ] Smooth preview updates
- [ ] Clear visual feedback
- [ ] Intuitive controls
- [ ] Helpful error messages

## Technical Considerations

### Rotation Mathematics

For rotating a voxel position around a pivot:

```rust
fn rotate_position(
    pos: IVec3,
    pivot: Vec3,
    axis: RotationAxis,
    angle: f32,
) -> IVec3 {
    // Convert to Vec3 relative to pivot
    let relative = pos.as_vec3() - pivot;
    
    // Create rotation quaternion
    let rotation = match axis {
        RotationAxis::X => Quat::from_rotation_x(angle),
        RotationAxis::Y => Quat::from_rotation_y(angle),
        RotationAxis::Z => Quat::from_rotation_z(angle),
    };
    
    // Apply rotation
    let rotated = rotation * relative;
    
    // Convert back to grid position
    let world_pos = rotated + pivot;
    IVec3::new(
        world_pos.x.round() as i32,
        world_pos.y.round() as i32,
        world_pos.z.round() as i32,
    )
}
```

### Collision Detection

```rust
fn check_collision(
    new_positions: &[(i32, i32, i32)],
    existing_voxels: &[VoxelData],
    original_positions: &HashSet<(i32, i32, i32)>,
) -> Vec<(i32, i32, i32)> {
    let mut collisions = Vec::new();
    
    for &new_pos in new_positions {
        // Skip if this was an original position (moving into self)
        if original_positions.contains(&new_pos) {
            continue;
        }
        
        // Check if position is occupied
        if existing_voxels.iter().any(|v| v.pos == new_pos) {
            collisions.push(new_pos);
        }
    }
    
    collisions
}
```

### Preview Material

```rust
fn create_preview_material(
    materials: &mut Assets<StandardMaterial>,
    is_valid: bool,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: if is_valid {
            Color::srgba(0.0, 1.0, 0.0, 0.3) // Green
        } else {
            Color::srgba(1.0, 0.0, 0.0, 0.3) // Red
        },
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    })
}
```

## Keyboard Shortcuts Summary

| Key | Action |
|-----|--------|
| `G` | Start move mode |
| `R` | Start rotate mode |
| `X` | Rotate around X-axis (in rotate mode) |
| `Y` | Rotate around Y-axis (in rotate mode) |
| `Z` | Rotate around Z-axis (in rotate mode) |
| `Shift + X/Y/Z` | Rotate counter-clockwise |
| `P` | Toggle pivot point (in transform mode) |
| `Arrow Keys` | Move 1 unit |
| `Shift + Arrows` | Move 5 units |
| `Enter` | Confirm transformation |
| `Escape` | Cancel transformation |
| `Left Click` | Confirm transformation |
| `Right Click` | Cancel transformation |

## UI Mockup

```
┌─────────────────────────────────────────┐
│ Properties Panel                        │
├─────────────────────────────────────────┤
│ Select Tool                             │
│ Click to select voxels                  │
│                                         │
│ Selected: 5 voxels                      │
│ Positions:                              │
│   (1, 2, 3)                            │
│   (2, 2, 3)                            │
│   ...                                   │
│                                         │
│ ┌─────────┐ ┌─────────┐               │
│ │ 🔄 Move │ │ 🔃 Rotate│               │
│ └─────────┘ └─────────┘               │
│                                         │
│ ┌─────────┐ ┌─────────┐               │
│ │🗑 Delete│ │  Clear  │               │
│ └─────────┘ └─────────┘               │
│                                         │
│ ─────────────────────────────────────  │
│ Transform Mode: Move                    │
│ Offset: (2, 0, -1)                     │
│ Collisions: None                        │
│                                         │
│ Press Enter to confirm                  │
│ Press Escape to cancel                  │
│                                         │
│ ┌─────────┐ ┌─────────┐               │
│ │ Confirm │ │ Cancel  │               │
│ └─────────┘ └─────────┘               │
└─────────────────────────────────────────┘
```

## Success Criteria

### Functional Requirements
- ✅ Can move selected voxels with mouse
- ✅ Can move selected voxels with arrow keys
- ✅ Can rotate selected voxels 90° on any axis
- ✅ Preview shows before confirmation
- ✅ Collisions are detected and prevented
- ✅ Operations can be undone/redone
- ✅ UI provides clear feedback

### Performance Requirements
- ✅ Preview updates at 60 FPS with 100+ voxels
- ✅ Collision detection completes in < 16ms
- ✅ Transform confirmation is instant

### UX Requirements
- ✅ Controls are intuitive and discoverable
- ✅ Visual feedback is clear
- ✅ Errors are communicated clearly
- ✅ Keyboard shortcuts match industry standards

## Future Enhancements (Post-Phase 2)

### Phase 3 Possibilities
- **Duplicate**: Ctrl+D to duplicate selection
- **Mirror**: Flip selection along axis
- **Scale**: Resize selection (challenging with voxels)
- **Array**: Create multiple copies in a pattern
- **Align**: Align selection to grid or other voxels
- **Snap to Surface**: Move selection to nearest surface

### Advanced Features
- **Multi-axis rotation**: Rotate on multiple axes
- **Free rotation**: Rotate by arbitrary angles
- **Pivot manipulation**: Move pivot point manually
- **Transform gizmo**: 3D widget for visual manipulation
- **Numeric input**: Type exact offset/rotation values

## Dependencies

### Required Systems
- Selection tool (✅ implemented)
- History system (✅ implemented)
- Cursor ray casting (✅ implemented)
- Renderer (✅ implemented)

### Required Components
- `EditorState` (✅ exists)
- `EditorHistory` (✅ exists)
- `SelectionHighlight` (✅ exists)

### New Dependencies
- None (all functionality can be implemented with existing Bevy features)

## Risk Assessment

### High Risk
- **Rotation complexity**: Rotating voxels around arbitrary pivots can be mathematically complex
  - *Mitigation*: Start with 90° rotations only, use well-tested quaternion math

### Medium Risk
- **Collision detection performance**: Checking collisions for large selections
  - *Mitigation*: Use spatial hashing, early exit on first collision

- **Preview rendering**: Many ghost voxels could impact performance
  - *Mitigation*: Use instanced rendering, limit preview complexity

### Low Risk
- **Move operation**: Straightforward translation
- **UI integration**: Well-established patterns
- **History integration**: System already supports batch operations

## Testing Plan

### Unit Tests
- [ ] Test rotation math for all axes
- [ ] Test collision detection accuracy
- [ ] Test history action inverse operations
- [ ] Test grid snapping logic

### Integration Tests
- [ ] Test move → undo → redo chain
- [ ] Test rotate → undo → redo chain
- [ ] Test move + rotate combinations
- [ ] Test collision prevention

### Manual Tests
- [ ] Move single voxel
- [ ] Move large selection (50+ voxels)
- [ ] Rotate symmetric shape
- [ ] Rotate asymmetric shape
- [ ] Test all keyboard shortcuts
- [ ] Test UI buttons
- [ ] Test collision scenarios
- [ ] Test undo/redo extensively

---

**Document Version**: 1.0.0  
**Created**: 2025-10-21  
**Status**: Planning Complete - Ready for Implementation