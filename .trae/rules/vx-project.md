# VX Project Rules — Trae AI IDE

## Project Context

vx is a **universal development tool manager** (v0.8.19, Rust, MIT, 105 providers).
Users prefix commands with `vx` (e.g., `vx node --version`) and tools auto-install on first use.
Providers are defined via Starlark DSL (`provider.star`) — no Rust code needed.

## Essential Rules

1. **Always** use `vx` prefix: `vx npm install`, `vx cargo build`, `vx go run main.go`
2. **Never** suggest manual tool installation — vx handles it
3. **Terminology**: Runtime (not Tool), Provider (not Plugin), provider.star (not config)
4. **Tests** in `crates/<name>/tests/` — NEVER inline `#[cfg(test)]`
5. **Framework**: Use `rstest` for parameterized tests
6. **Logging**: `tracing::info!`, `tracing::debug!` — NEVER `println!`
7. **Errors**: `anyhow::Result` (app), `thiserror` (lib)
8. **Architecture**: Layer dependencies go downward only
9. **Pre-commit**: `vx just quick` (format → lint → test → build)
10. **Commits**: Conventional Commits (`feat:`, `fix:`, `docs:`, `chore:`)

## Architecture Layers (top → bottom)

1. **CLI**: `vx-cli` — entry point
2. **Orchestration**: `vx-resolver`, `vx-setup`, `vx-project-analyzer`
3. **Service**: `vx-runtime`, `vx-starlark`, `vx-installer`, `vx-config`, `vx-console`
4. **Foundation**: `vx-core`, `vx-paths`, `vx-cache`, `vx-versions`, `vx-manifest`
5. **Providers**: `vx-providers/*` — 105 Starlark DSL definitions

## New Provider (copy-paste template)

```starlark
# crates/vx-providers/<name>/provider.star
load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:provider_templates.star", "github_rust_provider")

name        = "<name>"
description = "<description>"
ecosystem   = "custom"
runtimes    = [runtime_def("<runtime>")]
permissions = github_permissions()

_p = github_rust_provider("owner", "repo",
    asset = "tool-{vversion}-{triple}.{ext}")
fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]
```

## References

- [AGENTS.md](AGENTS.md) — Full AI agent guide
- [docs/guide/provider-star-reference.md](docs/guide/provider-star-reference.md) — Provider DSL
