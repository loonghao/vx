# 🦀 vx-tool-rust

<div align="center">

**Rust Programming Language Tool Plugin for vx Universal Tool Manager**

[![Crates.io](https://img.shields.io/crates/v/vx-tool-rust.svg)](https://crates.io/crates/vx-tool-rust)
[![Documentation](https://docs.rs/vx-tool-rust/badge.svg)](https://docs.rs/vx-tool-rust)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/loonghao/vx/workflows/CI/badge.svg)](https://github.com/loonghao/vx/actions)

*Blazing fast Rust development with beautiful installation experience and zero configuration*

</div>

## Overview

`vx-tool-rust` provides Rust programming language support for vx, enabling automatic installation, version management, and execution of Rust toolchain commands through the vx interface.

## Features

- **Rust Toolchain**: Complete Rust compiler and tools
- **Cargo Integration**: Built-in Cargo package manager and build tool
- **Auto-Installation**: Automatic download and installation of Rust versions
- **Cross-Platform**: Windows, macOS, and Linux support
- **Version Management**: Install and switch between multiple Rust versions
- **Target Support**: Cross-compilation for multiple targets
- **Component Management**: Install additional Rust components

## Supported Commands

### Rust Compiler
```bash
# Compile Rust code
vx rustc main.rs
vx rustc --version
vx rustc --explain E0308

# Check code without building
vx rustc --emit=metadata main.rs
```

### Cargo Package Manager
```bash
# Project management
vx cargo new myproject
vx cargo init
vx cargo build
vx cargo run

# Testing
vx cargo test
vx cargo test --release
vx cargo bench

# Package management
vx cargo add serde
vx cargo remove serde
vx cargo update
```

### Development Tools
```bash
# Code formatting
vx cargo fmt
vx cargo fmt --check

# Linting
vx cargo clippy
vx cargo clippy -- -D warnings

# Documentation
vx cargo doc
vx cargo doc --open

# Publishing
vx cargo publish
vx cargo package
```

## Installation

### Through vx CLI
```bash
# Install latest stable
vx install rust

# Install specific version
vx install rust@1.75.0
vx install rust@1.74.1

# Install latest version
vx install rust@latest
```

### Version Constraints
```bash
# Semantic version ranges
vx install rust@^1.75.0   # Latest 1.75.x
vx install rust@~1.75.0   # Latest 1.75.0.x
vx install rust@>=1.70.0  # 1.70.0 or higher
```

## Configuration

### Project Configuration (.vx.toml)
```toml
[tools]
rust = "1.75.0"           # Specific version
# rust = "stable"         # Latest stable
# rust = "beta"           # Beta channel
# rust = "nightly"        # Nightly channel

[tools.rust]
auto_install = true
default_target = "x86_64-unknown-linux-gnu"
```

### Global Configuration
```toml
[tools.rust]
default_version = "stable"
auto_install = true
install_timeout = 600

[rust.settings]
default_toolchain = "stable"
profile = "default"
components = ["rustfmt", "clippy"]
targets = ["wasm32-unknown-unknown"]
```

## Toolchain Management

### Channels
- **stable**: Latest stable release (recommended)
- **beta**: Beta releases (6-week cycle)
- **nightly**: Nightly builds (daily)

### Components
```bash
# Install additional components
vx rustup component add rustfmt
vx rustup component add clippy
vx rustup component add rust-src
vx rustup component add rust-analyzer

# List components
vx rustup component list
vx rustup component list --installed
```

### Targets
```bash
# Add compilation targets
vx rustup target add wasm32-unknown-unknown
vx rustup target add x86_64-pc-windows-gnu
vx rustup target add aarch64-apple-darwin

# List targets
vx rustup target list
vx rustup target list --installed
```

## Platform Support

### Windows
- **x64**: Full support
- **x86**: Legacy support
- **ARM64**: Windows 11 ARM support

### macOS
- **x64**: Intel Mac support
- **ARM64**: Apple Silicon (M1/M2) support

### Linux
- **x64**: All major distributions
- **ARM64**: ARM-based systems
- **ARMv7**: Raspberry Pi support

## Cross-Compilation

### Common Targets
```bash
# WebAssembly
vx rustup target add wasm32-unknown-unknown
vx cargo build --target wasm32-unknown-unknown

# Windows from Linux
vx rustup target add x86_64-pc-windows-gnu
vx cargo build --target x86_64-pc-windows-gnu

# macOS from Linux
vx rustup target add x86_64-apple-darwin
vx cargo build --target x86_64-apple-darwin
```

### Build Configuration
```toml
# .cargo/config.toml
[build]
target = "x86_64-unknown-linux-gnu"

[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"

[target.wasm32-unknown-unknown]
runner = "wasm-pack"
```

## Integration

### With vx-core
```rust
use vx_core::{Tool, ToolManager};
use vx_tool_rust::RustTool;

let rust_tool = RustTool::new();
let manager = ToolManager::new();

// Install Rust
manager.install_tool(&rust_tool, "1.75.0").await?;

// Execute Rust commands
manager.execute_tool(&rust_tool, &["--version"]).await?;
```

### Plugin Registration
```rust
use vx_core::{Plugin, PluginManager};
use vx_tool_rust::RustPlugin;

let plugin = RustPlugin::new();
let mut manager = PluginManager::new();

manager.register_plugin(Box::new(plugin))?;
```

## Development

### Building
```bash
cd crates/vx-tool-rust
cargo build
```

### Testing
```bash
cargo test
```

### Integration Testing
```bash
# Test with actual Rust installation
cargo test --features integration-tests
```

## Implementation Details

### Tool Structure
- **RustTool**: Main Rust compiler (rustc)
- **CargoTool**: Cargo package manager and build tool
- **RustupTool**: Rustup toolchain manager

### Version Resolution
1. **Project Config**: Check `.vx.toml` for version specification
2. **Global Config**: Fall back to global default
3. **Stable Channel**: Use latest stable if no version specified
4. **Auto-Install**: Download and install if not available

### Installation Process
1. **Rustup Download**: Download rustup installer
2. **Toolchain Install**: Install specified Rust version
3. **Component Setup**: Install default components
4. **Verification**: Verify installation integrity
5. **Environment Setup**: Configure CARGO_HOME and RUSTUP_HOME

## Project Templates

### Binary Application
```bash
# Create new binary project
vx cargo new myapp
cd myapp

# Add dependencies
vx cargo add clap serde

# Build and run
vx cargo build
vx cargo run
```

### Library Crate
```bash
# Create new library
vx cargo new --lib mylib
cd mylib

# Add dependencies
vx cargo add serde --features derive

# Test
vx cargo test
```

### WebAssembly Project
```bash
# Install wasm target
vx rustup target add wasm32-unknown-unknown

# Create project
vx cargo new --lib wasm-project
cd wasm-project

# Configure for wasm
cat >> Cargo.toml << EOF
[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
EOF

# Build for wasm
vx cargo build --target wasm32-unknown-unknown
```

## Error Handling

### Common Errors
- **Network Issues**: Download failures, registry timeouts
- **Permission Errors**: Installation directory access
- **Compilation Errors**: Code compilation failures
- **Dependency Conflicts**: Cargo dependency resolution

### Recovery
```bash
# Reinstall Rust
vx install rust@1.75.0 --force

# Clear Cargo cache
vx cargo clean

# Update dependencies
vx cargo update

# Use system Rust as fallback
vx --use-system-path cargo --version
```

## Performance

- **Fast Compilation**: Rust's efficient compilation
- **Incremental Builds**: Cargo's incremental compilation
- **Parallel Downloads**: Concurrent dependency downloads
- **Build Caching**: Shared build cache across projects

## Security

- **Checksum Verification**: SHA256 verification of downloads
- **HTTPS Only**: Secure downloads from official sources
- **Crate Verification**: Cargo registry verification
- **Sandboxed Builds**: Isolated build environments

## Troubleshooting

### Installation Issues
```bash
# Check Rust installation
vx rustc --version
vx cargo --version

# Verify toolchain
vx rustup show

# Check environment
vx rustup which rustc
vx rustup which cargo

# Force reinstall
vx remove rust@1.75.0
vx install rust@1.75.0
```

### Build Issues
```bash
# Check project
vx cargo check

# Clean build
vx cargo clean
vx cargo build

# Update dependencies
vx cargo update

# Debug build
vx cargo build --verbose
```

### Component Issues
```bash
# List components
vx rustup component list --installed

# Reinstall component
vx rustup component remove rustfmt
vx rustup component add rustfmt

# Update toolchain
vx rustup update
```

## Best Practices

### Project Structure
```
myproject/
├── Cargo.toml
├── Cargo.lock
├── src/
│   ├── main.rs
│   └── lib.rs
├── tests/
│   └── integration_test.rs
├── benches/
│   └── benchmark.rs
└── examples/
    └── example.rs
```

### Cargo.toml Configuration
```toml
[package]
name = "myproject"
version = "0.1.0"
edition = "2021"
rust-version = "1.70"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "my_benchmark"
harness = false
```

## License

This project is licensed under the MIT License - see the [LICENSE](../../../LICENSE) file for details.

## Contributing

Contributions are welcome! Please see the [contributing guidelines](../../../CONTRIBUTING.md) for more information.

## Related Crates

- [`vx-core`](../../vx-core/README.md) - Core functionality
- [`vx-cli`](../../vx-cli/README.md) - Command-line interface
- [`vx-tool-node`](../vx-tool-node/README.md) - Node.js tool
- [`vx-tool-go`](../vx-tool-go/README.md) - Go tool
