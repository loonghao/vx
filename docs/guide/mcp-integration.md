# MCP Integration Guide

vx is **MCP-ready** — replace `npx` or `uvx` with `vx` in any MCP server configuration to eliminate runtime pre-installation requirements.

## Why vx for MCP?

Traditional MCP server configs require users to pre-install Node.js, Python, or specific tools. With vx:

- **Zero setup**: Users don't need to install any runtime — vx handles it automatically
- **Version pinning**: Lock to exact tool versions for reproducibility
- **Cross-platform**: Same config works on Windows, macOS, and Linux

## Basic Pattern

Replace `npx` or `uvx` with `vx`:

```json
// Before (requires Node.js pre-installed)
{ "command": "npx", "args": ["-y", "@example/mcp-server@latest"] }

// After (vx auto-installs Node.js if needed)
{ "command": "vx", "args": ["npx", "-y", "@example/mcp-server@latest"] }
```

| Original | vx-powered |
|----------|-----------|
| `"command": "npx"` | `"command": "vx", "args": ["npx", ...]` |
| `"command": "uvx"` | `"command": "vx", "args": ["uvx", ...]` |
| `"command": "node"` | `"command": "vx", "args": ["node", ...]` |
| `"command": "python"` | `"command": "vx", "args": ["python", ...]` |

## Claude Desktop

Edit `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows):

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "vx",
      "args": ["npx", "-y", "@modelcontextprotocol/server-filesystem", "/path/to/allowed"]
    },
    "github": {
      "command": "vx",
      "args": ["npx", "-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "your-token"
      }
    },
    "puppeteer": {
      "command": "vx",
      "args": ["npx", "-y", "@modelcontextprotocol/server-puppeteer"]
    },
    "browsermcp": {
      "command": "vx",
      "args": ["npx", "-y", "@browsermcp/mcp@latest"]
    },
    "context7": {
      "command": "vx",
      "args": ["npx", "-y", "@upstash/context7-mcp@latest"]
    }
  }
}
```

## Cursor

Edit `~/.cursor/mcp.json` or `.cursor/mcp.json` in your project:

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "vx",
      "args": ["npx", "-y", "@modelcontextprotocol/server-filesystem", "."]
    },
    "sequential-thinking": {
      "command": "vx",
      "args": ["npx", "-y", "@modelcontextprotocol/server-sequential-thinking"]
    }
  }
}
```

## Windsurf

Edit `~/.codeium/windsurf/mcp_config.json`:

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "vx",
      "args": ["npx", "-y", "@modelcontextprotocol/server-filesystem", "/path/to/workspace"]
    }
  }
}
```

## Python-based MCP Servers

For Python-based MCP servers, use `uvx`:

```json
{
  "mcpServers": {
    "mcp-server-git": {
      "command": "vx",
      "args": ["uvx", "mcp-server-git", "--repository", "/path/to/repo"]
    },
    "mcp-qdrant": {
      "command": "vx",
      "args": ["uvx", "mcp-server-qdrant"],
      "env": {
        "QDRANT_URL": "http://localhost:6333"
      }
    }
  }
}
```

## Real-World Examples

### Full Development Stack

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "vx",
      "args": ["npx", "-y", "@modelcontextprotocol/server-filesystem", "~", "/workspace"]
    },
    "github": {
      "command": "vx",
      "args": ["npx", "-y", "@modelcontextprotocol/server-github"],
      "env": { "GITHUB_PERSONAL_ACCESS_TOKEN": "${GITHUB_TOKEN}" }
    },
    "git": {
      "command": "vx",
      "args": ["uvx", "mcp-server-git", "--repository", "."]
    },
    "memory": {
      "command": "vx",
      "args": ["npx", "-y", "@modelcontextprotocol/server-memory"]
    },
    "brave-search": {
      "command": "vx",
      "args": ["npx", "-y", "@modelcontextprotocol/server-brave-search"],
      "env": { "BRAVE_API_KEY": "${BRAVE_API_KEY}" }
    }
  }
}
```

### AI-native Output (Token-optimized)

When AI agents call `vx list` or `vx analyze`, use these flags for optimal output:

```bash
vx list --json           # JSON format for programmatic parsing
vx list --format toon    # Token-optimized (saves 40-60% tokens vs default)
vx analyze --json        # Project analysis as JSON
```

## Troubleshooting

**vx not found in PATH after MCP server starts**

Add the vx binary directory to the MCP server environment:

```json
{
  "mcpServers": {
    "example": {
      "command": "vx",
      "args": ["npx", "-y", "@example/mcp-server"],
      "env": {
        "PATH": "/usr/local/bin:/usr/bin:/bin:${PATH}"
      }
    }
  }
}
```

**Windows: vx not recognized**

Ensure vx is in PATH. Install it first:

```powershell
powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
```

Then verify:
```powershell
vx --version
```

**First-run latency**

The first time an MCP server uses a tool via vx, it downloads and installs the runtime. Subsequent runs are instant. To pre-warm:

```bash
vx install node@latest
vx install uv@latest
```

## Version Pinning for Stability

Pin tool versions for reproducible MCP configs:

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "vx",
      "args": ["node@22", "path/to/server.js"]
    }
  }
}
```

Or via `vx.toml` in your project:

```toml
[tools]
node = "22"
uv = "0.5"
```

Then vx will always use the pinned version from `vx.toml`.
