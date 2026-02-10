# 支持的工具概览

vx 开箱即支持 **50+ 个工具**，涵盖语言运行时、包管理器、DevOps 工具、构建系统等。所有工具通过相同的统一接口管理。

## 一览

| 分类 | 工具 | 数量 |
|------|------|------|
| [语言运行时](#语言运行时) | Node.js, Python, Go, Rust, Deno, Zig, Java, .NET | 8 |
| [包管理器](#包管理器) | npm, pnpm, yarn, bun, uv, pip, cargo, nuget | 8 |
| [DevOps](#devops) | Terraform, kubectl, Helm, Docker CLI, Git | 5 |
| [云 CLI](#云-cli) | AWS CLI, Azure CLI, Google Cloud CLI | 3 |
| [构建工具](#构建工具) | CMake, Ninja, Just, Task, Make, Meson, protoc, MSBuild | 8 |
| [代码质量](#代码质量) | pre-commit, Vite | 2 |
| [AI](#ai) | Ollama | 1 |
| [科学计算 & HPC](#科学计算--hpc) | Spack, Rez | 2 |
| [媒体](#媒体) | FFmpeg, ImageMagick | 2 |
| [系统工具](#系统工具) | jq, gh, curl, pwsh, Git, NASM, OpenSSL, x-cmd | 8+ |
| [Windows 专属](#windows-专属) | choco, winget, rcedit, MSVC Build Tools | 4 |

## 语言运行时

| 工具 | 版本来源 | 平台 | 文档 |
|------|---------|------|------|
| **Node.js** | nodejs.org API | 全平台 | [详情 →](./nodejs) |
| **Python** | python-build-standalone | 全平台 | [详情 →](./python) |
| **Go** | go.dev API | 全平台 | [详情 →](./go) |
| **Rust** | static.rust-lang.org | 全平台 | [详情 →](./rust) |
| **Deno** | GitHub Releases | 全平台 | [详情 →](./other) |
| **Zig** | GitHub Releases | 全平台 | [详情 →](./other) |
| **Java** | Adoptium API | 全平台 | [详情 →](./other) |
| **.NET SDK** | dotnet API | 全平台 | [详情 →](./build-tools) |

## 包管理器

| 工具 | 生态系统 | 依赖 | 文档 |
|------|---------|------|------|
| **npm** | Node.js | node | [详情 →](./nodejs) |
| **npx** | Node.js | node | [详情 →](./nodejs) |
| **pnpm** | Node.js | node | [详情 →](./nodejs) |
| **yarn** | Node.js | node | [详情 →](./nodejs) |
| **bun** | Node.js | — | [详情 →](./nodejs) |
| **uv** | Python | — | [详情 →](./python) |
| **uvx** | Python | uv | [详情 →](./python) |
| **cargo** | Rust | rust | [详情 →](./rust) |
| **nuget** | .NET | — | [详情 →](./build-tools) |

## DevOps

| 工具 | 描述 | 文档 |
|------|------|------|
| **Terraform** | 基础设施即代码 | [详情 →](./devops) |
| **kubectl** | Kubernetes CLI | [详情 →](./devops) |
| **Helm** | Kubernetes 包管理器 | [详情 →](./devops) |
| **Docker** | 容器 CLI（需要 Docker Engine） | [详情 →](./devops) |
| **Git** | 版本控制（Windows 使用 MinGit） | [详情 →](./devops) |
| **Dagu** | 基于 DAG 的工作流执行器 | — |

## 云 CLI

| 工具 | 云提供商 | 文档 |
|------|---------|------|
| **AWS CLI** | Amazon Web Services | [详情 →](./cloud) |
| **Azure CLI** | Microsoft Azure | [详情 →](./cloud) |
| **Google Cloud CLI** | Google Cloud Platform | [详情 →](./cloud) |

## 构建工具

| 工具 | 描述 | 文档 |
|------|------|------|
| **CMake** | 跨平台构建系统生成器 | [详情 →](./build-tools) |
| **Ninja** | 小而快的构建系统 | [详情 →](./build-tools) |
| **Just** | 命令运行器（现代 Make） | [详情 →](./build-tools) |
| **Task** | 任务运行器（go-task） | [详情 →](./build-tools) |
| **Make** | GNU Make | [详情 →](./build-tools) |
| **Meson** | 构建系统 | [详情 →](./build-tools) |
| **protoc** | Protocol Buffers 编译器 | [详情 →](./build-tools) |
| **MSBuild** | Microsoft 构建引擎 | [详情 →](./build-tools) |
| **MSVC Build Tools** | Microsoft C/C++ 编译器工具链 | [详情 →](./build-tools) |
| **Vite** | 前端构建工具 | [详情 →](./build-tools) |

## 代码质量

| 工具 | 描述 | 文档 |
|------|------|------|
| **pre-commit** | 多语言预提交钩子 | [详情 →](./quality) |

## AI

| 工具 | 描述 | 文档 |
|------|------|------|
| **Ollama** | 本地运行 LLM（Llama、Mistral、Gemma） | [详情 →](./ai) |

## 科学计算 & HPC

| 工具 | 描述 | 文档 |
|------|------|------|
| **Spack** | HPC 包管理器 | [详情 →](./scientific) |
| **Rez** | VFX/动画包管理器 | [详情 →](./scientific) |

## 媒体

| 工具 | 描述 | 文档 |
|------|------|------|
| **FFmpeg** | 音视频处理 | [详情 →](./media) |
| **ImageMagick** | 图像处理 | [详情 →](./media) |

## 系统工具

| 工具 | 描述 |
|------|------|
| **jq** | JSON 处理器 |
| **gh** | GitHub CLI |
| **curl** | HTTP 客户端 |
| **pwsh** | PowerShell |
| **NASM** | Netwide 汇编器 |
| **OpenSSL** | 密码学工具包 |
| **x-cmd** | 命令行工具箱，100+ 模块，集成 AI |

## Windows 专属

| 工具 | 描述 |
|------|------|
| **choco** | Chocolatey 包管理器 |
| **winget** | Windows 包管理器 |
| **rcedit** | Windows 资源编辑器 |
| **MSVC Build Tools** | cl、link、lib、nmake、ml64、dumpbin、editbin |

## 使用模式

所有工具遵循相同模式：

```bash
# 直接执行（如需自动安装）
vx <tool> [args...]

# 安装指定版本
vx install <tool>@<version>

# 在 vx.toml 中指定版本
[tools]
<tool> = "<version>"
```

## 自定义工具

你可以通过[声明式 Provider](/zh/guide/manifest-driven-providers) 添加任何工具的支持：

```toml
# ~/.vx/providers/mytool/provider.toml
[provider]
name = "mytool"
description = "我的自定义工具"

[[runtimes]]
name = "mytool"
executable = "mytool"

[runtimes.version_source]
type = "github_releases"
owner = "myorg"
repo = "mytool"
```

参见 [Provider 开发](/zh/advanced/plugin-development) 了解如何构建基于 Rust 的 Provider。
