# Supported Tools Overview

vx supports **50+ tools** out of the box, spanning language runtimes, package managers, DevOps tools, build systems, and more. All tools are managed through the same unified interface.

## At a Glance

| Category | Tools | Count |
|----------|-------|-------|
| [Language Runtimes](#language-runtimes) | Node.js, Python, Go, Rust, Deno, Zig, Java, .NET | 8 |
| [Package Managers](#package-managers) | npm, pnpm, yarn, bun, uv, pip, cargo, nuget | 8 |
| [DevOps](#devops) | Terraform, kubectl, Helm, Docker CLI, Git | 5 |
| [Cloud CLI](#cloud-cli) | AWS CLI, Azure CLI, Google Cloud CLI | 3 |
| [Build Tools](#build-tools) | CMake, Ninja, Just, Task, Make, Meson, protoc, MSBuild | 8 |
| [Code Quality](#code-quality) | pre-commit, Vite | 2 |
| [AI](#ai) | Ollama | 1 |
| [Scientific & HPC](#scientific--hpc) | Spack, Rez | 2 |
| [Media](#media) | FFmpeg, ImageMagick | 2 |
| [System Tools](#system-tools) | jq, gh, curl, pwsh, Git, NASM, OpenSSL, x-cmd | 8+ |
| [Windows-specific](#windows-specific) | choco, winget, rcedit, MSVC Build Tools | 4 |

## Language Runtimes

| Tool | Version Source | Platforms | Documentation |
|------|---------------|-----------|---------------|
| **Node.js** | nodejs.org API | All | [Details →](./nodejs) |
| **Python** | python-build-standalone | All | [Details →](./python) |
| **Go** | go.dev API | All | [Details →](./go) |
| **Rust** | static.rust-lang.org | All | [Details →](./rust) |
| **Deno** | GitHub Releases | All | [Details →](./other) |
| **Zig** | GitHub Releases | All | [Details →](./other) |
| **Java** | Adoptium API | All | [Details →](./other) |
| **.NET SDK** | dotnet API | All | [Details →](./build-tools) |

## Package Managers

| Tool | Ecosystem | Depends On | Documentation |
|------|-----------|------------|---------------|
| **npm** | Node.js | node | [Details →](./nodejs) |
| **npx** | Node.js | node | [Details →](./nodejs) |
| **pnpm** | Node.js | node | [Details →](./nodejs) |
| **yarn** | Node.js | node | [Details →](./nodejs) |
| **bun** | Node.js | — | [Details →](./nodejs) |
| **uv** | Python | — | [Details →](./python) |
| **uvx** | Python | uv | [Details →](./python) |
| **cargo** | Rust | rust | [Details →](./rust) |
| **nuget** | .NET | — | [Details →](./build-tools) |

## DevOps

| Tool | Description | Documentation |
|------|-------------|---------------|
| **Terraform** | Infrastructure as Code | [Details →](./devops) |
| **kubectl** | Kubernetes CLI | [Details →](./devops) |
| **Helm** | Kubernetes package manager | [Details →](./devops) |
| **Docker** | Container CLI (requires Docker Engine) | [Details →](./devops) |
| **Git** | Version control (MinGit on Windows) | [Details →](./devops) |
| **Dagu** | DAG-based workflow executor | — |

## Cloud CLI

| Tool | Cloud Provider | Documentation |
|------|---------------|---------------|
| **AWS CLI** | Amazon Web Services | [Details →](./cloud) |
| **Azure CLI** | Microsoft Azure | [Details →](./cloud) |
| **Google Cloud CLI** | Google Cloud Platform | [Details →](./cloud) |

## Build Tools

| Tool | Description | Documentation |
|------|-------------|---------------|
| **CMake** | Cross-platform build system generator | [Details →](./build-tools) |
| **Ninja** | Small, fast build system | [Details →](./build-tools) |
| **Just** | Command runner (modern Make) | [Details →](./build-tools) |
| **Task** | Task runner (go-task) | [Details →](./build-tools) |
| **Make** | GNU Make | [Details →](./build-tools) |
| **Meson** | Build system | [Details →](./build-tools) |
| **protoc** | Protocol Buffers compiler | [Details →](./build-tools) |
| **MSBuild** | Microsoft Build Engine | [Details →](./build-tools) |
| **MSVC Build Tools** | Microsoft C/C++ compiler toolchain | [Details →](./build-tools) |
| **Vite** | Frontend build tool | [Details →](./build-tools) |

## Code Quality

| Tool | Description | Documentation |
|------|-------------|---------------|
| **pre-commit** | Multi-language pre-commit hooks | [Details →](./quality) |

## AI

| Tool | Description | Documentation |
|------|-------------|---------------|
| **Ollama** | Run LLMs locally (Llama, Mistral, Gemma) | [Details →](./ai) |

## Scientific & HPC

| Tool | Description | Documentation |
|------|-------------|---------------|
| **Spack** | HPC package manager | [Details →](./scientific) |
| **Rez** | VFX/animation package manager | [Details →](./scientific) |

## Media

| Tool | Description | Documentation |
|------|-------------|---------------|
| **FFmpeg** | Audio/video processing | [Details →](./media) |
| **ImageMagick** | Image processing | [Details →](./media) |

## System Tools

| Tool | Description |
|------|-------------|
| **jq** | JSON processor |
| **gh** | GitHub CLI |
| **curl** | HTTP client |
| **pwsh** | PowerShell |
| **NASM** | Netwide Assembler |
| **OpenSSL** | Cryptography toolkit |
| **x-cmd** | Command-line toolbox with 100+ modules and AI |

## Windows-Specific

| Tool | Description |
|------|-------------|
| **choco** | Chocolatey package manager |
| **winget** | Windows Package Manager |
| **rcedit** | Windows resource editor |
| **MSVC Build Tools** | cl, link, lib, nmake, ml64, dumpbin, editbin |

## Usage Pattern

All tools follow the same pattern:

```bash
# Direct execution (auto-installs if needed)
vx <tool> [args...]

# Install specific version
vx install <tool>@<version>

# Version in vx.toml
[tools]
<tool> = "<version>"
```

## Custom Tools

You can add support for any tool through [Manifest-Driven Providers](/guide/manifest-driven-providers):

```toml
# ~/.vx/providers/mytool/provider.toml
[provider]
name = "mytool"
description = "My custom tool"

[[runtimes]]
name = "mytool"
executable = "mytool"

[runtimes.version_source]
type = "github_releases"
owner = "myorg"
repo = "mytool"
```

See [Provider Development](/advanced/plugin-development) for building Rust-based providers.
