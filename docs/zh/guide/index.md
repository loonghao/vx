# 简介

## 什么是 vx？

**vx** 是一个通用开发工具管理器，提供**零学习成本**的体验，用于管理各平台上的编程语言运行时、包管理器和开发工具。

无需学习和配置多个工具安装器 — `nvm` 管理 Node.js、`pyenv` 管理 Python、`rustup` 管理 Rust、`gvm` 管理 Go — 你只需在任何命令前加上 `vx`，一切就能正常工作。

```bash
# 只需在命令前加上 vx — 工具会自动安装
vx node --version        # 如果需要，自动安装 Node.js
vx python --version      # 如果需要，自动安装 Python
vx go version            # 如果需要，自动安装 Go
vx cargo build           # 如果需要，自动安装 Rust
```

## 为什么选择 vx？

### 问题

现代软件开发需要复杂的工具链：

- **多种语言运行时** — Node.js、Python、Go、Rust、.NET、Java、Zig 等
- **包管理器** — npm、pnpm、yarn、uv、pip、cargo 等
- **DevOps 工具** — Terraform、kubectl、Helm、Docker CLI 等
- **构建工具** — CMake、Ninja、Just、Task、protoc 等
- **云 CLI** — AWS CLI、Azure CLI、Google Cloud CLI 等

每个工具都有自己的安装器、版本管理器和配置方式。团队花费大量时间调试"在我机器上能跑"的问题。

### 解决方案

vx 提供**一个工具管理所有**：

| 功能 | vx | 传统方式 |
|------|-----|---------|
| 安装工具 | `vx install node@22` | 下载安装器、配置 PATH |
| 使用工具 | `vx node index.js` | 祈祷激活了正确的版本 |
| 切换版本 | `vx switch node 20` | `nvm use 20` / `fnm use 20` / 编辑 `.nvmrc` |
| 团队一致性 | 仓库中的 `vx.toml` | README、wiki、口口相传 |
| CI/CD | `uses: loonghao/vx@main` | 多个 setup-* actions |

## 核心特性

### 零学习成本
使用你已经熟悉的命令 — 只需在前面加上 `vx`：
```bash
vx npm install           # 和 npm install 一样，但版本受管理
vx uvx ruff check .      # 和 uvx ruff check 一样，但自动安装
vx go build ./...        # 和 go build 一样，但可移植
```

### 50+ 工具支持
从语言运行时到 DevOps 工具，vx 用统一接口管理所有工具。查看[完整工具列表](/zh/tools/overview)。

### 声明式配置
在 `vx.toml` 中定义项目的工具链：
```toml
[tools]
node = "22"
python = "3.12"
uv = "latest"
just = "latest"
```

### 自动依赖解析
vx 理解工具之间的依赖关系并自动安装：
```bash
vx npm --version         # 自动先安装 Node.js
vx cargo build           # 自动先安装 Rust
vx uvx ruff check .      # 自动先安装 uv
```

### 增强脚本系统
定义并运行带有强大变量插值功能的项目脚本：
```toml
[scripts]
dev = "vx node server.js --port {{PORT}}"
test = "vx uv run pytest {{args}}"
build = "vx cargo build --release"
lint = "vx uvx ruff check . {{args}}"
```

### 跨平台
在 Windows、macOS 和 Linux 上行为一致。

### 可扩展
通过 TOML 清单或 Rust 插件创建自定义 Provider，使用扩展系统增加功能。

## 工作原理

```
┌─────────────┐    ┌─────────────┐    ┌──────────────┐
│  vx node    │───>│  解析器      │───>│  Provider    │
│  --version  │    │  (查找工具   │    │  (安装 &     │
│             │    │   和依赖)    │    │   执行)      │
└─────────────┘    └─────────────┘    └──────────────┘
                         │                    │
                   ┌─────▼─────┐    ┌────────▼────────┐
                   │ 版本       │    │ 内容寻址存储      │
                   │ 解析       │    │ (~/.vx/)        │
                   └───────────┘    └─────────────────┘
```

1. **解析** — vx 识别运行时和命令
2. **解析** — 查找所需版本并检查依赖
3. **安装** — 下载并安装缺失的工具（如需要）
4. **执行** — 透明地转发命令

## 快速示例

```bash
# 安装 vx
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash

# 立即使用任何工具 — 无需手动设置
vx node --version        # v22.x.x
vx python --version      # Python 3.12.x
vx go version            # go1.23.x

# 设置项目
cd my-project
vx config init           # 创建 vx.toml
vx setup                 # 安装所有项目工具

# 运行项目脚本
vx run dev               # 启动开发服务器
vx run test              # 运行测试
```

## 下一步

- [安装](/zh/guide/installation) — 在你的系统上安装 vx
- [快速上手](/zh/guide/getting-started) — 5 分钟内开始使用
- [核心概念](/zh/guide/concepts) — 了解 vx 的工作原理
- [CLI 参考](/zh/cli/overview) — 完整的命令文档
