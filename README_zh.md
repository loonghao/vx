# vx - 通用开发工具管理器

[English](README.md)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![CI](https://github.com/loonghao/vx/workflows/CI/badge.svg)](https://github.com/loonghao/vx/actions)
[![Release](https://github.com/loonghao/vx/workflows/Release/badge.svg)](https://github.com/loonghao/vx/actions)
[![GitHub release](https://img.shields.io/github/release/loonghao/vx.svg)](https://github.com/loonghao/vx/releases)
[![GitHub downloads](https://img.shields.io/github/downloads/loonghao/vx/total.svg)](https://github.com/loonghao/vx/releases)

> 🚀 终极开发工具管理器 - 一个工具统治所有工具

`vx` 是一个强大、快速且可扩展的开发工具管理器，为跨不同语言和生态系统的开发工具管理、安装和执行提供统一接口。可以将其视为 `nvm`、`rustup`、`pyenv` 和包管理器的组合，全部集成在一个闪电般快速的工具中。

## ✨ 特性

### 🎯 核心特性
- **🔄 通用接口**: 通过单一、一致的接口执行任何支持的工具
- **📦 多版本管理**: 安装、管理和切换工具的多个版本
- **⚡ 零配置**: 开箱即用，具有智能默认设置
- **🚀 自动安装**: 自动下载和安装缺失的工具
- **🎯 项目特定**: 支持项目级工具配置
- **🔌 插件架构**: 模块化设计，具有可扩展的插件系统

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
irm https://raw.githubusercontent.com/loonghao/vx/main/install-release.ps1 | iex
```

#### 包管理器

**Homebrew (macOS/Linux):**
```bash
brew install loonghao/tap/vx
```

**Scoop (Windows):**
```powershell
scoop bucket add loonghao https://github.com/loonghao/scoop-bucket
scoop install vx
```

**Cargo (从源码):**
```bash
cargo install --git https://github.com/loonghao/vx
```

### 基本用法

```bash
# 通过 vx 执行工具 - 如果缺失会自动安装！
vx uv pip install requests
vx npm install react
vx node app.js
vx go build
vx cargo run

# 列出支持的工具和插件
vx list
vx plugin list

# 安装特定版本
vx install uv@0.5.26
vx install node@20.11.0
vx install go@1.21.6

# 在版本之间切换
vx switch uv@0.5.26
vx switch node@18.19.0

# 项目配置
vx init
vx config
```

## 📖 支持的工具

### 🔧 内置插件

| 工具 | 命令 | 类别 | 自动安装 | 描述 |
|------|----------|----------|--------------|-------------|
| **UV** | `vx uv pip`, `vx uv venv`, `vx uv run`, `vx uv add` | Python | ✅ | 极快的 Python 包安装器 |
| **Node.js** | `vx node`, `vx npm`, `vx npx` | JavaScript | ✅ | JavaScript 运行时和包管理器 |
| **Go** | `vx go build`, `vx go run`, `vx go test` | Go | ✅ | Go 编程语言工具链 |
| **Rust** | `vx cargo build`, `vx cargo run`, `vx cargo test` | Rust | ✅ | Rust 编程语言和 Cargo |

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

### 当前状态 (v0.1.0)
- ✅ 核心插件架构
- ✅ 4 个内置插件（UV、Node.js、Go、Rust）
- ✅ 自动安装系统
- ✅ 多版本包管理
- ✅ 项目配置支持

### 即将推出的功能
- [ ] 更多内置插件（Python、Java、.NET、Docker）
- [ ] 外部插件支持（.dll、.so、脚本）
- [ ] 插件市场
- [ ] GUI 界面
- [ ] CI/CD 集成
- [ ] 团队配置同步

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
- 📧 **联系**: hal.long@outlook.com

---

**由开发者为开发者制作，充满 ❤️**
