# Headroom — AI 上下文优化

Headroom 是一个集成到 vx 中的 AI 上下文压缩工具。它通过压缩对话历史、工具输出和日志来优化 LLM 上下文窗口，可在保留语义的同时减少 40-60% 的 token 消耗。

## 快速开始

```bash
# 安装 headroom
vx ai headroom install

# 验证环境
vx ai headroom doctor

# 启动代理（后台运行）
vx ai headroom proxy start
```

## 安装

### 前置条件

- **Python 3.11+**（由 vx 通过 uv 自动管理）
- **uv**（latest，由 vx 自动管理）

### 安装 Headroom

```bash
vx ai headroom install
```

这会通过 `uv tool install` 安装 `headroom-ai[proxy]`，使用 Python-first 桥接。安装程序会：

1. 解析 Python 版本（默认：3.11）
2. 安装 headroom-ai（含 proxy extras）
3. 运行 `doctor` 检查验证环境
4. 安装 `mcpcall`（0.4.0+）用于 MCP 冒烟测试

安装指定版本：

```bash
vx ai headroom install --version 0.22.3 --python 3.12
```

强制重新安装：

```bash
vx ai headroom install --force
```

## 命令

### Doctor — 环境诊断

`doctor` 命令执行三层检查：

```bash
# 完整检查（代理 + MCP）
vx ai headroom doctor

# 快速检查（跳过代理启动和 MCP 探测）
vx ai headroom doctor --quick

# JSON 输出（机器可读）
vx ai headroom doctor --json
```

**检查层次：**

1. **环境** — headroom-ai 安装、uv 可用性、Python 版本
2. **代理** — 代理健康检查（端口 8787，`--quick` 时跳过）
3. **MCP** — MCP 服务器可用性（端口 8765，`--quick` 时跳过）

### Proxy — 生命周期管理

```bash
# 启动代理（默认后台运行）
vx ai headroom proxy start

# 前台运行并指定自定义端口
vx ai headroom proxy start --host 0.0.0.0 --port 8888 --foreground

# 查看代理状态
vx ai headroom proxy status
vx ai headroom proxy status --json

# 停止代理
vx ai headroom proxy stop
```

**选项：**

| 参数 | 默认值 | 描述 |
|------|--------|------|
| `--host` | `127.0.0.1` | 代理绑定地址 |
| `--port` | `8787` | 代理端口 |
| `--foreground` | `false` | 前台运行 |
| `--no-optimize` | `false` | 禁用优化透传 |
| `--log-file` | — | 日志文件路径 |

### MCP — 上下文服务器

以 stdio 模式运行 headroom MCP 服务器：

```bash
vx ai headroom mcp stdio
```

冒烟测试 MCP 工具：

```bash
# 测试所有工具
vx ai headroom mcp test

# 使用示例文件并输出 JSON
vx ai headroom mcp test --sample-file ./large-log.txt --json
```

测试验证三个 MCP 工具：
- `headroom_compress` — 压缩内容以节省上下文
- `headroom_retrieve` — 通过哈希检索原始内容
- `headroom_stats` — 显示压缩统计信息

### Setup — AI Agent 配置

为 AI Agent 生成 MCP 配置模板：

```bash
# 预览所有支持 Agent 的配置（dry-run）
vx ai headroom setup

# 为指定 Agent 应用配置
vx ai headroom setup --agent claude-code --agent cursor --apply

# 自定义端口
vx ai headroom setup --port 8787 --mcp-port 8765 --apply
```

**支持的 Agent：**

- Codebuddy
- Claude Code（claude-code）
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

## 使用场景

### AI Agent 上下文优化

配置你的 AI Agent 使用 headroom 作为 MCP 服务器，实现自动上下文压缩：

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

### CI 日志压缩

在 Agent 检查前压缩大型 CI 日志：

```bash
vx --compact gh run view <run-id> --log | head -2000 > build.log
vx ai headroom mcp test --sample-file build.log --json
```

### 省 Token 的开发流程

配合 vx compact 模式使用 headroom，实现最大 token 节省：

```bash
# 紧凑工具列表（节省 40-60%）
vx list --format toon

# 紧凑子进程过滤
vx --compact gh run view <run-id> --log

# 通过 MCP 进行 headroom 压缩
vx ai headroom mcp test --json
```

## vx.toml 配置

```toml
[tools]
uv = "latest"

[scripts]
# 安装和验证 headroom
headroom-setup = "vx ai headroom install"
headroom-doctor = "vx ai headroom doctor"
headroom-start = "vx ai headroom proxy start"
headroom-status = "vx ai headroom proxy status"
headroom-stop = "vx ai headroom proxy stop"
headroom-test = "vx ai headroom mcp test"
```

## 遥测

Headroom 遥测**默认关闭**。除非明确选择开启，否则不会收集任何数据。

验证遥测状态：

```bash
vx ai headroom doctor --json
```

## 故障排除

### 安装失败

```bash
# 检查 Python 可用性
vx uv tool install --from 'headroom-ai[proxy]==latest' headroom

# 尝试指定 Python 版本
vx ai headroom install --python 3.12 --force
```

### 代理无法启动

```bash
# 检查端口可用性
vx ai headroom proxy status

# 尝试不同端口
vx ai headroom proxy start --port 8888
```

### MCP 工具不可用

```bash
# 验证 headroom 安装
vx ai headroom doctor

# 重新运行 MCP 冒烟测试
vx ai headroom mcp test
```

## 成功标准（Phase 1）

- [x] 通过 `vx ai headroom install` 安装 headroom-ai
- [x] 通过 `vx ai headroom doctor` 进行环境诊断
- [x] 代理生命周期管理（启动/停止/状态）
- [x] stdio 模式的 MCP 服务器
- [x] MCP 冒烟测试（compress, retrieve, stats）
- [x] AI Agent 配置模板
- [x] 遥测默认关闭
