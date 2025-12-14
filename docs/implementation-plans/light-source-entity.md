# Light Source Entity Implementation Plan

## Overview

Implement a point light entity that emits light in all directions (spherical/omnidirectional). Light sources can be placed in maps via the map editor and will illuminate the environment at runtime using Bevy's built-in `PointLight` component.

## Requirements

### Functional Requirements
1. Light sources can be placed in maps via the map editor (EntityType::LightSource)
2. Light emits uniformly in all directions (spherical pattern)
3. Light has configurable properties (color, intensity, range, shadows)
4. Multiple light sources can exist in a scene simultaneously
5. Light sources are saved/loaded with the map

### Non-Functional Requirements
1. Efficient rendering using Bevy's native lighting system
2. Editor shows visual indicator at light position
3. Minimal performance impact when multiple lights are present

## Architecture

### Entity Structure
```
LightSource Entity (Parent)
├── LightSource Component (light data: color, intensity, range, shadows_enabled)
├── Transform (position)
└── PointLight (Bevy's built-in point light)
```

### Data Flow
1. Map file defines light spawns with `entity_type: LightSource`
2. Map loader parses light entity data
3. Spawner creates light entities with PointLight component
4. Bevy's rendering system handles lighting automatically

## Implementation Steps

### Phase 1: Entity Type & Map Format (Priority: High)

#### Step 1.1: Add LightSource EntityType

**File:** `src/systems/game/map/format.rs`

Add `LightSource` variant to the `EntityType` enum:

```rust
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityType {
    /// Player spawn point
    PlayerSpawn,
    /// NPC spawn point (static, non-moving characters)
    Npc,
    /// Enemy spawn point
    Enemy,
    /// Item spawn point
    Item,
    /// Trigger volume
    Trigger,
    /// Point light source (omnidirectional)
    LightSource,  // NEW
}
```

### Phase 2: Light Source Component (Priority: High)

#### Step 2.1: Create LightSource Component

**File:** `src/systems/game/components.rs`

```rust
/// Component for light source entities.
/// Light sources emit light in all directions (point light / omnidirectional).
#[derive(Component)]
pub struct LightSource {
    /// Light color (RGB, 0.0-1.0)
    pub color: Color,
    /// Light intensity in lumens (typical range: 100-10000)
    pub intensity: f32,
    /// Maximum range/radius of the light in world units
    pub range: f32,
    /// Whether this light casts shadows
    pub shadows_enabled: bool,
}

impl Default for LightSource {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1000.0,
            range: 10.0,
            shadows_enabled: false, // Disabled by default for performance
        }
    }
}
```

### Phase 3: Light Source Spawning (Priority: High)

#### Step 3.1: Implement Light Source Spawning

**File:** `src/systems/game/map/spawner.rs`

Add the `spawn_light_source` function:

```rust
/// Spawn a light source entity with a point light.
///
/// Light sources emit light uniformly in all directions (spherical).
/// Properties can customize color, intensity, range, and shadow casting.
fn spawn_light_source(
    ctx: &mut EntitySpawnContext,
    position: Vec3,
    properties: &std::collections::HashMap<String, String>,
) {
    // Parse light properties with defaults
    let intensity = properties
        .get("intensity")
        .and_then(|i| i.parse::<f32>().ok())
        .unwrap_or(1000.0)
        .clamp(0.0, 100000.0);

    let range = properties
        .get("range")
        .and_then(|r| r.parse::<f32>().ok())
        .unwrap_or(10.0)
        .clamp(0.1, 100.0);

    let shadows_enabled = properties
        .get("shadows")
        .map(|s| s == "true" || s == "1")
        .unwrap_or(false);

    // Parse color (format: "r,g,b" with values 0.0-1.0)
    let color = properties
        .get("color")
        .and_then(|c| {
            let parts: Vec<f32> = c.split(',')
                .filter_map(|p| p.trim().parse().ok())
                .collect();
            if parts.len() == 3 {
                Some(Color::srgb(parts[0], parts[1], parts[2]))
            } else {
                None
            }
        })
        .unwrap_or(Color::WHITE);

    // Spawn light source entity
    ctx.commands.spawn((
        Transform::from_translation(position),
        Visibility::default(),
        LightSource {
            color,
            intensity,
            range,
            shadows_enabled,
        },
        PointLight {
            color,
            intensity,
            range,
            radius: 0.0, // Point light (no physical size)
            shadows_enabled,
            ..default()
        },
    ));

    info!(
        "Spawned light source at {:?} (intensity: {}, range: {}, shadows: {})",
        position, intensity, range, shadows_enabled
    );
}
```

#### Step 3.2: Update Entity Spawning Match

**File:** `src/systems/game/map/spawner.rs`

Add the `LightSource` case to the `spawn_entities` function's match statement:

```rust
EntityType::LightSource => {
    spawn_light_source(ctx, Vec3::new(x, y, z), &entity_data.properties);
}
```

### Phase 4: Editor Integration (Priority: Medium)

#### Step 4.1: Add LightSource to Editor Toolbar

**File:** `src/editor/ui/toolbar.rs`

In the `render_tool_options` function, add LightSource to the entity type dropdown:

```rust
// In the EntityPlace match arm's ComboBox show_ui closure:
changed |= ui
    .selectable_value(entity_type, EntityType::LightSource, "💡 Light Source")
    .changed();
```

Update the `entity_type_display` function:

```rust
fn entity_type_display(entity_type: &EntityType) -> &'static str {
    match entity_type {
        EntityType::PlayerSpawn => "🟢 Player Spawn",
        EntityType::Npc => "🔵 NPC",
        EntityType::Enemy => "🔴 Enemy",
        EntityType::Item => "🟡 Item",
        EntityType::Trigger => "🟣 Trigger",
        EntityType::LightSource => "💡 Light Source",  // NEW
    }
}
```

#### Step 4.2: Add LightSource Marker to Editor Renderer

**File:** `src/editor/renderer.rs`

In the `render_entities_system` function, add LightSource to the color/size match:

```rust
let (color, size) = match entity_data.entity_type {
    EntityType::PlayerSpawn => (Color::srgba(0.0, 1.0, 0.0, 0.8), 0.4),
    EntityType::Npc => (Color::srgba(0.0, 0.5, 1.0, 0.8), 0.35),
    EntityType::Enemy => (Color::srgba(1.0, 0.0, 0.0, 0.8), 0.35),
    EntityType::Item => (Color::srgba(1.0, 1.0, 0.0, 0.8), 0.25),
    EntityType::Trigger => (Color::srgba(1.0, 0.0, 1.0, 0.5), 0.5),
    EntityType::LightSource => (Color::srgba(1.0, 1.0, 0.8, 0.9), 0.3),  // NEW: Warm yellow/white
};
```

### Phase 5: Properties Panel (Priority: Low, Optional)

#### Step 5.1: Add Light Properties Panel

**File:** `src/editor/ui/properties.rs`

Add a properties panel for light source entities when selected, allowing adjustment of:
- Color (color picker or RGB sliders)
- Intensity (slider 0-10000 lumens)
- Range (slider 0.1-100 units)
- Shadows enabled (checkbox)

```rust
// When a LightSource entity is selected:
if let Some(selected_idx) = editor_state.selected_entities.first() {
    if let Some(entity_data) = editor_state.current_map.entities.get_mut(*selected_idx) {
        if entity_data.entity_type == EntityType::LightSource {
            ui.heading("Light Source Properties");
            
            // Intensity slider
            let mut intensity: f32 = entity_data.properties
                .get("intensity")
                .and_then(|i| i.parse().ok())
                .unwrap_or(1000.0);
            if ui.add(egui::Slider::new(&mut intensity, 0.0..=10000.0).text("Intensity")).changed() {
                entity_data.properties.insert("intensity".to_string(), intensity.to_string());
            }
            
            // Range slider
            let mut range: f32 = entity_data.properties
                .get("range")
                .and_then(|r| r.parse().ok())
                .unwrap_or(10.0);
            if ui.add(egui::Slider::new(&mut range, 0.1..=100.0).text("Range")).changed() {
                entity_data.properties.insert("range".to_string(), range.to_string());
            }
            
            // Shadows checkbox
            let mut shadows = entity_data.properties
                .get("shadows")
                .map(|s| s == "true")
                .unwrap_or(false);
            if ui.checkbox(&mut shadows, "Cast Shadows").changed() {
                entity_data.properties.insert("shadows".to_string(), shadows.to_string());
            }
        }
    }
}
```

## File Changes Summary

| File | Action | Description |
|------|--------|-------------|
| `src/systems/game/map/format.rs` | Modify | Add `LightSource` to `EntityType` enum |
| `src/systems/game/components.rs` | Modify | Add `LightSource` component |
| `src/systems/game/map/spawner.rs` | Modify | Add `spawn_light_source` function and match case |
| `src/editor/ui/toolbar.rs` | Modify | Add LightSource to entity type dropdown and display function |
| `src/editor/renderer.rs` | Modify | Add LightSource marker rendering |
| `src/editor/ui/properties.rs` | Modify | Add LightSource properties panel (optional) |

## Light Source Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `color` | String (r,g,b) | "1.0,1.0,1.0" | Light color in RGB (0.0-1.0 per channel) |
| `intensity` | String (float) | "1000.0" | Light intensity in lumens |
| `range` | String (float) | "10.0" | Maximum light range in world units |
| `shadows` | String (bool) | "false" | Whether light casts shadows |

## Map Format Example

```ron
entities: [
    (
        entity_type: PlayerSpawn,
        position: (2.5, 1.0, 2.5),
        properties: {},
    ),
    (
        entity_type: LightSource,
        position: (5.0, 3.0, 5.0),
        properties: {
            "color": "1.0,0.9,0.8",
            "intensity": "2000.0",
            "range": "15.0",
            "shadows": "true",
        },
    ),
    (
        entity_type: LightSource,
        position: (10.0, 2.0, 8.0),
        properties: {
            "intensity": "500.0",
            "range": "5.0",
        },
    ),
]
```

## Testing Checklist

- [ ] LightSource spawns at correct position from map file
- [ ] Light illuminates surrounding geometry in all directions
- [ ] Light color property works correctly
- [ ] Light intensity scales appropriately
- [ ] Light range limits illumination distance
- [ ] Shadow casting works when enabled
- [ ] Multiple light sources can coexist
- [ ] Editor shows light marker at correct position
- [ ] Editor toolbar includes LightSource option
- [ ] Default values work when properties are omitted
- [ ] Map save/load preserves light properties

## Performance Considerations

1. **Shadow Casting**: Shadows are disabled by default (`shadows_enabled: false`) because point light shadows are computationally expensive. Each shadow-casting point light requires 6 shadow map renders (one per cube face).

2. **Light Count**: Bevy has a default limit of ~256 lights per scene. For scenes with many lights, consider:
   - Using lower intensity and smaller range
   - Disabling shadows on most lights
   - Grouping lights in areas

3. **Range Optimization**: Keep `range` as small as practical. Bevy skips lighting calculations for fragments outside the light's range.

## Future Enhancements (Out of Scope)

1. **Spotlight Support**: Add `EntityType::Spotlight` for directional cone lights
2. **Light Animation**: Flickering, pulsing, or color-cycling effects
3. **Light Volumes**: Area lights for soft shadows
4. **Light Groups**: Enable/disable groups of lights together
5. **Baked Lighting**: Pre-compute lighting for static lights
6. **IES Profiles**: Support photometric light distribution profiles

## Estimated Effort

| Phase | Estimated Time |
|-------|---------------|
| Phase 1: Entity Type | 15 minutes |
| Phase 2: Component | 15 minutes |
| Phase 3: Spawning | 30 minutes |
| Phase 4: Editor Integration | 30 minutes |
| Phase 5: Properties Panel (optional) | 45 minutes |
| **Total (MVP)** | **~1.5 hours** |
| **Total (with properties panel)** | **~2.5 hours** |
