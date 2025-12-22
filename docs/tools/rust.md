# Rust

vx provides support for the Rust programming language and its toolchain.

## Supported Tools

| Tool | Description |
|------|-------------|
| `rust` | Rust toolchain (rustc, cargo, etc.) |
| `cargo` | Rust package manager |
| `rustc` | Rust compiler |

## Installation

```bash
vx install rust stable
vx install rust nightly
vx install rust 1.75.0
```

## Version Specifiers

```bash
rust stable      # Stable channel
rust beta        # Beta channel
rust nightly     # Nightly channel
rust 1.75.0      # Specific version
```

## Cargo

### Basic Commands

```bash
vx cargo --version
vx cargo new my-project
vx cargo init
```

### Building

```bash
vx cargo build
vx cargo build --release
vx cargo build --target x86_64-unknown-linux-musl
```

### Running

```bash
vx cargo run
vx cargo run --release
vx cargo run -- --arg value
```

### Testing

```bash
vx cargo test
vx cargo test --release
vx cargo test -- --nocapture
```

### Dependencies

```bash
vx cargo add serde
vx cargo add tokio --features full
vx cargo remove serde
vx cargo update
```

### Publishing

```bash
vx cargo publish --dry-run
vx cargo publish
```

## Rustc

```bash
vx rustc --version
vx rustc main.rs -o main
```

## Project Configuration

```toml
[tools]
rust = "stable"

[scripts]
build = "cargo build --release"
test = "cargo test"
lint = "cargo clippy -- -D warnings"
format = "cargo fmt"
doc = "cargo doc --open"
```

## Common Workflows

### New Project

```bash
vx cargo new my-project
cd my-project
vx cargo run
```

### Library Project

```bash
vx cargo new --lib my-lib
cd my-lib
vx cargo test
```

### Web Server with Axum

```bash
vx cargo new my-server
cd my-server
vx cargo add axum tokio --features tokio/full
# Create src/main.rs
vx cargo run
```

### CLI Tool with Clap

```bash
vx cargo new my-cli
cd my-cli
vx cargo add clap --features derive
# Create src/main.rs
vx cargo build --release
```

## Cross-Compilation

```bash
# Add target
vx rustup target add x86_64-unknown-linux-musl

# Build for target
vx cargo build --release --target x86_64-unknown-linux-musl
```

## Code Quality

```bash
# Format code
vx cargo fmt

# Lint with Clippy
vx cargo clippy

# Check without building
vx cargo check
```

## Tips

1. **Use stable for production**: Unless you need nightly features
2. **Run clippy regularly**: Catches common mistakes
3. **Use cargo fmt**: Consistent formatting
4. **Pin rust-toolchain.toml**: For team consistency
