# 支持的工具概览

vx 支持广泛的开发工具和运行时。所有工具在首次使用时自动安装。

## 工具分类

### 语言运行时

| 工具 | 命令 | 描述 | 自动安装 |
|------|----------|-------------|--------------|
| `node` | `node`, `npm`, `npx` | Node.js JavaScript 运行时 | ✅ |
| `bun` | `bun`, `bunx` | 快速全能 JavaScript 运行时 | ✅ |
| `deno` | `deno` | 安全的 JavaScript/TypeScript 运行时 | ✅ |
| `go` | `go` | Go 编程语言 | ✅ |
| `rust` | `cargo`, `rustc`, `rustup` | Rust 编程语言 | ✅ |
| `java` | `java`, `javac` | Java 开发工具包 | ✅ |
| `zig` | `zig` | Zig 编程语言 | ✅ |

### 包管理器

| 工具 | 命令 | 描述 | 依赖 |
|------|----------|-------------|----------|
| `npm` | `npm` | Node.js 包管理器 | node |
| `npx` | `npx` | Node.js 包运行器 | node |
| `pnpm` | `pnpm`, `pnpx` | 快速、磁盘高效的包管理器 | - |
| `yarn` | `yarn` | JavaScript 包管理器 | - |
| `uv` | `uv` | 快速 Python 包管理器 | - |
| `uvx` | `uvx` | Python 工具运行器 | uv |
| `cargo` | `cargo` | Rust 包管理器 | rust |

### 构建工具

| 工具 | 命令 | 描述 | 自动安装 |
|------|----------|-------------|--------------|
| `vite` | `vite` | 下一代前端工具 | ✅ |
| `just` | `just` | 项目任务命令运行器 | ✅ |
| `task` | `task` | 任务运行器 / 构建工具 (go-task) | ✅ |
| `cmake` | `cmake` | 跨平台构建系统生成器 | ✅ |
| `ninja` | `ninja` | 专注于速度的小型构建系统 | ✅ |
| `protoc` | `protoc` | Protocol Buffers 编译器 | ✅ |

### DevOps 工具

| 工具 | 命令 | 描述 | 自动安装 |
|------|----------|-------------|--------------|
| `docker` | `docker` | 容器运行时和工具 | ✅ |
| `terraform` | `terraform` | 基础设施即代码 | ✅ |
| `kubectl` | `kubectl` | Kubernetes CLI | ✅ |
| `helm` | `helm` | Kubernetes 包管理器 | ✅ |

### 云 CLI 工具

| 工具 | 命令 | 描述 | 自动安装 |
|------|----------|-------------|--------------|
| `awscli` | `aws` | 亚马逊云服务 CLI | ✅ |
| `azcli` | `az` | 微软 Azure CLI | ✅ |
| `gcloud` | `gcloud` | 谷歌云平台 CLI | ✅ |

### 代码质量工具

| 工具 | 命令 | 描述 | 自动安装 |
|------|----------|-------------|--------------|
| `pre-commit` | `pre-commit` | 预提交钩子框架 | ✅ |

### 其他工具

| 工具 | 命令 | 描述 | 自动安装 |
|------|----------|-------------|--------------|
| `vscode` | `code` | Visual Studio Code | ✅ |
| `rez` | `rez` | 包管理系统 | ✅ |
| `rcedit` | `rcedit` | Windows 资源编辑器 | ✅ |

## 检查可用工具

```bash
# 列出所有支持的工具
vx list

# 显示安装状态
vx list --status

# 显示特定工具的详情
vx list node
```

## 工具依赖

某些工具依赖于其他工具：

```
npm, npx → node
cargo, rustc, rustup → rust
uvx → uv
```

vx 会在需要时自动安装依赖。

## 版本支持

每个工具支持不同的版本说明符：

```bash
vx install node 20          # 主版本号
vx install node 20.10       # 次版本号
vx install node 20.10.0     # 精确版本
vx install node latest      # 最新稳定版
vx install node lts         # LTS 版本 (Node.js)
vx install rust stable      # 频道 (Rust)
```

## 添加新工具

vx 使用基于 provider 的插件系统来支持工具。请参阅 [Provider 开发](/zh/advanced/plugin-development) 了解如何添加新工具。
