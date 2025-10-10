# VSCode Debugging Setup for Adrakestory (Rust/Bevy)

## Prerequisites

- **VSCode** (latest recommended)
- **Rust toolchain** (`rustup`, `cargo`, `rustc`)
- **LLDB** (comes with Xcode Command Line Tools on macOS)
- **Recommended Extensions** (auto-suggested by VSCode):
  - `rust-lang.rust-analyzer`
  - `vadimcn.vscode-lldb`
  - `serayuzgur.crates`
  - `tamasfe.even-better-toml`

## Debugging Configurations

### 1. CodeLLDB (Recommended)
- **Debug (CodeLLDB, Debug Build):**  
  Builds and runs the debug binary with full debug symbols.
- **Debug (CodeLLDB, Release Build):**  
  Builds and runs the release binary (optimized, but debuggable).
- **Attach to Process (CodeLLDB):**  
  Attach to a running process (e.g., if you start the game manually).

### 2. Native LLDB
- **Debug (Native LLDB, Debug Build):**  
  Uses the built-in LLDB debugger (no extension dependency).

## Build Tasks

- **cargo build**: Build debug binary.
- **cargo build --release**: Build optimized release binary.
- **cargo run**: Build and run the game.
- **cargo test**: Run tests.
- **cargo clean**: Clean build artifacts.

## How to Use

1. **Install all recommended extensions** (VSCode will prompt you).
2. **Open the Run & Debug panel** (Ctrl+Shift+D or Cmd+Shift+D).
3. **Select a configuration** (e.g., "Debug (CodeLLDB, Debug Build)").
4. **Set breakpoints** in your Rust code.
5. **Start debugging** (F5 or green play button).

## Tips for Bevy Debugging

- Set breakpoints in your systems, not just in `main.rs`.
- Use the "Attach to Process" config if you launch the game outside VSCode.
- Use the "Debug (Release Build)" config to debug performance issues.
- Use the "Problems" panel for Rust compiler errors and warnings.

## Troubleshooting

- If breakpoints are not hit, ensure you are building in debug mode.
- If you see "source not found," check that your build is not stripped.
- For advanced LLDB commands, open the Debug Console.

## References

- [Bevy Debugging Guide](https://bevyengine.org/learn/book/getting-started/debugging/)
- [CodeLLDB Extension](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb)
- [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
