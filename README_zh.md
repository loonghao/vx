# 🚀 vx - 通用开发工具管理器

<div align="center">

**一个命令统治所有工具 — 零设置，零学习成本**

*为 AI 原生时代而生：Unix 哲学与可脚本化的完美结合*

[English](README.md) | [📖 文档](https://docs.rs/vx) | [🚀 快速开始](#-快速开始) | [💡 示例](#-实际示例)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.93+-blue.svg)](https://www.rust-lang.org)
[![Test](https://github.com/loonghao/vx/workflows/Test/badge.svg)](https://github.com/loonghao/vx/actions)
[![Release](https://github.com/loonghao/vx/workflows/Release/badge.svg)](https://github.com/loonghao/vx/actions)
[![codecov](https://codecov.io/gh/loonghao/vx/branch/main/graph/badge.svg)](https://codecov.io/gh/loonghao/vx)
[![GitHub release](https://img.shields.io/github/release/loonghao/vx.svg)](https://github.com/loonghao/vx/releases)
[![GitHub downloads](https://img.shields.io/github/downloads/loonghao/vx/total.svg)](https://github.com/loonghao/vx/releases)

</div>

---

## 🤖 为 AI 原生开发而生

> *"Claude Code 被设计为低层级且不强制特定工作流的工具……创建一个灵活、可定制、可脚本化且安全的强力工具。"*
> — [Anthropic 工程博客：Claude Code 最佳实践](https://www.anthropic.com/engineering/claude-code-best-practices)

vx 遵循 Anthropic 为 AI 原生开发工具推荐的 **Unix 哲学** 和 **可脚本化（Scriptability）** 原则：

| 原则 | vx 如何实现 |
|------|------------|
| **Unix 哲学** | 一个工具，一个职责 — `vx` 透明管理所有运行时 |
| **可脚本化** | 完整的 bash 集成，CI/CD 就绪，支持无头模式 |
| **可组合性** | 与任何 AI 编码助手协作（Claude Code、Cursor、Copilot） |
| **零配置** | AI 代理可以直接使用任何工具，无需环境设置 |

### 为什么这对 AI 编码助手很重要

当 Claude Code 等 AI 代理需要跨不同生态系统执行命令时：

```bash
# 没有 vx：AI 必须处理复杂的环境设置
# "首先安装 Node.js，然后配置 npm，设置 PATH..."

# 有了 vx：AI 直接运行命令
vx npx create-react-app my-app  # 立即可用
vx uvx ruff check .             # 立即可用
vx cargo build --release        # 立即可用
```

**vx 让 AI 具备完整的全栈开发能力，无需为环境管理和依赖发愁。**

---

## 💡 设计理念

### 我们解决的问题

每次开始新的开发项目时，我们都面临同样令人沮丧的循环：

- 为前端工具安装 Node.js 和 npm
- 为脚本和自动化设置 Python 和 pip/uv
- 为后端服务配置 Go
- 为系统工具管理 Rust 工具链
- 处理版本冲突和 PATH 问题
- 在不同机器和环境中重复这个过程

**随着 MCP（模型上下文协议）的兴起**，这个问题变得更加突出。许多 MCP 服务器需要 `uvx` 用于 Python 工具，需要 `npx` 用于 Node.js 包，迫使开发者管理多个工具生态系统才能让 AI 辅助正常工作。

### 我们的解决方案：零学习成本

vx 在保持**零学习成本**的同时消除了这种复杂性：

```bash
# 不再需要学习和管理多个工具：
npx create-react-app my-app     # 需要 Node.js 设置
uvx ruff check .                # 需要 Python/UV 设置
go run main.go                  # 需要 Go 安装

# 只需使用 vx 和您已经知道的相同命令：
vx npx create-react-app my-app  # 需要时自动安装 Node.js
vx uvx ruff check .             # 需要时自动安装 UV
vx go run main.go               # 需要时自动安装 Go
```

---

## 🚀 快速开始

### 安装

**Linux/macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
```

**Windows (PowerShell):**

```powershell
powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
```

### 在限流网络中稳定安装

```bash
# 1）固定稳定安装版本（推荐用于 CI 与企业网络）
VX_VERSION="0.8.4" curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash

# 2）配置多源发布镜像（逗号分隔）
VX_RELEASE_BASE_URLS="https://mirror.example.com/vx/releases,https://github.com/loonghao/vx/releases" \
  curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
```

```powershell
# Windows 镜像回退（逗号或分号分隔）
$env:VX_RELEASE_BASE_URLS="https://mirror.example.com/vx/releases,https://github.com/loonghao/vx/releases"
powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
```

> 安装器会自动遍历所有配置的发布基址，并对不同资产命名模式执行回退重试。

### 立即开始使用


```bash
# 无需设置 - 只需在命令前加上 'vx'
vx node --version               # 自动安装 Node.js
vx python --version             # 通过 UV 自动安装 Python
vx go version                   # 自动安装 Go
vx cargo --version              # 自动安装 Rust
```

---

## 🎯 两种使用方式

### 1️⃣ 直接执行（用于快速任务）

只需在任何命令前加上 `vx` — 工具在首次使用时自动安装：

```bash
# 即时运行任何工具
vx npx create-react-app my-app
vx uvx ruff check .
vx go run main.go
vx cargo build --release
```

### 2️⃣ 项目开发环境（用于团队协作）

创建 `vx.toml` 文件来定义项目的工具需求：

```bash
# 初始化新项目
vx init

# 或手动创建 vx.toml
cat > vx.toml << 'EOF'
[tools]
node = "20"
python = "3.12"
uv = "latest"
go = "1.21"

[scripts]
dev = "npm run dev"
test = "npm test"
lint = "uvx ruff check ."
EOF
```

然后使用开发环境命令：

```bash
# 一键设置：安装所有项目工具
vx setup

# 进入开发 shell，所有工具都可用
vx dev

# 运行项目脚本
vx run dev
vx run test
vx run lint

# 管理项目工具
vx add bun                      # 添加工具
vx remove go                    # 移除工具
vx sync                         # 同步工具与 vx.toml
```

---

## 📋 命令参考

### 工具执行

| 命令 | 描述 |
|---------|-------------|
| `vx <runtime>[@version] [args...]` | 执行运行时（需要时自动安装） |
| `vx <runtime>[@version]::<executable> [args...]` | 执行运行时中的特定可执行文件 |
| `vx <ecosystem>:<package>[::executable] [args...]` | 执行包（RFC 0027） |
| `vx --with <runtime>[@version] <command>` | 为本次调用注入伴随运行时 |
| `vx install <runtime>@<version>` | 安装特定运行时版本 |
| `vx uninstall <runtime>[@version]` | 卸载运行时版本 |
| `vx switch <runtime>@<version>` | 切换到不同版本 |
| `vx which <runtime>` | 显示正在使用的版本 |
| `vx versions <runtime>` | 显示可用版本 |
| `vx list` | 列出所有支持的运行时 |
| `vx search <query>` | 搜索可用运行时 |

### Shell 与环境

| 命令 | 描述 |
|---------|-------------|
| `vx shell launch <runtime>[@version] [shell]` | 启动带有运行时环境的 shell（规范形式） |
| `vx dev` | 进入带有项目工具的开发 shell |
| `vx dev -c <cmd>` | 在开发环境中运行命令 |

### 全局包管理 (`vx pkg`)

| 命令 | 描述 |
|---------|-------------|
| `vx pkg install <ecosystem>:<package>` | 安装全局包 |
| `vx pkg uninstall <ecosystem>:<package>` | 卸载全局包 |
| `vx pkg list` | 列出全局安装的包 |
| `vx pkg info <ecosystem>:<package>` | 显示包信息 |

### 项目管理

| 命令 | 描述 |
|---------|-------------|
| `vx init` | 初始化项目配置（`vx.toml`） |
| `vx setup` | 安装 `vx.toml` 中定义的所有工具 |
| `vx sync` | 同步已安装工具与 `vx.toml` |
| `vx lock` | 生成或更新 `vx.lock` 以确保可复现性 |
| `vx check` | 检查版本约束和工具可用性 |
| `vx add <runtime>` | 添加运行时到项目配置 |
| `vx remove <runtime>` | 从项目配置移除运行时 |
| `vx run <script>` | 运行 `vx.toml` 中定义的脚本 |

### 系统管理

| 命令 | 描述 |
|---------|-------------|
| `vx cache info` | 显示磁盘使用和缓存统计信息 |
| `vx cache prune` | 清理缓存和孤立包 |
| `vx config` | 管理全局配置 |
| `vx self-update` | 更新 vx 本身 |
| `vx provider list` | 列出可用的 provider |

---

## 📁 项目配置（`vx.toml`）

```toml
# VX 项目配置
# 运行 'vx setup' 安装所有工具
# 运行 'vx dev' 进入开发环境

[tools]
node = "20"                     # 主版本号
python = "3.12"                 # 次版本号
uv = "latest"                   # 始终最新
go = "1.21.6"                   # 精确版本
rustup = "latest"               # Rust 工具链管理器

[settings]
auto_install = true             # 在 dev shell 中自动安装缺失工具
parallel_install = true         # 并行安装工具

[env]
NODE_ENV = "development"
DEBUG = "true"

[scripts]
dev = "npm run dev"
test = "npm test && cargo test"
build = "npm run build"
lint = "uvx ruff check . && npm run lint"
format = "uvx black . && npm run format"
# 增强：使用 {{args}} 处理复杂工具参数
test-pkgs = "cargo test {{args}}"
lint-fix = "eslint {{args}}"
```

### 🚀 增强的脚本系统

vx 现在支持**高级参数传递**，适用于复杂的工具工作流：

```bash
# 直接向工具传递复杂参数
vx run test-pkgs -p vx-runtime --lib
vx run lint-fix --fix --ext .js,.ts src/

# 获取脚本特定帮助
vx run test-pkgs -H

# 列出所有可用脚本
vx run --list
```

**主要特性：**
- ✅ **零冲突**：直接向脚本传递 `-p`、`--lib`、`--fix` 等参数
- ✅ **脚本帮助**：使用 `-H` 获取脚本特定文档
- ✅ **灵活参数**：在脚本定义中使用 `{{args}}` 获得最大灵活性
- ✅ **向后兼容**：现有脚本继续正常工作

---

## 🔌 MCP 集成

vx 在设计时就考虑了 MCP（模型上下文协议）。只需将命令从工具名改为 `vx`：

### 之前（需要复杂设置）

```json
{
  "mcpServers": {
    "browsermcp": {
      "command": "npx",
      "args": ["-y", "@browsermcp/mcp@latest"]
    },
    "python-tool": {
      "command": "uvx",
      "args": ["some-python-tool@latest"]
    }
  }
}
```

### 之后（使用 vx 零设置）

```json
{
  "mcpServers": {
    "browsermcp": {
      "command": "vx",
      "args": ["npx", "-y", "@browsermcp/mcp@latest"]
    },
    "python-tool": {
      "command": "vx",
      "args": ["uvx", "some-python-tool@latest"]
    }
  }
}
```

---

## 🎯 实际示例

### 团队入职

```bash
# 新团队成员加入项目
git clone https://github.com/your-org/your-project
cd your-project

# 一个命令设置所有东西
vx setup

# 开始开发
vx dev
```

### 多语言项目

```bash
# 前端 (Node.js) + 后端 (Go) + 脚本 (Python)
cat > vx.toml << 'EOF'
[tools]
node = "20"
go = "1.21"
uv = "latest"

[scripts]
frontend = "npm run dev"
backend = "go run cmd/server/main.go"
migrate = "uvx alembic upgrade head"
EOF

# 安装所有东西
vx setup

# 运行不同部分
vx run frontend
vx run backend
vx run migrate
```

### Python 开发

```bash
vx uv init my-python-app
cd my-python-app
vx uv add fastapi uvicorn
vx uv add --dev pytest black ruff
vx uv run uvicorn main:app --reload
vx uvx ruff check .
```

### Node.js 开发

```bash
vx npx create-react-app my-app
cd my-app
vx npm install
vx npm run dev
```

### Go 开发

```bash
vx go mod init my-go-app
vx go run main.go
vx go build -o app
```

### Rust 开发

```bash
vx cargo new my-rust-app
cd my-rust-app
vx cargo add serde tokio
vx cargo run
```

---

## 📖 支持的工具

### 语言运行时

| 工具 | 命令 | 描述 |
|------|----------|-------------|
| **Node.js** | `node`, `npm`, `npx` | JavaScript 运行时和包管理器 |
| **Bun** | `bun`, `bunx` | 快速全能 JavaScript 运行时 |
| **Deno** | `deno` | 安全的 JavaScript/TypeScript 运行时 |
| **Go** | `go` | Go 编程语言 |
| **Rust** | `cargo`, `rustc`, `rustup` | Rust 工具链 |
| **Java** | `java`, `javac` | Java 开发工具包 |
| **Zig** | `zig` | Zig 编程语言 |

### 包管理器

| 工具 | 命令 | 描述 |
|------|----------|-------------|
| **UV** | `uv`, `uvx` | 快速 Python 包管理器 |
| **pnpm** | `pnpm`, `pnpx` | 快速、磁盘高效的包管理器 |
| **Yarn** | `yarn` | JavaScript 包管理器 |

### 构建工具

| 工具 | 命令 | 描述 |
|------|----------|-------------|
| **Vite** | `vite` | 下一代前端工具 |
| **Just** | `just` | 项目任务命令运行器 |
| **Task** | `task` | 任务运行器 / 构建工具 (go-task) |
| **CMake** | `cmake` | 跨平台构建系统生成器 |
| **Ninja** | `ninja` | 专注于速度的小型构建系统 |
| **protoc** | `protoc` | Protocol Buffers 编译器 |

### DevOps 工具

| 工具 | 命令 | 描述 |
|------|----------|-------------|
| **Podman** | `podman` | 容器运行时和工具 |
| **Terraform** | `terraform` | 基础设施即代码 |
| **kubectl** | `kubectl` | Kubernetes 命令行工具 |
| **Helm** | `helm` | Kubernetes 包管理器 |

### 云 CLI 工具

| 工具 | 命令 | 描述 |
|------|----------|-------------|
| **AWS CLI** | `aws` | 亚马逊云服务 CLI |
| **Azure CLI** | `az` | 微软 Azure CLI |
| **gcloud** | `gcloud` | 谷歌云平台 CLI |

### 代码质量工具

| 工具 | 命令 | 描述 |
|------|----------|-------------|
| **pre-commit** | `pre-commit` | 预提交钩子框架 |

### 其他工具

| 工具 | 命令 | 描述 |
|------|----------|-------------|
| **VS Code** | `code` | Visual Studio Code 编辑器 |
| **Rez** | `rez` | 包管理系统 |
| **rcedit** | `rcedit` | Windows 资源编辑器 |

---

## 🌟 为什么选择 vx？

| 特性 | vx | nvm/pyenv 等 |
|---------|-----|----------------|
| **零学习成本** | ✅ 使用您熟悉的命令 | ❌ 需要学习新命令 |
| **多语言支持** | ✅ 一个工具管理所有 | ❌ 每种语言一个工具 |
| **自动安装** | ✅ 首次使用时安装 | ❌ 手动安装 |
| **项目配置** | ✅ `vx.toml` | ❌ 因工具而异 |
| **团队同步** | ✅ `vx setup` | ❌ 手动协调 |
| **MCP 就绪** | ✅ 只需添加 `vx` | ❌ 复杂设置 |
| **跨平台** | ✅ Windows/macOS/Linux | ⚠️ 因工具而异 |

---

## ⚙️ 高级配置

### 全局配置

`~/.config/vx/config.toml`:

```toml
[defaults]
auto_install = true
check_updates = true
update_interval = "24h"

[tools.node]
version = "20"

[tools.uv]
version = "latest"
```

### Shell 集成

```bash
# 添加到您的 shell 配置文件以启用自动补全
eval "$(vx shell init bash)"   # Bash
eval "$(vx shell init zsh)"    # Zsh
vx shell init fish | source    # Fish
```

### 使用 GitHub Token 自更新

```bash
# 在共享环境中避免速率限制
vx self-update --token ghp_your_token_here

# 或设置环境变量
export GITHUB_TOKEN=ghp_your_token_here
vx self-update
```

---

## 📦 安装选项

### 包管理器

```bash
# Windows
winget install loonghao.vx
choco install vx
scoop install vx

# macOS
brew tap loonghao/vx && brew install vx

# Arch Linux
yay -S vx-bin

# Cargo
cargo install --git https://github.com/loonghao/vx
```

### 容器镜像

```bash
podman pull loonghao/vx:latest
podman run --rm loonghao/vx --version
```

### GitHub Actions

在 CI/CD 工作流中使用 vx：

```yaml
- uses: loonghao/vx@main
  with:
    github-token: ${{ secrets.GITHUB_TOKEN }}

- run: vx node --version
- run: vx npm ci
- run: vx npm test
```

> **注意**: 请使用 `@main` 获取最新版本，或使用具体的版本标签（如 `@vx-v0.8.15`）。查看 [releases](https://github.com/loonghao/vx/releases) 获取最新版本。

详细文档请参阅 [GitHub Action 指南](docs/guides/github-action.md)。

---

## 🧪 测试

vx 提供了覆盖所有 provider 的完整测试套件：

```bash
# 在干净的临时环境中测试所有 provider
vx just test-providers

# 输出详细日志
vx just test-providers-verbose

# 仅测试指定 provider
vx just test-providers-filter "node"

# 保留缓存以便排查
vx just test-providers-keep
```

测试套件特性：
- ✅ 使用临时 VX_HOME（测试后自动清理）
- ✅ 自动发现源码中的所有 provider
- ✅ 验证命令执行与自动安装链路
- ✅ 生成详细测试报告
- ✅ 支持 CI/CD，包含退出码与 JSON 输出

详细说明请参阅 [scripts/README.md](scripts/README.md)。

---

## 🤝 贡献

我们欢迎贡献！请参阅 [CONTRIBUTING.md](CONTRIBUTING.md) 了解指南。

1. **报告问题**: [提交问题](https://github.com/loonghao/vx/issues)
2. **功能请求**: [开始讨论](https://github.com/loonghao/vx/discussions)
3. **代码贡献**: 提交拉取请求

---

## 📄 许可证

MIT 许可证 - 详情请参见 [LICENSE](LICENSE)。

## 📞 支持

- 📖 **文档**: [GitHub Wiki](https://github.com/loonghao/vx/wiki)
- 💬 **讨论**: [GitHub Discussions](https://github.com/loonghao/vx/discussions)
- 🐛 **问题**: [错误报告](https://github.com/loonghao/vx/issues)
- 📧 **联系**: <hal.long@outlook.com>

---

<div align="center">

**由开发者为开发者制作，充满 ❤️**

</div>
