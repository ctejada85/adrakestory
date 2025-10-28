# Camera Follow System

## Overview

The camera follow system provides smooth, responsive camera tracking of the player character while maintaining the game's isometric perspective. The camera maintains a fixed offset from the player and smoothly interpolates its position for a polished gameplay experience.

## Architecture

### Components

#### GameCamera Component
Located in [`src/systems/game/components.rs`](../../../src/systems/game/components.rs)

```rust
pub struct GameCamera {
    pub original_rotation: Quat,
    pub target_rotation: Quat,
    pub rotation_speed: f32,
    pub follow_offset: Vec3,      // Offset from player in local camera space
    pub follow_speed: f32,         // Interpolation speed (higher = more responsive)
    pub target_position: Vec3,     // Current follow target (player position)
}
```

**Key Fields:**
- `follow_offset`: The camera's offset from the player in local camera space (before rotation is applied)
- `follow_speed`: Controls how quickly the camera catches up to the player (default: 5.0)
- `target_position`: The current position being followed (updated each frame to player position)

### Systems

#### follow_player_camera System
Located in [`src/systems/game/camera.rs`](../../../src/systems/game/camera.rs)

**Purpose:** Smoothly moves the camera to follow the player's position.

**Execution Order:** Runs first in the Camera phase, before rotation.

**Algorithm:**
1. Query the player's current position
2. Update `target_position` to player position
3. Calculate world-space offset by rotating `follow_offset` with camera's current rotation
4. Calculate target camera position: `player_position + rotated_offset`
5. Smoothly interpolate camera position using lerp: `lerp(current, target, follow_speed * delta_time)`

**Key Features:**
- Maintains relative camera position during rotation
- Smooth interpolation prevents jarring camera movements
- Offset is rotated with camera, preserving isometric view

#### rotate_camera System
Located in [`src/systems/game/camera.rs`](../../../src/systems/game/camera.rs)

**Purpose:** Handles camera rotation around the player when Delete key is pressed.

**Execution Order:** Runs second in the Camera phase, after following.

**Changes from Original:**
- Rotation center changed from fixed world point (1.5, 0.0, 1.5) to `target_position` (player position)
- Camera now rotates around the player instead of a fixed point
- Maintains smooth rotation interpolation

## System Ordering

The camera systems run in the Camera phase, which is the last phase in the game loop:

```
Input → Movement → Physics → Visual → Camera
                                       ├─ follow_player_camera
                                       └─ rotate_camera
```

This ordering ensures:
1. Player movement is fully processed before camera updates
2. Camera follows the final player position after physics
3. Rotation is applied after following, maintaining correct pivot point

## Configuration

### Default Values

Set in [`spawn_camera`](../../../src/systems/game/map/spawner.rs) function:

```rust
follow_offset: Vec3 // Calculated from initial camera position
follow_speed: 5.0   // Medium responsiveness
rotation_speed: 5.0 // Existing rotation speed
```

### Adjusting Camera Behavior

**More Responsive Following:**
```rust
follow_speed: 8.0  // Camera catches up faster
```

**Smoother/Cinematic Following:**
```rust
follow_speed: 3.0  // Camera lags behind more
```

**Different Camera Angle:**
Modify the initial camera position in the map file (`assets/maps/default.ron`):
```ron
camera: (
    position: (5.0, 6.0, 5.0),  // Further back and higher
    look_at: (1.5, 0.0, 1.5),
    rotation_offset: 0.0,
)
```

## Implementation Details

### Offset Calculation

The `follow_offset` is calculated at initialization from the camera's starting position:

```rust
let initial_offset = camera_position - look_at_point;
let follow_offset = camera_rotation.inverse() * initial_offset;
```

This ensures the camera maintains its initial relative position to the player.

### Rotation Handling

When the camera rotates (Delete key pressed):
1. `follow_player_camera` updates camera position to follow player
2. `rotate_camera` rotates the camera around the player's position
3. The offset is automatically rotated, maintaining the isometric view

### Smooth Interpolation

Both systems use interpolation for smooth movement:
- **Position:** `lerp(current, target, speed * delta_time)`
- **Rotation:** `slerp(current, target, speed * delta_time)`

The `delta_time` multiplication ensures frame-rate independent movement.

## Testing

### Manual Testing

1. **Basic Following:**
   - Run the game: `cargo run`
   - Move the player with WASD/Arrow keys
   - Verify camera smoothly follows the player

2. **Rotation During Following:**
   - Move the player
   - Hold Delete key to rotate camera
   - Verify camera continues following while rotating
   - Verify camera rotates around player, not fixed point

3. **Responsiveness:**
   - Make quick directional changes
   - Verify camera smoothly catches up
   - Check for no jarring movements

### Expected Behavior

✅ Camera maintains fixed distance from player
✅ Camera smoothly interpolates position changes
✅ Camera continues following during rotation
✅ Rotation pivots around player position
✅ Isometric view is maintained at all times
✅ No stuttering or jarring movements

## Troubleshooting

### Camera Too Slow/Fast

Adjust `follow_speed` in [`spawn_camera`](../../../src/systems/game/map/spawner.rs):
```rust
follow_speed: 5.0, // Increase for faster, decrease for slower
```

### Camera Too Close/Far

Modify the initial camera position in the map file to change the offset.

### Camera Not Following

Check that:
1. Player entity has the `Player` component
2. Camera entity has the `GameCamera` component
3. `follow_player_camera` system is registered in [`main.rs`](../../../src/main.rs)
4. System runs in the Camera phase

### Rotation Issues

Verify:
1. `follow_player_camera` runs before `rotate_camera`
2. `target_position` is being updated each frame
3. Rotation center uses `target_position`, not fixed point

## Future Enhancements

Potential improvements to the camera system:

1. **Dead Zone:** Don't follow small player movements
2. **Look-Ahead:** Camera leads player movement direction
3. **Camera Shake:** Add impact effects on landing/damage
4. **Dynamic Zoom:** Adjust distance based on player velocity
5. **Collision Avoidance:** Prevent camera clipping through walls
6. **Smooth Zoom:** Allow player to adjust camera distance
7. **Multiple Camera Modes:** Switch between follow styles

## Related Systems

- **Player Movement:** [`src/systems/game/player_movement.rs`](../../../src/systems/game/player_movement.rs)
- **Physics:** [`src/systems/game/physics.rs`](../../../src/systems/game/physics.rs)
- **Map Spawning:** [`src/systems/game/map/spawner.rs`](../../../src/systems/game/map/spawner.rs)

## References

- [Bevy Transform Documentation](https://docs.rs/bevy/latest/bevy/transform/components/struct.Transform.html)
- [Camera Systems in Game Development](https://en.wikipedia.org/wiki/Virtual_camera_system)