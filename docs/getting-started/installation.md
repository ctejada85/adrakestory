# Installation Guide

This guide will help you set up A Drake's Story on your system.

## Prerequisites

### Required Software

- **Rust** (latest stable version)
  - Install from [rustup.rs](https://rustup.rs/)
  - Includes `cargo` (Rust's package manager)

### Platform-Specific Requirements

#### macOS
- **Xcode Command Line Tools**
  ```bash
  xcode-select --install
  ```

#### Linux
- **Development Libraries**
  - Follow the [Bevy setup guide](https://bevyengine.org/learn/book/getting-started/setup/) for your distribution
  - Common packages: `libudev-dev`, `libasound2-dev`, `libxcb-shape0-dev`, `libxcb-xfixes0-dev`

#### Windows
- **Visual Studio Build Tools**
  - Download from [Microsoft](https://visualstudio.microsoft.com/downloads/)
  - Select "Desktop development with C++"

## Installation Steps

### 1. Clone the Repository

```bash
git clone <repository-url>
cd adrakestory
```

### 2. Build the Project

#### Debug Build (Faster Compilation)
```bash
cargo build
```

#### Release Build (Better Performance)
```bash
cargo build --release
```

**Note**: The first build will take several minutes as Cargo downloads and compiles all dependencies.

### 3. Run the Game

#### Debug Mode
```bash
cargo run
```

#### Release Mode (Recommended)
```bash
cargo run --release
```

## Build Profiles

The project uses optimized build profiles for better development experience:

```toml
[profile.dev]
opt-level = 1              # Slight optimization for dev builds

[profile.dev.package."*"]
opt-level = 3              # Full optimization for dependencies
```

**Benefits:**
- Faster compilation during development
- Better runtime performance for dependencies (especially Bevy)
- Reasonable debug build performance

## Verifying Installation

After running the game, you should see:
1. **Intro Animation** - Opening splash screen
2. **Title Screen** - Main menu with "New Game" option
3. **Loading Screen** - Map loading progress
4. **Game World** - 3D voxel environment

If you encounter issues, see the [Troubleshooting Guide](../user-guide/troubleshooting.md).

## Next Steps

- [Quick Start Guide](quick-start.md) - Learn how to play
- [Controls Reference](controls.md) - Master the controls
- [Gameplay Guide](../user-guide/gameplay.md) - Explore features

## Development Setup

For developers who want to contribute:
- [Debugging Setup](../developer-guide/debugging.md) - Configure VSCode debugging
- [Architecture Overview](../developer-guide/architecture.md) - Understand the codebase
- [Contributing Guidelines](../developer-guide/contributing.md) - Contribution workflow

---

**Need Help?** Check the [Troubleshooting Guide](../user-guide/troubleshooting.md) or open an issue on GitHub.