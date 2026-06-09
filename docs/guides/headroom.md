# Headroom — AI Context Optimization

Headroom is an AI context compression tool integrated into vx. It optimizes LLM context windows by compressing conversation history, tool outputs, and logs — reducing token usage by 40-60% while preserving semantic meaning.

## Quick Start

```bash
# Install headroom
vx ai headroom install

# Verify the environment
vx ai headroom doctor

# Start the proxy (detached)
vx ai headroom proxy start
```

## Installation

### Prerequisites

- **Python 3.11+** (auto-managed by vx via uv)
- **uv** (latest, auto-managed by vx)

### Install Headroom

```bash
vx ai headroom install
```

This installs `headroom-ai[proxy]` via `uv tool install` with the Python-first bridge. The installer:

1. Resolves the Python version (default: 3.11)
2. Installs headroom-ai with proxy extras
3. Runs `doctor` checks to validate the environment
4. Installs `mcpcall` (0.4.0+) for MCP smoke tests

To install a specific version:

```bash
vx ai headroom install --version 0.22.3 --python 3.12
```

To force reinstall:

```bash
vx ai headroom install --force
```

## Commands

### Doctor — Environment Diagnostics

The `doctor` command performs three-layer checks:

```bash
# Full check (proxy + MCP)
vx ai headroom doctor

# Quick check (skip proxy startup and MCP probe)
vx ai headroom doctor --quick

# JSON output for machine parsing
vx ai headroom doctor --json
```

**Check layers:**

1. **Environment** — headroom-ai installation, uv availability, Python version
2. **Proxy** — proxy health on port 8787 (skipped with `--quick`)
3. **MCP** — MCP server availability on port 8765 (skipped with `--quick`)

### Proxy — Lifecycle Management

```bash
# Start proxy (detached by default)
vx ai headroom proxy start

# Start in foreground with custom port
vx ai headroom proxy start --host 0.0.0.0 --port 8888 --foreground

# Check proxy status
vx ai headroom proxy status
vx ai headroom proxy status --json

# Stop proxy
vx ai headroom proxy stop
```

**Options:**

| Flag | Default | Description |
|------|---------|-------------|
| `--host` | `127.0.0.1` | Proxy bind address |
| `--port` | `8787` | Proxy port |
| `--foreground` | `false` | Run in foreground |
| `--no-optimize` | `false` | Disable optimization passthrough |
| `--log-file` | — | Log file path |

### MCP — Context Server

Run the headroom MCP server in stdio mode:

```bash
vx ai headroom mcp stdio
```

Smoke-test MCP tools:

```bash
# Test all tools
vx ai headroom mcp test

# Test with sample file and JSON output
vx ai headroom mcp test --sample-file ./large-log.txt --json
```

The test validates three MCP tools:
- `headroom_compress` — Compress content to save context
- `headroom_retrieve` — Retrieve original content by hash
- `headroom_stats` — Show compression statistics

### Setup — AI Agent Configuration

Generate MCP configuration templates for AI agents:

```bash
# Preview configs for all supported agents (dry-run)
vx ai headroom setup

# Apply config for specific agents
vx ai headroom setup --agent claude-code --agent cursor --apply

# Custom ports
vx ai headroom setup --port 8787 --mcp-port 8765 --apply
```

**Supported agents:**

- Codebuddy
- Claude Code (claude-code)
- Cursor
- Codex
- Windsurf
- GitHub Copilot
- OpenCode
- Trae
- Gemini CLI
- AMP
- Roo
- Cline

## Use Cases

### AI Agent Context Optimization

Configure your AI agent to use headroom as an MCP server for automatic context compression:

```json
{
  "mcpServers": {
    "headroom": {
      "command": "vx",
      "args": ["ai", "headroom", "mcp", "stdio"]
    }
  }
}
```

### CI Log Compression

Compress large CI logs before agent inspection:

```bash
vx --compact gh run view <run-id> --log | head -2000 > build.log
vx ai headroom mcp test --sample-file build.log --json
```

### Token-Efficient Development

Use headroom with vx compact mode for maximum token savings:

```bash
# Compact tool list (saves 40-60%)
vx list --format toon

# Compact subprocess filtering
vx --compact gh run view <run-id> --log

# Headroom compression via MCP
vx ai headroom mcp test --json
```

## vx.toml Configuration

```toml
[tools]
uv = "latest"

[scripts]
# Install and verify headroom
headroom-setup = "vx ai headroom install"
headroom-doctor = "vx ai headroom doctor"
headroom-start = "vx ai headroom proxy start"
headroom-status = "vx ai headroom proxy status"
headroom-stop = "vx ai headroom proxy stop"
headroom-test = "vx ai headroom mcp test"
```

## Telemetry

Headroom telemetry is **disabled by default**. No data is collected unless explicitly opted in.

To verify telemetry status:

```bash
vx ai headroom doctor --json
```

## Troubleshooting

### Installation Fails

```bash
# Check Python availability
vx uv tool install --from 'headroom-ai[proxy]==latest' headroom

# Try with explicit Python version
vx ai headroom install --python 3.12 --force
```

### Proxy Won't Start

```bash
# Check port availability
vx ai headroom proxy status

# Try different port
vx ai headroom proxy start --port 8888
```

### MCP Tools Unavailable

```bash
# Verify headroom installation
vx ai headroom doctor

# Re-run MCP smoke test
vx ai headroom mcp test
```

## Success Criteria (Phase 1)

- [x] Install headroom-ai via `vx ai headroom install`
- [x] Environment diagnostics via `vx ai headroom doctor`
- [x] Proxy lifecycle management (start/stop/status)
- [x] MCP server in stdio mode
- [x] MCP smoke tests (compress, retrieve, stats)
- [x] AI agent configuration templates
- [x] Telemetry disabled by default
