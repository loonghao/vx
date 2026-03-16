# VX Coding Conventions

> This is the **source of truth** for coding standards in vx.
> All conventions are enforced by CI linters where possible.

## Terminology (Enforced)

| ✅ Use          | ❌ Never use           | Context          |
|-----------------|------------------------|------------------|
| Runtime         | Tool, VxTool           | Executable tools |
| Provider        | Plugin, Bundle         | Tool providers   |
| ProviderRegistry | BundleRegistry        | Registry         |
| RuntimeSpec     | ToolSpec               | Specifications   |
| provider.star   | provider config file   | DSL files        |

## Naming Conventions

### Crate Names
- Prefix: `vx-` (e.g., `vx-core`, `vx-resolver`)
- Case: kebab-case
- Providers: `vx-providers/<name>/` directory with `vx-provider-<name>` package

### Rust Code
- Structs/Traits: `PascalCase` (e.g., `NodeProvider`, `ManifestDrivenRuntime`)
- Functions/Methods: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`, filename matches module name

### Starlark (provider.star)
- Functions: `snake_case`
- Constants/Dicts: `_SCREAMING_SNAKE` (prefixed with `_` if private)
- Exported symbols: no underscore prefix

## File Organization

### Source Code
```
crates/vx-<name>/
├── src/
│   └── lib.rs           # Module docs required
├── tests/               # All tests here (NEVER inline)
│   └── *_tests.rs       # Use rstest framework
└── Cargo.toml
```

### Tests — NEVER Inline
```rust
// ❌ FORBIDDEN — no inline tests
#[cfg(test)]
mod tests { }

// ✅ CORRECT — tests in crates/<name>/tests/
// File: crates/vx-resolver/tests/resolver_tests.rs
use rstest::rstest;

#[rstest]
#[case("node", Ecosystem::NodeJs)]
fn test_ecosystem(#[case] name: &str, #[case] expected: Ecosystem) {
    // ...
}
```

### Provider Files
```
crates/vx-providers/<name>/
├── provider.star     # Required — Starlark DSL definition
├── provider.toml     # Provider manifest metadata
└── src/lib.rs        # Rust glue (only if needed)
```

## Architecture Rules

### Layer Dependencies
Dependencies flow **downward only**. Never import from a higher layer.

```
Layer 4: vx-cli                    (Application)
Layer 3: vx-resolver, vx-setup     (Orchestration)
Layer 2: vx-runtime, vx-starlark   (Services)
Layer 1: vx-config, vx-env         (Infrastructure)
Layer 0: vx-core, vx-paths         (Foundation)
```

### File Size Limits
- **Source files**: Max 500 lines (split into modules if larger)
- **Test files**: Max 800 lines
- **provider.star**: Max 200 lines (use templates to reduce size)

## Error Handling

### Libraries (crates)
```rust
// Use thiserror for specific error types
#[derive(Debug, thiserror::Error)]
pub enum ResolverError {
    #[error("runtime '{name}' not found")]
    RuntimeNotFound { name: String },
    #[error("dependency cycle detected: {cycle}")]
    DependencyCycle { cycle: String },
}
```

### Application (vx-cli)
```rust
// Use anyhow::Result for convenience
pub async fn run() -> anyhow::Result<()> {
    let config = load_config().context("failed to load configuration")?;
    Ok(())
}
```

## Logging

### Use tracing, NEVER print/eprintln
```rust
// ❌ FORBIDDEN
eprintln!("DEBUG: {}", value);
println!("Installing...");

// ✅ CORRECT
tracing::debug!("loaded {} manifests", count);
tracing::info!("Installing {}@{}", name, version);
```

### Log Levels
| Level | Use Case | Example |
|-------|----------|---------|
| `trace!` | Verbose debug (dev only) | `trace!("item: {:?}", item)` |
| `debug!` | Troubleshooting info | `debug!("path: {}", path.display())` |
| `info!` | Important operations | `info!("Installing {}@{}", n, v)` |
| `warn!` | Non-fatal issues | `warn!("fallback to default")` |
| `error!` | Failures needing attention | `error!("download failed: {}", e)` |

## Dependencies

### Workspace Dependencies
All shared dependencies are declared in root `Cargo.toml` under `[workspace.dependencies]`.
Individual crates reference them with `{ workspace = true }`.

### Version Sync
All internal crates share the same version number (currently in `[workspace.package]`).

## Git & PR Conventions

### Commit Messages
Follow [Conventional Commits](https://www.conventionalcommits.org/):
```
feat: add starship provider
fix: resolve Windows path handling in vx-paths
docs: update architecture overview
refactor: extract sccache setup into composite action
ci: optimize change detection for providers
```

### PR Requirements
- Title follows Conventional Commits
- Tests added/updated for code changes
- CI passes (green)
- No new inline tests (`#[cfg(test)]`)
