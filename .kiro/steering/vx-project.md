# VX Project Steering — Kiro AI IDE

## Project Context

vx is a **universal development tool manager** (v0.8.19, Rust, MIT) that ships 105 providers.
Users prefix commands with `vx` and tools auto-install on first use. Providers use Starlark DSL.

## Critical Rules

1. **Always** use `vx` prefix: `vx npm install`, `vx cargo build`
2. **Never** suggest manual tool installation
3. **Terminology**: Runtime (not Tool), Provider (not Plugin), provider.star (not config)
4. **Tests** in `crates/<name>/tests/` — never inline `#[cfg(test)]`
5. **Logging**: `tracing` macros only — never `println!`
6. **Errors**: `anyhow::Result` (app), `thiserror` (lib)
7. **Layers go downward**: CLI → Orchestration → Service → Foundation → Providers
8. **New providers**: Starlark DSL only, no Rust code needed
9. **Pre-commit**: `vx just quick` (format → lint → test → build)
10. **Commits**: Conventional Commits format

## Key Paths

| Task | Where |
|------|-------|
| CLI subcommand | `crates/vx-cli/src/commands/` |
| Execution logic | `crates/vx-resolver/src/executor.rs` |
| Starlark stdlib | `crates/vx-starlark/stdlib/*.star` |
| New provider | `crates/vx-providers/<name>/provider.star` |
| Project detection | `crates/vx-project-analyzer/src/frameworks/` |
| Console output | `crates/vx-console/src/` |

## Provider Template

```starlark
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

## Architecture (5 Layers)

```
CLI:           vx-cli (entry point)
Orchestration: vx-resolver, vx-setup, vx-project-analyzer
Service:       vx-runtime, vx-starlark, vx-installer, vx-config, vx-console
Foundation:    vx-core, vx-paths, vx-cache, vx-versions, vx-manifest
Providers:     vx-providers/* (105 Starlark DSL definitions)
```

## References

- Full guide: `AGENTS.md`
- Provider DSL: `docs/guide/provider-star-reference.md`
- Architecture: `docs/architecture/OVERVIEW.md`
