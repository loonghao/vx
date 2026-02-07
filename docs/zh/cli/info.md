# info

显示系统信息、能力概览和构建诊断。

## 用法

```bash
vx info [选项]
```

## 描述

`vx info` 命令显示关于你的 vx 安装的全面信息，包括：

- **vx 版本**和平台详情
- **托管运行时**，按生态系统（Node.js、Python、Go、Rust 等）分组，显示安装状态
- **系统工具**，显示系统中发现的可用工具
- **功能开关**（自动安装、Shell 模式、扩展等）

此命令在调试问题、分享环境信息到 Bug 报告、以及 AI 工具需要程序化发现可用能力时特别有用。

## 选项

| 选项 | 描述 |
|------|------|
| `--json` | 以 JSON 格式输出（推荐 AI 和脚本使用） |
| `--warnings` | 显示构建警告和诊断信息 |

## 示例

### 基本用法

```bash
# 以可读格式显示系统信息
vx info
```

输出示例：

```
ℹ vx 0.7.0 capabilities

Platform: windows (x86_64)

ℹ Managed Runtimes:
  NodeJs:
    ✅ node (v22.0.0) - Node.js JavaScript runtime
    ❌ bun - Fast JavaScript runtime and package manager
  Python:
    ✅ uv (0.5.14) - Fast Python package manager
  Go:
    ✅ go (1.22.0) - Go programming language
  ...

ℹ System Tools (available):
    git [vcs] @ C:\Program Files\Git\cmd\git.exe
    cmake [build] @ C:\Program Files\CMake\bin\cmake.exe
    ...

ℹ Features:
    auto_install: true
    shell_mode: true
    project_config: true
    extensions: true
    virtual_environments: true
```

### JSON 输出（适用于 AI 和脚本）

```bash
vx info --json
```

返回包含所有能力的结构化 JSON：

```json
{
  "version": "0.7.0",
  "platform": { "os": "windows", "arch": "x86_64" },
  "runtimes": {
    "node": {
      "name": "node",
      "description": "Node.js JavaScript runtime",
      "version": "22.0.0",
      "installed": true,
      "ecosystem": "NodeJs",
      "commands": ["node", "nodejs"]
    }
  },
  "system_tools": {
    "available": [...],
    "unavailable": [...]
  },
  "features": {
    "auto_install": true,
    "shell_mode": true,
    "project_config": true,
    "extensions": true,
    "virtual_environments": true
  }
}
```

### 显示构建诊断信息

```bash
vx info --warnings
```

显示 Provider 注册表初始化过程中发生的错误或警告。这在诊断自定义 Provider 或 manifest 文件的问题时非常有用。

一切正常时的输出：

```
✓ No build warnings or errors.
```

存在问题时的输出：

```
✗ Build Errors (1):
  • missing factory for provider 'my-custom-provider'

⚠ Build Warnings (2):
  • provider 'legacy-tool' has deprecated configuration format
  • runtime 'old-tool' uses unsupported platform constraint syntax

Summary: 3 total diagnostic(s). Use --debug for verbose output.
```

## 高级用法

### 通过管道配合其他工具使用

```bash
# 获取已安装的运行时列表
vx info --json | jq '.runtimes | to_entries[] | select(.value.installed) | .key'

# 检查特定运行时是否可用
vx info --json | jq '.runtimes.node.installed'

# 获取平台信息
vx info --json | jq '.platform'
```

### 在 CI/CD 中使用

```yaml
# GitHub Actions 示例
- name: 检查 vx 环境
  run: vx info --json > vx-env.json

- name: 验证工具已安装
  run: |
    vx info --warnings
    vx info --json | jq -e '.runtimes.node.installed'
```

### 调试 Provider 问题

当自定义 Provider 加载失败或行为异常时：

```bash
# 检查构建错误
vx info --warnings

# 启用调试日志查看更多详情
VX_LOG=debug vx info --warnings
```

## 相关命令

- [`vx list`](./list.md) — 列出可用和已安装的工具
- [`vx config show`](./config.md) — 显示当前配置
