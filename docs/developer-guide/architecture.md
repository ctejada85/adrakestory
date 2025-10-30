# Architecture Overview

Complete guide to the system architecture and design of A Drake's Story.

## Table of Contents

- [Overview](#overview)
- [Technology Stack](#technology-stack)
- [ECS Architecture](#ecs-architecture)
- [Game States](#game-states)
- [System Organization](#system-organization)
- [Core Components](#core-components)
- [Resources](#resources)
- [Module Structure](#module-structure)
- [Data Flow](#data-flow)
- [Design Patterns](#design-patterns)

## Overview

A Drake's Story is built using the **Entity Component System (ECS)** architecture pattern via the Bevy game engine. This architecture provides:

- **Modularity**: Systems are independent and composable
- **Performance**: Data-oriented design for cache efficiency
- **Flexibility**: Easy to add/remove features
- **Maintainability**: Clear separation of concerns

## Technology Stack

### Core Technologies

- **Language**: Rust 2021 Edition
- **Game Engine**: Bevy 0.15
- **Build System**: Cargo
- **Serialization**: Serde + RON

### Key Dependencies

```toml
[dependencies]
bevy = { version = "0.15", features = ["bevy_gltf"] }  # Game engine with GLTF support
serde = { version = "1.0", features = ["derive"] }     # Serialization
ron = "0.8"                                            # Map format
thiserror = "1.0"                                      # Error handling
```

### Development Tools

- **Debugger**: CodeLLDB (VSCode)
- **Linter**: Clippy
- **Formatter**: rustfmt
- **Documentation**: rustdoc

## ECS Architecture

### Entity Component System Basics

**Entities**: Unique identifiers for game objects
```rust
// Examples: player, voxel, camera, UI element
let entity = commands.spawn(/* components */);
```

**Components**: Data attached to entities
```rust
#[derive(Component)]
pub struct Player {
    pub velocity: Vec3,
    pub is_grounded: bool,
    pub radius: f32,
}
```

**Systems**: Logic that operates on components
```rust
fn player_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Player)>,
) {
    // Update player positions
}
```

### Bevy's ECS Features

- **Queries**: Efficient component access
- **Resources**: Global state
- **Events**: Communication between systems
- **States**: Game state management
- **Schedules**: System execution order

## Game States

The game uses a finite state machine for flow control:

```rust
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    IntroAnimation,  // Opening splash screen
    TitleScreen,     // Main menu
    LoadingMap,      // Map loading with progress
    InGame,          // Active gameplay
    Paused,          // Pause menu
    Settings,        // Settings menu (planned)
}
```

### State Transitions

```
IntroAnimation → TitleScreen → LoadingMap → InGame ⇄ Paused
                                                ↓
                                          TitleScreen
```

### State-Specific Systems

Systems can be configured to run only in specific states:

```rust
app.add_systems(OnEnter(GameState::InGame), setup_game)
   .add_systems(Update, player_movement.run_if(in_state(GameState::InGame)))
   .add_systems(OnExit(GameState::InGame), cleanup_game);
```

### Camera Management Across States

The game uses different cameras for different states to prevent rendering conflicts:

- **2D Camera**: Used for UI-only states (IntroAnimation, TitleScreen, LoadingMap, Paused)
  - Spawned at startup in [`setup()`](../../src/main.rs:121)
  - Automatically despawned when entering InGame state via [`cleanup_2d_camera()`](../../src/main.rs:127)

- **3D Camera**: Used for gameplay (InGame state)
  - Spawned when entering InGame state in [`spawn_camera()`](../../src/systems/game/map/spawner.rs:224)
  - Includes [`GameCamera`](../../src/systems/game/components.rs:38) component for rotation control

This separation prevents camera order ambiguity warnings and ensures only one camera is active per render target at any time.

## System Organization

### Module Hierarchy

```
src/
├── main.rs                 # Application entry point
├── states.rs               # Game state definitions
├── systems/
│   ├── mod.rs              # Systems module root
│   ├── game/               # Core gameplay systems
│   │   ├── mod.rs
│   │   ├── components.rs   # Game components
│   │   ├── resources.rs    # Game resources
│   │   ├── systems.rs      # System re-exports
│   │   ├── camera.rs       # Camera control
│   │   ├── character/      # Character model system
│   │   │   └── mod.rs      # CharacterModel component
│   │   ├── collision.rs    # Collision detection
│   │   ├── input.rs        # Input handling
│   │   ├── physics.rs      # Physics simulation
│   │   ├── player_movement.rs  # Player controls
│   │   └── map/            # Map loading system
│   │       ├── mod.rs
│   │       ├── format.rs   # Map data structures
│   │       ├── loader.rs   # Map file loading
│   │       ├── spawner.rs  # World instantiation
│   │       ├── validation.rs  # Map validation
│   │       └── error.rs    # Error types
│   ├── intro_animation/    # Intro screen
│   ├── title_screen/       # Title screen
│   ├── loading_screen/     # Loading UI
│   └── pause_menu/         # Pause menu
└── components/             # Shared components
```

### System Categories

**1. Game Systems** (`systems/game/`)
- Core gameplay logic
- Character model management
- Physics and collision
- Player movement
- Camera control
- Map loading

**2. UI Systems** (`systems/*/`)
- Menu interfaces
- Loading screens
- HUD (planned)

**3. Rendering Systems** (Bevy built-in)
- 3D rendering
- Lighting
- Camera projection

## Core Components

### Player Component

```rust
#[derive(Component)]
pub struct Player {
    pub speed: f32,          // Movement speed
    pub velocity: Vec3,      // Current velocity
    pub is_grounded: bool,   // On ground?
    pub radius: f32,         // Collision radius (0.3)
}
```

**Purpose**: Represents the player character with physics state. The visual representation is handled separately by the [`CharacterModel`](#charactermodel-component) component.

### CharacterModel Component

```rust
#[derive(Component)]
pub struct CharacterModel {
    pub scene_handle: Handle<Scene>,  // GLB/GLTF scene handle
    pub scale: f32,                   // Model scale factor
    pub offset: Vec3,                 // Position offset
}
```

**Purpose**: Manages the player's 3D visual model (GLB/GLTF format). The model is spawned as a child entity of the player, separating visuals from physics. See [Character System](systems/character-system.md) for details.

### Voxel Component

```rust
#[derive(Component)]
pub struct Voxel;
```

**Purpose**: Marker component for voxel entities.

### SubVoxel Component

```rust
#[derive(Component)]
pub struct SubVoxel {
    pub bounds: (Vec3, Vec3),  // (min, max) AABB bounds
}
```

**Purpose**: Represents individual sub-voxels within a voxel (8×8×8 grid). The bounds are pre-calculated and cached at spawn time for efficient collision detection, eliminating the need to compute them during physics updates. See [`physics-analysis.md`](systems/physics-analysis.md) for optimization details.

### GameCamera Component

```rust
#[derive(Component)]
pub struct GameCamera {
    pub original_rotation: Quat,  // Original rotation quaternion
    pub target_rotation: Quat,    // Target rotation for smooth transitions
    pub rotation_speed: f32,      // Speed of rotation interpolation
}
```

**Purpose**: Camera control and rotation state with smooth interpolation.

### CollisionBox Component

```rust
#[derive(Component)]
pub struct CollisionBox;
```

**Purpose**: Debug visualization for collision boundaries.

## Resources

### LoadedMapData

```rust
#[derive(Resource)]
pub struct LoadedMapData {
    pub map: MapData,
}
```

**Purpose**: Stores the currently loaded map data.

### MapLoadProgress

```rust
#[derive(Resource, Default)]
pub struct MapLoadProgress {
    pub current: Option<LoadProgress>,
    pub events: Vec<LoadProgress>,
}
```

**Purpose**: Tracks map loading progress for UI display.

### SpatialGrid

```rust
#[derive(Resource)]
pub struct SpatialGrid {
    cells: HashMap<IVec3, Vec<Entity>>,
    cell_size: f32,
}
```

**Purpose**: Spatial partitioning for efficient collision detection.

### GameInitialized

```rust
#[derive(Resource, Default)]
pub struct GameInitialized(pub bool);
```

**Purpose**: Prevents duplicate world setup. Tuple struct for simple boolean flag.

## Module Structure

### Game Module (`systems/game/`)

**Responsibilities:**
- Core gameplay logic
- Character model management
- Physics simulation
- Collision detection
- Player movement
- Camera control
- Map loading

**Key Files:**
- `components.rs`: Game-specific components
- `resources.rs`: Game-specific resources
- `systems.rs`: System function re-exports
- `camera.rs`: Camera control system
- `character/mod.rs`: Character model component
- `collision.rs`: Collision detection
- `input.rs`: Input handling
- `physics.rs`: Physics simulation
- `player_movement.rs`: Player controls

### Map Module (`systems/game/map/`)

**Responsibilities:**
- Map file loading
- Map validation
- World spawning
- Progress tracking

**Key Files:**
- `format.rs`: Map data structures
- `loader.rs`: File I/O and parsing
- `spawner.rs`: Entity instantiation
- `validation.rs`: Map validation
- `error.rs`: Error types

### UI Modules

Each UI module follows a consistent pattern:

```
module/
├── mod.rs          # Module exports
├── components.rs   # UI components
├── resources.rs    # UI resources (optional)
└── systems.rs      # UI systems
```

## Data Flow

### Startup Flow

```
1. main() → App initialization
2. Setup systems run
3. IntroAnimation state entered
4. Intro systems run
5. Transition to TitleScreen
```

### Game Start Flow

```
1. User clicks "New Game"
2. Transition to LoadingMap state
3. Map loader starts
4. Progress updates emitted
5. Map validated
6. World spawned
7. Transition to InGame state
```

### Game Loop Flow

```
1. Input systems read controls
2. Physics systems update velocities
3. Collision systems check boundaries
4. Movement systems update positions
5. Camera systems follow player
6. Rendering systems draw frame
```

### Pause Flow

```
1. User presses ESC
2. Transition to Paused state
3. Game systems stop
4. Pause menu systems run
5. User selects option
6. Transition back or to TitleScreen
```

## Design Patterns

### State Pattern

Game states control system execution:

```rust
app.add_systems(
    Update,
    player_movement.run_if(in_state(GameState::InGame))
);
```

### Observer Pattern

Events for system communication:

```rust
fn emit_event(mut events: EventWriter<CustomEvent>) {
    events.send(CustomEvent { /* ... */ });
}

fn handle_event(mut events: EventReader<CustomEvent>) {
    for event in events.read() {
        // Handle event
    }
}
```

### Component Pattern

Data-oriented design with components:

```rust
#[derive(Component)]
struct Health(f32);

#[derive(Component)]
struct Damage(f32);

fn damage_system(
    mut query: Query<&mut Health>,
    damage_query: Query<&Damage>,
) {
    // Apply damage to health
}
```

### Resource Pattern

Global state management:

```rust
#[derive(Resource)]
struct GameSettings {
    volume: f32,
    difficulty: Difficulty,
}
```

### Builder Pattern

Map creation:

```rust
let map = MapData {
    metadata: MapMetadata { /* ... */ },
    world: WorldData { /* ... */ },
    // ...
};
```

## Performance Considerations

### Spatial Partitioning

The `SpatialGrid` resource divides the world into cells for efficient collision queries:

```rust
// O(n²) → O(n) for collision checks
let nearby = spatial_grid.query_cell(position);
```

### Sub-Voxel Rendering

Each voxel contains 8×8×8 sub-voxels for high detail without excessive entity count.

### Build Profiles

Optimized profiles balance compilation speed and runtime performance:

```toml
[profile.dev]
opt-level = 1              # Slight optimization

[profile.dev.package."*"]
opt-level = 3              # Full optimization for deps
```

## Extension Points

### Adding New Systems

1. Create system function
2. Add to appropriate module
3. Register in `main.rs`
4. Configure state conditions

### Adding New Components

1. Define component struct
2. Derive `Component` trait
3. Add to `components.rs`
4. Use in systems

### Adding New States

1. Add variant to `GameState` enum
2. Create state-specific systems
3. Define transitions
4. Add UI if needed

### Adding New Entity Types

1. Define in `map/format.rs`
2. Add spawning logic in `map/spawner.rs`
3. Create component if needed
4. Add systems for behavior

## Best Practices

### System Design

1. **Single Responsibility**: Each system does one thing
2. **Query Efficiency**: Use specific queries
3. **State Conditions**: Run systems only when needed
4. **Error Handling**: Use `Result` types

### Component Design

1. **Data Only**: Components are pure data
2. **Small**: Keep components focused
3. **Composable**: Combine for complex behavior
4. **Derive Traits**: Use `#[derive]` when possible

### Resource Design

1. **Global State**: Only for truly global data
2. **Initialization**: Use `Default` or `FromWorld`
3. **Access**: Minimize mutable access
4. **Lifetime**: Consider cleanup

## Testing Strategy

### Unit Tests

Test individual functions:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_detection() {
        // Test collision logic
    }
}
```

### Integration Tests

Test system interactions:

```rust
#[test]
fn test_player_movement() {
    let mut app = App::new();
    // Setup and test
}
```

### Manual Testing

- Test in debug mode
- Use collision visualization
- Test all game states
- Verify map loading

## Documentation

### Code Documentation

```rust
/// Handles player movement based on input.
///
/// # Arguments
/// * `time` - Game time resource
/// * `input` - Keyboard input resource
/// * `query` - Query for player entities
pub fn player_movement_system(/* ... */) {
    // Implementation
}
```

### Module Documentation

```rust
//! # Player Movement Module
//!
//! Handles all player movement logic including:
//! - Keyboard input processing
//! - Velocity calculation
//! - Ground detection
```

## Related Documentation

- **[Debugging Guide](debugging.md)** - Debug setup and tips
- **[Contributing Guide](contributing.md)** - Contribution workflow
- **[Character System](systems/character-system.md)** - Character model management
- **[Map Loader System](systems/map-loader.md)** - Map system details
- **[Map Editor Documentation](systems/map-editor/README.md)** - Complete map editor guide

---

**Architecture Version:** 1.0.0
**Last Updated:** 2025-10-22