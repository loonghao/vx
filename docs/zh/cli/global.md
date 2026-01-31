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
vx install rust    # 用于 cargo 包
```

### 权限问题

```bash
# 检查目录权限
ls -la ~/.vx/packages/

# 使用正确的权限重新创建
chmod -R u+rwX ~/.vx/packages/
```

## 相关命令

- [install](./install) - 安装运行时版本
- [list](./list) - 列出可用的运行时
- [env](./env) - 管理环境
