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
}
```

Handles physics and movement data, independent of visual representation.

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
- **Scale:** 0.3 (30% of original size)
- **Y-Offset:** -0.3 units (aligns feet with ground)

### Why These Values?

- **Collision Sphere Radius:** 0.3 units
- **Model Scale (0.3):** Matches the collision sphere size for visual consistency
- **Y-Offset (-0.3):** Places the model's feet at ground level while keeping the sphere center at the player position for physics calculations

## Physics vs Visuals Separation

### Design Philosophy

The character system separates physics from visuals for several key benefits:

1. **Simple Physics:** Sphere collision is computationally efficient and predictable
2. **Flexible Visuals:** Models can be swapped without affecting gameplay
3. **Future-Proof:** Easy to add animations, different characters, or customization
4. **Performance:** Physics calculations remain fast regardless of model complexity

### Collision Detection

The physics system uses an **invisible sphere collider** (radius: 0.3) for all collision detection:
- Player movement ([`player_movement.rs`](../../../src/systems/game/player_movement.rs))
- Gravity and physics ([`physics.rs`](../../../src/systems/game/physics.rs))
- Collision checks ([`collision.rs`](../../../src/systems/game/collision.rs))

The visual model is purely cosmetic and doesn't participate in collision detection.

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
├── components.rs           # Player component
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
- [ ] Multiple character models/skins
- [ ] Character customization system
- [ ] Model LOD (Level of Detail) for performance
- [ ] Dynamic model swapping during gameplay

### Animation System

When animations are added, the system will need:
1. Animation controller component
2. State machine for animation transitions
3. Blend trees for smooth transitions
4. Integration with movement system

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
2. Current scale: `0.3` (30% of original)
3. Recommended range: `0.2` to `0.5`

### Model Position Issues

If the model floats or sinks into the ground:
1. Adjust the Y-offset in `Transform::from_translation(Vec3::new(0.0, y_offset, 0.0))`
2. Current offset: `-0.3` (one collision radius down)
3. Positive values move up, negative values move down

## Related Systems

- [Map Loader](map-loader.md) - Spawns the player during map loading
- [Camera Follow](camera-follow.md) - Camera tracks the player entity
- [Physics Analysis](physics-analysis.md) - Physics system overview

## References

- Bevy GLTF Documentation: https://docs.rs/bevy/latest/bevy/gltf/
- Character Model Component: [`src/systems/game/character/mod.rs`](../../../src/systems/game/character/mod.rs)
- Player Spawning: [`src/systems/game/map/spawner.rs`](../../../src/systems/game/map/spawner.rs)