# Map Editor WASD Movement Implementation Plan

## Status: 📋 PLANNED

## Overview

This document outlines the implementation plan for replacing the orbit camera with a permanent first-person flying camera in the Map Editor. The camera will always be in fly mode, providing a Minecraft Creative mode-style editing experience with WASD movement and mouse look.

## Goal

Replace the orbit camera system with a permanent fly camera, enabling keyboard and mouse users to fly around the map and edit with familiar FPS-style controls (WASD + mouse look).

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
| Look Around | Mouse Movement | Rotate camera view (yaw/pitch) - always active |

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

## Implementation Phases

### Phase 1: Replace Orbit Camera with Fly Camera
**Estimated time: 2-3 hours**

1. Modify `EditorCamera` to store position and rotation directly instead of orbit parameters
   ```rust
   #[derive(Resource)]
   pub struct EditorCamera {
       pub position: Vec3,
       pub yaw: f32,    // Horizontal rotation (radians)
       pub pitch: f32,  // Vertical rotation (radians, clamped)
       pub move_speed: f32,
       pub look_sensitivity: f32,
   }
   ```

2. Remove orbit camera logic:
   - Remove `target`, `distance`, `target_distance` fields
   - Remove `orbit()`, `pan()`, `zoom()` methods
   - Remove orbit-related input handling

3. Implement mouse look (always active):
   - Track mouse delta movement every frame
   - Apply to yaw (horizontal) and pitch (vertical)
   - Clamp pitch to prevent camera flip (-89° to +89°)

4. Update `calculate_position()` and transform sync to use new position-based system

### Phase 2: WASD Movement
**Estimated time: 1-2 hours**

1. Create `handle_wasd_movement` system
   ```rust
   pub fn handle_wasd_movement(
       keys: Res<ButtonInput<KeyCode>>,
       time: Res<Time>,
       mut editor_camera: ResMut<EditorCamera>,
       egui_context: Query<&EguiContext>,
   )
   ```

2. Movement logic:
   - Calculate forward vector from yaw (ignore pitch for horizontal movement)
   - Calculate right vector perpendicular to forward
   - W/S: Move along forward vector
   - A/D: Move along right vector
   - Space: Move up (+Y)
   - Ctrl: Move down (-Y)
   - Apply movement speed (~15-20 units/sec)
   - Scale by delta time for frame-independent movement

3. Skip movement when egui has focus (typing in text fields)

### Phase 3: Raycast Cursor (Center Screen)
**Estimated time: 1 hour**

1. Always use center-screen raycast for cursor positioning:
   - Cast ray from camera position in look direction
   - Find intersection with voxels or ground plane
   - Position cursor wireframe at hit point

2. Remove mouse-position-based cursor logic (no longer needed)

3. Reuse existing raycast logic from controller support

### Phase 4: Tool Actions (Right/Left Click)
**Estimated time: 1-2 hours**

1. Modify click handling:
   - Right Click → Primary action (place voxel, place entity, select)
   - Left Click → Secondary action (remove voxel)

2. Ensure actions use raycast cursor position (center of screen)

3. Add debounce/cooldown to prevent rapid-fire actions (0.15-0.2 sec)

4. Remove old mouse-based tool action logic that used cursor position

### Phase 5: Q/E Pattern Cycling
**Estimated time: 30 minutes**

1. Create `handle_qe_cycling` system
   ```rust
   pub fn handle_qe_cycling(
       keys: Res<ButtonInput<KeyCode>>,
       mut editor_state: ResMut<EditorState>,
       mut tool_memory: ResMut<ToolMemory>,
       time: Res<Time>,
       mut last_cycle: Local<f32>,
   )
   ```

2. Reuse pattern/entity cycling logic from controller support:
   - Q key: Previous pattern/entity
   - E key: Next pattern/entity
   - 0.2 second cooldown between cycles

3. Active when using Voxel Place or Entity Place tools

### Phase 6: Cleanup & Polish
**Estimated time: 1-2 hours**

1. Remove all orbit camera code and related UI
   - Remove orbit controls from Camera tool
   - Remove zoom controls (or repurpose for move speed)
   - Update/remove pan controls

2. Update UI panels that referenced orbit camera

3. Ensure controller support still works alongside WASD:
   - Controller and keyboard/mouse should work simultaneously
   - Same camera system used by both

4. Update keyboard edit mode (`I` key) if it conflicts

5. Test edge cases:
   - UI focus (egui text fields should block movement)
   - Menu shortcuts should still work
   - Tool switching should still work

## System Registration Order

```rust
// In main.rs, replace orbit camera systems with:
.add_systems(Update, camera::handle_mouse_look.after(ui_system::render_ui))
.add_systems(Update, camera::handle_wasd_movement.after(camera::handle_mouse_look))
.add_systems(Update, camera::handle_fly_mode_actions.after(camera::handle_wasd_movement))
.add_systems(Update, camera::handle_qe_cycling.after(camera::handle_wasd_movement))
.add_systems(Update, camera::sync_camera_transform.after(camera::handle_wasd_movement))
```

## File Changes

### Modified Files
- `src/editor/camera.rs` - Replace orbit camera with fly camera, add WASD/mouse look systems
- `src/bin/map_editor/main.rs` - Update system registration
- `src/editor/cursor/mod.rs` - Update to always use center-screen raycast
- `docs/user-guide/map-editor/controls.md` - Document new controls, remove orbit camera docs

### Removed Functionality
- Orbit camera mode (target + distance + rotation)
- Middle-click pan
- Right-click orbit
- Scroll wheel zoom
- Camera tool orbit controls

## Interaction with Existing Systems

### Controller Support
- Controller and keyboard/mouse use the same fly camera
- Both can be used simultaneously or interchangeably
- Controller stick input adds to keyboard movement
- Controller look adds to mouse look

### Keyboard Edit Mode
- `I` key keyboard edit mode may need adjustment or removal
- The new WASD system provides similar functionality
- Consider keeping for precise grid-based cursor navigation without camera movement

## Testing Plan

1. **Unit Tests**
   - Test movement vector calculations
   - Test pitch clamping at ±89°
   - Test Q/E cycling wrapping

2. **Integration Tests**
   - Test camera position updates correctly
   - Test cursor raycast hits correct voxels
   - Test tool actions work at cursor position

3. **Manual Testing**
   - Verify smooth movement at various frame rates
   - Verify no input conflicts with UI text fields
   - Verify controller still works
   - Test tool actions (place/remove voxels)

## Future Enhancements

1. **Configurable Keys** - Allow rebinding WASD to other keys
2. **Movement Speed Slider** - UI control for fly speed
3. **Sprint Key** - Shift to move faster
4. **Smooth Deceleration** - Gradual stop instead of instant

## Dependencies

- Existing raycast cursor system from controller support
- Existing tool memory and pattern cycling logic
- Bevy input systems

## Estimated Total Time

**6-10 hours** depending on how much orbit camera code needs to be removed/refactored.

## Success Criteria

1. ✅ WASD moves camera smoothly
2. ✅ Mouse look rotates camera view (always active)
3. ✅ Space/Ctrl moves up/down
4. ✅ Right click executes tool primary action
5. ✅ Left click removes voxels
6. ✅ Q/E cycles patterns and entities
7. ✅ Cursor always at center-screen raycast position
8. ✅ No conflicts with UI text input
9. ✅ Controller still works alongside keyboard/mouse
10. ✅ All orbit camera code removed
