# AI Agent Guide (vx)

> Detailed reference for AI agents working **on vx** or **with vx**.
> The short navigation map lives in [`AGENTS.md`](https://github.com/loonghao/vx/blob/main/AGENTS.md) — start there.
> Coding standards live in [`docs/CONVENTIONS.md`](../CONVENTIONS.md).

## Mental model

vx is a transparent proxy: it ensures the right tool version is available, then
forwards the command unchanged. The user never needs to know what happened.

```
User Command: vx npm install
                 │
                 ▼
          ┌─────────────┐
          │   vx CLI     │  ← Parses "npm" as the runtime name
          └──────┬───────┘
                 │
          ┌──────▼───────┐
          │   Resolver    │  ← Looks up "npm" → bundled with "node" Provider
          └──────┬───────┘
                 │
          ┌──────▼───────┐
          │  Is node      │  ← Checks ~/.vx/store/node/<version>/
          │  installed?   │
          └──┬────────┬──┘
          No │        │ Yes
             ▼        ▼
       ┌──────────┐  ┌──────────┐
       │ Install   │  │ Prepare  │  ← Sets PATH, env vars from provider.star
       │ node      │  │ env      │
       └─────┬────┘  └────┬─────┘
             └──────┬─────┘
                    ▼
          ┌─────────────┐
          │ Forward cmd  │  ← Executes: npm install (with correct PATH)
          └─────────────┘
```

Step by step:

1. CLI parses `node` as the runtime name.
2. Resolver looks up `node` in `ProviderRegistry` (via `provider.star`).
3. Resolver checks whether it is installed under `~/.vx/store/node/<version>/`.
4. If missing → installer downloads from the URL returned by `download_url()`.
5. Environment is prepared via `environment()`.
6. The command is forwarded to the real binary with all args.

## Decision framework

```
Is the user working ON vx (developing vx itself)?
├── YES → Use `vx just <task>` for builds, tests, linting
│         Tests go in crates/<name>/tests/ (never inline)
│         Use rstest, tracing, and correct terminology
│         New providers: create provider.star in crates/vx-providers/<name>/
│
└── NO → Is the user working WITH vx (using vx in their project)?
    ├── YES → Always prefix commands: `vx npm install`, `vx cargo build`
    │         Check vx.toml for project tool requirements
    │         Use `vx run <script>` for project scripts
    │         Never suggest manual tool installation
    │
    └── UNCLEAR → Check for vx.toml or .vx/ in project root
                  If found → treat as vx-managed project
                  If not → ask user or suggest `vx init`
```

### Provider development decision tree

```
Need to add a new tool to vx?
├── Tool releases on GitHub?
│   ├── Rust target triple naming?       → github_rust_provider (most common)
│   ├── Go goreleaser style?             → github_go_provider
│   ├── Single binary (no archive)?      → github_binary_provider
│   ├── Irregular / unknown naming?      → github_smart_provider
│   └── Want auto-detect with fallback?  → github_smart_provider
├── System package manager only?         → system_provider
└── Custom download source?              → Hand-write download_url function
```

### Version resolution priority

```
1. Command-line: vx node@22 app.js   (highest)
2. Project vx.toml: [tools] node = "22"
3. Parent directory vx.toml (traverses up)
4. User global: ~/.config/vx/config.toml
5. Provider default: latest stable   (lowest)
```

## Key concepts

| Concept | Definition |
|---|---|
| **Runtime** | An executable tool managed by vx (node, go, uv, ripgrep…) |
| **Provider** | A module that defines how to install/manage a Runtime |
| **provider.star** | Starlark DSL file that declaratively describes a Provider |
| **Ecosystem** | A language/tool family (nodejs, python, rust, go, system, custom) |
| **Bundled Runtime** | A Runtime shipped inside another (npm bundled with node) |
| **Descriptor** | Dict returned by Starlark (phase 1) → interpreted by Rust (phase 2) |
| **Package Alias** | Short command that routes to an ecosystem package (e.g. `vx vite` = `vx npm:vite`) |

## Starlark provider system

vx uses a **two-phase execution model**: `provider.star` runs as pure Starlark
computation (no I/O) returning descriptor dicts, which the Rust runtime then
interprets for actual downloads, installs, and process execution. Five
high-level templates cover 90% of cases.

### Adding a new provider

1. Create `crates/vx-providers/<name>/provider.star`.
2. Use a template (covers 90% of cases):

   ```starlark
   load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
   load("@vx//stdlib:provider_templates.star", "github_rust_provider")

   name        = "<name>"
   description = "<description>"
   ecosystem   = "custom"  # nodejs, python, rust, go, system, custom
   runtimes    = [runtime_def("<runtime>", aliases = ["<alias>"])]
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

3. Define metadata: `name`, `description`, `runtimes`, `permissions`.
4. Test: `vx <runtime> --version`.

### Available templates

| Template | Use case | Placeholders |
|---|---|---|
| `github_rust_provider` | Rust target triple naming | `{version}`, `{vversion}`, `{triple}`, `{ext}`, `{exe}` |
| `github_go_provider` | Go/goreleaser naming | `{version}`, `{os}`, `{arch}`, `{ext}` |
| `github_binary_provider` | Single binary download | `{exe}` |
| `github_smart_provider` | Irregular or unknown asset naming, auto-detect with fallback | same as above |
| `system_provider` | System package manager only | N/A |

### Platform constraints

Return `None` from `download_url` for unsupported platforms:

```starlark
def download_url(ctx, version):
    platform = platform_map(ctx, _PLATFORMS)
    if not platform:
        return None  # Unsupported platform
    return "https://example.com/v{}/tool-{}.tar.gz".format(version, platform)
```

Complete DSL reference: [`docs/guide/provider-star-reference.md`](../guide/provider-star-reference.md).
Full walkthrough: [`docs/guide/creating-provider.md`](../guide/creating-provider.md).

## Common scenarios

| Scenario | What to do |
|---|---|
| User says "install Node.js" | `vx node --version` (auto-installs) or `vx install node@22` |
| User says "run npm test" | `vx npm test` |
| User says "global install CLI (all languages)" | Prefer ecosystem-native global form (`vx npm install -g <pkg>`, `vx pip install --user <pkg>`, `vx cargo install <pkg>`, `vx go install <module>@<ver>`, `vx gem install <pkg>`) |
| User says "set up project" | Check for `vx.toml`, then `vx setup` |
| User says "add Python to project" | `vx add python@3.12` then `vx sync` |
| User says "use vite" | `vx vite` (package alias, auto-routes to `vx npm:vite`) |
| MCP server needs `npx` | Use `"command": "vx", "args": ["npx", ...]` in the MCP config |
| Need to check tool version | `vx which <tool>` or `vx <tool> --version` |
| CI/CD setup | Use `loonghao/vx@main` GitHub Action with `setup: 'true'` |
| Developing vx itself | `vx just quick` for format → lint → test → build |
| User encounters errors | `vx doctor` first, then `vx --debug <command>` |
| Need to update vx itself | `vx self-update` |
| Analyze project structure | `vx analyze --json` |

## Command syntax guardrails

**Single source of truth**: [`docs/guide/command-syntax-rules.md`](../guide/command-syntax-rules.md).

| Pattern | Example |
|---|---|
| `vx <runtime>[@version] [args...]` | `vx node@22 app.js` |
| `vx <runtime>[@version]::<executable> [args...]` | `vx msvc@14.42::cl main.cpp` |
| `vx <ecosystem>:<package>[@version] [args...]` | `vx uvx:pyinstaller --version` |
| `vx --with <runtime> <target_command>` | `vx --with bun@1.1.0 node app.js` |
| `vx shell launch <runtime>[@version] [shell]` | `vx shell launch node@22 powershell` |
| `vx pkg <install\|uninstall\|list\|info\|update> ...` | `vx pkg install npm:typescript` |
| `vx run`, `vx sync`, `vx lock`, `vx check` | Project-aware execution |

## MCP integration

When configuring MCP servers, always use `vx` instead of `npx` or `uvx`:

```json
{
  "mcpServers": {
    "server-name": {
      "command": "vx",
      "args": ["npx", "-y", "@scope/package@latest"]
    }
  }
}
```

Benefits: no Node.js/Python pre-install required, one config across Windows,
macOS and Linux, and tool versions managed by `vx.toml` when present.

| Replace | With |
|---|---|
| `"command": "npx"` | `"command": "vx", "args": ["npx", ...]` |
| `"command": "uvx"` | `"command": "vx", "args": ["uvx", ...]` |
| `"command": "node"` | `"command": "vx", "args": ["node", ...]` |
| `"command": "python"` | `"command": "vx", "args": ["python", ...]` |

Common servers:

```json
{
  "filesystem": { "command": "vx", "args": ["npx", "-y", "@modelcontextprotocol/server-filesystem", "/path"] },
  "github": { "command": "vx", "args": ["npx", "-y", "@modelcontextprotocol/server-github"] },
  "sqlite": { "command": "vx", "args": ["uvx", "mcp-server-sqlite", "--db-path", "db.sqlite"] }
}
```

## Testing conventions

See [`docs/CONVENTIONS.md`](../CONVENTIONS.md) for code style, error handling,
logging and dependency rules. Testing specifics:

- Tests go in `crates/<name>/tests/` — **never** inline `#[cfg(test)]`.
- Use `rstest` for parameterized tests.
- Name tests `test_<function_name>_<scenario>()`:

  ```rust
  #[test]
  fn test_ecosystem_detection_returns_nodejs() { }

  #[tokio::test]
  async fn test_resolve_missing_runtime_errors() { }
  ```

- Mock network calls in unit tests — never use real HTTP.
- Use the mock utilities in `vx-runtime::testing` where available.
- E2E tests use `trycmd`.
- Static provider checks: `vx just test-providers-static`.

## GitHub Actions integration

```yaml
- uses: loonghao/vx@main
  with:
    tools: 'node@22 uv'
    setup: 'true'
    cache: 'true'
    github-token: ${{ secrets.GITHUB_TOKEN }}
- run: vx node --version
- run: vx npm test
```

Use `@main` for latest, or pin to a release tag.
Full guide: [`docs/guides/github-action.md`](../guides/github-action.md).

## Multi-agent development (`vx wt`)

```bash
vx wt switch feat/add-new-provider   # Create worktree + branch
vx wt list                           # List all worktrees
vx wt merge                          # Merge current worktree
vx wt remove                         # Remove worktree
```

Typical workflow: create worktrees → agents work in parallel → merge
independently → remove worktrees.

## Process introspection (`vx witr`)

```bash
vx witr nginx                # Inspect by name
vx witr --pid 1234           # Inspect by PID
vx witr --port 5432          # Find process on port
vx witr postgres --tree      # Show process ancestry
vx witr --json               # JSON output for scripting
```

See [`docs/tools/witr.md`](../tools/witr.md).

## Search, build, package commands

```bash
# Search in project
vx rg "pattern"              # ripgrep (fast text search)
vx fd "filename"             # fd-find (fast file search)
vx fzf "pattern"             # fzf (fuzzy search)

# Build commands
vx cargo build               # Rust build
vx cmake --build .           # CMake build
vx make                      # Make build
vx ninja                     # Ninja build
vx meson compile             # Meson build

# Package management
vx cargo package             # Create Rust crate package
vx npm pack                  # Create npm package
vx python -m build           # Python package (with build)
vx dpkg-deb --build          # Debian package

# Containers
vx podman build -t app .     # Build container image
vx docker build -t app .     # Docker build
```

## Diagnostics

```bash
vx doctor                                # 1. Check vx health
vx list --installed                      # 2. Check installed tools
vx which node && vx node --version       # 3. Verify a specific tool
vx --debug node --version                # 4. Verbose debug output
vx cache clean && vx install node --force  # 5. Clean & retry
vx check --json                          # 6. Check project config
```

For a full error-by-error decision tree, see
[`docs/appendix/troubleshooting.md`](../appendix/troubleshooting.md).

### Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | General error |
| 2 | Tool not found |
| 3 | Installation failed |
| 4 | Version not found |
| 5 | Network error |
| 6 | Permission error |
| 7 | Configuration error |

### Token-efficient output

```bash
vx list --json                   # JSON output
vx list --output-format toon     # Token-oriented structured output
vx --compact gh run view 123 --log  # Compact forwarded subprocess output
vx analyze --json                # Project analysis
```

For CI and other huge logs, use this order: `vx gh ... --json --jq` for status,
`vx gh ... --log | vx rg -n -m 80 "error|failed|panic|Traceback"` for focused
matches, then `vx --compact ...` when broad context is still needed. Default
`vx git` and `vx gh` remain transparent forwarded commands; compacting is an
explicit opt-in.

### Efficient git/GitHub patterns

```powershell
# PowerShell (Windows) — keep only last N lines
vx git checkout main 2>&1 | Select-Object -Last 3
vx git pull --ff-only 2>&1 | Select-Object -Last 2
vx git checkout -b feat/my-feature 2>&1 | Select-Object -Last 2
vx gh pr list --json number,title,state --jq '.[:5]'
```

```bash
# Unix (bash/zsh) — keep only last N lines
vx git checkout main 2>&1 | tail -3
vx git pull --ff-only 2>&1 | tail -2
vx git checkout -b feat/my-feature 2>&1 | tail -2

# Cross-platform
vx git log --oneline -5                                  # concise history
vx gh pr list --json number,title,state --jq '.[:5]'     # structured, minimal
vx gh run list --json status,conclusion,name --jq '.[:3]' # CI status
```

Never dump a full git log or `gh` output into agent context.

## Allowed vs. needs-approval actions

### Allowed without asking

- Read any file.
- `vx just quick` / `test` / `lint` / `format`; `vx cargo check/test -p <crate>`.
- Create or modify `crates/vx-providers/` (new providers) or `crates/*/tests/`.
- Run `vx <tool> --version`.

### Ask first

- Deleting files.
- Modifying `Cargo.toml` workspace dependencies.
- `vx git push`.
- Modifying CI workflows.
- Installing system packages.
- Running the full E2E suite.
- Changing layer boundaries.

## Security considerations

- Downloads come from official sources (GitHub Releases, official APIs);
  checksums are verified automatically.
- `permissions` in `provider.star` declares which network hosts a provider may
  access.
- Never run `sudo vx install` — vx manages user-level installations under `~/.vx/`.
- Set `GITHUB_TOKEN` to avoid GitHub API rate limits.

See [`docs/advanced/security.md`](security.md) for more.
