# Fire Entity System Implementation Plan

## Overview

Implement a dynamic fire entity type that can be placed in maps via the map editor and rendered at runtime using a GPU particle system. The fire effect will be based on the Three.js particle fire simulation (`fire-simulation.html`), adapted for Bevy's rendering system.

## Reference: HTML Fire Simulation Analysis

The `fire-simulation.html` demonstrates a particle-based fire effect with the following characteristics:

### Core Mechanics
1. **Particle Count**: ~2000 particles for a realistic fire appearance
2. **Particle Physics**:
   - Each particle has velocity (x, y, z components)
   - Upward bias with random horizontal spread
   - Life/decay system for particle recycling
3. **Visual Effects**:
   - Additive blending for bright overlapping regions
   - Color gradient: White core → Orange → Red → Fade out
   - Procedural radial gradient texture
   - Depth write disabled for proper transparency
4. **Dynamic Light**: Point light with flickering intensity

### Key Equations
- **Position Update**: `P_new = P_old + Velocity * deltaTime`
- **Life Decay**: `life -= deltaTime`
- **Color Fading**: Green channel = `life / maxLife` (Yellow → Red transition)
- **Reset Conditions**: When `life <= 0` or `y > maxHeight`

## Requirements

### Functional Requirements
1. Fire entities can be placed in maps via the map editor (EntityType::Fire)
2. Fire renders as an animated particle system at runtime
3. Fire emits dynamic point light with flickering
4. Fire can have configurable properties (intensity, size, color temperature)
5. Multiple fire entities can exist in a scene simultaneously

### Non-Functional Requirements
1. GPU-accelerated particle rendering for performance
2. Consistent visual quality across different hardware
3. Minimal CPU overhead per fire instance
4. Editor shows static fire indicator (full particle system only at runtime)

## Architecture

### Entity Structure
```
Fire Entity (Parent)
├── Fire Component (fire data: intensity, size, particle_count)
├── Transform (position)
├── FireParticleSystem Component (particle state)
└── Child: PointLight (dynamic flickering light)
```

### Data Flow
1. Map file defines fire spawns with `entity_type: Fire`
2. Map loader parses fire entity data
3. Spawner creates fire entities with particle systems
4. Fire system updates particles and light each frame

## Implementation Steps

### Phase 1: Entity Type & Map Format (Priority: High)

#### Step 1.1: Add Fire EntityType

**File:** `src/systems/game/map/format.rs`

```rust
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityType {
    PlayerSpawn,
    Npc,
    Enemy,
    Item,
    Trigger,
    Fire,  // NEW
}
```

#### Step 1.2: Add Fire to Map Validation

**File:** `src/systems/game/map/validation.rs`

Ensure fire entities pass validation (no special requirements beyond position).

### Phase 2: Fire Components (Priority: High)

#### Step 2.1: Create Fire Component

**File:** `src/systems/game/components.rs`

```rust
/// Component for fire entities.
/// Fire entities emit light and render as animated particle effects.
#[derive(Component)]
pub struct Fire {
    /// Intensity multiplier for light and particle count (0.5 to 2.0)
    pub intensity: f32,
    /// Base size/radius of the fire effect
    pub size: f32,
    /// Color temperature: 0.0 = red/orange, 1.0 = white/blue
    pub temperature: f32,
}

impl Default for Fire {
    fn default() -> Self {
        Self {
            intensity: 1.0,
            size: 0.5,
            temperature: 0.3, // Warm orange by default
        }
    }
}
```

#### Step 2.2: Create Fire Particle System Component

**File:** `src/systems/game/components.rs`

```rust
/// Individual particle state for fire simulation
#[derive(Clone)]
pub struct FireParticle {
    /// Current position offset from fire origin
    pub position: Vec3,
    /// Current velocity
    pub velocity: Vec3,
    /// Remaining life (0.0 to max_life)
    pub life: f32,
    /// Maximum life for this particle
    pub max_life: f32,
}

/// Fire particle system state
#[derive(Component)]
pub struct FireParticleSystem {
    /// All particles in this fire
    pub particles: Vec<FireParticle>,
    /// Time accumulator for spawning
    pub spawn_timer: f32,
    /// Cached particle mesh handle
    pub mesh: Handle<Mesh>,
    /// Cached particle material handle  
    pub material: Handle<StandardMaterial>,
}
```

### Phase 3: Fire Spawning (Priority: High)

#### Step 3.1: Implement Fire Spawning

**File:** `src/systems/game/map/spawner.rs`

```rust
/// Spawn a fire entity with particle system and point light.
fn spawn_fire(
    ctx: &mut EntitySpawnContext,
    position: Vec3,
    properties: &HashMap<String, String>,
) {
    // Parse fire properties with defaults
    let intensity = properties
        .get("intensity")
        .and_then(|i| i.parse::<f32>().ok())
        .unwrap_or(1.0)
        .clamp(0.5, 2.0);

    let size = properties
        .get("size")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.5)
        .clamp(0.1, 2.0);

    let temperature = properties
        .get("temperature")
        .and_then(|t| t.parse::<f32>().ok())
        .unwrap_or(0.3)
        .clamp(0.0, 1.0);

    // Calculate particle count based on intensity
    let particle_count = ((500.0 * intensity) as usize).clamp(100, 2000);

    // Initialize particles
    let particles = initialize_fire_particles(particle_count, size);

    // Create particle mesh (small billboard quad)
    let mesh = ctx.meshes.add(create_fire_particle_mesh());

    // Create particle material with additive blending
    let material = ctx.materials.add(create_fire_material(temperature));

    // Spawn fire entity
    let fire_entity = ctx.commands.spawn((
        Transform::from_translation(position),
        Visibility::default(),
        Fire {
            intensity,
            size,
            temperature,
        },
        FireParticleSystem {
            particles,
            spawn_timer: 0.0,
            mesh: mesh.clone(),
            material: material.clone(),
        },
    )).id();

    // Spawn point light as child
    let light_color = fire_color_from_temperature(temperature);
    let light_intensity = 5000.0 * intensity; // Lumens

    ctx.commands.spawn((
        PointLight {
            color: light_color,
            intensity: light_intensity,
            radius: 10.0 * size,
            shadows_enabled: false, // Performance optimization
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, size * 0.5, 0.0)),
    )).set_parent(fire_entity);

    info!("Spawned fire at {:?} with intensity {}", position, intensity);
}

fn initialize_fire_particles(count: usize, size: f32) -> Vec<FireParticle> {
    let mut particles = Vec::with_capacity(count);
    let mut rng = || rand::random::<f32>();
    
    for _ in 0..count {
        particles.push(FireParticle {
            position: Vec3::new(
                (rng() - 0.5) * size * 0.4,
                rng() * size * 0.5,
                (rng() - 0.5) * size * 0.4,
            ),
            velocity: Vec3::new(
                (rng() - 0.5) * 0.5,
                rng() * 2.0 + 1.0, // Strong upward
                (rng() - 0.5) * 0.5,
            ),
            life: rng() * 0.5 + 0.5,
            max_life: rng() * 0.5 + 0.5,
        });
    }
    particles
}
```

#### Step 3.2: Update Entity Spawning Match

**File:** `src/systems/game/map/spawner.rs`

Add the `Fire` case to `spawn_entities`:

```rust
EntityType::Fire => {
    spawn_fire(ctx, Vec3::new(x, y, z), &entity_data.properties);
}
```

### Phase 4: Fire Particle System (Priority: High)

#### Step 4.1: Create Fire Systems Module

**File:** `src/systems/game/fire.rs` (new file)

```rust
//! Fire particle system simulation and rendering.

use super::components::{Fire, FireParticle, FireParticleSystem};
use bevy::prelude::*;

/// System to update fire particle physics each frame.
pub fn update_fire_particles(
    time: Res<Time>,
    mut fire_query: Query<(&Fire, &mut FireParticleSystem, &Transform)>,
) {
    let delta = time.delta_secs();

    for (fire, mut system, transform) in fire_query.iter_mut() {
        for particle in system.particles.iter_mut() {
            // Decay life
            particle.life -= delta;

            // Update position
            particle.position += particle.velocity * delta;

            // Reset dead or escaped particles
            if particle.life <= 0.0 || particle.position.y > fire.size * 2.5 {
                reset_particle(particle, fire.size);
            }
        }
    }
}

fn reset_particle(particle: &mut FireParticle, size: f32) {
    let rng = || rand::random::<f32>();
    
    // Reset to emitter base
    particle.position = Vec3::new(
        (rng() - 0.5) * size * 0.4,
        0.0,
        (rng() - 0.5) * size * 0.4,
    );
    
    // New upward velocity with turbulence
    particle.velocity = Vec3::new(
        (rng() - 0.5) * 0.5,
        rng() * 2.0 + 1.0,
        (rng() - 0.5) * 0.5,
    );
    
    // Reset life
    particle.max_life = rng() * 0.5 + 0.5;
    particle.life = particle.max_life;
}

/// System to update fire point light flickering.
pub fn update_fire_lights(
    time: Res<Time>,
    fire_query: Query<(&Fire, &Children)>,
    mut light_query: Query<&mut PointLight>,
) {
    let t = time.elapsed_secs();

    for (fire, children) in fire_query.iter() {
        for child in children.iter() {
            if let Ok(mut light) = light_query.get_mut(*child) {
                // Flicker effect using noise-like pattern
                let flicker = 0.8 + 0.4 * (t * 10.0).sin() * (t * 17.0).cos();
                light.intensity = 5000.0 * fire.intensity * flicker;
            }
        }
    }
}
```

#### Step 4.2: Fire Particle Rendering

**File:** `src/systems/game/fire.rs`

```rust
/// System to render fire particles using instanced meshes.
/// 
/// For performance, we use GPU instancing to render all particles
/// in a single draw call per fire entity.
pub fn render_fire_particles(
    mut commands: Commands,
    fire_query: Query<(Entity, &Fire, &FireParticleSystem, &Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_particles: Query<Entity, With<FireParticleMarker>>,
) {
    // Clear existing particle visuals (inefficient, see optimization notes)
    for entity in existing_particles.iter() {
        commands.entity(entity).despawn();
    }

    // Spawn particle visuals for each fire
    for (fire_entity, fire, system, transform) in fire_query.iter() {
        for particle in &system.particles {
            let life_ratio = (particle.life / particle.max_life).clamp(0.0, 1.0);
            
            // Color: fade from white-yellow to red-orange to transparent
            let color = fire_particle_color(life_ratio, fire.temperature);
            
            // Size: slightly smaller as particle ages
            let size = 0.1 * fire.size * (0.5 + 0.5 * life_ratio);
            
            // World position
            let world_pos = transform.translation + particle.position;

            commands.spawn((
                Mesh3d(system.mesh.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    emissive: color.into(),
                    alpha_mode: AlphaMode::Add, // Additive blending
                    unlit: true,
                    ..default()
                })),
                Transform::from_translation(world_pos)
                    .with_scale(Vec3::splat(size)),
                FireParticleMarker,
            ));
        }
    }
}

#[derive(Component)]
pub struct FireParticleMarker;

fn fire_particle_color(life_ratio: f32, temperature: f32) -> Color {
    // Interpolate based on life and temperature
    // life_ratio: 1.0 = just spawned, 0.0 = about to die
    // temperature: 0.0 = red/orange, 1.0 = white/blue
    
    let base_red = 1.0;
    let base_green = 0.3 + life_ratio * 0.5 + temperature * 0.2;
    let base_blue = temperature * life_ratio * 0.5;
    let alpha = life_ratio * 0.8;
    
    Color::srgba(base_red, base_green, base_blue, alpha)
}
```

### Phase 5: Editor Integration (Priority: Medium)

#### Step 5.1: Add Fire to Editor Toolbar

**File:** `src/editor/ui/toolbar.rs`

```rust
// In render_tool_options, EntityPlace match arm:
EditorTool::EntityPlace { entity_type } => {
    ui.label("Entity:");
    egui::ComboBox::from_id_salt("toolbar_entity_type")
        .selected_text(entity_type_display(entity_type))
        .width(120.0)
        .show_ui(ui, |ui| {
            ui.selectable_value(entity_type, EntityType::PlayerSpawn, "🟢 Player Spawn");
            ui.selectable_value(entity_type, EntityType::Npc, "🔵 NPC");
            ui.selectable_value(entity_type, EntityType::Enemy, "🔴 Enemy");
            ui.selectable_value(entity_type, EntityType::Item, "🟡 Item");
            ui.selectable_value(entity_type, EntityType::Trigger, "🟣 Trigger");
            ui.selectable_value(entity_type, EntityType::Fire, "🔥 Fire");  // NEW
        });
}

// Update entity_type_display function:
fn entity_type_display(entity_type: &EntityType) -> &'static str {
    match entity_type {
        EntityType::PlayerSpawn => "🟢 Player Spawn",
        EntityType::Npc => "🔵 NPC",
        EntityType::Enemy => "🔴 Enemy",
        EntityType::Item => "🟡 Item",
        EntityType::Trigger => "🟣 Trigger",
        EntityType::Fire => "🔥 Fire",  // NEW
    }
}
```

#### Step 5.2: Add Fire Marker to Editor Renderer

**File:** `src/editor/renderer.rs`

```rust
// In render_entities_system, update the color/size match:
let (color, size) = match entity_data.entity_type {
    EntityType::PlayerSpawn => (Color::srgba(0.0, 1.0, 0.0, 0.8), 0.4),
    EntityType::Npc => (Color::srgba(0.0, 0.5, 1.0, 0.8), 0.35),
    EntityType::Enemy => (Color::srgba(1.0, 0.0, 0.0, 0.8), 0.35),
    EntityType::Item => (Color::srgba(1.0, 1.0, 0.0, 0.8), 0.25),
    EntityType::Trigger => (Color::srgba(1.0, 0.0, 1.0, 0.5), 0.5),
    EntityType::Fire => (Color::srgba(1.0, 0.5, 0.0, 0.9), 0.35),  // NEW: Orange flame color
};
```

#### Step 5.3: Add Fire Properties Panel (Optional)

**File:** `src/editor/ui/properties.rs`

Add a properties panel for fire entities when selected, allowing adjustment of:
- Intensity (slider 0.5-2.0)
- Size (slider 0.1-2.0)
- Temperature (slider 0.0-1.0)

### Phase 6: Optimization (Priority: Low, Post-MVP)

#### Step 6.1: GPU Instancing

Replace per-particle spawning with GPU instancing for better performance:
- Use `bevy_hanabi` crate for GPU-accelerated particles
- Or implement custom instance buffer with compute shaders

#### Step 6.2: Particle Pooling

Implement object pooling for particle entities:
- Pre-spawn all particle entities
- Toggle visibility instead of spawn/despawn
- Reduces allocation overhead

#### Step 6.3: LOD System

Reduce particle count based on camera distance:
- Full particles within 20 units
- 50% particles at 20-50 units
- 25% particles at 50+ units
- Disable particles beyond 100 units (keep light only)

## File Changes Summary

| File | Action | Description |
|------|--------|-------------|
| `src/systems/game/map/format.rs` | Modify | Add `Fire` to `EntityType` enum |
| `src/systems/game/components.rs` | Modify | Add `Fire`, `FireParticle`, `FireParticleSystem` components |
| `src/systems/game/map/spawner.rs` | Modify | Add `spawn_fire` function and match case |
| `src/systems/game/fire.rs` | Create | New fire particle system module |
| `src/systems/game/mod.rs` | Modify | Add `fire` module declaration |
| `src/systems/game/systems.rs` | Modify | Re-export fire systems |
| `src/editor/ui/toolbar.rs` | Modify | Add Fire to entity type dropdown |
| `src/editor/renderer.rs` | Modify | Add Fire marker rendering |
| `src/editor/ui/properties.rs` | Modify | Add Fire properties panel (optional) |
| `Cargo.toml` | Modify | Add `rand` dependency for particle randomization |

## Dependencies

### Required
- `rand = "0.8"` - For particle randomization

### Optional (Performance Optimization)
- `bevy_hanabi = "0.15"` - GPU particle system (if Bevy 0.15 compatible)

## Testing Plan

### Unit Tests
1. Fire particle reset logic
2. Fire color interpolation
3. Fire property parsing

### Integration Tests
1. Fire entity spawning from map file
2. Multiple fires in same scene
3. Fire persistence through save/load

### Manual Tests
1. Place fire in editor, verify marker appears
2. Load map with fire, verify particles render
3. Verify light flickering effect
4. Test performance with 10+ fires

## Timeline Estimate

| Phase | Estimated Time |
|-------|---------------|
| Phase 1: Entity Type | 1 hour |
| Phase 2: Components | 1 hour |
| Phase 3: Spawning | 2 hours |
| Phase 4: Particle System | 4 hours |
| Phase 5: Editor Integration | 2 hours |
| Phase 6: Optimization | 4+ hours |
| **Total (MVP)** | **~10 hours** |

## Future Enhancements

1. **Fire Spreading**: Fire can spread to adjacent flammable voxels
2. **Smoke Particles**: Secondary particle system for smoke above flames
3. **Audio**: Crackling fire sound effect
4. **Interaction**: Player takes damage when touching fire
5. **Water Interaction**: Fire extinguished by water entities
6. **Day/Night Impact**: Fire more prominent at night with bloom effect
