# list 命令

列出可用工具和已安装版本。

## 语法

```bash
vx list [tool] [options]
```

## 参数

| 参数 | 描述 |
|------|------|
| `tool` | 可选，特定工具名称 |

## 选项

| 选项 | 描述 |
|------|------|
| `--status` | 显示安装状态和版本详情 |
| `--all, -a` | 显示所有工具，包括当前平台不支持的 |
| `--installed` | 仅显示已安装的工具 |
| `--available` | 仅显示可用但未安装的工具 |

## 示例

```bash
# 列出当前平台支持的工具
vx list

# 列出所有工具（包括不支持的）
vx list --all
vx list -a

# 列出已安装的工具
vx list --installed

# 列出特定工具的可用版本
vx list node --status

# 显示安装状态摘要
vx list --status
```

## 输出示例

### 默认输出

```
📦 Available Tools (windows-x64):
  ✅ node - JavaScript runtime built on Chrome's V8 engine
  ❌ go - Go programming language
  ✅ uv - Fast Python package installer

   2 tools hidden (not supported on windows-x64). Use --all to show all.
```

### 使用 --all 显示所有工具

```
📦 Available Tools (showing all, including 2 unsupported):
  ✅ node - JavaScript runtime built on Chrome's V8 engine
  ❌ go - Go programming language
  ⚠️  choco - Chocolatey package manager (not supported on linux-x64)
```

### 状态输出

```
📦 Available Tools (windows-x64):
  ✅ node - JavaScript runtime built on Chrome's V8 engine
     Versions: 18.17.0, 20.10.0
  ❌ go - Go programming language

📊 Summary: 1/10 tools installed
   2 tools hidden (not supported on windows-x64). Use --all to show all.
```

## 状态图标

| 图标 | 含义 |
|------|------|
| ✅ | 已安装 |
| ❌ | 未安装（支持当前平台） |
| ⚠️ | 当前平台不支持（仅 --all 模式） |

## 支持的工具

### Node.js 生态

- **node** - Node.js 运行时
- **npm** - Node.js 包管理器
- **npx** - Node.js 包运行器
- **pnpm** - 快速包管理器
- **yarn** - Yarn 包管理器
- **bun** - Bun JavaScript 运行时

### Python 生态

- **python** - Python 解释器
- **uv** - 快速 Python 包管理器
- **uvx** - UV 工具运行器
- **pip** - Python 包安装器

### Go 生态

- **go** - Go 编程语言

### Rust 生态

- **cargo** - Rust 包管理器
- **rustc** - Rust 编译器

### Windows 专属

- **choco** - Chocolatey 包管理器（仅 Windows）
- **rcedit** - Windows 资源编辑器（仅 Windows）

## 参见

- [install](./install) - 安装工具
- [search](./overview) - 搜索工具
