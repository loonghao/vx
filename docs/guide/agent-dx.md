# Agent DX: vx CLI for AI Agents

> This document describes vx's "Agent Developer Experience" (Agent DX) features —
> a set of improvements that make vx predictable and safe for AI coding agents,
> following [Google Cloud's best practices](https://justin.poehnelt.com/posts/rewrite-your-cli-for-ai-agents/)
> (Justin Poehnelt, March 2026).

## Core Principle

**Human DX** optimizes for discoverability.  
**Agent DX** optimizes for predictability.

AI agents fail differently than humans:
- Agents hallucinate paths (`../../.ssh/id_rsa`) instead of typos
- Agents embed query params in IDs (`node?version=20`)
- Agents can't handle rich text output — they need machine-readable data
- Agents consume context-window tokens on static docs they can't verify

vx addresses all of these.

---

## 1. Auto Token-Oriented Output (TTY Detection)

When stdout is **not a TTY** (piped to an agent, script, or CI pipeline), vx
automatically switches from human-readable text to TOON — a token-oriented
structured format for LLM prompts.

```bash
# Human (TTY): rich text with emoji
vx list

# Agent (pipe): token-oriented TOON
vx list | cat
# runtimes[...]{name,installed,description}:
```

### Override

```bash
# Force text even in pipelines
VX_OUTPUT=text vx list | cat

# Force JSON even in terminal
vx list --json
vx list --output-format json

# Force TOON or compact output
vx list --toon
vx list --output-format toon
vx --compact list node
```

---

## 2. Field Masks (`--fields`)

Agents have limited context windows. Use `--fields` to return only the fields you need:

```bash
# Only return name and version — saves ~80% tokens
vx list --json --fields name,installed

# Only return version string and lts flag
vx versions node --json --fields version,lts

# Only return path
vx which node --json --fields path
```

The `--fields` flag is global and applies to all JSON-outputting commands.

---

## 3. Compact Subprocess Output

Forwarded tools keep their native stdout by default. When a command can produce
huge logs and no better structured output is available, opt in to compact mode:

```bash
# GitHub Actions status first, without logs
vx gh run view 123 --json status,conclusion,jobs --jq '.jobs[] | {name,conclusion}'

# Capped log search next
vx gh run view 123 --log | vx rg -n -m 80 "error|failed|panic|Traceback|FAILED|warning"

# Broad fallback with RTK-style filtering
vx --compact gh run view 123 --log
vx --compact --filter-level aggressive cargo test
```

Compact mode strips ANSI noise, collapses repeated lines, preserves
error-looking lines, and applies a line budget. Use `--filter-level light`,
`normal`, or `aggressive` to tune how much output is suppressed.

---

## 4. Schema Introspection (`vx schema`)

Agents can discover what vx accepts at runtime — no static docs, no hallucinations:

```bash
# Schema for the node runtime
vx schema node

# All runtimes as NDJSON (one per line, streamable)
vx schema --all

# All vx sub-commands as JSON
vx schema --commands
```

Example output for `vx schema node`:
```json
{
  "name": "node",
  "aliases": ["nodejs"],
  "description": "Node.js - JavaScript runtime",
  "ecosystem": "nodejs",
  "platform_supported": true,
  "version_examples": [
    "vx node --version",
    "vx install node@latest",
    "vx install node@<semver>"
  ],
  "install_command": "vx install node",
  "execute_example": "vx node --version",
  "bundled_runtimes": ["npm", "npx"]
}
```

---

## 5. Input Validation (Defense Against Hallucinations)

vx treats all CLI inputs as untrusted — agents can hallucinate adversarial values.
The `input_validation` module rejects:

| Attack Pattern | Example | Status |
|----------------|---------|--------|
| Path traversal | `../../.ssh/id_rsa` | `error` |
| Control characters | `node\x01` | `error` |
| Embedded query params | `node?version=20` | `error` |
| URL encoding | `%2enode` | `error` |
| Shell metacharacters | `node;rm -rf /` | `error` |
| Null bytes | `node\x00` | `error` |

Errors are clear and actionable:
```
Error: Invalid runtime name 'node?version=20': embedded query parameters not allowed
```

---

## 6. Disable Auto-Install (`--no-auto-install`)

Agents in CI/CD pipelines should have explicit install steps. Use this to prevent
silent auto-installation:

```bash
# Fail if node is not already installed
vx --no-auto-install node --version

# Or via environment variable (for CI/CD)
VX_NO_AUTO_INSTALL=1 vx node --version
```

---

## 7. Dry-Run Mode (`--dry-run`)

Agents can validate operations before executing them, preventing data loss from
hallucinated parameters:

```bash
# Preview what would be installed
vx install node@20 --dry-run

# Preview sync changes
vx sync --dry-run

# Preview init configuration
vx init --dry-run
```

---

## 8. Environment Variable Authentication

Agents cannot perform browser-based OAuth flows. Use environment variables to
inject credentials:

```bash
# GitHub token (for version fetching and self-update)
export GITHUB_TOKEN=ghp_xxx

# Or use vx auth
vx auth login github --token ghp_xxx
```

---

## 9. AI Context (`vx ai context`)

Get a structured project context dump optimized for AI agents:

```bash
# Full context (includes env vars)
vx ai context --json

# Minimal context (saves tokens)
vx ai context --minimal --json
```

---

## 10. Cross-Language Global Install Contract

vx preserves ecosystem-native global install syntax while keeping installs inside
vx-managed isolation and shim workflows:

```bash
vx npm install -g @tencent-ai/codebuddy-code
vx pnpm add -g eslint
vx yarn global add typescript
vx pip install --user ruff
vx cargo install ripgrep
vx go install golang.org/x/tools/gopls@latest
vx gem install bundler
```

For agents, the contract is:
- Keep user-native syntax
- Register package in vx global package registry
- Generate/update shims in `~/.vx/shims`
- Ensure resulting executables are invokable consistently

---

## Quick Reference for Agents

```bash
# Discover what runtimes are available
vx schema --all | <filter>

# Check if a specific runtime is available  
vx schema node

# Install with dry-run first
vx install node@20 --dry-run && vx install node@20

# List installed tools in machine-readable format
vx list --json --fields name,installed,version

# Check if project tools are up to date
vx check --json

# Get project context
vx ai context --minimal --json
```

---

## Environment Variables Reference

| Variable | Values | Description |
|----------|--------|-------------|
| `VX_OUTPUT` | `json`, `text`, `toon`, `compact` | Override output format |
| `VX_FILTER_LEVEL` | `light`, `normal`, `aggressive` | Tune compact subprocess filtering |
| `VX_NO_AUTO_INSTALL` | `1`, `true`, `yes` | Disable auto-installation |
| `VX_OUTPUT_JSON` | `1` | Legacy: force JSON (prefer `VX_OUTPUT=json`) |
| `GITHUB_TOKEN` | `ghp_...` | GitHub API token for version fetching |
| `GH_TOKEN` | `ghp_...` | Alternative GitHub token (same effect) |

---

## References

- [You Need to Rewrite Your CLI for AI Agents](https://justin.poehnelt.com/posts/rewrite-your-cli-for-ai-agents/) — Justin Poehnelt, Google Cloud (March 2026)
- [RFC 0031: Unified Output Format](../rfcs/0031-unified-structured-output.md)
- [RFC 0035: AI Integration Optimization](../rfcs/0035-ai-integration-optimization.md)
