# 简介

**vx** 是一个通用开发工具管理器，消除了管理多个开发运行时的复杂性。无需学习和配置 Node.js、Python、Go、Rust 等单独的工具，只需在命令前加上 `vx`，一切就能正常工作。

## 为什么选择 vx？

### 传统方式

```bash
# 分别安装和管理多个工具
nvm install 20
nvm use 20
npm install -g typescript

pyenv install 3.11
pyenv local 3.11
pip install uv

# 处理 PATH 冲突、版本不匹配...
```

### vx 方式

```bash
# 直接使用工具 - vx 处理一切
vx node --version
vx python --version
vx npx create-react-app my-app
vx uvx ruff check .
```

## 两种使用方式

### 1. 直接执行（适用于快速任务）

只需在任何命令前加上 `vx` — 工具在首次使用时自动安装：

```bash
vx npx create-react-app my-app
vx uvx ruff check .
vx go run main.go
```

### 2. 项目开发环境（适用于团队）

创建 `vx.toml` 文件来定义项目的工具需求：

```toml
[project]
name = "my-project"

[tools]
node = "20"
uv = "latest"
go = "1.21"

[scripts]
dev = "npm run dev"
test = "pytest"
build = "go build -o app"
```

然后运行：

```bash
vx setup     # 安装所有项目工具
vx run dev   # 运行定义的脚本
```

## 支持的工具

vx 支持广泛的开发工具：

| 生态系统 | 工具 |
|----------|------|
| **Node.js** | node, npm, npx, pnpm, yarn, bun |
| **Python** | python, uv, uvx, pip |
| **Go** | go |
| **Rust** | cargo, rustc |
| **DevOps** | kubectl, helm, terraform |
| **实用工具** | just, jq, ripgrep 等 |

## 下一步

- [安装](/zh/guide/installation) - 在你的系统上安装 vx
- [快速上手](/zh/guide/getting-started) - 几分钟内开始使用
- [配置](/zh/guide/configuration) - 了解 `vx.toml`
