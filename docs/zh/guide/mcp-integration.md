# MCP 集成指南

vx **原生支持 MCP（Model Context Protocol）** — 只需将任意 MCP 服务器配置中的 `npx` 或 `uvx` 替换为 `vx`，即可消除运行时预装需求。

## 为什么用 vx 运行 MCP 服务器？

传统 MCP 服务器配置要求用户预先安装 Node.js、Python 或特定工具。使用 vx：

- **零配置**：用户无需安装任何运行时，vx 自动处理
- **版本锁定**：精确锁定工具版本，保证可重现性
- **跨平台一致**：同一份配置在 Windows、macOS 和 Linux 上均有效

## 基本替换模式

将 `npx` 或 `uvx` 替换为 `vx`：

```json
// 替换前（需要预先安装 Node.js）
{ "command": "npx", "args": ["-y", "@example/mcp-server@latest"] }

// 替换后（vx 自动安装 Node.js）
{ "command": "vx", "args": ["npx", "-y", "@example/mcp-server@latest"] }
```

| 原始命令 | vx 等效命令 |
|----------|------------|
| `"command": "npx"` | `"command": "vx", "args": ["npx", ...]` |
| `"command": "uvx"` | `"command": "vx", "args": ["uvx", ...]` |
| `"command": "node"` | `"command": "vx", "args": ["node", ...]` |
| `"command": "python"` | `"command": "vx", "args": ["python", ...]` |

## Claude Desktop

编辑配置文件：
- **macOS**：`~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**：`%APPDATA%\Claude\claude_desktop_config.json`

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

编辑 `~/.cursor/mcp.json` 或项目目录下的 `.cursor/mcp.json`：

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

编辑 `~/.codeium/windsurf/mcp_config.json`：

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

## Python 版 MCP 服务器

对于基于 Python 的 MCP 服务器，使用 `uvx`：

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

## 完整开发栈示例

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

## AI Agent 优化输出

当 AI Agent 调用 `vx list` 或 `vx analyze` 时，使用以下标志获得最优输出：

```bash
vx list --json           # JSON 格式，便于程序解析
vx list --format toon    # Token 优化输出（相比默认减少 40-60% Token）
vx analyze --json        # 以 JSON 格式输出项目分析
```

## 版本锁定

为保证 MCP 配置的可重现性，可以锁定工具版本：

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

或在项目 `vx.toml` 中锁定：

```toml
[tools]
node = "22"
uv = "0.5"
```

vx 会始终使用 `vx.toml` 中指定的版本。

## 常见问题排查

### MCP 服务器启动后找不到 vx

在 MCP 服务器的 `env` 中添加 vx 二进制目录：

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

### Windows 上 vx 未识别

确保 vx 在 PATH 中。先安装 vx：

```powershell
powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
```

验证安装：

```powershell
vx --version
```

### 首次运行延迟

MCP 服务器通过 vx 首次使用某个工具时，需要下载并安装运行时，后续运行几乎即时完成。可以预热常用工具：

```bash
vx install node@latest
vx install uv@latest
```
