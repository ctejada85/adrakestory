# Map Editor WASD Movement Implementation Plan

## Status: 📋 PLANNED

## Overview

This document outlines the implementation plan for adding WASD keyboard movement to the Map Editor, providing the same first-person flying experience as the controller support. This allows seamless editing with keyboard and mouse, mimicking Minecraft Creative mode movement.

## Goal

Enable keyboard users to fly around the map and edit with the same fluidity as controller users, using familiar FPS-style controls (WASD + mouse look).

## Control Mapping

### Movement Keys
| Action | Key | Description |
|--------|-----|-------------|
| Move Forward | `W` | Fly forward in camera direction |
| Move Backward | `S` | Fly backward |
| Strafe Left | `A` | Fly left |
| Strafe Right | `D` | Fly right |
| Fly Up | `Space` | Ascend vertically |
| Fly Down | `Ctrl` | Descend vertically |

### Camera Look
| Action | Control | Description |
|--------|---------|-------------|
| Look Around | Mouse Movement | Rotate camera (yaw/pitch) when in fly mode |

### Tool Actions
| Action | Control | Description |
|--------|---------|-------------|
| Primary Action | Right Click | Execute current tool's action (place voxel, place entity, select, etc.) |
| Secondary Action | Left Click | Remove voxel (always, regardless of current tool) |

### Pattern/Entity Cycling
| Action | Key | Description |
|--------|-----|-------------|
| Next Pattern/Entity | `E` | Cycle to next pattern (Voxel Place) or entity type (Entity Place) |
| Previous Pattern/Entity | `Q` | Cycle to previous pattern or entity type |

### Mode Toggle
| Action | Key | Description |
|--------|-----|-------------|
| Toggle Fly Mode | `F` | Enter/exit first-person fly mode |
| Exit Fly Mode | `Escape` | Return to orbit camera mode |

## Implementation Phases

### Phase 1: Fly Mode State & Camera Control
**Estimated time: 1-2 hours**

1. Add `FlyModeState` resource to track when WASD fly mode is active
   ```rust
   #[derive(Resource, Default)]
   pub struct FlyModeState {
       pub active: bool,
       pub mouse_captured: bool,
       pub last_mouse_position: Option<Vec2>,
   }
   ```

2. Add system to toggle fly mode with `F` key
   - When entering fly mode: capture mouse, hide cursor
   - When exiting fly mode: release mouse, show cursor, restore orbit camera

3. Implement mouse look when in fly mode
   - Track mouse delta movement
   - Apply to camera yaw (horizontal) and pitch (vertical)
   - Clamp pitch to prevent camera flip (-89° to +89°)

### Phase 2: WASD Movement
**Estimated time: 1-2 hours**

1. Create `handle_wasd_movement` system
   ```rust
   pub fn handle_wasd_movement(
       fly_state: Res<FlyModeState>,
       keys: Res<ButtonInput<KeyCode>>,
       time: Res<Time>,
       mut camera_query: Query<&mut Transform, With<EditorCamera>>,
       mut editor_camera: ResMut<EditorCamera>,
   )
   ```

2. Movement logic:
   - Get camera's forward and right vectors from current rotation
   - W/S: Move along forward vector (ignore Y for horizontal movement, or include for true flight)
   - A/D: Move along right vector
   - Space/Ctrl: Move along world Y axis
   - Apply movement speed multiplier (configurable, ~10-20 units/sec)
   - Scale by delta time for frame-independent movement

3. Update `EditorCamera` target to match new position (keep orbit camera in sync)

### Phase 3: Raycast Cursor for Fly Mode
**Estimated time: 1 hour**

1. Reuse existing raycast logic from controller support
   - Cast ray from camera center in look direction
   - Find intersection with voxels or ground plane
   - Position cursor wireframe at hit point

2. Update cursor system to work in fly mode
   - When fly mode active: use center-screen raycast
   - When fly mode inactive: use mouse position raycast (existing behavior)

### Phase 4: Tool Actions (Right/Left Click)
**Estimated time: 1-2 hours**

1. Modify click handling when in fly mode:
   - Right Click → Primary action (place voxel, place entity, select)
   - Left Click → Secondary action (remove voxel)

2. Ensure actions use raycast cursor position, not mouse position

3. Add debounce/cooldown to prevent rapid-fire actions (0.15-0.2 sec)

### Phase 5: Q/E Pattern Cycling
**Estimated time: 30 minutes**

1. Create `handle_qe_cycling` system
   ```rust
   pub fn handle_qe_cycling(
       fly_state: Res<FlyModeState>,
       keys: Res<ButtonInput<KeyCode>>,
       mut editor_state: ResMut<EditorState>,
       mut tool_memory: ResMut<ToolMemory>,
       time: Res<Time>,
       mut last_cycle: Local<f32>,
   )
   ```

2. Reuse pattern/entity cycling logic from controller support
   - Q key: Previous pattern/entity
   - E key: Next pattern/entity
   - 0.2 second cooldown between cycles

3. Only active when in fly mode OR when using Voxel Place / Entity Place tools

### Phase 6: Integration & Polish
**Estimated time: 1 hour**

1. Add visual indicator when fly mode is active (status bar: "✈ FLY MODE")

2. Ensure fly mode doesn't conflict with:
   - Existing keyboard edit mode (`I` key)
   - Text input fields (egui focus)
   - Menu shortcuts

3. Register all new systems in `main.rs` with proper ordering

4. Add smooth camera transition when entering/exiting fly mode

## System Registration Order

```rust
// In main.rs, add after existing camera systems:
.add_systems(Update, camera::toggle_fly_mode.after(ui_system::render_ui))
.add_systems(Update, camera::handle_fly_mode_mouse_look.after(camera::toggle_fly_mode))
.add_systems(Update, camera::handle_wasd_movement.after(camera::handle_fly_mode_mouse_look))
.add_systems(Update, camera::handle_fly_mode_actions.after(camera::handle_wasd_movement))
.add_systems(Update, camera::handle_qe_cycling.after(camera::handle_wasd_movement))
```

## File Changes

### Modified Files
- `src/editor/camera.rs` - Add FlyModeState, movement systems, mouse look
- `src/editor/state.rs` - Add fly mode state resource
- `src/bin/map_editor/main.rs` - Register new systems
- `docs/user-guide/map-editor/controls.md` - Document new controls

### New Code Locations
All fly mode code should be added to `src/editor/camera.rs` to keep camera-related functionality together.

## Interaction with Existing Systems

### Controller Support
- Fly mode and controller mode are mutually exclusive
- Using controller deactivates fly mode
- Using WASD/mouse deactivates controller mode

### Keyboard Edit Mode
- `I` key keyboard edit mode remains separate
- Fly mode focuses on camera movement
- Keyboard edit mode focuses on cursor grid navigation

### Orbit Camera
- Fly mode temporarily overrides orbit camera behavior
- Exiting fly mode restores last orbit camera state
- Orbit camera target is updated when exiting fly mode to current position

## Testing Plan

1. **Unit Tests**
   - Test movement vector calculations
   - Test pitch clamping
   - Test Q/E cycling wrapping

2. **Integration Tests**
   - Test fly mode toggle
   - Test cursor raycast in fly mode
   - Test tool actions work correctly

3. **Manual Testing**
   - Verify smooth movement at various frame rates
   - Verify no input conflicts with UI
   - Verify seamless transition between fly/orbit modes
   - Test on different keyboard layouts (QWERTY, AZERTY considerations)

## Future Enhancements

1. **Configurable Keys** - Allow rebinding WASD to other keys
2. **Movement Speed Slider** - UI control for fly speed
3. **Sprint Key** - Shift to move faster
4. **Smooth Deceleration** - Gradual stop instead of instant
5. **Collision Detection** - Optional "noclip" toggle

## Dependencies

- Existing `EditorCamera` struct and systems
- Existing raycast cursor system from controller support
- Existing tool memory and pattern cycling logic

## Estimated Total Time

**4-8 hours** depending on polish level and edge case handling.

## Success Criteria

1. ✅ WASD moves camera smoothly in fly mode
2. ✅ Mouse look rotates camera view
3. ✅ Space/Ctrl moves up/down
4. ✅ Right click executes tool primary action
5. ✅ Left click removes voxels
6. ✅ Q/E cycles patterns and entities
7. ✅ F toggles fly mode on/off
8. ✅ Escape exits fly mode
9. ✅ No conflicts with existing shortcuts
10. ✅ Cursor shows correct placement position
