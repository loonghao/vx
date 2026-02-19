# Contributing

Thank you for your interest in contributing to vx!

## Getting Started

### Prerequisites

- Rust 1.80+
- Git

### Clone and Build

```bash
git clone https://github.com/loonghao/vx.git
cd vx
cargo build
```

### Cross-Platform Build Notes

vx uses **rustls** (pure Rust TLS implementation) instead of OpenSSL, which enables:

- **No system dependencies**: Cross-compilation to musl targets works out of the box
- **Smaller binaries**: No need to bundle OpenSSL
- **Consistent behavior**: Same TLS implementation across all platforms

The HTTP client (`reqwest`) is configured with:
- `rustls-tls`: Pure Rust TLS backend
- `rustls-tls-native-roots`: Uses system certificate store for trust roots

This means you can build static musl binaries without installing OpenSSL:

```bash
# Build for Linux musl (static binary)
cross build --release --target x86_64-unknown-linux-musl

# Build for ARM64 musl
cross build --release --target aarch64-unknown-linux-musl
```

### Run Tests

```bash
cargo test
```

### Run Clippy

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

### Format Code

```bash
cargo fmt
```

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/my-feature
```

### 2. Set Up Pre-commit Hooks

vx uses [prek](https://prek.j178.dev/) for pre-commit hooks. Install them once after cloning:

```bash
vx prek install
```

This installs hooks that automatically check your code before every commit. See [Pre-commit Hooks](pre-commit-hooks) for the full list of checks.

### 3. Make Changes

- Write code
- Add tests
- Update documentation

### 4. Test Locally

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### 5. Check Code Quality

```bash
# Format
cargo fmt

# Lint
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Check documentation
cargo doc --all-features --no-deps

# Run all pre-commit hooks manually
vx prek run --all-files
```

### 6. Keep workspace-hack in Sync

After adding or updating dependencies in any `Cargo.toml`, regenerate the workspace-hack:

```bash
just hakari-generate
# or manually:
cargo hakari generate
cargo hakari manage-deps
```

The pre-commit hook will catch this automatically if you forget.

### 7. Submit PR

- Push your branch
- Create a pull request
- Fill out the PR template

## Code Guidelines

### Rust Style

- Follow Rust conventions
- Use `rustfmt` for formatting
- Address all Clippy warnings
- Document public APIs

### Testing

- Place tests in `tests/` directories
- Use `rstest` for parameterized tests
- Aim for good coverage

### Documentation

- Document public functions and types
- Include examples in doc comments
- Update user documentation as needed

## Project Structure

```
vx/
├── crates/
│   ├── vx-cli/         # CLI application
│   ├── vx-core/        # Core types and traits
│   ├── vx-paths/       # Path management
│   ├── vx-resolver/    # Version resolution
│   ├── vx-runtime/     # Runtime management
│   └── vx-providers/   # Tool providers
├── book/               # Documentation (mdBook)
├── tests/              # Integration tests
└── examples/           # Example configurations
```

## Adding a New Provider

1. Create crate in `crates/vx-providers/`
2. Implement `Provider` trait
3. Add tests
4. Register in `vx-cli/src/registry.rs`
5. Update documentation

See [Plugin Development](plugin-development) for details.

## Commit Messages

Use conventional commits:

```
feat: add support for new tool
fix: resolve version parsing issue
docs: update installation guide
test: add provider tests
refactor: simplify version resolution
```

## Pull Request Process

1. Ensure CI passes
2. Update documentation
3. Add tests for new features
4. Request review
5. Address feedback

## CI Pipeline

The CI pipeline is optimized with **crate-level change detection** to minimize build times:

### How It Works

1. **Change Detection**: The CI automatically detects which crates have changed
2. **Dependency Analysis**: It understands the dependency graph between crates
3. **Targeted Testing**: Only affected crates are tested

### Crate Dependency Layers

```
┌─────────────────────────────────────────────────────────────┐
│                      vx-cli (Application)                    │
├─────────────────────────────────────────────────────────────┤
│  vx-resolver │ vx-extension │ vx-project-analyzer │ ...     │
├─────────────────────────────────────────────────────────────┤
│                    vx-runtime (Infrastructure)               │
├─────────────────────────────────────────────────────────────┤
│              vx-core │ vx-paths (Foundation)                 │
└─────────────────────────────────────────────────────────────┘
```

### Impact Rules

| Changed Crate | Affected Crates |
|--------------|-----------------|
| `vx-core` | All crates that depend on it (runtime, resolver, extension, etc.) |
| `vx-paths` | runtime, resolver, env, setup, migration, args, extension, cli |
| `vx-runtime` | resolver, extension, cli, all providers |
| `vx-config` | project-analyzer, cli |
| Provider crates | Only the changed provider and cli |
| `vx-cli` | Only cli itself |

### CI Jobs

| Job | Condition | Description |
|-----|-----------|-------------|
| `test-targeted` | Specific crates changed | Tests only affected crates |
| `test-full` | Core crates changed or CI config changed | Full workspace test |
| `code-quality` | Any Rust code changed | Format and Clippy checks |
| `dogfood` | Any Rust code changed | Integration tests with real tools |
| `cross-build` | Main branch only | Cross-compilation for ARM/musl |
| `coverage` | Main branch only | Code coverage report |

### Force Full CI

To run all tests regardless of changes:

1. Go to Actions tab
2. Select "CI" workflow
3. Click "Run workflow"
4. Check "Force full CI run"

## Dependency Constraints

Some dependencies have version constraints that cannot be automatically upgraded:

### bincode (v1.3 → v3.x blocked)

The `bincode` crate is pinned to v1.3 because:

- **msvc-kit** (used for MSVC Build Tools installation on Windows) depends on `bincode ^1.3`
- `bincode v3` has a completely different API (the crate was restructured)
- Until `msvc-kit` releases a version supporting `bincode v3`, we cannot upgrade

**Workaround**: Renovate is configured to skip major `bincode` updates. See [PR #378](https://github.com/loonghao/vx/pull/378) for details.

**Resolution**: Monitor `msvc-kit` releases for `bincode v3` support. Once available:
1. Update `msvc-kit` to the new version
2. Remove the Renovate rule for `bincode`
3. Migrate code to `bincode v3` API

## Reporting Issues

When reporting bugs:

1. Check existing issues
2. Include vx version (`vx --version`)
3. Include OS and shell
4. Provide reproduction steps
5. Include error messages

## Feature Requests

1. Check existing issues/discussions
2. Describe the use case
3. Propose a solution if possible

## Community

- [GitHub Issues](https://github.com/loonghao/vx/issues)
- [GitHub Discussions](https://github.com/loonghao/vx/discussions)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
