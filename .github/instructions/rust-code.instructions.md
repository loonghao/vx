---
applyTo: "**/*.rs"
---

# Rust Code Instructions for vx

## vx-specific Rules

- **Logging**: Always use `tracing` macros (`tracing::info!`, `tracing::debug!`, `tracing::warn!`, `tracing::error!`). NEVER use `println!`, `eprintln!`, or `dbg!`.
- **Error handling**: Use `anyhow::Result` for application code (vx-cli, vx-resolver). Use `thiserror` for library error types (vx-core, vx-paths, etc.).
- **Tests**: Place tests in `crates/<name>/tests/` directories. NEVER write inline `#[cfg(test)]` modules in source files.
- **Testing framework**: Use `rstest` for parameterized tests. Naming: `test_<function>_<scenario>()`.
- **Import order**: stdlib → external crates → internal crates, separated by blank lines.
- **File size**: Keep source files under 500 lines. Split large files into modules.
- **Async**: Use Tokio-based async/await for all I/O operations.

## Terminology (enforced in code, docs, and comments)

- **Runtime** (not Tool, not VxTool) — an executable managed by vx
- **Provider** (not Plugin, not Bundle) — a module that defines how to install/manage a Runtime
- **provider.star** (not "provider config") — the Starlark DSL file
- **ProviderRegistry** (not BundleRegistry)

## Architecture Layer Rule

Layer dependencies go **downward only** — never import from a higher layer:

1. CLI: `vx-cli`
2. Orchestration: `vx-resolver`, `vx-setup`, `vx-project-analyzer`
3. Service: `vx-runtime`, `vx-starlark`, `vx-installer`, `vx-config`, `vx-console`
4. Foundation: `vx-core`, `vx-paths`, `vx-cache`, `vx-versions`, `vx-manifest`
5. Providers: `vx-providers/*`

## Commands

```bash
vx cargo check -p vx-cli          # Type-check one crate
vx cargo test -p vx-starlark      # Test one crate
vx cargo clippy -p vx-resolver -- -D warnings  # Lint one crate
vx just quick                      # format → lint → test → build
```
