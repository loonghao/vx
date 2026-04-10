# CLAUDE.md — vx Project Instructions for Claude Code

> This file is read by Claude Code at the start of every conversation.
> For the full AI agent guide, see @AGENTS.md.

## Project

vx is a **universal development tool manager** (v0.8.20, Rust, MIT). Users prefix any command with `vx` and tools auto-install on first use. 119 providers defined via Starlark DSL (`provider.star`).

```bash
vx node --version     # Auto-installs Node.js
vx cargo build        # Auto-installs Rust
vx npm install        # npm is bundled with Node.js
vx uv pip install x   # Auto-installs uv
```

## Commands You'll Need

```bash
vx just quick         # format → lint → test → build (pre-commit cycle)
vx just fmt           # Format code (rustfmt)
vx just lint          # Run clippy
vx just test          # Run tests
vx just build         # Build debug binary
vx cargo test -p vx-starlark   # Test a single crate
vx cargo check -p vx-cli       # Type-check a single crate
```

## Core Rules

1. **Always use `vx` prefix** — `vx npm install`, never `npm install`
2. **Terminology** — Runtime (not Tool), Provider (not Plugin), provider.star (not config)
3. **Tests in `crates/<name>/tests/`** — NEVER use inline `#[cfg(test)]` modules
4. **Use `rstest`** for parameterized tests
5. **Logging** — `tracing::info!`, `tracing::debug!` — NEVER `println!` or `eprintln!`
6. **Error handling** — `anyhow::Result` (application), `thiserror` (library)
7. **Commits** — Conventional Commits: `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`
8. **Layer deps go downward only** — CLI → Orchestration → Service → Foundation → Providers
9. **New providers** — Starlark DSL only (`provider.star`), no Rust code needed
10. **File size** — Keep source files under 500 lines

## Architecture (5 Layers)

```
CLI:           vx-cli (entry point, command parsing)
Orchestration: vx-resolver, vx-setup, vx-project-analyzer
Service:       vx-runtime, vx-starlark, vx-installer, vx-config, vx-console
Foundation:    vx-core, vx-paths, vx-cache, vx-versions, vx-manifest
Providers:     vx-providers/* (105 Starlark DSL definitions)
```

## Key Paths

| Task | Path |
|------|------|
| Add CLI subcommand | `crates/vx-cli/src/commands/` |
| Modify execution logic | `crates/vx-resolver/src/executor.rs` |
| Add Starlark stdlib function | `crates/vx-starlark/stdlib/*.star` |
| Add new provider | `crates/vx-providers/<name>/provider.star` |
| Add project detection | `crates/vx-project-analyzer/src/frameworks/` |
| Change console output | `crates/vx-console/src/` |
| View RFCs (design decisions) | `docs/rfcs/` (50 RFCs) |

## New Provider Template

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

## Common Pitfalls

- **Don't suggest manual tool installation** — vx handles it. Say `vx node --version`, not "install Node.js first".
- **Don't use `sudo` with vx** — it manages user-level installations under `~/.vx/`.
- **Don't import upward** — Foundation crates cannot import from Service layer.
- **Provider count is 105** — update any docs that reference old counts (78, 73, 70+, 50+).
- **Don't use `println!`** — always use `tracing::info!`, `tracing::debug!` etc.
- **Don't write inline tests** — tests go in `crates/<name>/tests/`, never `#[cfg(test)]`.

## MCP Integration

Replace `npx`/`uvx` with `vx` in MCP server configs for zero-config tool management:

```json
{
  "mcpServers": {
    "server": {
      "command": "vx",
      "args": ["npx", "-y", "@example/mcp-server@latest"]
    }
  }
}
```

## Deeper References

- @AGENTS.md — Full AI agent guide with execution flow, mental model, and scenarios
- @docs/guide/provider-star-reference.md — Complete Starlark DSL reference
- @docs/architecture/OVERVIEW.md — Architecture deep dive
- @docs/CONVENTIONS.md — Coding conventions
- @llms.txt — LLM-friendly project index
