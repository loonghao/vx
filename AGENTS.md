# AGENTS.md — vx

> Zero-config universal development tool manager (Rust, MIT). Users prefix any
> command with `vx` (`vx node --version`, `vx cargo build`) and vx installs,
> manages and forwards to the correct tool version. Providers are defined in
> Starlark DSL (`provider.star`).
> Navigation map for AI agents, not a reference manual. Follow the links; do not
> read everything up front.

**Key insight**: vx is a transparent proxy. The user writes the exact commands
they already know, just prepended with `vx`. There is no new syntax to learn.

## Build & test

```bash
vx just quick                  # format -> lint -> test -> build (run before a PR)
vx just build                  # debug build (vx cargo build)
vx just test                   # cargo nextest run --workspace --no-fail-fast
vx just lint                   # cargo clippy --workspace --all-targets -D warnings
vx just format                 # cargo fmt
vx just pre-merge              # format-check + lint + check-architecture + test-fast
vx just test-providers-static  # static provider checks
vx just doctor                 # diagnose the development environment
```

Full recipe list: `vx just --list`. Never invent a recipe — check the `justfile`.
Single crate: `vx cargo test -p <crate-name>`.

## Repo layout

| Path | Role |
|---|---|
| `crates/vx-cli/` | Application layer — CLI entry point |
| `crates/vx-resolver/`, `vx-setup/`, `vx-project-analyzer/` | Orchestration — resolve, execute, env setup, project detection |
| `crates/vx-runtime/`, `vx-starlark/`, `vx-installer/`, `vx-config/`, `vx-console/` | Services — runtime registry, DSL engine, install, config, output |
| `crates/vx-runtime-core/`, `vx-paths/`, `vx-cache/`, `vx-versions/`, `vx-manifest/` | Foundation — traits, paths, cache, semver, provider manifests |
| `crates/vx-providers/` | Provider definitions, one `provider.star` per runtime |
| `crates/vx-starlark/stdlib/` | Starlark standard library (facade, 5 templates, 14 modules) |
| `tests/` | Workspace-level integration tests |
| `docs/` | Documentation site; `docs/zh/` is the Chinese mirror |
| `llms.txt` / `llms-full.txt` | LLM-friendly project index (concise / full) |

**Dependency rule**: each layer may only depend on layers *below* it. Never upward.

## Reference map

| I need to… | Read this |
|---|---|
| Full AI-agent reference (decision trees, diagnostics, exit codes, MCP) | [`docs/advanced/ai-agent-guide.md`](docs/advanced/ai-agent-guide.md) |
| Coding standards (source of truth) | [`docs/CONVENTIONS.md`](docs/CONVENTIONS.md) |
| Understand the architecture | [`docs/architecture/OVERVIEW.md`](docs/architecture/OVERVIEW.md) |
| Design decisions (RFCs) | [`docs/rfcs/`](docs/rfcs/) |
| Add a new provider | [`docs/guide/creating-provider.md`](docs/guide/creating-provider.md) |
| Starlark DSL reference | [`docs/guide/provider-star-reference.md`](docs/guide/provider-star-reference.md) |
| Command syntax guardrails | [`docs/guide/command-syntax-rules.md`](docs/guide/command-syntax-rules.md) |
| All CLI commands | [`docs/cli/`](docs/cli/) |
| All providers | [`docs/tools/overview.md`](docs/tools/overview.md) |
| `vx.toml` configuration | [`docs/config/vx-toml.md`](docs/config/vx-toml.md) |
| CI pipeline | [`.github/workflows/ci.yml`](.github/workflows/ci.yml) |
| Contributing | [`docs/advanced/contributing.md`](docs/advanced/contributing.md) |
| Troubleshooting | [`docs/appendix/troubleshooting.md`](docs/appendix/troubleshooting.md) |
| Metrics & telemetry | [`docs/advanced/metrics-analysis.md`](docs/advanced/metrics-analysis.md) |

## Release

- release-please drives versioning from Conventional Commits on `main`.
- `feat:` → minor, `fix:` → patch, `chore:`/`docs:`/`ci:` → **no release**.
- Use `chore:`/`docs:` for config and doc work so release-please does not cut a
  valueless version.
- release-please also bumps `Cargo.toml`. Never edit that version by hand; the
  current version lives in [`.release-please-manifest.json`](.release-please-manifest.json).

## Do / Don't

- **Do** single-source agent instructions here. This is the only agent contract
  file at the repo root.
- **Don't** add `CLAUDE.md` / `GEMINI.md` / `CURSOR.md` / `ANTHROPIC.md` /
  `OPENAI.md` / `COPILOT.md` / `CODEBUDDY.md` / `.cursorrules` / `.clinerules` /
  `.windsurfrules` at the root. Vendor-specific notes live under
  [`docs/integrations/`](docs/integrations/), linked from here.
- **Do** prefix every command with `vx` — `vx npm install`, never bare `npm install`.
  This includes git and GitHub: `vx git status`, `vx gh pr create`.
- **Don't** tell the user to install a tool manually. Just run it: `vx node --version`
  auto-installs.
- **Do** use the enforced terminology: Runtime (not Tool), Provider (not Plugin),
  `provider.star` (not provider config), ProviderRegistry (not BundleRegistry).
- **Do** check `vx.toml` before suggesting commands in a vx-managed project.
- **Do** put tests in `crates/<name>/tests/` — **never** inline `#[cfg(test)]`.
  Use `rstest`, name tests `test_<function_name>_<scenario>()`, and mock network
  calls instead of using real HTTP.
- **Do** order imports stdlib → external → internal, separated by blank lines.
- **Do** use `anyhow::Result` in `vx-cli` and `thiserror` in library crates; log
  with `tracing::info!`, never `println!`.
- **Do** return `None` from `download_url` for unsupported platforms; do not
  fabricate a URL.
- **Do** use `vx` instead of `npx` / `uvx` in MCP server configs
  (`"command": "vx", "args": ["npx", ...]`).
- **Don't** quote a provider or version count from memory — read
  `crates/vx-providers/` and `.release-please-manifest.json`.
- **Don't** hardcode an exact version in tests (`assert __version__ == "X.Y.Z"`)
  — release-please bumps will break it. Use `>=` or read package metadata.
- **Don't** commit build artifacts to the repo root (`*.o`, `coverage.json`,
  `audit-result.json`, `clippy_check.txt`, `commit_msg.txt`).

## Vendor-specific notes

| Agent | Notes |
|---|---|
| Claude Code | [`docs/integrations/claude.md`](docs/integrations/claude.md) |
| Google Gemini | [`docs/integrations/gemini.md`](docs/integrations/gemini.md) |
| Cursor | [`docs/integrations/cursor.md`](docs/integrations/cursor.md) |
| Windsurf | [`docs/integrations/windsurf.md`](docs/integrations/windsurf.md) |
| Cline | [`docs/integrations/cline.md`](docs/integrations/cline.md) |

## Quick orientation for agents using vx

```bash
vx setup                 # install all tools declared in vx.toml
vx dev                   # enter the dev environment
vx run <script>          # run a project script
vx install node@22       # version always with @, never a bare second argument
vx vite                  # package alias, same as: vx npm:vite
vx doctor                # first stop when something fails
```

This file follows the [AGENTS.md](https://agents.md/) open standard.
