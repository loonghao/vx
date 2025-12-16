# 🚀 vx - 通用开发工具管理器

<div align="center">

**终极开发工具管理器 - 一个工具统治所有工具**

[English](README.md) | [📖 文档](https://docs.rs/vx) | [🚀 快速开始](#-快速开始) | [💡 示例](#-实际示例)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.80+-blue.svg)](https://www.rust-lang.org)
[![Test](https://github.com/loonghao/vx/workflows/Test/badge.svg)](https://github.com/loonghao/vx/actions)
[![Release](https://github.com/loonghao/vx/workflows/Release/badge.svg)](https://github.com/loonghao/vx/actions)
[![codecov](https://codecov.io/gh/loonghao/vx/branch/main/graph/badge.svg)](https://codecov.io/gh/loonghao/vx)
[![Security audit](https://github.com/loonghao/vx/workflows/Security%20audit/badge.svg)](https://github.com/loonghao/vx/actions)
[![GitHub release](https://img.shields.io/github/release/loonghao/vx.svg)](https://github.com/loonghao/vx/releases)
[![GitHub downloads](https://img.shields.io/github/downloads/loonghao/vx/total.svg)](https://github.com/loonghao/vx/releases)
[![Crates.io](https://img.shields.io/crates/v/vx.svg)](https://crates.io/crates/vx)
[![Documentation](https://docs.rs/vx/badge.svg)](https://docs.rs/vx)

*闪电般快速、格式无关的开发工具管理器，具有美观的进度跟踪*

</div>

---

## 🎯 什么是 vx？

**vx** 是一个强大、快速且可扩展的开发工具管理器，为跨不同语言和生态系统的开发工具管理、安装和执行提供统一接口。可以将其视为 `nvm`、`rustup`、`pyenv` 和包管理器的组合，全部集成在一个闪电般快速的工具中。

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

### 🌟 为什么选择 vx？

- **🔄 通用接口**: 通过单一、一致的接口执行任何支持的工具
- **📚 零学习成本**: 使用您已经知道的完全相同的命令（`npx`、`uvx`、`go` 等）
- **⚡ 闪电般快速**: 使用 Rust 构建，采用异步优先架构，实现最大性能
- **🚀 自动安装**: 自动下载和安装缺失的工具，具有美观的进度条
- **🔒 环境隔离**: 所有工具在 vx 管理的环境中运行（无系统 PATH 冲突）
- **📦 格式无关**: 支持 ZIP、TAR.GZ、TAR.XZ、TAR.BZ2 和原始二进制文件
- **🎨 美观的用户体验**: 丰富的进度条、彩色输出和直观的命令
- **🤖 MCP 就绪**: 非常适合 MCP 服务器 - 只需在命令前加上 `vx`

## ✨ 特性

### 🚀 最新改进 (v0.3.0)

- **🔄 自更新系统**: 内置自更新功能，支持 GitHub 令牌以避免速率限制
- **📁 统一路径管理**: 新的 vx-paths 系统，提供标准化工具安装路径
- **📊 增强的工具发现**: 改进的 `list` 和 `which` 命令，提供详细状态信息
- **🏗️ 模块化架构**: 使用 vx-installer 引擎完全重写，提高可维护性
- **📊 高级进度跟踪**: 美观的进度条，显示 ETA 和传输速率
- **🔧 增强的安装系统**: 支持多种存档格式和安装方法
- **🔌 插件系统**: 可扩展架构，支持内置和外部插件
- **🛡️ 安全优先**: 内置校验和验证和安全下载
- **🌍 跨平台**: 在 Windows、macOS 和 Linux 上无缝运行

### 🎯 核心特性

- **🔄 通用接口**: 通过单一、一致的接口执行任何支持的工具
- **📦 多版本管理**: 安装、管理和切换工具的多个版本
- **⚡ 零配置**: 开箱即用，具有智能默认设置
- **🚀 自动安装**: 自动下载和安装缺失的工具
- **🎯 项目特定**: 支持项目级工具配置
- **🔌 插件架构**: 模块化设计，具有可扩展的插件系统
- **🔄 自更新系统**: 内置更新功能，支持 GitHub 令牌以避免 API 速率限制
- **📁 统一路径管理**: 跨所有平台的标准化工具安装路径

### 🛠️ 高级特性

- **📊 包管理**: 类似 Chocolatey 的分层包管理
- **🔍 智能发现**: 自动工具检测和版本解析
- **⚙️ 配置管理**: 全局和项目级配置支持
- **📈 依赖跟踪**: 跟踪和管理工具依赖关系
- **🧹 清理工具**: 孤立包清理和维护
- **📋 丰富的 CLI**: 全面的命令行界面，输出有用信息

## 🚀 快速开始

### 安装

#### 快速安装（推荐）

**Linux/macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
```

**Windows (PowerShell):**

```powershell
powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
```

#### 高级安装选项

**安装特定版本:**

```bash
# Linux/macOS
VX_VERSION="0.1.0" curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash

# Windows
$env:VX_VERSION="0.1.0"; powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
```

**安装到自定义目录:**

```bash
# Linux/macOS
VX_INSTALL_DIR="$HOME/bin" curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash

# Windows
$env:VX_INSTALL_DIR="C:\tools\vx"; powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
```

#### 包管理器

**Chocolatey (Windows):**

```powershell
choco install vx
```

**Scoop (Windows):**

```powershell
scoop bucket add loonghao https://github.com/loonghao/scoop-vx.git
scoop install vx
```

**WinGet (Windows):**

```powershell
winget install loonghao.vx
```

**Homebrew (macOS):**

```bash
brew tap loonghao/vx
brew install vx
```

**Arch Linux (AUR):**

```bash
# 使用 yay
yay -S vx-bin

# 使用 paru
paru -S vx-bin
```

**Cargo (从源码):**

```bash
cargo install --git https://github.com/loonghao/vx
```

**Docker:**

```bash
# Docker Hub
docker pull loonghao/vx:latest
docker run --rm loonghao/vx --version

# GitHub Container Registry
docker pull ghcr.io/loonghao/vx:latest
docker run --rm ghcr.io/loonghao/vx --version
```

**Debian/Ubuntu (DEB):**

```bash
# 下载并安装最新版本
curl -fsSLO https://github.com/loonghao/vx/releases/latest/download/vx_amd64.deb
sudo dpkg -i vx_amd64.deb
```

**Fedora/RHEL (RPM):**

```bash
# 下载并安装最新版本
curl -fsSLO https://github.com/loonghao/vx/releases/latest/download/vx.x86_64.rpm
sudo rpm -i vx.x86_64.rpm
```

**Nix:**

```bash
# 使用 flakes
nix profile install github:loonghao/vx

# 或直接运行
nix run github:loonghao/vx -- --version
```

### ⚡ 快速示例：相同命令，更好体验

```bash
# 🎯 使用您已经知道的完全相同的命令 - 只需添加 'vx'！

# Python 开发（无需 Python 设置）
vx uv pip install requests           # 需要时自动安装 UV
vx uvx ruff check .                  # 通过 UV 自动安装 ruff
vx uvx black --check .               # 通过 UV 自动安装 black

# Node.js 开发（无需 Node.js 设置）
vx npm install react                 # 需要时自动安装 Node.js
vx npx create-react-app my-app       # 自动安装 create-react-app
vx npx -y cowsay "Hello from vx!"    # 一次性工具执行

# Go 开发（无需 Go 设置）
vx go build                          # 需要时自动安装 Go
vx go run main.go                    # 您知道的相同命令

# Rust 开发（无需 Rust 设置）
vx cargo run                         # 需要时自动安装 Rust
vx cargo build --release             # 相同的 Cargo 命令

# 🤖 非常适合 MCP 服务器 - 只需在前面加上 'vx'：
# 不再使用: npx @browsermcp/mcp@latest
# 改为使用: vx npx @browsermcp/mcp@latest
# 不再使用: uvx some-python-tool
# 改为使用: vx uvx some-python-tool

# 🔧 需要时的高级功能
vx --use-system-path python --version  # 需要时使用系统工具
vx list --status                      # 显示所有工具及安装状态
vx which node --all                   # 显示工具的所有已安装版本
vx stats                              # 包统计和使用情况

# 🔄 支持 GitHub 令牌的自更新（解决速率限制问题）
vx self-update --check                # 检查更新
vx self-update --token ghp_xxxx       # 使用 GitHub 令牌更新（推荐团队使用）
vx self-update --prerelease           # 包含预发布版本

# 🎯 具有美观进度条的版本管理
vx install uv@0.7.12                 # 安装特定版本
vx install node@20.0.0               # 丰富的进度跟踪
vx switch node@18.19.0               # 即时版本切换

# ⚙️ 项目配置
vx init                               # 初始化项目配置
vx config                             # 管理全局设置
```

## 📖 支持的工具

### 🔧 内置插件

| 工具 | 命令 | 类别 | 自动安装 | 描述 |
|------|----------|----------|--------------|-------------|
| **UV** | `vx uv pip`, `vx uv venv`, `vx uv run`, `vx uv add` | Python | ✅ | 极快的 Python 包安装器 |
| **Node.js** | `vx node`, `vx npm`, `vx npx` | JavaScript | ✅ | JavaScript 运行时和包管理器 |
| **Go** | `vx go build`, `vx go run`, `vx go test` | Go | ✅ | Go 编程语言工具链 |
| **Rust** | `vx cargo build`, `vx cargo run`, `vx cargo test` | Rust | ✅ | Rust 编程语言和 Cargo |

## 🔌 MCP 集成：完美解决方案

vx 在设计时就考虑了 MCP（模型上下文协议）。许多 MCP 服务器需要 `uvx` 和 `npx`，但设置这些工具可能复杂且容易出错。vx 通过**零配置**和**零学习成本**解决了这个问题。

### MCP 挑战

MCP 服务器通常需要多个工具生态系统：

```bash
# 传统设置需要管理多个工具：
npm install -g some-package     # 需要 Node.js 设置
uvx install some-python-tool    # 需要 Python/UV 设置
# 还要处理 PATH 冲突、版本不匹配等问题
```

### vx 解决方案：只需添加 `vx`

使用 vx，您只需在现有命令前加上 `vx` - **无学习成本，无配置**：

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

### 🎯 您获得的好处

- **📚 零学习成本**: 使用您已经知道的完全相同的 `npx` 和 `uvx` 命令
- **🚀 零配置**: 无需安装 Node.js、Python、UV 或管理 PATH
- **🔒 完全隔离**: MCP 工具在隔离环境中运行，无冲突
- **📊 美观进度**: 通过丰富的进度条查看具体发生的情况
- **🛡️ 安全优先**: 自动校验和验证和安全下载
- **🌍 跨平台**: 在 Windows、macOS 和 Linux 上行为完全一致
- **⚡ 闪电般快速**: 并发下载和安装

## 🔄 自更新系统

vx 包含强大的自更新系统，解决了在共享环境中常见的 GitHub API 速率限制问题。

### 🚀 快速更新

```bash
# 检查更新
vx self-update --check

# 更新到最新版本
vx self-update

# 使用 GitHub 令牌更新（推荐团队/共享网络使用）
vx self-update --token ghp_your_github_token_here

# 包含预发布版本
vx self-update --prerelease

# 强制更新，即使已经是最新版本
vx self-update --force
```

### 🔐 GitHub 令牌支持

**问题**: GitHub 的公共 API 对未认证用户有每小时 60 次请求的速率限制。在共享环境（办公室、学校、公寓）中，多个用户可能会达到此限制。

**解决方案**: 使用 GitHub 令牌将速率限制提高到每小时 5,000 次请求：

```bash
# 在此处创建 GitHub 令牌：https://github.com/settings/tokens
# 不需要特殊权限 - 只需基本访问权限

# 使用令牌进行更新
vx self-update --token ghp_xxxxxxxxxxxxxxxxxxxx

# 或设置为环境变量
export GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxx
vx self-update
```

### 📊 更新功能

- **🔍 智能检测**: 自动检测当前版本和最新版本
- **📦 跨平台**: 支持 Windows、macOS 和 Linux 二进制文件
- **🔒 安全下载**: 仅使用 HTTPS，自动验证
- **📋 发布说明**: 显示新版本的更新日志
- **🔄 备份和回滚**: 自动备份当前版本
- **⚡ 快速下载**: 并发下载，带进度条
- **🎯 格式支持**: ZIP、TAR.GZ 存档和原始二进制文件

## ⚙️ 配置

### 全局配置

`~/.config/vx/config.toml`:

```toml
[defaults]
auto_install = true        # 自动安装缺失的工具
check_updates = true       # 检查更新
update_interval = "24h"    # 更新检查频率

[tools.uv]
version = "0.5.26"
install_method = "official"

[tools.node]
version = "20.11.0"
install_method = "official"

[tools.go]
version = "1.21.6"
```

### 项目配置

`.vx.toml`:

```toml
[tools]
uv = "0.5.26"
node = "20.11.0"
go = "1.21.6"

[defaults]
auto_install = true
```

## 🎯 实际示例

### Python 开发（使用 UV）

```bash
# 创建新的 Python 项目
vx uv init my-python-app
cd my-python-app

# 添加依赖
vx uv add fastapi uvicorn
vx uv add --dev pytest black

# 运行应用程序
vx uv run uvicorn main:app --reload

# 运行测试
vx uv run pytest
```

### Node.js 开发

```bash
# 安装和使用 Node.js
vx npm install express
vx node server.js

# 使用 npx 执行一次性工具
vx npx create-react-app my-app
vx npx -y typescript --init
```

## 🛠️ 开发

### 前提条件

- Rust 1.70+
- Cargo

### 构建

```bash
git clone https://github.com/loonghao/vx
cd vx
cargo build --release
```

### 测试

```bash
cargo test
cargo run -- --help
```

## 🚀 路线图

### 当前状态 (v0.3.0)

- ✅ **核心插件架构** 具有基于特征的可扩展性
- ✅ **6 个内置工具**（UV、UVX、Node.js、NPX、Go、Rust）
- ✅ **完整环境隔离系统** 具有完整的 PATH 管理
- ✅ **🆕 自更新系统** 支持 GitHub 令牌以避免速率限制
- ✅ **🆕 统一路径管理** 使用 vx-paths 系统（`~/.vx/tools/<tool>/<version>/`）
- ✅ **🆕 增强的工具发现** 改进的 `list --status` 和 `which --all` 命令
- ✅ **vx-installer 引擎** 具有通用格式支持
- ✅ **美观的进度条** 具有 ETA 和传输速率
- ✅ **安全优先下载** 具有校验和验证
- ✅ **异步安装系统** 具有并发操作
- ✅ **多版本包管理** 具有智能切换
- ✅ **MCP 集成支持** 用于无缝代理使用
- ✅ **包运行器支持**（npx、uvx）具有环境隔离
- ✅ **项目配置支持** 具有基于 TOML 的配置

### 即将推出的功能

#### 🔧 新工具支持（第7阶段）

- [ ] **just** - 现代命令运行器和构建工具（`vx just --list`、`vx just build`）
- [ ] **kubectl** - Kubernetes 命令行工具（`vx kubectl get pods`、`vx kubectl apply`）
- [ ] **deno** - 现代 JavaScript/TypeScript 运行时（`vx deno run`、`vx deno task`）
- [ ] **podman** - 容器管理（`vx podman run`、`vx podman build`）
- [ ] **zig** - 系统编程语言（`vx zig build`、`vx zig run`）

#### 🚀 增强功能

- [ ] **增强的包管理器**: pnpm、yarn、bun 与完整的 vx-installer 集成
- [ ] **系统包管理器**: Homebrew、Chocolatey、apt、yum 支持
- [ ] **专业工具**: 用于 VFX 的 Rez、用于 HPC 环境的 Spack
- [ ] **外部插件支持**: .dll、.so 和基于脚本的插件
- [ ] **插件市场**: 社区驱动的插件生态系统
- [ ] **高级安装方法**: Docker、容器和虚拟环境
- [ ] **GUI 界面**: 具有可视化工具管理的桌面应用程序
- [ ] **CI/CD 集成**: GitHub Actions、GitLab CI、Jenkins 插件
- [ ] **团队配置同步**: 共享配置和工具版本
- [ ] **性能优化**: 缓存、并行操作和智能更新

## 🤝 贡献

我们欢迎贡献！以下是您可以帮助的方式：

1. **报告问题**: 发现了 bug？[提交问题](https://github.com/loonghao/vx/issues)
2. **功能请求**: 有想法？[开始讨论](https://github.com/loonghao/vx/discussions)
3. **插件开发**: 为新工具创建插件
4. **文档**: 改进文档和示例
5. **代码贡献**: 提交拉取请求

详细指南请参见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 📄 许可证

本项目采用 MIT 许可证 - 详情请参见 [LICENSE](LICENSE) 文件。

## 🙏 致谢

- 受 `asdf`、`mise`、`proto` 和 `chocolatey` 等工具启发
- 使用 Rust 和现代开发实践构建，充满 ❤️
- 特别感谢 Rust 社区和所有贡献者

## 📞 支持

- 📖 **文档**: [完整文档](https://github.com/loonghao/vx/wiki)
- 💬 **讨论**: [GitHub 讨论](https://github.com/loonghao/vx/discussions)
- 🐛 **问题**: [错误报告](https://github.com/loonghao/vx/issues)
- 📧 **联系**: <hal.long@outlook.com>

---

**由开发者为开发者制作，充满 ❤️**
