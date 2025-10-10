# A Drake's Story

A 3D voxel-based game built with Rust and the Bevy game engine, featuring sub-voxel rendering, physics simulation, and an isometric camera view.

## Overview

A Drake's Story is an experimental 3D game that explores voxel-based world generation with high-resolution sub-voxel rendering. The game features a physics-driven player character navigating through a procedurally structured world with various terrain types including staircases, platforms, and pillars.

## Features

### Gameplay
- **3D Voxel World**: Explore a voxel-based environment with detailed sub-voxel rendering (8×8×8 sub-voxels per voxel)
- **Physics System**: Realistic gravity and collision detection for smooth player movement
- **Varied Terrain**: Navigate through different structure types:
  - Progressive staircases with increasing height
  - Thin platforms for precise jumping
  - Small centered pillars for challenging navigation
  - Full solid blocks for stable ground
- **Isometric Camera**: Dynamic camera with rotation controls for better spatial awareness

### Technical Features
- **Map Loader System**: Load custom maps from RON files with progress tracking
  - Human-readable RON (Rusty Object Notation) format
  - Support for voxels, entities, lighting, and camera configuration
  - Real-time loading progress with visual feedback
  - Map validation and error handling
- **State Management**: Clean game flow through multiple states (Intro Animation → Title Screen → Loading Map → In-Game → Pause Menu)
- **Spatial Grid Optimization**: Efficient collision detection using spatial partitioning
- **Modular Architecture**: Well-organized system modules for maintainability
- **Debug Tools**: Collision box visualization for development and testing

## Technical Stack

- **Language**: Rust (2021 Edition)
- **Game Engine**: [Bevy](https://bevyengine.org/) 0.15
- **Architecture**: Entity Component System (ECS)
- **Build System**: Cargo with optimized profiles

### Dependencies

```toml
bevy = "0.15"
serde = { version = "1.0", features = ["derive"] }
ron = "0.8"
thiserror = "1.0"
```

## Getting Started

### Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **Cargo**: Comes with Rust installation
- **Platform-specific requirements**:
  - **macOS**: Xcode Command Line Tools
  - **Linux**: Development libraries (see [Bevy setup guide](https://bevyengine.org/learn/book/getting-started/setup/))
  - **Windows**: Visual Studio Build Tools

### Building

```bash
# Clone the repository
git clone <repository-url>
cd adrakestory

# Build in debug mode (faster compilation, slower runtime)
cargo build

# Build in release mode (optimized for performance)
cargo build --release
```

### Running

```bash
# Run in debug mode
cargo run

# Run in release mode (recommended for better performance)
cargo run --release
```

## Game Controls

### Movement
- **W** - Move forward
- **A** - Move left
- **S** - Move backward
- **D** - Move right

### Camera
- **Mouse Movement** - Rotate camera view
- **Q/E** - Rotate camera around player (alternative)

### System
- **ESC** - Pause game / Return to menu
- **C** - Toggle collision box visualization (debug)

### Menu Navigation
- **Arrow Keys** / **Tab** - Navigate menu options
- **Enter** / **Space** - Select menu option
- **Mouse Click** - Select menu buttons

## Project Structure

```
adrakestory/
├── src/
│   ├── main.rs                 # Application entry point
│   ├── states.rs               # Game state definitions
│   ├── systems/
│   │   ├── mod.rs              # Systems module root
│   │   ├── game/               # Core gameplay systems
│   │   │   ├── mod.rs
│   │   │   ├── components.rs  # Game entity components
│   │   │   ├── resources.rs   # Game resources
│   │   │   ├── systems.rs     # System re-exports
│   │   │   ├── camera.rs      # Camera control
│   │   │   ├── collision.rs   # Collision detection
│   │   │   ├── input.rs       # Input handling
│   │   │   ├── physics.rs     # Physics simulation
│   │   │   ├── player_movement.rs  # Player controls
│   │   │   ├── world_generation.rs # Legacy world setup
│   │   │   └── map/           # Map loading system
│   │   │       ├── mod.rs     # Map module exports
│   │   │       ├── format.rs  # Map data structures
│   │   │       ├── loader.rs  # Map file loading
│   │   │       ├── spawner.rs # World instantiation
│   │   │       ├── validation.rs # Map validation
│   │   │       └── error.rs   # Error types
│   │   ├── intro_animation/   # Intro screen systems
│   │   ├── title_screen/      # Title screen systems
│   │   ├── loading_screen/    # Map loading UI
│   │   └── pause_menu/        # Pause menu systems
│   └── components/            # Shared components
├── assets/
│   ├── maps/                  # Map files (RON format)
│   │   ├── default.ron        # Default game map
│   │   └── simple_test.ron    # Minimal test map
│   ├── audio/                 # Sound effects and music
│   ├── fonts/                 # UI fonts
│   └── textures/              # Textures and sprites
├── Cargo.toml                 # Project dependencies
├── MAP_LOADER_DESIGN.md      # Map system documentation
├── DEBUG_SETUP.md            # VSCode debugging guide
└── README.md                 # This file
```

## Development

### Architecture

The project follows a modular ECS architecture with clear separation of concerns:

#### Game States
- **IntroAnimation**: Opening splash screen with fade effects
- **TitleScreen**: Main menu with keyboard and mouse navigation
- **LoadingMap**: Map loading with progress indicator
- **InGame**: Active gameplay with physics and controls
- **Paused**: Pause menu overlay
- **Settings**: Configuration screen (planned)

#### System Organization
- **Game Systems**: Core gameplay logic (physics, collision, movement)
- **UI Systems**: User interface and menu handling
- **Rendering**: Handled by Bevy's built-in systems

#### Key Components
- **Player**: Player entity with velocity, grounding state, and radius
- **Voxel**: Marker for voxel entities
- **SubVoxel**: Individual sub-voxel with parent and local coordinates
- **GameCamera**: Camera with rotation state and controls
- **CollisionBox**: Debug visualization component

#### Resources
- **LoadedMapData**: Currently loaded map data
- **MapLoadProgress**: Real-time loading progress tracking
- **SpatialGrid**: Spatial partitioning for efficient collision queries
- **GameInitialized**: Prevents duplicate world setup

### Debugging

Comprehensive debugging setup is available for VSCode. See [`DEBUG_SETUP.md`](DEBUG_SETUP.md) for:
- CodeLLDB configuration
- Native LLDB setup
- Build tasks
- Bevy-specific debugging tips
- Troubleshooting guide

#### Quick Debug Start
1. Install recommended VSCode extensions (rust-analyzer, CodeLLDB)
2. Open Run & Debug panel (Ctrl+Shift+D / Cmd+Shift+D)
3. Select "Debug (CodeLLDB, Debug Build)"
4. Set breakpoints in your code
5. Press F5 to start debugging

### Build Profiles

The project uses optimized build profiles for better development experience:

```toml
[profile.dev]
opt-level = 1              # Slight optimization for dev builds

[profile.dev.package."*"]
opt-level = 3              # Full optimization for dependencies
```

This configuration provides:
- Faster compilation times during development
- Better runtime performance for dependencies (especially Bevy)
- Reasonable debug build performance

### Contributing

Contributions are welcome! When contributing:

1. Follow Rust naming conventions and style guidelines
2. Maintain the modular architecture
3. Add documentation for new systems and components
4. Test changes in both debug and release builds
5. Update this README if adding new features or changing architecture

## Performance Tips

- **Use Release Mode**: Run with `cargo run --release` for optimal performance
- **Bevy Dynamic Linking**: For faster compile times during development, consider enabling Bevy's dynamic linking feature
- **Asset Optimization**: Keep textures and audio files optimized for size

## Map System

### Creating Custom Maps

Maps are defined in RON (Rusty Object Notation) format in the `assets/maps/` directory. See [`MAP_LOADER_DESIGN.md`](MAP_LOADER_DESIGN.md) for detailed documentation.

#### Basic Map Structure

```ron
(
    metadata: (
        name: "My Custom Map",
        author: "Your Name",
        version: "1.0.0",
        description: "A custom map description",
    ),
    world: (
        voxels: [
            (pos: (0, 0, 0), pattern: Some(Full)),
            (pos: (1, 0, 0), pattern: Some(Platform)),
            // Add more voxels...
        ],
    ),
    entities: [
        (entity_type: PlayerSpawn, position: (0.5, 1.5, 0.5)),
        // Add more entities...
    ],
    lighting: (
        ambient_intensity: 0.3,
        directional_light: Some((
            direction: (-0.5, -1.0, -0.5),
            illuminance: 10000.0,
            color: (1.0, 1.0, 0.9),
        )),
    ),
    camera: (
        position: (8.0, 10.0, 8.0),
        look_at: (0.0, 0.0, 0.0),
        rotation_offset: 0.0,
    ),
)
```

#### Sub-Voxel Patterns

- **Full**: Solid 8×8×8 block
- **Platform**: Thin 8×8×1 platform
- **Staircase**: Progressive steps
- **Pillar**: Small centered 2×2×2 pillar

#### Entity Types

- **PlayerSpawn**: Player starting position
- **Enemy**: Enemy spawn point (planned)
- **Item**: Item pickup location (planned)
- **Trigger**: Event trigger zone (planned)

### Loading Custom Maps

To load a custom map, place your `.ron` file in `assets/maps/` and update the map path in [`src/main.rs`](src/main.rs):

```rust
commands.insert_resource(MapToLoad("maps/your_map.ron".to_string()));
```

## Roadmap

Future planned features:
- [x] Map loader system with RON format
- [x] Loading screen with progress tracking
- [ ] Map editor tool
- [ ] Procedural world generation
- [ ] Multiple biomes and terrain types
- [ ] Player inventory system
- [ ] Save/load functionality
- [ ] Multiplayer support
- [ ] Advanced physics (jumping, climbing)
- [ ] Sound effects and music
- [ ] Settings menu with graphics options

## License

This project is currently unlicensed. Please contact the project maintainers for licensing information.

## Acknowledgments

- Built with [Bevy](https://bevyengine.org/) - A refreshingly simple data-driven game engine
- Developed by Kibound

---

**Note**: This is an experimental project under active development. Features and architecture may change significantly between versions.
