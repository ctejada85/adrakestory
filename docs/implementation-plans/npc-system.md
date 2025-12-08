# NPC System Implementation Plan

## Overview
Implement static NPCs (Non-Player Characters) that can be placed in maps. NPCs will have a visual model, collision with the player, and be defined in the map format.

## Requirements
- NPCs can be placed in maps via the map format
- NPCs have a 3D visual model (GLB/GLTF)
- Player collides with NPCs (cannot walk through them)
- NPCs do not move (static, stationary)
- NPCs can have custom properties (name, dialogue ID, etc.)

## Architecture

### Component Structure
```
NPC Entity (Parent)
├── Npc Component (NPC data: name, radius)
├── Transform (position)
└── Child: SceneRoot (GLB visual model)
```

### Data Flow
1. Map file defines NPC spawns with `entity_type: Npc`
2. Map loader parses NPC entity data
3. Spawner creates NPC entities with collision
4. Physics system checks player-NPC collision

## Implementation Steps

### Step 1: Add NPC EntityType

**File:** `src/systems/game/map/format.rs`

Add `Npc` variant to the `EntityType` enum:

```rust
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityType {
    /// Player spawn point
    PlayerSpawn,
    /// NPC spawn point
    Npc,
    /// Enemy spawn point
    Enemy,
    /// Item spawn point
    Item,
    /// Trigger volume
    Trigger,
}
```

### Step 2: Create NPC Component

**File:** `src/systems/game/components.rs`

Add an `Npc` component to identify NPC entities:

```rust
/// Component for NPC entities.
/// NPCs are static characters that the player can interact with.
#[derive(Component)]
pub struct Npc {
    /// Display name of the NPC
    pub name: String,
    /// Collision radius for player collision
    pub radius: f32,
}

impl Default for Npc {
    fn default() -> Self {
        Self {
            name: "NPC".to_string(),
            radius: 0.3,
        }
    }
}
```

### Step 3: Implement NPC Spawning

**File:** `src/systems/game/map/spawner.rs`

Add a `spawn_npc` function similar to `spawn_player`:

```rust
/// Spawn an NPC entity with a 3D character model.
fn spawn_npc(ctx: &mut EntitySpawnContext, position: Vec3, properties: &HashMap<String, String>) {
    let npc_radius = properties
        .get("radius")
        .and_then(|r| r.parse::<f32>().ok())
        .unwrap_or(0.3);
    
    let npc_name = properties
        .get("name")
        .cloned()
        .unwrap_or_else(|| "NPC".to_string());

    // Get model path from properties or use default
    let model_path = properties
        .get("model")
        .cloned()
        .unwrap_or_else(|| "characters/base_basic_pbr.glb".to_string());

    // Load the NPC model
    let npc_scene: Handle<Scene> = ctx
        .asset_server
        .load(GltfAssetLabel::Scene(0).from_asset(&model_path));

    // Spawn NPC entity
    let npc_entity = ctx
        .commands
        .spawn((
            Transform::from_translation(position),
            Visibility::default(),
            Npc {
                name: npc_name.clone(),
                radius: npc_radius,
            },
        ))
        .id();

    // Spawn model as child
    ctx.commands
        .spawn((
            SceneRoot(npc_scene),
            Transform::from_translation(Vec3::new(0.0, -0.3, 0.0))
                .with_scale(Vec3::splat(0.5)),
        ))
        .set_parent(npc_entity);

    info!("Spawned NPC '{}' at position: {:?}", npc_name, position);
}
```

Update `spawn_entities` to handle the `Npc` entity type:

```rust
EntityType::Npc => {
    spawn_npc(ctx, Vec3::new(x, y, z), &entity_data.properties);
}
```

### Step 4: Add NPC Collision Detection

**File:** `src/systems/game/physics.rs`

Modify `apply_physics` to check collision with NPCs:

```rust
pub fn apply_physics(
    time: Res<Time>,
    spatial_grid: Res<SpatialGrid>,
    sub_voxel_query: Query<&SubVoxel, Without<Player>>,
    npc_query: Query<(&Npc, &Transform), Without<Player>>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
) {
    // ... existing code ...

    // Check collision with NPCs (sphere-sphere collision)
    for (npc, npc_transform) in &npc_query {
        let distance = player_pos.distance(npc_transform.translation);
        let min_distance = player.radius + npc.radius;
        
        if distance < min_distance {
            // Push player away from NPC
            let direction = (player_pos - npc_transform.translation).normalize_or_zero();
            let penetration = min_distance - distance;
            player_transform.translation += direction * penetration;
        }
    }
}
```

### Step 5: Update Documentation

**Files to update:**
- `docs/api/map-format-spec.md` - Add Npc to EntityType documentation
- `docs/user-guide/maps/map-format.md` - Document NPC placement
- `docs/user-guide/maps/creating-maps.md` - Add NPC examples

## Map Format Example

```ron
entities: [
    (
        entity_type: PlayerSpawn,
        position: (2.5, 1.0, 2.5),
        properties: {},
    ),
    (
        entity_type: Npc,
        position: (5.0, 1.0, 5.0),
        properties: {
            "name": "Village Elder",
            "model": "characters/base_basic_pbr.glb",
            "radius": "0.3",
        },
    ),
]
```

## NPC Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `name` | String | "NPC" | Display name of the NPC |
| `model` | String | "characters/base_basic_pbr.glb" | Path to GLB model |
| `radius` | String (float) | "0.3" | Collision radius |

## File Changes Summary

| File | Change |
|------|--------|
| `src/systems/game/map/format.rs` | Add `Npc` to `EntityType` enum |
| `src/systems/game/components.rs` | Add `Npc` component |
| `src/systems/game/map/spawner.rs` | Add `spawn_npc` function, handle in `spawn_entities` |
| `src/systems/game/physics.rs` | Add NPC collision detection |

## Testing Checklist

- [ ] NPC spawns at correct position from map file
- [ ] NPC model is visible in game
- [ ] Player cannot walk through NPC
- [ ] Player is pushed back when colliding with NPC
- [ ] Multiple NPCs can exist in same map
- [ ] NPC properties (name, model, radius) are parsed correctly
- [ ] Default values work when properties are omitted

## Future Enhancements (Out of Scope)

- NPC dialogue system
- NPC facing direction / rotation
- NPC animations (idle)
- NPC movement / pathfinding
- NPC interaction (E key to talk)
- NPC spawn in map editor

## Estimated Effort

~1-2 hours for basic implementation
