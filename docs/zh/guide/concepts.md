# 核心概念

理解 vx 背后的核心概念有助于你更有效地使用和扩展它。

## 架构概览

```
┌────────────────────────────────────────────────────┐
│                    vx CLI                           │
│  vx <runtime> [args]  │  vx run <script>           │
└──────────┬─────────────┴───────────────┬────────────┘
           │                             │
     ┌─────▼──────┐              ┌───────▼────────┐
     │   解析器    │              │   脚本引擎     │
     │  (依赖 +   │              │  (插值 +       │
     │   版本)    │              │   .env)        │
     └─────┬──────┘              └───────┬────────┘
           │                             │
     ┌─────▼──────────────────────────────▼──────┐
     │            Provider 注册表                  │
     │  ┌────────┐ ┌────────┐ ┌────────┐        │
     │  │ Node   │ │ Python │ │  Go    │  ...   │
     │  │Provider│ │Provider│ │Provider│        │
     │  └───┬────┘ └───┬────┘ └───┬────┘        │
     │      │          │          │              │
     │  ┌───▼──┐  ┌────▼───┐ ┌───▼──┐          │
     │  │node  │  │python  │ │ go   │  ...     │
     │  │npm   │  │uv      │ │gofmt │          │
     │  │npx   │  │uvx     │ └──────┘          │
     │  └──────┘  └────────┘                    │
     └──────────────────────┬────────────────────┘
                            │
     ┌──────────────────────▼────────────────────┐
     │            内容寻址存储                      │
     │  ~/.vx/store/<runtime>/<version>/          │
     └───────────────────────────────────────────┘
```

## Provider

**Provider** 是提供一个或多个相关运行时的模块，是 vx 的组织单元。

```
Provider (例如 NodeProvider)
├── Runtime: node       (Node.js 运行时)
├── Runtime: npm        (Node 包管理器)
└── Runtime: npx        (Node 包执行器)
```

每个 Provider 负责：
- **版本发现** — 从上游获取可用版本
- **安装** — 下载和解压二进制文件
- **执行** — 使用正确的环境运行命令
- **平台支持** — 处理操作系统/架构差异

### 内置 Provider

vx 内置了 **48+ 个 Provider**，覆盖主要生态系统：

| 生态系统 | Provider |
|----------|----------|
| **Node.js** | node, npm, npx, pnpm, yarn, bun |
| **Python** | python, uv, uvx |
| **Go** | go, gofmt |
| **Rust** | rust (rustc, cargo, rustup) |
| **.NET** | dotnet, msbuild, nuget |
| **DevOps** | terraform, kubectl, helm, docker |
| **云** | awscli, azcli, gcloud |
| **构建** | cmake, ninja, just, task, make, meson, protoc |
| **媒体** | ffmpeg, imagemagick |
| **AI** | ollama |
| **其他** | git, jq, deno, zig, java, gh, curl, pwsh... |

### 声明式 Provider

你可以使用 TOML 清单定义自定义 Provider，无需编写 Rust 代码：

```toml
# ~/.vx/providers/mytool/provider.toml
[provider]
name = "mytool"
description = "我的自定义工具"

[[runtimes]]
name = "mytool"
executable = "mytool"
description = "我的工具"

[runtimes.version_source]
type = "github_releases"
owner = "myorg"
repo = "mytool"
```

详见[声明式 Provider](/zh/guide/manifest-driven-providers)。

## Runtime

**Runtime** 是由 Provider 管理的单个可执行工具。每个 Runtime 具有：

- **名称** — 主要标识符（如 `node`、`python`、`go`）
- **别名** — 替代名称（如 `nodejs` → `node`、`golang` → `go`）
- **生态系统** — 所属生态系统（Node.js、Python、Go 等）
- **依赖** — 需要的其他运行时（如 `npm` 依赖 `node`）

### 运行时依赖

vx 自动解析并安装依赖：

```
npm ──依赖──> node
npx ──依赖──> node
uvx ──依赖──> uv
cargo ──依赖──> rust
gofmt ──依赖──> go
```

当你运行 `vx npm install` 时，vx 会确保 Node.js 已安装。

## 版本解析

vx 支持多种版本规格格式：

| 格式 | 示例 | 说明 |
|------|------|------|
| 精确版本 | `22.11.0` | 指定版本 |
| 主版本 | `22` | 最新 22.x.x |
| 次版本 | `22.11` | 最新 22.11.x |
| 范围 | `^22.0.0` | 兼容 22.x.x |
| 范围 | `~22.11.0` | 兼容 22.11.x |
| 最新版 | `latest` | 最新稳定版本 |
| LTS | `lts` | 最新 LTS 版本（Node.js） |
| 通道 | `stable` / `beta` / `nightly` | 发布通道（Rust） |

### 版本解析顺序

确定使用哪个版本时，vx 按以下顺序检查：

1. **命令行** — `vx install node@22`
2. **环境变量** — `VX_NODE_VERSION=22`
3. **项目配置** — 当前或父目录中的 `vx.toml`
4. **锁文件** — `vx.lock` 中精确锁定的版本
5. **全局配置** — `~/.config/vx/config.toml`
6. **自动检测** — 最新稳定版本

## 内容寻址存储

所有工具存储在全局**内容寻址存储**中：

```
~/.vx/
├── store/                      # 全局工具存储
│   ├── node/
│   │   ├── 22.11.0/           # 完整安装
│   │   └── 20.18.0/
│   ├── python/
│   │   └── 3.12.8/
│   └── go/
│       └── 1.23.4/
├── cache/                      # 下载缓存
│   └── downloads/
├── bin/                        # 全局 shims
└── config/                     # 配置
```

### 优势

- **去重** — 相同版本只存储一份，跨项目共享
- **隔离** — 每个版本独立目录，无冲突
- **快速** — 环境通过符号链接创建，而非复制
- **可恢复** — `vx setup` 可从 `vx.toml` 重新安装

## 项目配置

`vx.toml` 文件定义项目的工具需求：

```toml
[tools]
node = "22"
python = "3.12"
uv = "latest"
just = "latest"

[scripts]
dev = "vx node server.js"
test = "vx uv run pytest"
lint = "vx uvx ruff check ."
build = "vx node scripts/build.js"

[env]
NODE_ENV = "development"
```

完整参考请见[配置](/zh/guide/configuration)。

## 执行模型

当你运行 `vx <tool> [args...]` 时：

1. **工具查找** — 找到管理该工具的 Provider
2. **版本解析** — 确定使用哪个版本
3. **依赖检查** — 确保所有依赖可用
4. **自动安装** — 如果启用了 `auto_install`，安装缺失的工具
5. **环境设置** — 设置 PATH 和环境变量
6. **转发执行** — 使用原始参数运行工具
7. **退出码透传** — 返回工具的退出码

执行过程是**透明的** — 工具的行为与直接运行完全一致。

## 生态系统

**生态系统**将相关工具分组：

| 生态系统 | 工具 |
|----------|------|
| `NodeJs` | node, npm, npx, yarn, pnpm, bun, vite, deno |
| `Python` | python, uv, uvx, pip |
| `Rust` | rust, cargo, rustc, rustup |
| `Go` | go, gofmt |
| `DotNet` | dotnet, msbuild, nuget |
| `System` | git, jq, curl, pwsh |

生态系统帮助 vx 理解工具之间的关系，优化依赖解析。

## 下一步

- [直接执行](/zh/guide/direct-execution) — 命令转发的工作原理
- [版本管理](/zh/guide/version-management) — 高级版本控制
- [项目环境](/zh/guide/project-environments) — 团队协作
- [CLI 参考](/zh/cli/overview) — 完整的命令文档
