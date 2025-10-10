# Debugging Setup Guide

Complete guide to debugging A Drake's Story in VSCode with LLDB.

## Prerequisites

### Required Software

- **VSCode** (latest version recommended)
- **Rust toolchain** (`rustup`, `cargo`, `rustc`)
- **LLDB debugger**
  - macOS: Comes with Xcode Command Line Tools
  - Linux: Install via package manager
  - Windows: Comes with Visual Studio Build Tools

### Required VSCode Extensions

The project includes recommended extensions that VSCode will prompt you to install:

1. **rust-analyzer** (`rust-lang.rust-analyzer`)
   - Rust language support
   - Code completion and analysis
   - Inline error checking

2. **CodeLLDB** (`vadimcn.vscode-lldb`)
   - LLDB debugger integration
   - Breakpoint support
   - Variable inspection

3. **crates** (`serayuzgur.crates`)
   - Cargo.toml dependency management
   - Version checking

4. **Even Better TOML** (`tamasfe.even-better-toml`)
   - TOML syntax highlighting
   - Better Cargo.toml editing

## Quick Start

### 1. Install Extensions

When you open the project, VSCode will show a notification:

```
This workspace has extension recommendations.
[Install All] [Show Recommendations]
```

Click **Install All** to install all recommended extensions.

### 2. Open Debug Panel

- Press **Ctrl+Shift+D** (Windows/Linux) or **Cmd+Shift+D** (macOS)
- Or click the Debug icon in the sidebar

### 3. Select Configuration

Choose from the dropdown at the top of the Debug panel:
- **Debug (CodeLLDB, Debug Build)** - Recommended for most debugging
- **Debug (CodeLLDB, Release Build)** - For performance debugging
- **Attach to Process (CodeLLDB)** - Attach to running process
- **Debug (Native LLDB, Debug Build)** - Alternative without extension

### 4. Set Breakpoints

Click in the gutter (left of line numbers) to set breakpoints:
- Red dot = Active breakpoint
- Gray dot = Disabled breakpoint
- Hollow dot = Unverified breakpoint

### 5. Start Debugging

- Press **F5** or click the green play button
- The game will build and launch with debugger attached
- Execution will pause at breakpoints

## Debug Configurations

The project includes several pre-configured debug setups in `.vscode/launch.json`.

### Debug (CodeLLDB, Debug Build)

**Best for:** General development and debugging

```json
{
    "type": "lldb",
    "request": "launch",
    "name": "Debug (CodeLLDB, Debug Build)",
    "cargo": {
        "args": ["build", "--bin=adrakestory"]
    },
    "args": [],
    "cwd": "${workspaceFolder}"
}
```

**Features:**
- Full debug symbols
- Fast compilation
- Complete variable inspection
- All breakpoints work

**When to use:**
- Debugging logic errors
- Inspecting variables
- Step-through debugging
- Understanding code flow

### Debug (CodeLLDB, Release Build)

**Best for:** Performance debugging and optimization

```json
{
    "type": "lldb",
    "request": "launch",
    "name": "Debug (CodeLLDB, Release Build)",
    "cargo": {
        "args": ["build", "--release", "--bin=adrakestory"]
    },
    "args": [],
    "cwd": "${workspaceFolder}"
}
```

**Features:**
- Optimized code
- Better performance
- Some debug info available
- May skip some breakpoints

**When to use:**
- Performance issues
- Frame rate problems
- Release-specific bugs
- Optimization verification

### Attach to Process (CodeLLDB)

**Best for:** Debugging already-running game

```json
{
    "type": "lldb",
    "request": "attach",
    "name": "Attach to Process (CodeLLDB)",
    "pid": "${command:pickProcess}"
}
```

**Features:**
- Attach to running process
- No restart needed
- Useful for long-running sessions

**When to use:**
- Game already running
- Don't want to restart
- Debugging specific state
- Testing after manual setup

### Debug (Native LLDB, Debug Build)

**Best for:** When CodeLLDB extension isn't available

```json
{
    "type": "lldb",
    "request": "launch",
    "name": "Debug (Native LLDB, Debug Build)",
    "program": "${workspaceFolder}/target/debug/adrakestory",
    "args": [],
    "cwd": "${workspaceFolder}",
    "preLaunchTask": "cargo build"
}
```

**Features:**
- Uses system LLDB
- No extension dependency
- Basic debugging features

**When to use:**
- CodeLLDB not working
- Extension conflicts
- Minimal setup needed

## Build Tasks

The project includes pre-configured build tasks in `.vscode/tasks.json`.

### Available Tasks

Access via **Terminal → Run Task** or **Ctrl+Shift+B**:

1. **cargo build** - Build debug binary
2. **cargo build --release** - Build optimized binary
3. **cargo run** - Build and run debug
4. **cargo test** - Run tests
5. **cargo clean** - Clean build artifacts

### Using Tasks

**Keyboard Shortcut:**
- Press **Ctrl+Shift+B** (Windows/Linux) or **Cmd+Shift+B** (macOS)
- Select task from list

**Command Palette:**
- Press **Ctrl+Shift+P** (Windows/Linux) or **Cmd+Shift+P** (macOS)
- Type "Run Task"
- Select task

## Debugging Workflow

### Basic Debugging Session

1. **Set Breakpoints**
   ```rust
   pub fn player_movement_system(/* ... */) {
       // Click here to set breakpoint
       let input = keyboard_input.pressed(KeyCode::W);
   }
   ```

2. **Start Debugging**
   - Press **F5**
   - Wait for build to complete
   - Game launches with debugger

3. **Trigger Breakpoint**
   - Play the game
   - Perform action that hits breakpoint
   - Execution pauses

4. **Inspect State**
   - Hover over variables
   - Check Variables panel
   - Evaluate expressions in Debug Console

5. **Control Execution**
   - **F10** - Step Over (next line)
   - **F11** - Step Into (enter function)
   - **Shift+F11** - Step Out (exit function)
   - **F5** - Continue (resume execution)

### Advanced Debugging

**Conditional Breakpoints:**
```
Right-click breakpoint → Edit Breakpoint → Add condition
Example: player.velocity.y < 0.0
```

**Logpoints:**
```
Right-click gutter → Add Logpoint
Example: Player position: {transform.translation}
```

**Watch Expressions:**
```
Debug panel → Watch section → Add expression
Example: player.is_grounded
```

## Debugging Bevy Systems

### System Debugging Tips

**1. Set Breakpoints in Systems, Not main()**

```rust
// ❌ Don't set breakpoints here
fn main() {
    App::new()
        .add_systems(Update, player_movement)
        .run();
}

// ✅ Set breakpoints here
fn player_movement(
    mut query: Query<(&mut Transform, &Player)>,
) {
    // Breakpoint here will trigger every frame
    for (mut transform, player) in query.iter_mut() {
        // Debug player movement
    }
}
```

**2. Use Conditional Breakpoints**

Systems run every frame, so use conditions:

```rust
// Condition: player.velocity.length() > 5.0
fn player_movement(/* ... */) {
    // Only breaks when player moving fast
}
```

**3. Inspect Query Results**

```rust
fn debug_system(query: Query<&Player>) {
    let count = query.iter().count();
    // Breakpoint here to check entity count
}
```

### Common Bevy Debugging Scenarios

**Debugging Entity Spawning:**
```rust
fn spawn_map_system(mut commands: Commands) {
    let entity = commands.spawn((
        Voxel,
        Transform::default(),
    ));
    // Breakpoint here to verify spawning
}
```

**Debugging Component Updates:**
```rust
fn physics_system(mut query: Query<&mut Player>) {
    for mut player in query.iter_mut() {
        player.velocity.y -= 9.81;
        // Breakpoint to check velocity changes
    }
}
```

**Debugging State Transitions:**
```rust
fn check_state(state: Res<State<GameState>>) {
    // Breakpoint to verify current state
    println!("Current state: {:?}", state.get());
}
```

## Debug Console

### Using the Debug Console

Access via **View → Debug Console** or **Ctrl+Shift+Y**.

**Evaluate Expressions:**
```
> player.velocity
Vec3(1.0, 0.0, 0.5)

> transform.translation.length()
5.2
```

**Call Functions:**
```
> player.is_grounded
true
```

**LLDB Commands:**
```
> expr player.velocity.y
(f32) $0 = -2.5
```

## Troubleshooting

### Breakpoints Not Hit

**Problem:** Breakpoints show as gray/unverified or aren't hit

**Solutions:**

1. **Ensure Debug Build**
   ```bash
   cargo build  # Not cargo build --release
   ```

2. **Check Optimization Level**
   - Debug builds should have `opt-level = 0` or `1`
   - Release builds may skip breakpoints

3. **Verify Code is Executed**
   - Add `println!` to confirm code runs
   - Check system is registered
   - Verify state conditions

4. **Rebuild**
   ```bash
   cargo clean
   cargo build
   ```

### Source Not Found

**Problem:** Debugger can't find source files

**Solutions:**

1. **Check Build Artifacts**
   - Ensure binary isn't stripped
   - Verify debug symbols present

2. **Rebuild with Debug Info**
   ```toml
   [profile.release]
   debug = true
   ```

3. **Check Working Directory**
   - Ensure `cwd` in launch.json is correct

### Variables Not Visible

**Problem:** Can't see variable values

**Solutions:**

1. **Use Debug Build**
   - Release builds optimize away variables
   - Use debug build for full inspection

2. **Check Optimization**
   ```toml
   [profile.dev]
   opt-level = 0  # No optimization
   ```

3. **Use Debug Console**
   - Manually evaluate expressions
   - Use LLDB commands

### Debugger Crashes

**Problem:** Debugger or game crashes

**Solutions:**

1. **Update Extensions**
   - Update CodeLLDB
   - Update rust-analyzer

2. **Check LLDB Version**
   ```bash
   lldb --version
   ```

3. **Try Native LLDB**
   - Use "Debug (Native LLDB)" configuration

4. **Check System Resources**
   - Close other applications
   - Free up memory

### Performance Issues

**Problem:** Debugging is very slow

**Solutions:**

1. **Disable Unnecessary Breakpoints**
   - Remove breakpoints in hot loops
   - Use conditional breakpoints

2. **Use Release Build**
   - Better performance
   - Some debug info still available

3. **Reduce Logging**
   - Disable verbose logging
   - Remove debug prints

## Platform-Specific Notes

### macOS

**LLDB Location:**
```bash
/Library/Developer/CommandLineTools/usr/bin/lldb
```

**Xcode Command Line Tools:**
```bash
xcode-select --install
```

**Code Signing:**
May need to sign binary for debugging:
```bash
codesign -s - target/debug/adrakestory
```

### Linux

**Install LLDB:**
```bash
# Ubuntu/Debian
sudo apt-get install lldb

# Fedora
sudo dnf install lldb

# Arch
sudo pacman -S lldb
```

**Permissions:**
May need to adjust ptrace permissions:
```bash
echo 0 | sudo tee /proc/sys/kernel/yama/ptrace_scope
```

### Windows

**LLDB Location:**
Comes with Visual Studio Build Tools

**Path Issues:**
Ensure LLDB is in PATH:
```powershell
$env:PATH += ";C:\Program Files\LLVM\bin"
```

## Best Practices

### Effective Debugging

1. **Start Simple**: Use `println!` first
2. **Isolate Issues**: Narrow down problem area
3. **Use Breakpoints Wisely**: Don't break in hot loops
4. **Check Assumptions**: Verify what you think is true
5. **Read Error Messages**: They often tell you exactly what's wrong

### Debugging Strategy

1. **Reproduce**: Ensure you can trigger the bug
2. **Isolate**: Find minimal reproduction
3. **Hypothesize**: Form theory about cause
4. **Test**: Use debugger to verify theory
5. **Fix**: Implement solution
6. **Verify**: Confirm fix works

### Performance Debugging

1. **Profile First**: Use profiling tools
2. **Measure**: Don't guess at performance
3. **Optimize Hot Paths**: Focus on frequently-called code
4. **Verify**: Measure improvement

## Additional Resources

### Documentation

- [Bevy Debugging Guide](https://bevyengine.org/learn/book/getting-started/debugging/)
- [CodeLLDB Documentation](https://github.com/vadimcn/vscode-lldb/blob/master/MANUAL.md)
- [LLDB Tutorial](https://lldb.llvm.org/use/tutorial.html)

### Related Guides

- [Architecture Overview](architecture.md) - Understand the codebase
- [Contributing Guide](contributing.md) - Contribution workflow
- [Troubleshooting](../user-guide/troubleshooting.md) - Common issues

---

**Happy debugging!** Remember: debugging is a skill that improves with practice.