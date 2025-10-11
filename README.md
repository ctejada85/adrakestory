# A Drake's Story

A 3D voxel-based game built with Rust and the Bevy game engine, featuring sub-voxel rendering, physics simulation, and an isometric camera view.

![Rust](https://img.shields.io/badge/rust-2021-orange.svg)
![Bevy](https://img.shields.io/badge/bevy-0.15-blue.svg)
![License](https://img.shields.io/badge/license-unlicensed-lightgrey.svg)

## Overview

A Drake's Story is an experimental 3D game exploring voxel-based world generation with high-resolution sub-voxel rendering (8×8×8 sub-voxels per voxel). Navigate through procedurally structured worlds with realistic physics, varied terrain types, and dynamic camera controls.

## Quick Start

```bash
# Clone the repository
git clone <repository-url>
cd adrakestory

# Build and run (release mode recommended)
cargo run --release
```

**First time?** See the [Installation Guide](docs/getting-started/installation.md) for detailed setup instructions.

## Features

- **3D Voxel World** - High-detail sub-voxel rendering
- **Physics System** - Realistic gravity and collision detection
- **Varied Terrain** - Staircases, platforms, pillars, and solid blocks
- **Isometric Camera** - Dynamic camera with rotation controls
- **Map Loader** - Load custom maps from RON files with progress tracking
- **Map Editor** - Visual editor for creating and editing maps
- **State Management** - Clean game flow through multiple states

## Documentation

📚 **[Complete Documentation](docs/README.md)** - Full documentation hub

### Quick Links

- **[Installation Guide](docs/getting-started/installation.md)** - Setup and prerequisites
- **[Quick Start](docs/getting-started/quick-start.md)** - Your first game
- **[Controls](docs/getting-started/controls.md)** - Keyboard and mouse controls
- **[Map Editor](docs/user-guide/map-editor/getting-started.md)** - Visual map creation tool
- **[Creating Maps](docs/user-guide/maps/creating-maps.md)** - Manual map creation
- **[Architecture](docs/developer-guide/architecture.md)** - System design
- **[Contributing](docs/developer-guide/contributing.md)** - How to contribute

## Controls

| Action | Key/Input |
|--------|-----------|
| Move | **WASD** |
| Camera | **Mouse** or **Q/E** |
| Pause | **ESC** |
| Debug | **C** (collision boxes) |

Full controls: [Controls Reference](docs/getting-started/controls.md)

## Technology Stack

- **Language**: Rust 2021 Edition
- **Engine**: [Bevy](https://bevyengine.org/) 0.15
- **Architecture**: Entity Component System (ECS)
- **Map Format**: RON (Rusty Object Notation)

## Project Structure

```
adrakestory/
├── src/                    # Source code
│   ├── main.rs            # Game entry point
│   ├── lib.rs             # Shared library
│   ├── states.rs          # Game states
│   ├── systems/           # Game systems
│   ├── editor/            # Map editor modules
│   └── bin/
│       └── map_editor.rs  # Map editor binary
├── assets/                # Game assets
│   ├── maps/             # Map files (.ron)
│   ├── textures/         # Textures and sprites
│   └── fonts/            # UI fonts
├── docs/                  # Documentation
└── Cargo.toml            # Dependencies
```

## Development

### Prerequisites

- Rust (latest stable)
- Platform-specific requirements:
  - **macOS**: Xcode Command Line Tools
  - **Linux**: Development libraries ([details](docs/getting-started/installation.md))
  - **Windows**: Visual Studio Build Tools

### Building

```bash
# Build the game (debug mode)
cargo build

# Build the game (release mode - recommended)
cargo build --release

# Build the map editor
cargo build --bin map_editor --release
```

### Running

```bash
# Run the game
cargo run --release

# Run the map editor
cargo run --bin map_editor --release
```

### Running Tests

```bash
cargo test
```

### Debugging

See the [Debugging Guide](docs/developer-guide/debugging.md) for VSCode setup and debugging tips.

## Creating Custom Maps

Maps are defined in RON format in `assets/maps/`:

```ron
(
    metadata: (
        name: "My Map",
        author: "Your Name",
        version: "1.0.0",
        // ...
    ),
    world: (
        width: 10,
        height: 5,
        depth: 10,
        voxels: [
            (pos: (0, 0, 0), voxel_type: Grass, pattern: Some(Full)),
            // ...
        ],
    ),
    // ...
)
```

**Learn more**: [Creating Maps Guide](docs/user-guide/maps/creating-maps.md)

## Contributing

Contributions are welcome! Please read our [Contributing Guide](docs/developer-guide/contributing.md) for:

- Development setup
- Code style guidelines
- Submission process
- Areas for contribution

## Roadmap

- [x] Map loader system with RON format
- [x] Loading screen with progress tracking
- [x] Map editor tool (core implementation complete)
- [ ] Map editor integration (file I/O, voxel rendering)
- [ ] Procedural world generation
- [ ] Player inventory system
- [ ] Save/load functionality
- [ ] Multiplayer support
- [ ] Sound effects and music

## License

This project is currently unlicensed. Please contact the project maintainers for licensing information.

## Acknowledgments

- Built with [Bevy](https://bevyengine.org/) - A refreshingly simple data-driven game engine
- Developed by Kibound

## Support

- **Documentation**: [docs/](docs/README.md)
- **Issues**: [GitHub Issues](https://github.com/yourusername/adrakestory/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/adrakestory/discussions)

---

**Note**: This is an experimental project under active development. Features and architecture may change significantly between versions.
