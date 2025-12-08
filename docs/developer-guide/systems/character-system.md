# Character System

## Overview

The character system manages the player's visual representation using 3D models (GLB/GLTF format) while maintaining a separate physics collision system for gameplay.

## Architecture

### Component Structure

```
Player Entity (Parent)
├── Player Component (physics data)
├── Transform (position)
├── CharacterModel Component (scene handle)
└── Child: SceneRoot (GLB visual model)

Separate: CollisionBox (debug visualization)
```

### Key Components

#### CharacterModel Component
Located in [`src/systems/game/character/mod.rs`](../../../src/systems/game/character/mod.rs)

```rust
#[derive(Component)]
pub struct CharacterModel {
    pub scene_handle: Handle<Scene>,
    pub scale: f32,
    pub offset: Vec3,
}
```

Tracks the loaded 3D model scene and its transform properties.

#### Player Component
Located in [`src/systems/game/components.rs`](../../../src/systems/game/components.rs)

```rust
#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub velocity: Vec3,
    pub is_grounded: bool,
    pub radius: f32,  // Collision sphere radius
    pub target_rotation: f32,      // Target angle in radians
    pub current_rotation: f32,     // Current angle in radians
    pub start_rotation: f32,       // Angle when rotation started
    pub rotation_elapsed: f32,     // Time elapsed since rotation started
    pub rotation_duration: f32,    // Fixed duration for all rotations (0.2s)
}
```

Handles physics, movement, and rotation data, independent of visual representation.

## Model Loading

### Current Implementation

The character model is loaded in [`src/systems/game/map/spawner.rs`](../../../src/systems/game/map/spawner.rs) during map spawning:

```rust
use bevy::gltf::GltfAssetLabel;

// Load the character model with explicit scene specification
let character_scene: Handle<Scene> = asset_server.load(
    GltfAssetLabel::Scene(0).from_asset("characters/base_basic_pbr.glb")
);

// Spawn player entity (parent)
let player_entity = commands.spawn((
    Transform::from_translation(position),
    Visibility::default(),
    Player {
        speed: 3.0,
        velocity: Vec3::ZERO,
        is_grounded: true,
        radius: 0.3,
        target_rotation: 0.0,
        current_rotation: 0.0,
        start_rotation: 0.0,
        rotation_elapsed: 0.0,
        rotation_duration: 0.2,  // 0.2 seconds for snappy rotation
    },
    CharacterModel::new(character_scene.clone()),
)).id();

// Spawn model as child with scale and offset
commands.spawn((
    SceneRoot(character_scene),
    Transform::from_translation(Vec3::new(0.0, -0.3, 0.0))
        .with_scale(Vec3::splat(0.3)),
)).set_parent(player_entity);
```

### Model Configuration

**Current Model:** `assets/characters/base_basic_pbr.glb`
- **Format:** GLB (binary GLTF)
- **Materials:** PBR (Physically Based Rendering)
- **Scale:** 0.5 (50% of original size)
- **Y-Offset:** -0.3 units (aligns feet with ground)
- **Default Orientation:** Faces right (+Z direction)

### Why These Values?

- **Collision Sphere Radius:** 0.3 units
- **Model Scale (0.5):** Sized appropriately for the game world
- **Y-Offset (-0.3):** Places the model's feet at ground level while keeping the sphere center at the player position for physics calculations
- **Rotation Offset:** -90° applied to compensate for model's default right-facing orientation

## Character Rotation System

### Overview

The character model smoothly rotates to face the direction of movement using a fixed-duration rotation system with ease-in-out cubic easing. The rotation duration is input-aware: faster for gamepad (0.08s) for responsive analog control, and slower for keyboard (0.2s) for a snappier digital feel.

### Rotation Architecture

Located in [`src/systems/game/character_rotation.rs`](../../../src/systems/game/character_rotation.rs)

**Key Features:**
- **Input-Aware Duration:** 0.08s for gamepad, 0.2s for keyboard (snappy, arcade-like feel)
- **Ease-in-Out Cubic:** Smooth acceleration and deceleration
- **Shortest Path:** Always rotates the shortest way around
- **Direction-Based:** Automatically faces movement direction
- **Easing Reset:** Restarts smoothly when direction changes
- **Analog Support:** Full 360° directional control with gamepad stick

### Rotation Algorithm

```rust
// 1. Calculate target rotation from movement direction
let new_target = direction.z.atan2(-direction.x) - FRAC_PI_2;

// 2. Detect direction change and reset easing
if (new_target - player.target_rotation).abs() > 0.01 {
    player.start_rotation = player.current_rotation;
    player.rotation_elapsed = 0.0;
    player.target_rotation = new_target;
}

// 3. Update rotation with easing
player.rotation_elapsed += delta_time;
let progress = (player.rotation_elapsed / player.rotation_duration).min(1.0);
let eased_progress = ease_in_out_cubic(progress);

// 4. Interpolate from start to target
player.current_rotation = player.start_rotation + (angle_diff * eased_progress);
```

### Easing Function

```rust
fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let f = -2.0 * t + 2.0;
        1.0 - f * f * f / 2.0
    }
}
```

This creates a smooth "slow-fast-slow" rotation curve.

### Movement Direction Mapping

| Input | Direction | Angle | Character Faces |
|-------|-----------|-------|-----------------|
| W | +X | 0° | Forward |
| S | -X | 180° | Backward |
| A | -Z | 90° | Left |
| D | +Z | -90° | Right |
| W+D | +X+Z | -45° | Forward-Right |
| W+A | +X-Z | 45° | Forward-Left |
| S+D | -X+Z | -135° | Backward-Right |
| S+A | -X-Z | 135° | Backward-Left |

### Rotation Timing

Rotation duration is input-source aware for optimal feel:

**Keyboard (0.2 seconds):**
- 45° turn: 0.2 seconds
- 90° turn: 0.2 seconds
- 180° turn: 0.2 seconds

**Gamepad (0.08 seconds):**
- All rotations: 0.08 seconds
- Provides responsive analog stick control
- Prevents sluggish feeling with continuous stick input

This creates consistent, predictable rotation behavior optimized for each input type.

### Integration with Movement

The rotation system is integrated into the player movement system:

1. **Input Detection:** WASD keys or gamepad left stick determine movement direction
2. **Target Calculation:** Movement direction converted to target rotation angle
3. **Change Detection:** System detects when target rotation changes (threshold varies by input: 0.01 rad for keyboard, 0.15 rad for gamepad to prevent jitter)
4. **Easing Reset:** Rotation animation restarts from current position
5. **Duration Selection:** Faster rotation for gamepad (0.08s), standard for keyboard (0.2s)
6. **Visual Update:** Character model rotates smoothly to face direction

## Physics vs Visuals Separation

### Design Philosophy

The character system separates physics from visuals for several key benefits:

1. **Simple Physics:** Sphere collision is computationally efficient and predictable
2. **Flexible Visuals:** Models can be swapped without affecting gameplay
3. **Independent Rotation:** Visual rotation doesn't affect physics collision
4. **Future-Proof:** Easy to add animations, different characters, or customization
5. **Performance:** Physics calculations remain fast regardless of model complexity

### Collision Detection

The physics system uses an **invisible sphere collider** (radius: 0.3) for all collision detection:
- Player movement ([`player_movement.rs`](../../../src/systems/game/player_movement.rs))
- Gravity and physics ([`physics.rs`](../../../src/systems/game/physics.rs))
- Collision checks ([`collision.rs`](../../../src/systems/game/collision.rs))

The visual model is purely cosmetic and doesn't participate in collision detection. The character rotation is purely visual and doesn't affect the physics sphere.

## Dependencies

### Cargo.toml Configuration

```toml
[dependencies]
bevy = { version = "0.15", features = ["bevy_gltf"] }
```

The `bevy_gltf` feature must be enabled to load GLB/GLTF files.

### Required Imports

```rust
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
```

## File Structure

```
src/systems/game/
├── character/
│   └── mod.rs              # CharacterModel component
├── character_rotation.rs   # Character rotation system
├── components.rs           # Player component with rotation fields
├── player_movement.rs      # Movement and rotation target calculation
├── map/
│   └── spawner.rs         # Player spawning logic
└── ...

assets/
└── characters/
    ├── base_basic_pbr.glb  # Current character model
    └── base_basic_shaded.glb
```

## Future Enhancements

### Planned Features

- [ ] Character animations (walk, jump, idle)
  - Walk animation triggered by movement
  - Idle animation when stationary
  - Jump animation (when jumping is implemented)
  - Animation blending with rotation
- [ ] Multiple character models/skins
- [ ] Character customization system
- [ ] Model LOD (Level of Detail) for performance
- [ ] Dynamic model swapping during gameplay
- [ ] Configurable rotation speed/duration
- [ ] Different easing curves for different situations

### Animation System

When animations are added, the system will need:
1. Animation controller component
2. State machine for animation transitions
3. Blend trees for smooth transitions
4. Integration with movement and rotation systems
5. Root motion support for realistic movement

## Troubleshooting

### Model Not Loading

**Error:** `Could not find an asset loader matching`

**Solutions:**
1. Ensure `bevy_gltf` feature is enabled in Cargo.toml
2. Verify GLB file exists in `assets/characters/`
3. Use `GltfAssetLabel::Scene(0)` syntax for loading
4. Check file path is relative to `assets/` directory

### Model Size Issues

If the model appears too large or small:
1. Adjust the scale value in `Transform::from_scale(Vec3::splat(scale))`
2. Current scale: `0.5` (50% of original)
3. Recommended range: `0.3` to `0.7`

### Model Position Issues

If the model floats or sinks into the ground:
1. Adjust the Y-offset in `Transform::from_translation(Vec3::new(0.0, y_offset, 0.0))`
2. Current offset: `-0.3` (one collision radius down)
3. Positive values move up, negative values move down

### Rotation Issues

**Character not rotating:**
1. Verify `rotate_character_model` system is registered in `main.rs`
2. Check rotation fields are initialized in spawner
3. Ensure movement system calculates `target_rotation`
4. Verify `SceneRoot` component exists on child entity

**Character facing wrong direction:**
1. Adjust rotation offset in `player_movement.rs` (currently `-FRAC_PI_2`)
2. Check model's default orientation in 3D modeling software
3. Verify coordinate system mapping (W=+X, S=-X, A=-Z, D=+Z)

**Rotation too fast/slow:**
1. For keyboard: Adjust `rotation_duration` in Player component (currently 0.2 seconds)
2. For gamepad: Adjust `effective_duration` in `character_rotation.rs` (currently 0.08 seconds)
3. Lower values = faster rotation
4. Higher values = slower rotation
5. Recommended range: 0.05 to 0.15 for gamepad, 0.1 to 0.5 for keyboard

## Related Systems

- [Map Loader](map-loader.md) - Spawns the player during map loading
- [Camera Follow](camera-follow.md) - Camera tracks the player entity
- [Physics Analysis](physics-analysis.md) - Physics system overview

## References

- Bevy GLTF Documentation: https://docs.rs/bevy/latest/bevy/gltf/
- Character Model Component: [`src/systems/game/character/mod.rs`](../../../src/systems/game/character/mod.rs)
- Character Rotation System: [`src/systems/game/character_rotation.rs`](../../../src/systems/game/character_rotation.rs)
- Player Movement System: [`src/systems/game/player_movement.rs`](../../../src/systems/game/player_movement.rs)
- Player Spawning: [`src/systems/game/map/spawner.rs`](../../../src/systems/game/map/spawner.rs)
- Easing Functions: https://easings.net/ (reference for easing curves)