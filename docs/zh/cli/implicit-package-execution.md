# 隐式包执行

执行全局安装包或按需运行包，无需显式安装，类似于 `npx` 和 `uvx`，但支持跨语言。

## 概述

隐式包执行功能允许你使用统一语法直接运行包。与 `npx`（仅 Node.js）或 `uvx`（仅 Python）不同，vx 支持多个生态系统，提供一致的界面。

**主要优势：**
- 🚀 **一键执行**：无需预先安装即可运行包
- 🌍 **跨语言**：适用于 npm、pip、cargo、go 等
- 📦 **自动安装**：首次运行时自动安装包
- 🔒 **隔离性**：每个包都安装在自己的隔离环境中
- 🎯 **版本控制**：指定确切版本以确保可重现性

## 语法

```
vx <生态系统>[@运行时版本]:<包名>[@版本][::可执行文件] [参数...]
```

### 语法组件

| 组件 | 描述 | 示例 |
|------|------|------|
| `生态系统` | 包生态系统（npm、pip、cargo、go 等） | `npm`、`pip` |
| `@运行时版本` | （可选）要使用的运行时版本 | `@20`、`@3.11` |
| `包名` | 包名称 | `typescript`、`ruff` |
| `@版本` | （可选）包版本 | `@5.3`、`@0.3` |
| `::可执行文件` | （可选）可执行文件名（如果与包名不同） | `::tsc`、`::rg` |

## 基本用法

### 运行已安装的工具

通过 `vx global install` 安装包后，可以直接运行：

```bash
# 通过可执行文件名运行已安装工具
vx tsc --version
vx black --check .
vx rg "pattern" ./src
```

### 显式包语法

当包名与可执行文件名不同时，使用完整语法：

```bash
# 包名 ≠ 可执行文件名
vx npm:typescript::tsc --version      # typescript 包，tsc 可执行文件
vx pip:httpie::http GET example.com   # httpie 包，http 命令
vx cargo:ripgrep::rg "pattern"        # ripgrep 包，rg 可执行文件
```

### 自动检测与安装

如果包尚未安装，vx 会自动下载并安装：

```bash
# 首次运行 - 自动安装 typescript
vx npm:typescript --version

# 首次运行 - 自动安装 ruff
vx pip:ruff check .

# 包会被缓存以供后续使用
```

## 支持的生态系统

| 生态系统 | 别名 | 运行时 | 示例包 |
|----------|------|--------|--------|
| `npm` | `node` | Node.js | `npm:typescript` |
| `pip` | `python`、`pypi` | Python | `pip:black` |
| `uv` | - | Python（通过 uv） | `uv:ruff` |
| `cargo` | `rust`、`crates` | Rust | `cargo:ripgrep` |
| `go` | `golang` | Go | `go:golangci-lint` |
| `bun` | - | Bun | `bun:typescript` |
| `yarn` | - | Node.js | `yarn:typescript` |
| `pnpm` | - | Node.js | `pnpm:typescript` |

## 常见用例

### TypeScript/Node.js

```bash
# 编译 TypeScript（如需要自动安装）
vx npm:typescript::tsc --version

# 运行 ESLint
vx npm:eslint .

# 使用指定 Node 版本创建 React 应用
vx npm@18:create-react-app my-app

# 运行作用域包（@biomejs/biome）
vx npm:@biomejs/biome::biome check .

# 运行指定版本的 TypeScript
vx npm:typescript@5.3::tsc --version
```

### Python

```bash
# 使用 black 格式化代码
vx pip:black .

# 使用 ruff 检查（指定版本）
vx pip:ruff@0.3 check .

# 运行 pytest
vx pip:pytest -v

# 使用指定 Python 版本
vx pip@3.11:black .

# 使用 uv（更快）
vx uv:ruff check .

# HTTP 客户端
vx pip:httpie::http GET example.com
```

### Rust

```bash
# 使用 ripgrep 搜索
vx cargo:ripgrep::rg "pattern" ./src

# 使用 fd 查找文件
vx cargo:fd-find::fd ".rs$" .

# 使用 bat 语法高亮
vx cargo:bat::bat src/main.rs
```

### Go

```bash
# 运行 linter
vx go:golangci-lint run

# 运行语言服务器
vx go:gopls version
```

## `::` 分隔符说明

许多包提供的可执行文件名与包名不同。`::` 分隔符让你可以指定确切的可执行文件：

| 包名 | 可执行文件 | 完整命令 | 简写（如已安装） |
|------|------------|----------|------------------|
| `typescript` | `tsc` | `vx npm:typescript::tsc` | `vx tsc` |
| `typescript` | `tsserver` | `vx npm:typescript::tsserver` | `vx tsserver` |
| `httpie` | `http` | `vx pip:httpie::http` | `vx http` |
| `ripgrep` | `rg` | `vx cargo:ripgrep::rg` | `vx rg` |
| `fd-find` | `fd` | `vx cargo:fd-find::fd` | `vx fd` |
| `bat` | `bat` | `vx cargo:bat::bat` | `vx bat` |

### 何时使用 `::`

**使用 `::` 的情况：**
- 包名与可执行文件名不同（如 `typescript` → `tsc`）
- 包有多个可执行文件（如 `typescript` 有 `tsc` 和 `tsserver`）
- 你想明确指定运行哪个可执行文件

**省略 `::` 的情况：**
- 包名等于可执行文件名（如 `eslint`、`ruff`）
- 安装后通过简写运行

## 版本规范

### 包版本

```bash
# 最新版本（默认）
vx npm:typescript --version

# 指定版本
vx npm:typescript@5.3 --version

# 版本范围
vx npm:typescript@^5.0 --version
```

### 运行时版本

```bash
# 使用指定 Node.js 版本
vx npm@20:typescript::tsc --version
vx npm@18:eslint .

# 使用指定 Python 版本
vx pip@3.11:black .
vx pip@3.12:ruff check .

# 使用最新运行时（默认）
vx npm:typescript --version
```

### 组合规范

```bash
# 完整规范：生态系统@运行时:包名@版本::可执行文件
vx npm@20:typescript@5.3::tsc --version
# │    │  │          │   │  │
# │    │  │          │   │  └── 可执行文件
# │    │  │          │   └───── 包版本
# │    │  │          └───────── 包名
# │    │  └──────────────────── 运行时版本
# │    └─────────────────────── 运行时
# └──────────────────────────── 生态系统
```

## 作用域 npm 包

对于带作用域的 npm 包（@组织/包）：

```bash
# Biome（JavaScript 工具链）
vx npm:@biomejs/biome::biome check .

# OpenAI Codex
vx npm:@openai/codex::codex

# TypeScript Go 实现
vx npm:@aspect-build/tsgo::tsgo check .
```

## 与现有工具对比

### vx vs npx

| 场景 | npx | vx |
|------|-----|-----|
| 基本执行 | `npx eslint` | `vx npm:eslint` 或 `vx eslint`（已安装） |
| 不同可执行文件 | `npx -p typescript tsc` | `vx npm:typescript::tsc` |
| 指定版本 | `npx eslint@8` | `vx npm:eslint@8` |
| 运行时版本 | ❌ 不支持 | `vx npm@20:eslint` |
| 其他生态系统 | ❌ 不支持 | ✅ pip、cargo、go 等 |

### vx vs uvx

| 场景 | uvx | vx |
|------|-----|-----|
| 基本执行 | `uvx ruff` | `vx pip:ruff` 或 `vx ruff`（已安装） |
| 不同可执行文件 | `uvx --from httpie http` | `vx pip:httpie::http` |
| 指定版本 | `uvx ruff@0.3` | `vx pip:ruff@0.3` |
| 运行时版本 | `uvx --python 3.11 ruff` | `vx pip@3.11:ruff` |
| 其他生态系统 | ❌ 不支持 | ✅ npm、cargo、go 等 |

## 项目级配置

对于项目，可以在 `vx.toml` 中声明常用包：

```toml
[tools.global]
typescript = "5.3"
eslint = "8"
black = "24.1"
ruff = "0.3"
```

然后直接使用：

```bash
vx sync    # 安装所有声明的全局工具

vx tsc --version    # 使用项目的 typescript 版本
vx eslint .
vx black .
```

## 环境变量

| 变量 | 描述 |
|------|------|
| `VX_AUTO_INSTALL` | 启用/禁用自动安装（默认：`true`） |
| `VX_GLOBAL_CACHE` | 覆盖全局包缓存目录 |

## 故障排除

### "找不到包"

```bash
# 确保使用正确的生态系统
vx npm:my-package      # 用于 npm 包
vx pip:my-package      # 用于 Python 包

# 检查包是否存在
vx global list
```

### "运行时未安装"

```bash
# 首先安装所需的运行时
vx install node        # 用于 npm 包
vx install python      # 用于 pip 包
vx install rust        # 用于 cargo 包
```

### 命令冲突

如果命令与运行时名称冲突：

```bash
# 使用显式语法
vx npm:node             # 运行 'node' 包，而非 node 运行时

# 或使用全局命令
vx global install npm:node
vx node                 # 现在运行的是该包
```

## 最佳实践

### 1. 固定版本以确保可重现性

```bash
# 好：指定版本
vx npm:typescript@5.3.3 --version

# 不太可预测：最新版本
vx npm:typescript --version
```

### 2. 在脚本中使用显式语法

```bash
# 在 CI/CD 或共享脚本中，保持明确
vx npm:typescript@5.3::tsc --project tsconfig.json
```

### 3. 对于频繁使用的工具，优先使用 `vx global install`

```bash
# 一次安装，多次使用
vx global install npm:typescript@5.3

# 然后使用简写
vx tsc --version
```

### 4. 使用 `vx dev` 进行项目隔离

```bash
# 进入项目环境
vx dev

# 所有工具都可用，无需前缀
tsc --version
black .
ruff check .
```

## 相关命令

- [`vx global`](./global) - 管理全局包
- [`vx install`](./install) - 安装运行时版本
- [RFC 0027: 隐式包执行](../rfcs/0027-implicit-package-execution.md)
- [RFC 0025: 跨语言包隔离](../rfcs/0025-cross-language-package-isolation.md)
