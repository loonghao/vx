# GitHub Copilot Instructions for vx

## Project Overview

vx is a **universal development tool manager** (v0.8.25, Rust, MIT, 129 providers).
Users prefix any command with `vx` (e.g., `vx node --version`, `vx cargo build`) and vx automatically installs, manages, and forwards to the correct tool version. Providers defined via Starlark DSL (`provider.star`).

## Critical Rules

### Commands
- **Always use `vx` prefix**: `vx npm install`, `vx cargo build`, `vx go run main.go`
- **Never suggest manual tool installation** — vx handles it automatically
- **Task runner**: Use `vx just <task>` (see `justfile` for all commands)
- **Quick pre-commit cycle**: `vx just quick` (format → lint → test → build)

### Terminology (enforced)
- Use **Runtime** (not Tool, not VxTool)
- Use **Provider** (not Plugin, not Bundle)
- Use **provider.star** (not provider config)
- Use **ProviderRegistry** (not BundleRegistry)

### Code Style
- **Language**: Rust (edition 2024, MSRV 1.93+)
- **Error handling**: `anyhow::Result` for application code, `thiserror` for library error types
- **Async**: Tokio-based async/await for all I/O
- **Logging**: Always use `tracing` macros (`tracing::info!`, `tracing::debug!`), never `println!` or `eprintln!`
- **Imports**: Group by stdlib → external crates → internal crates, separated by blank lines
- **File size**: Keep source files under 500 lines

### Testing
- Tests go in `crates/<name>/tests/` directories — **NEVER** inline `#[cfg(test)]` modules in source files
- Use `rstest` for parameterized tests
- Test file naming: `<feature>_tests.rs`
- Mock network calls in unit tests — never use real HTTP

### Architecture
- **Layer dependencies go downward only** — never import from a higher layer
- Layers (top to bottom): CLI → Orchestration → Service → Foundation → Providers

### New Providers
- Create `crates/vx-providers/<name>/provider.star` — no Rust code required
- Use templates: `github_rust_provider`, `github_go_provider`, `github_binary_provider`, `system_provider`
- Test with: `vx <runtime> --version`

## Key Files

| Task | File(s) |
|------|---------|
| Add CLI subcommand | `crates/vx-cli/src/commands/` |
| Modify execution logic | `crates/vx-resolver/src/executor.rs` |
| Add Starlark stdlib function | `crates/vx-starlark/stdlib/*.star` |
| Add new provider | `crates/vx-providers/<name>/provider.star` |
| Add project detection | `crates/vx-project-analyzer/src/frameworks/` |
| Change console output | `crates/vx-console/src/` |

## Commit Conventions

Use [Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`

## For More Details

See [AGENTS.md](../AGENTS.md) for the complete AI agent guide.
