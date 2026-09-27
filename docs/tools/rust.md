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

## Version Resolution Priority

Rust is the one ecosystem where vx is **not** the source of truth. If the repository
already names a toolchain, vx hands the command straight to rustup — it does not
install anything, does not put its own store on `PATH`, and never runs
`rustup default` (which would rewrite the default for every other project on the
machine).

Highest priority wins:

| Priority | Signal | Behaviour |
|---|---|---|
| 1 | `rust-toolchain.toml` / `rust-toolchain` | Delegate to rustup. Searched from the working directory upward. |
| 2 | `RUSTUP_TOOLCHAIN` environment variable | Delegate to rustup. |
| 3 | `rust = "rustup-managed"` in `vx.toml` | Delegate to rustup, explicitly. |
| 4 | `rust = "<version>"` in `vx.toml` | vx installs that toolchain into its own store. |

Notes:

- When both (1) and (2) are present, (2) wins — that is rustup's own precedence, and
  the toolchain vx reports is the one that will actually run.
- A numeric `vx.toml` pin that disagrees with the effective toolchain is reported on
  stderr, and `vx check` exits non-zero. The repository's toolchain file still wins;
  the warning exists so a dead pin cannot go unnoticed.
- Remove the `rust` pin (or set `rust = "rustup-managed"`) to record that rustup owns
  the toolchain and silence the warning.
- An explicit `vx cargo@1.90.0` is the caller naming the toolchain, so it always goes
  through vx rather than the toolchain file.

```toml
# Recommended for repositories that already pin with rustup
[tools]
rust = "rustup-managed"
```

## Package Execution Syntax (Rust Ecosystem)

```bash
# Run rust ecosystem packages on demand
vx cargo:ripgrep::rg --version
vx cargo:fd-find::fd .
```

