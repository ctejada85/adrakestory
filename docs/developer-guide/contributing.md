# Contributing Guide

Thank you for your interest in contributing to A Drake's Story! This guide will help you get started.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Code Style](#code-style)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Review Process](#review-process)

## Getting Started

### Prerequisites

Before contributing, ensure you have:

1. **Rust toolchain** installed ([rustup.rs](https://rustup.rs/))
2. **Git** for version control
3. **VSCode** (recommended) with extensions
4. **Basic Rust knowledge**
5. **Familiarity with Bevy** (helpful but not required)

### First Steps

1. **Fork the Repository**
   ```bash
   # Click "Fork" on GitHub
   ```

2. **Clone Your Fork**
   ```bash
   git clone https://github.com/YOUR_USERNAME/adrakestory.git
   cd adrakestory
   ```

3. **Add Upstream Remote**
   ```bash
   git remote add upstream https://github.com/ORIGINAL_OWNER/adrakestory.git
   ```

4. **Build and Test**
   ```bash
   cargo build
   cargo test
   cargo run
   ```

## Development Setup

### Recommended Tools

- **VSCode** with extensions:
  - rust-analyzer
  - CodeLLDB
  - Even Better TOML
  - crates

- **Command Line Tools**:
  - `cargo-watch` - Auto-rebuild on changes
  - `cargo-clippy` - Linting
  - `cargo-fmt` - Formatting

### Install Development Tools

```bash
# Cargo watch for auto-rebuild
cargo install cargo-watch

# Run with auto-rebuild
cargo watch -x run
```

### Project Structure

Familiarize yourself with the codebase:

```
src/
├── main.rs              # Entry point
├── states.rs            # Game states
├── systems/             # Game systems
│   ├── game/            # Core gameplay
│   ├── intro_animation/ # Intro screen
│   ├── title_screen/    # Main menu
│   ├── loading_screen/  # Loading UI
│   └── pause_menu/      # Pause menu
└── components/          # Shared components
```

See [Architecture Overview](architecture.md) for details.

## Code Style

### Rust Style Guidelines

Follow standard Rust conventions:

1. **Naming Conventions**
   ```rust
   // Types: PascalCase
   struct PlayerComponent { }
   enum GameState { }
   
   // Functions/variables: snake_case
   fn update_player_position() { }
   let player_velocity = Vec3::ZERO;
   
   // Constants: SCREAMING_SNAKE_CASE
   const MAX_VELOCITY: f32 = 10.0;
   ```

2. **Formatting**
   ```bash
   # Format code before committing
   cargo fmt
   ```

3. **Linting**
   ```bash
   # Check for common issues
   cargo clippy
   ```

### Bevy-Specific Conventions

1. **Component Naming**
   ```rust
   // Good: Descriptive, clear purpose
   #[derive(Component)]
   struct Player { }
   
   #[derive(Component)]
   struct Velocity(Vec3);
   ```

2. **System Naming**
   ```rust
   // Good: Verb + noun, describes action
   fn update_player_movement() { }
   fn handle_collision_detection() { }
   fn spawn_map_entities() { }
   ```

3. **Resource Naming**
   ```rust
   // Good: Noun, describes data
   #[derive(Resource)]
   struct GameSettings { }
   
   #[derive(Resource)]
   struct SpatialGrid { }
   ```

### Documentation

1. **Module Documentation**
   ```rust
   //! # Player Movement Module
   //!
   //! Handles all player movement logic including keyboard input,
   //! velocity calculation, and ground detection.
   ```

2. **Function Documentation**
   ```rust
   /// Updates player position based on velocity and input.
   ///
   /// # Arguments
   /// * `time` - Game time for delta calculations
   /// * `input` - Keyboard input state
   /// * `query` - Query for player entities
   pub fn update_player_movement(/* ... */) {
       // Implementation
   }
   ```

3. **Complex Logic**
   ```rust
   // Explain why, not what
   // Use spatial grid for O(n) collision detection instead of O(n²)
   let nearby_entities = spatial_grid.query_cell(position);
   ```

## Making Changes

### Branch Strategy

1. **Create Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/bug-description
   ```

2. **Keep Branches Focused**
   - One feature/fix per branch
   - Small, reviewable changes
   - Clear purpose

### Commit Messages

Follow conventional commit format:

```
type(scope): brief description

Longer explanation if needed.

Fixes #123
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance tasks

**Examples:**
```bash
git commit -m "feat(player): add jump mechanic"
git commit -m "fix(collision): resolve edge detection bug"
git commit -m "docs(readme): update installation instructions"
```

### Making Changes

1. **Write Code**
   - Follow style guidelines
   - Add documentation
   - Keep changes focused

2. **Test Locally**
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   cargo run
   ```

3. **Commit Changes**
   ```bash
   git add .
   git commit -m "feat(scope): description"
   ```

4. **Keep Updated**
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Writing Tests

1. **Unit Tests**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_collision_detection() {
           let pos1 = Vec3::ZERO;
           let pos2 = Vec3::new(1.0, 0.0, 0.0);
           assert!(check_collision(pos1, pos2, 0.5));
       }
   }
   ```

2. **Integration Tests**
   ```rust
   #[test]
   fn test_player_movement_system() {
       let mut app = App::new();
       app.add_systems(Update, player_movement);
       // Test system behavior
   }
   ```

### Manual Testing

1. **Test in Debug Mode**
   ```bash
   cargo run
   ```

2. **Test in Release Mode**
   ```bash
   cargo run --release
   ```

3. **Test All Game States**
   - Intro animation
   - Title screen
   - Map loading
   - Gameplay
   - Pause menu

4. **Test Edge Cases**
   - Empty maps
   - Large maps
   - Invalid input
   - Boundary conditions

## Submitting Changes

### Before Submitting

**Checklist:**
- [ ] Code follows style guidelines
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Code is formatted (`cargo fmt`)
- [ ] Documentation is updated
- [ ] Commit messages are clear
- [ ] Branch is up to date with main

### Create Pull Request

1. **Push to Your Fork**
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Open Pull Request on GitHub**
   - Click "New Pull Request"
   - Select your branch
   - Fill out template

3. **PR Description Template**
   ```markdown
   ## Description
   Brief description of changes
   
   ## Type of Change
   - [ ] Bug fix
   - [ ] New feature
   - [ ] Documentation update
   - [ ] Refactoring
   
   ## Testing
   How was this tested?
   
   ## Checklist
   - [ ] Tests pass
   - [ ] Code formatted
   - [ ] Documentation updated
   
   ## Related Issues
   Fixes #123
   ```

## Review Process

### What to Expect

1. **Initial Review** (1-3 days)
   - Maintainer reviews code
   - Provides feedback
   - Requests changes if needed

2. **Discussion**
   - Address feedback
   - Make requested changes
   - Push updates

3. **Approval**
   - Changes approved
   - PR merged
   - Branch deleted

### Responding to Feedback

1. **Be Respectful**
   - Thank reviewers
   - Ask questions if unclear
   - Explain your reasoning

2. **Make Changes**
   ```bash
   # Make requested changes
   git add .
   git commit -m "fix: address review feedback"
   git push origin feature/your-feature-name
   ```

3. **Mark Resolved**
   - Resolve conversations on GitHub
   - Explain what you changed

## Areas for Contribution

### Good First Issues

Look for issues labeled:
- `good first issue`
- `help wanted`
- `documentation`

### High-Priority Areas

1. **Features**
   - Jump mechanic
   - Inventory system
   - Save/load functionality
   - Additional entity types

2. **Improvements**
   - Performance optimization
   - Better error messages
   - UI enhancements
   - Map editor tools

3. **Documentation**
   - Code examples
   - Tutorial content
   - API documentation
   - Video guides

4. **Testing**
   - Unit tests
   - Integration tests
   - Performance tests
   - Map validation tests

## Code of Conduct

### Our Standards

- **Be Respectful**: Treat everyone with respect
- **Be Constructive**: Provide helpful feedback
- **Be Patient**: Everyone is learning
- **Be Inclusive**: Welcome all contributors

### Unacceptable Behavior

- Harassment or discrimination
- Trolling or insulting comments
- Personal attacks
- Spam or off-topic content

## Getting Help

### Resources

- **Documentation**: Read the [docs](../README.md)
- **Architecture**: See [Architecture Guide](architecture.md)
- **Debugging**: Check [Debugging Guide](debugging.md)
- **Issues**: Search existing GitHub issues

### Asking Questions

1. **Check Documentation First**
2. **Search Existing Issues**
3. **Ask in Discussions** (if available)
4. **Open an Issue** with:
   - Clear question
   - What you've tried
   - Relevant code/errors

## Recognition

Contributors will be:
- Listed in CONTRIBUTORS.md
- Mentioned in release notes
- Credited in documentation

Thank you for contributing to A Drake's Story!

---

**Questions?** Open an issue or discussion on GitHub.