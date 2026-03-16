# vx global - 全局包管理

跨不同生态系统管理全局安装的包，实现完全隔离。

## 概述

`vx global` 命令提供了一个统一的界面，用于安装、管理和使用来自多个生态系统（npm、pip、cargo、go、gem）的全局包，而不会污染你的运行时安装。

**核心特性：**
- 🔒 **完全隔离**：全局包永远不会污染运行时安装
- 🌍 **跨语言支持**：npm、pip、cargo、go 和 gem 统一体验
- 🔗 **基于 Shim 的访问**：自动创建 shim 实现无缝命令执行
- 📦 **版本共存**：同一个包的多个版本可以共存

## 语法

```bash
vx global <子命令> [选项]
```

## 子命令

| 子命令 | 别名 | 描述 |
|--------|------|------|
| `install` | - | 全局安装包（隔离） |
| `list` | `ls` | 列出全局安装的包 |
| `uninstall` | `rm` | 卸载全局包 |
| `info` | - | 显示全局包的信息 |
| `shim-update` | - | 手动更改后更新 shims |

---

## vx global install

以完全隔离的方式全局安装包。

### 语法

```bash
vx global install <包规格> [选项]
```

### 包规格格式

| 格式 | 描述 | 示例 |
|------|------|------|
| `package` | 自动检测生态系统，最新版本 | `typescript` |
| `package@version` | 自动检测生态系统，指定版本 | `typescript@5.3` |
| `ecosystem:package` | 显式生态系统，最新版本 | `npm:typescript` |
| `ecosystem:package@version` | 显式生态系统和版本 | `npm:typescript@5.3.3` |

### 支持的生态系统

| 生态系统 | 别名 | 包管理器 | 示例 |
|----------|------|----------|------|
| `npm` | `node` | npm, yarn, pnpm, bun | `npm:typescript@5.3` |
| `pip` | `python`, `pypi`, `uv` | pip, uv | `pip:black@24.1` |
| `cargo` | `rust`, `crates` | cargo | `cargo:ripgrep@14` |
| `go` | `golang` | go install | `go:golangci-lint@1.55` |
| `gem` | `ruby`, `rubygems` | gem | `gem:bundler@2.5` |

### 首选安装器

vx 自动为每个生态系统选择最佳安装器：

| 生态系统 | 首选 | 备用 | 说明 |
|----------|------|------|------|
| **Python** | `uv` | `pip` | uv 速度显著更快 |
| **Node.js** | `npm` | - | 使用显式 `yarn:`、`pnpm:` 或 `bun:` 选择替代方案 |

要使用特定安装器，请显式指定：

```bash
# 使用 uv（更快）安装 Python 包
vx global install uv:black@24.1
vx global install uv:ruff

# 使用 pip（标准）安装 Python 包
vx global install pip:black@24.1

# 使用 yarn 替代 npm
vx global install yarn:typescript

# 使用 pnpm
vx global install pnpm:eslint
```

### 选项

| 选项 | 简写 | 描述 |
|------|------|------|
| `--force` | `-f` | 即使已安装也强制重新安装 |
| `--verbose` | `-v` | 显示详细的安装进度 |
| `--` | - | 传递额外参数给包管理器 |

### 示例

```bash
# 安装 npm 包
vx global install typescript@5.3
vx global install npm:eslint
vx global install npm:@biomejs/biome@1.5

# 安装 Python 工具
vx global install pip:black@24.1
vx global install pip:ruff
vx global install uv:pytest  # 使用 uv 作为安装器

# 安装 Rust CLI 工具
vx global install cargo:ripgrep@14
vx global install cargo:fd-find
vx global install cargo:bat

# 安装 Go 工具
vx global install go:golangci-lint@1.55
vx global install go:gopls

# 安装 Ruby gems
vx global install gem:bundler@2.5
vx global install gem:rubocop

# 强制重新安装
vx global install typescript@5.3 --force

# 详细输出
vx global install pip:black -v

# 传递额外参数给包管理器
vx global install npm:some-package -- --legacy-peer-deps
```

### 自动检测

当未指定生态系统时，vx 会根据常见的包名自动检测：

```bash
# 这两个是等价的：
vx global install typescript@5.3
vx global install npm:typescript@5.3

# 这两个是等价的：
vx global install black@24.1
vx global install pip:black@24.1

# 对于未知的包，请显式指定：
vx global install npm:my-custom-package
```

---

## vx global list

列出所有全局安装的包。

### 语法

```bash
vx global list [选项]
```

### 选项

| 选项 | 简写 | 描述 |
|------|------|------|
| `--ecosystem <name>` | - | 按生态系统筛选 (npm, pip, cargo, go, gem) |
| `--format <format>` | - | 输出格式：`table`（默认）、`json`、`plain` |
| `--verbose` | `-v` | 显示详细信息包括路径 |

### 示例

```bash
# 列出所有包
vx global list
vx global ls

# 按生态系统筛选
vx global list --ecosystem npm
vx global list --ecosystem pip

# 不同输出格式
vx global list --format json
vx global list --format plain

# 详细输出
vx global list -v
```

### 输出示例

```
ECOSYSTEM    PACKAGE                  VERSION      EXECUTABLES
----------------------------------------------------------------------
npm          typescript               5.3.3        tsc, tsserver
npm          eslint                   8.56.0       eslint
pip          black                    24.1.0       black
pip          ruff                     0.3.0        ruff
cargo        ripgrep                  14.0.0       rg
cargo        fd-find                  9.0.0        fd
go           golangci-lint            1.55.0       golangci-lint

Total: 7 package(s)
```

---

## vx global uninstall

删除全局安装的包。

### 语法

```bash
vx global uninstall <包规格> [选项]
```

### 选项

| 选项 | 简写 | 描述 |
|------|------|------|
| `--force` | `-f` | 跳过确认提示 |
| `--verbose` | `-v` | 显示详细的删除进度 |

### 示例

```bash
# 按名称卸载（从注册表自动检测生态系统）
vx global uninstall typescript
vx global rm eslint

# 显式生态系统
vx global uninstall npm:typescript
vx global uninstall pip:black

# 强制删除，不需确认
vx global uninstall typescript --force
```

---

## vx global info

显示已安装包的详细信息。

### 语法

```bash
vx global info <包名或可执行文件名> [选项]
```

### 选项

| 选项 | 描述 |
|------|------|
| `--json` | 以 JSON 格式输出 |

### 示例

```bash
# 按包名查询
vx global info typescript
vx global info npm:typescript

# 按可执行文件名查询
vx global info tsc
vx global info rg

# JSON 输出
vx global info typescript --json
```

### 输出示例

```
Package: typescript
Version: 5.3.3
Ecosystem: npm
Installed at: 2024-01-15T10:30:00Z
Location: ~/.vx/packages/npm/typescript/5.3.3
Executables: tsc, tsserver
```

---

## vx global shim-update

手动同步 shims 与包注册表。通常不需要使用，因为在安装/卸载过程中会自动创建/删除 shims。

### 语法

```bash
vx global shim-update
```

### 使用场景

- 手动修改包目录后
- 如果 shims 不同步
- 系统恢复或还原后

---

## 安装目录结构

包被安装在隔离的目录中：

```
~/.vx/
├── packages/                    # 全局包
│   ├── npm/
│   │   └── typescript/
│   │       └── 5.3.3/
│   │           ├── node_modules/
│   │           └── bin/
│   │               ├── tsc
│   │               └── tsserver
│   ├── pip/
│   │   └── black/
│   │       └── 24.1.0/
│   │           ├── venv/
│   │           └── bin/
│   │               └── black
│   └── cargo/
│       └── ripgrep/
│           └── 14.0.0/
│               └── bin/
│                   └── rg
│
└── shims/                       # 全局 shims
    ├── tsc -> ../packages/npm/typescript/5.3.3/bin/tsc
    ├── black -> ../packages/pip/black/24.1.0/bin/black
    └── rg -> ../packages/cargo/ripgrep/14.0.0/bin/rg
```

## 使用已安装的工具

安装后，工具可通过 shims 使用：

```bash
# 将 shims 目录添加到 PATH（建议在 shell 配置中设置）
export PATH="$HOME/.vx/shims:$PATH"

# 现在可以直接使用工具
tsc --version
black --check .
rg "pattern" ./src
```

或者通过 vx 运行：

```bash
vx tsc --version
vx black --check .
```

## 自动安装行为

当你通过 vx 运行一个尚未安装的工具时，vx 可以自动为你安装（类似于 `npx` 或 `uvx`）。

### 显式包执行（RFC 0027）

使用 `ecosystem:package` 语法运行任何包，无需预先安装：

```bash
# 自动安装并运行（如果尚未安装）
vx npm:typescript::tsc --version
vx pip:ruff check .
vx cargo:ripgrep::rg "pattern" ./src

# 指定特定版本
vx npm:typescript@5.3::tsc --version
vx pip@3.11:black .

# 完整语法，包含运行时版本
vx npm@20:typescript@5.3::tsc --version
```

**工作原理：**
1. 检查包是否已安装
2. 如果未安装，自动安装（相当于 `vx global install`）
3. 使用正确的环境执行工具

### Shim 执行

对于已安装的包，直接使用可执行文件名：

```bash
# 安装后，以下命令等价
vx tsc --version          # 通过 vx shim
vx npm:typescript::tsc    # 通过 RFC 0027 语法
tsc --version             # 直接 shim（如果 PATH 已配置）
```

**参见 [隐式包执行](./implicit-package-execution.md) 获取完整文档。**

## 最佳实践

### 1. 为未知包指定生态系统

```bash
# 好：显式生态系统
vx global install npm:my-internal-package

# 可能失败：未知包
vx global install my-internal-package
```

### 2. 固定版本以确保可重现性

```bash
# 好：指定版本
vx global install typescript@5.3.3

# 不太可预测：最新版本
vx global install typescript
```

### 3. 使用首选包管理器

```bash
# Python: uv 比 pip 更快
vx global install uv:black@24.1

# Node.js: npm 是默认的，但你可以指定
vx global install npm:typescript
```

### 4. 保持 PATH 更新

添加到你的 shell 配置（`~/.bashrc`、`~/.zshrc` 等）：

```bash
# 将 vx shims 添加到 PATH
export PATH="$HOME/.vx/shims:$PATH"
```

## 与原生包管理器的对比

| 特性 | vx global | npm -g | pip | cargo install |
|------|-----------|--------|-----|---------------|
| 隔离性 | ✅ 完全隔离 | ❌ 污染 node | ❌ 污染 Python | ❌ 污染 ~/.cargo |
| 跨语言 | ✅ 统一 | ❌ 仅 npm | ❌ 仅 pip | ❌ 仅 cargo |
| 版本共存 | ✅ 多版本 | ❌ 单版本 | ❌ 单版本 | ❌ 单版本 |
| Shim 管理 | ✅ 自动 | ❌ 手动 | ❌ 手动 | ❌ 手动 |
| 清理 | ✅ 干净卸载 | ⚠️ 可能残留 | ⚠️ 可能残留 | ⚠️ 可能残留 |

## 故障排除

### Shims 不工作

```bash
# 检查 shims 目录是否在 PATH 中
echo $PATH | grep -q ".vx/shims" && echo "OK" || echo "缺失"

# 重建 shims
vx global shim-update
```

### 找不到包管理器

```bash
# 确保运行时已安装
vx install node    # 用于 npm 包
vx install python  # 用于 pip 包
vx install rustup  # 用于 cargo 包（由 rustup 管理）
```

### 权限问题

```bash
# 检查目录权限
ls -la ~/.vx/packages/

# 使用正确的权限重新创建
chmod -R u+rwX ~/.vx/packages/
```

## 架构

### vx-ecosystem-pm

`vx-ecosystem-pm` crate 为多个生态系统提供隔离的包安装：

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        vx-ecosystem-pm 架构                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  EcosystemInstaller Trait（生态系统安装器特质）                  │   │
│  │  ├── install(dir, package, version, options) -> Result          │   │
│  │  ├── is_available() -> bool                                     │   │
│  │  └── ecosystem() -> String                                      │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  安装器（按生态系统）                                            │   │
│  │  ├── npm.rs    - npm、yarn、pnpm、bun 支持                      │   │
│  │  ├── pip.rs    - 标准 pip 安装器                                │   │
│  │  ├── uv.rs     - 基于 uv 的快速 Python 安装器                   │   │
│  │  ├── cargo.rs  - Rust cargo 安装器                              │   │
│  │  ├── go.rs     - Go 安装器                                      │   │
│  │  └── gem.rs    - Ruby gem 安装器                                │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  隔离策略                                                        │   │
│  │  ├── npm:  NPM_CONFIG_PREFIX 重定向                             │   │
│  │  ├── pip:  隔离的虚拟环境                                        │   │
│  │  ├── uv:   UV_INSTALL_DIR 重定向                                │   │
│  │  ├── cargo: CARGO_INSTALL_ROOT 重定向                           │   │
│  │  ├── go:   GOBIN 重定向                                         │   │
│  │  └── gem:  GEM_HOME/GEM_PATH 重定向                             │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 目录结构

包安装在隔离目录中，使用环境变量重定向：

```
~/.vx/
├── packages/                    # 隔离的包安装
│   ├── npm/
│   │   └── typescript/
│   │       └── 5.3.3/          # NPM_CONFIG_PREFIX 设置为此目录
│   │           ├── lib/
│   │           │   └── node_modules/
│   │           │       └── typescript/
│   │           └── bin/
│   │               └── tsc -> ../lib/node_modules/typescript/bin/tsc
│   │
│   ├── pip/
│   │   └── black/
│   │       └── 24.1.0/         # VIRTUAL_ENV 设置为此目录
│   │           ├── venv/       # 隔离的 Python 虚拟环境
│   │           │   ├── bin/
│   │           │   │   ├── python -> ~/.vx/store/python/3.11.x/bin/python
│   │           │   │   └── black
│   │           │   └── lib/python3.11/site-packages/
│   │           │       └── black/
│   │           └── bin/
│   │               └── black -> ../venv/bin/black
│   │
│   ├── cargo/
│   │   └── ripgrep/
│   │       └── 14.0.0/         # CARGO_INSTALL_ROOT 设置为此目录
│   │           └── bin/
│   │               └── rg
│   │
│   └── go/
│       └── golangci-lint/
│           └── 1.55.0/         # GOBIN 设置为此目录
│               └── bin/
│                   └── golangci-lint
│
└── shims/                       # 全局可执行文件 shims
    ├── tsc -> ../packages/npm/typescript/5.3.3/bin/tsc
    ├── black -> ../packages/pip/black/24.1.0/bin/black
    └── rg -> ../packages/cargo/ripgrep/14.0.0/bin/rg
```

## 相关命令

- [install](./install) - 安装运行时版本
- [list](./list) - 列出可用的运行时
- [env](./env) - 管理环境
- [隐式包执行](./implicit-package-execution.md) - 无需安装即可运行包
