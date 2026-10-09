---
layout: home

hero:
  name: vx
  text: 通用开发工具管理器
  tagline: 一个命令统管所有 - 零配置，零学习成本
  image:
    src: /logo.svg
    alt: vx
  actions:
    - theme: brand
      text: 快速开始
      link: /zh/guide/getting-started
    - theme: alt
      text: 在 GitHub 上查看
      link: https://github.com/vx-org/vx

features:
  - icon: "🚀"
    title: 零配置
    details: 开箱即用，无需任何设置。只需在命令前加上 vx 即可。
  - icon: "🔧"
    title: 自动安装
    details: 工具在首次使用时自动安装，无需手动安装。
  - icon: "📦"
    title: 版本管理
    details: 通过 vx.toml 配置文件为每个项目指定特定版本。
  - icon: "🌐"
    title: 跨平台
    details: 在 Windows、macOS 和 Linux 上无缝运行。
  - icon: "⚡"
    title: 极速性能
    details: 使用 Rust 编写，性能卓越，开销极小。
  - icon: "🔩"
    title: 可扩展
    details: 插件系统支持添加自定义工具和工作流。
---

## 我们解决的问题

每次开始新的开发项目时，我们都面临同样令人沮丧的循环：

- 为前端工具安装 Node.js 和 npm
- 为脚本和自动化设置 Python 和 pip/uv
- 为后端服务配置 Go
- 为系统工具管理 Rust 工具链
- 处理版本冲突和 PATH 问题

**随着 MCP（模型上下文协议）的兴起**，这个问题变得更加突出。许多 MCP 服务器需要 `uvx` 来运行 Python 工具，需要 `npx` 来运行 Node.js 包。

## 我们的解决方案

```bash
# 不再需要学习和管理多个工具：
npx create-react-app my-app     # 需要设置 Node.js
uvx ruff check .                # 需要设置 Python/UV
go run main.go                  # 需要安装 Go

# 只需使用 vx，使用你已经熟悉的命令：
vx npx create-react-app my-app  # 如果需要，自动安装 Node.js
vx uvx ruff check .             # 如果需要，自动安装 UV
vx go run main.go               # 如果需要，自动安装 Go
```

## 快速安装

::: code-group

```bash [Linux/macOS]
curl -fsSL https://raw.githubusercontent.com/vx-org/vx/main/install.sh | bash
```

```powershell [Windows]
irm https://raw.githubusercontent.com/vx-org/vx/main/install.ps1 | iex
```

:::
