# Supported Tools Overview

vx supports **129 tools** out of the box, spanning language runtimes, package managers, DevOps tools, build systems, and more. All tools are managed through the same unified interface.

## At a Glance

| Category | Count | Example Tools |
|----------|-------|---------------|
| [Language Runtimes](#language-runtimes) | 9 | node, python, go, rust, bun, deno, java, zig, dotnet |
| [Package Managers](#package-managers) | 4 | uv, pnpm, yarn, nuget |
| [Build Tools](#build-tools) | 20 | cmake, ninja, just, task, meson, xmake, msbuild, conan, vcpkg, vite, nx, turbo |
| [Compiler Caches](#compiler-caches--speedup) | 3 | ccache, sccache, buildcache |
| [DevOps & Kubernetes](#devops--kubernetes) | 23 | terraform, kubectl, helm, k9s, kind, minikube, tilt, kustomize, flux |
| [Cloud CLI](#cloud-cli-tools) | 3 | awscli, azcli, gcloud |
| [Security & Signing](#security--signing) | 5 | cosign, trivy, grype, syft, gitleaks |
| [Code Quality](#code-quality--linting) | 8 | ruff, biome, oxlint, hadolint, pre-commit, prek |
| [Terminal Utilities](#terminal-utilities--cli-tools) | 25 | ripgrep, fd, bat, eza, fzf, zoxide, lazygit, jq, yq |
| [Git Tools](#version-control--git-tools) | 7 | git, gh, lazygit, jj, gws |
| [AI & ML](#ai--ml-tools) | 1 | ollama |
| [Database](#database-tools) | 2 | duckdb, usql |
| [Media](#media--creative) | 2 | ffmpeg, imagemagick |
| [Scientific & HPC](#scientific--hpc) | 2 | spack, rez |
| [System Tools](#system--platform-tools) | 9 | curl, bash, pwsh, 7zip, vscode, llvm |
| [Windows-specific](#windows-specific) | 5 | choco, winget, rcedit, wix, msvc |

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
| **Podman** | Container CLI and compose workflow | [Details →](./devops) |
| **nerdctl** | Docker-compatible CLI for containerd | [Details →](./devops) |
| **Git** | Version control (MinGit on Windows) | [Details →](./devops) |
| **Dagu** | DAG-based workflow executor | — |
| **skaffold** | Local Kubernetes Development | — |
| **k9s** | Terminal UI for Kubernetes clusters | — |
| **k3d** | Run k3s Kubernetes in Docker | — |
| **kind** | Kubernetes IN Docker | — |
| **minikube** | Run a local Kubernetes cluster | — |
| **ctlptl** | Making local Kubernetes clusters easy to set up | — |
| **tilt** | Toolkit for fixing microservice development pains | — |
| **kustomize** | Customization of Kubernetes YAML configurations | — |
| **flux** | GitOps for Kubernetes | — |
| **goreleaser** | Release engineering for Go projects | — |
| **golangci-lint** | Fast linters runner for Go | — |
| **cosign** | Container signing and verification (Sigstore) | — |
| **trivy** | Comprehensive security scanner | — |
| **dive** | Tool for exploring Docker image layers | — |
| **actionlint** | Static checker for GitHub Actions workflow files | — |
| **actrun** | Actionforge workflow runner CLI | — |

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
| **xmake** | Cross-platform build utility based on Lua | [Details →](./build-tools) |
| **protoc** | Protocol Buffers compiler | [Details →](./build-tools) |
| **buf** | The best way to work with Protocol Buffers | [Details →](./build-tools) |
| **MSBuild** | Microsoft Build Engine | [Details →](./build-tools) |
| **MSVC Build Tools** | Microsoft C/C++ compiler toolchain | [Details →](./build-tools) |
| **Vite** | Frontend build tool | [Details →](./build-tools) |
| **Nx** | Smart Monorepos · Fast CI | [Details →](./build-tools) |
| **Turborepo** | High-performance Build System for JS/TS | [Details →](./build-tools) |
| **Conan** | The C/C++ Package Manager | [Details →](./build-tools) |
| **vcpkg** | C++ library manager | [Details →](./build-tools) |
| **WiX** | Build Windows installation packages | [Details →](./build-tools) |
| **maturin** | Build and publish Rust-based Python packages | [Details →](./build-tools) |

## Compiler Caches & Speedup

| Tool | Description |
|------|-------------|
| **ccache** | C/C++ Compiler Cache for faster recompilation |
| **sccache** | Shared Compilation Cache (supports Rust, C/C++) |
| **buildcache** | Compiler Cache with excellent MSVC support |

## Security & Signing

| Tool | Description |
|------|-------------|
| **cosign** | Container signing and verification (Sigstore) |
| **trivy** | Comprehensive security scanner for containers and code |
| **grype** | Vulnerability scanner for container images and filesystems |
| **syft** | CLI tool for generating a Software Bill of Materials (SBOM) |
| **gitleaks** | Secrets scanner for git repos and files |
| **cargo-audit** | Audit Cargo.lock for security vulnerabilities |
| **cargo-deny** | Cargo plugin for linting dependencies |

## Code Quality & Linting

| Tool | Description | Documentation |
|------|-------------|---------------|
| **ruff** | An extremely fast Python linter and formatter | [Details →](./quality) |
| **biome** | Fast formatter and linter for JS/TS/JSON/CSS | [Details →](./quality) |
| **oxlint** | High-performance linting tools for JavaScript/TypeScript | [Details →](./quality) |
| **hadolint** | Dockerfile linter | [Details →](./quality) |
| **pre-commit** | Multi-language pre-commit hooks framework | [Details →](./quality) |
| **prek** | Pre-commit hook runner (vx's built-in hook system) | [Details →](./quality) |
| **cargo-nextest** | A next-generation test runner for Rust | [Details →](./quality) |
| **actionlint** | Static checker for GitHub Actions workflow files | [Details →](./quality) |

## Terminal Utilities & CLI Tools

| Tool | Commands | Description |
|------|----------|-------------|
| **ripgrep** | `rg` | Recursively searches directories for a regex pattern (extremely fast) |
| **fd** | `fd` | A simple, fast alternative to 'find' |
| **bat** | `bat` | A cat clone with syntax highlighting and Git integration |
| **eza** | `eza` | A modern, maintained replacement for ls |
| **delta** | `delta` | Syntax-highlighting pager for git, diff, grep |
| **sd** | `sd` | Intuitive find & replace CLI (sed alternative) |
| **fzf** | `fzf` | A command-line fuzzy finder |
| **zoxide** | `zoxide` | A smarter cd command that learns your habits |
| **atuin** | `atuin` | Magical shell history with SQLite and encrypted sync |
| **yazi** | `yazi` | A blazing fast terminal file manager |
| **zellij** | `zellij` | A modern terminal workspace (terminal multiplexer) |
| **helix** | `hx` | A post-modern modal text editor |
| **tealdeer** | `tldr` | A very fast Rust implementation of tldr |
| **tokei** | `tokei` | Count your code, quickly |
| **hyperfine** | `hyperfine` | A command-line benchmarking tool |
| **watchexec** | `watchexec` | Execute commands when watched files change |
| **bottom** | `btm` | Cross-platform graphical process/system monitor |
| **duf** | `duf` | Disk usage utility with a user-friendly interface |
| **dust** | `dust` | A more intuitive version of du written in Rust |
| **gping** | `gping` | Ping, but with a graph |
| **xh** | `xh` | A friendly and fast HTTP request tool |
| **grpcurl** | `grpcurl` | Like curl, but for gRPC |
| **trippy** | `trip` | Network path tracer combining traceroute and ping |
| **jq** | `jq` | Lightweight and flexible command-line JSON processor |
| **yq** | `yq` | A portable command-line YAML/JSON/XML/CSV/TOML processor |

## Version Control & Git Tools

| Tool | Description |
|------|-------------|
| **git** | Git version control (MinGit on Windows) |
| **gh** | GitHub CLI — GitHub's official command line tool |
| **lazygit** | Simple terminal UI for git commands |
| **jj** | A Git-compatible DVCS that is both simple and powerful |
| **gws** | Manage multiple git repositories |
| **lazydocker** | Simple terminal UI for both docker and docker-compose |

## Project & Environment Management

| Tool | Description |
|------|-------------|
| **mise** | Dev tools, env vars, task runner (asdf/direnv replacement) |
| **chezmoi** | Manage your dotfiles across multiple machines |
| **lefthook** | Fast and powerful Git hooks manager |

## AI & ML Tools

| Tool | Description | Documentation |
|------|-------------|---------------|
| **Ollama** | Run LLMs locally (Llama, Mistral, Gemma) | [Details →](./ai) |

## Database Tools

| Tool | Description |
|------|-------------|
| **duckdb** | DuckDB — In-process SQL OLAP database management system |
| **usql** | Universal command-line interface for SQL databases |

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

## System & Platform Tools

| Tool | Description |
|------|-------------|
| **curl** | Command-line tool for transferring data with URLs |
| **bash** | GNU Bourne Again SHell |
| **pwsh** | PowerShell — Cross-platform task automation |
| **7zip** | 7-Zip — High compression ratio file archiver |
| **nasm** | NASM — Netwide Assembler |
| **vscode** | Visual Studio Code (code editor) |
| **llvm** | LLVM compiler infrastructure |
| **systemctl** | systemd system and service manager (Linux) |
| **openclaw** | OpenClaw — Open-source 2D platformer game engine |

## Windows-Specific

| Tool | Description |
|------|-------------|
| **choco** | Chocolatey package manager |
| **winget** | Windows Package Manager |
| **rcedit** | Windows resource editor for PE files |
| **wix** | WiX Toolset — Build Windows installation packages |
| **msvc** | MSVC Build Tools — cl, link, lib, nmake, ml64, dumpbin |

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

You can add support for any tool through [Provider Development Guide](/guide/provider-star-reference):

```starlark
# ~/.vx/providers/mytool/provider.star
load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:provider_templates.star", "github_rust_provider")

name        = "mytool"
description = "My custom tool"
ecosystem   = "custom"

runtimes    = [runtime_def("mytool")]
permissions = github_permissions()

_p = github_rust_provider("myorg", "mytool",
    asset = "mytool-{vversion}-{triple}.{ext}")

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]
```

See [Provider Star Reference](/guide/provider-star-reference) for the full Starlark DSL documentation.
