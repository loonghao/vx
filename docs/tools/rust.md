# Rust

vx supports Rust through `rustup` and bundled runtimes like `cargo`/`rustc`.

## Runtime Relationship

| Runtime | Role | Recommended Usage |
|---|---|---|
| `rustup` | Toolchain manager / installer | `vx rustup ...` |
| `cargo` | Build, test, dependency management | `vx cargo ...` |
| `rustc` | Rust compiler | `vx rustc ...` |

> `vx rust` is currently an alias to `vx rustc`. For clarity, prefer `vx rustc` in docs/scripts.

## Installation

```bash
# Install rustup runtime (recommended)
vx install rustup
vx install rustup@latest
```

## Daily Commands

### Cargo

```bash
vx cargo --version
vx cargo build --release
vx cargo test
vx cargo run
```

### Rustc

```bash
vx rustc --version
vx rustc main.rs -o main
```

### Rustup (toolchain management)

```bash
vx rustup --version
vx rustup toolchain list
vx rustup target add x86_64-unknown-linux-musl
```

## vx.toml Recommendation

```toml
[tools]
rustup = "latest"

[scripts]
build = "cargo build --release"
test = "cargo test"
lint = "cargo clippy -- -D warnings"
format = "cargo fmt"
```

## Important Version Note

`rustup` version and `rustc` version are different.

- `rustup = "1.93.1"` is usually invalid (that is a Rust compiler version, not a rustup release).
- If you need a specific compiler toolchain, manage it via `vx rustup toolchain ...` commands.

## Package Execution Syntax (Rust Ecosystem)

```bash
# Run rust ecosystem packages on demand
vx cargo:ripgrep::rg --version
vx cargo:fd-find::fd .
```

