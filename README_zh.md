# 🚀 vx - 通用开发工具管理器

<div align="center">

**一个命令统治所有工具 — 零设置，零学习成本**

[English](README.md) | [📖 文档](https://docs.rs/vx) | [🚀 快速开始](#-快速开始) | [💡 示例](#-实际示例)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.80+-blue.svg)](https://www.rust-lang.org)
[![Test](https://github.com/loonghao/vx/workflows/Test/badge.svg)](https://github.com/loonghao/vx/actions)
[![Release](https://github.com/loonghao/vx/workflows/Release/badge.svg)](https://github.com/loonghao/vx/actions)
[![codecov](https://codecov.io/gh/loonghao/vx/branch/main/graph/badge.svg)](https://codecov.io/gh/loonghao/vx)
[![GitHub release](https://img.shields.io/github/release/loonghao/vx.svg)](https://github.com/loonghao/vx/releases)
[![GitHub downloads](https://img.shields.io/github/downloads/loonghao/vx/total.svg)](https://github.com/loonghao/vx/releases)

</div>

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

创建 `.vx.toml` 文件来定义项目的工具需求：

```bash
# 初始化新项目
vx init

# 或手动创建 .vx.toml
cat > .vx.toml << 'EOF'
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
vx rm-tool go                   # 移除工具
vx sync                         # 同步工具与 .vx.toml
```

---

## 📋 命令参考

### 工具执行

| 命令 | 描述 |
|---------|-------------|
| `vx <tool> [args...]` | 执行工具（需要时自动安装） |
| `vx install <tool>[@version]` | 安装特定工具版本 |
| `vx uninstall <tool> [version]` | 卸载工具版本 |
| `vx switch <tool>@<version>` | 切换到不同版本 |
| `vx which <tool>` | 显示正在使用的版本 |
| `vx versions <tool>` | 显示可用版本 |
| `vx list` | 列出所有支持的工具 |
| `vx search <query>` | 搜索可用工具 |

### 项目环境

| 命令 | 描述 |
|---------|-------------|
| `vx init` | 初始化项目配置（`.vx.toml`） |
| `vx setup` | 安装 `.vx.toml` 中定义的所有工具 |
| `vx dev` | 进入带有项目工具的开发 shell |
| `vx dev -c <cmd>` | 在开发环境中运行命令 |
| `vx sync` | 同步已安装工具与 `.vx.toml` |
| `vx add <tool>` | 添加工具到项目配置 |
| `vx rm-tool <tool>` | 从项目配置移除工具 |
| `vx run <script>` | 运行 `.vx.toml` 中定义的脚本 |

### 系统管理

| 命令 | 描述 |
|---------|-------------|
| `vx stats` | 显示磁盘使用和统计信息 |
| `vx clean` | 清理缓存和孤立包 |
| `vx config` | 管理全局配置 |
| `vx self-update` | 更新 vx 本身 |
| `vx plugin list` | 列出可用插件 |

---

## 📁 项目配置（`.vx.toml`）

```toml
# VX 项目配置
# 运行 'vx setup' 安装所有工具
# 运行 'vx dev' 进入开发环境

[tools]
node = "20"                     # 主版本号
python = "3.12"                 # 次版本号
uv = "latest"                   # 始终最新
go = "1.21.6"                   # 精确版本
rust = ">=1.70"                 # 版本范围

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
```

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
cat > .vx.toml << 'EOF'
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

### DevOps 工具

| 工具 | 命令 | 描述 |
|------|----------|-------------|
| **Terraform** | `terraform` | 基础设施即代码 |
| **kubectl** | `kubectl` | Kubernetes 命令行工具 |
| **Helm** | `helm` | Kubernetes 包管理器 |

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
| **项目配置** | ✅ `.vx.toml` | ❌ 因工具而异 |
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

### Docker

```bash
docker pull loonghao/vx:latest
docker run --rm loonghao/vx --version
```

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
