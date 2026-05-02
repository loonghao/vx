# Supported Tools Overview

vx supports **132 tools** out of the box, spanning language runtimes, package managers, DevOps tools, build systems, code quality tools, and more. All tools are managed through the same unified interface.

## At a Glance

| Category | Tools | Count |
|----------|-------|-------|
| [Language Runtimes](#language-runtimes) | Node.js, Python, Go, Rust, Deno, Zig, Java, .NET | 8 |
| [Package Managers](#package-managers) | npm, pnpm, yarn, bun, uv, pip, cargo, nuget | 8 |
| [Build Tools](#build-tools) | CMake, Ninja, Just, Task, Make, Meson, protoc, MSBuild, Vite, xmake | 10 |
| [Build Cache](#build-cache-tools) | sccache, ccache, buildcache, Nx, Turborepo | 7 |
| [DevOps](#devops--kubernetes) | Terraform, kubectl, Helm, Podman, Flux, kustomize | 6+ |
| [Kubernetes Tools](#kubernetes-tools) | k9s, kind, k3d, minikube, nerdctl | 7 |
| [Cloud CLI](#cloud-cli) | AWS CLI, Azure CLI, Google Cloud CLI | 3 |
| [Code Quality](#code-quality) | pre-commit, ruff, ripgrep, fd, bat, biome, golangci-lint | 7+ |
| [Git Tools](#git-tools) | lazygit, jj, delta, gitleaks, lefthook | 7 |
| [AI/ML](#aiml-tools) | Ollama, usql | 2 |
| [Scientific & HPC](#scientific--hpc) | Spack, Rez | 2 |
| [Media](#media) | FFmpeg, ImageMagick | 2 |
| [CLI Enhancements](#cli--terminal-enhancements) | jq, fzf, eza, duf, dust, sd, zoxide | 7+ |
| [System Tools](#system--terminal-tools) | curl, pwsh, NASM, x-cmd, 7zip, htop (bottom) | 6+ |
| [Security](#security-tools) | cosign, grype, syft, trivy, git-leaks, age, sops | 7 |
| [Windows-specific](#windows-specific) | choco, winget, rcedit, MSVC, Wix, vcpkg | 6 |
| [Editors & TUI](#editors--tui-tools) | VS Code, Helix, Zellij, Yazi | 4 |

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
| **conda** | Python/Multi-language | — | — |
| **micromamba** | Conda-compatible | — | — |
| **mamba** | Conda-compatible | — | — |
| **conan** | C/C++ | — | — |
| **vcpkg** | C/C++ | — | — |
| **mise** | Multi-language | — | — |

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
| **Vite** | Frontend build tool | [Details →](./build-tools) |
| **xmake** | C/C++ build system | — |
| **Nx** | Monorepo build system (package alias) | [Details →](./build-cache) |
| **Turborepo** | Monorepo build cache (package alias) | [Details →](./build-cache) |

## Build Cache Tools

See [Build Cache Guide](./build-cache) for detailed documentation.

| Tool | Languages | Best For | Documentation |
|------|-----------|----------|---------------|
| **sccache** | Rust, C/C++, CUDA | Cross-language, CI/CD | [Details →](./build-cache) |
| **ccache** | C/C++ | Native C/C++ projects | [Details →](./build-cache) |
| **buildcache** | C/C++, CUDA | MSVC, Visual Studio | [Details →](./build-cache) |

## DevOps & Kubernetes

| Tool | Description | Documentation |
|------|-------------|---------------|
| **Terraform** | Infrastructure as Code | [Details →](./devops) |
| **kubectl** | Kubernetes CLI | [Details →](./devops) |
| **Helm** | Kubernetes package manager | [Details →](./devops) |
| **Podman** | Container CLI and compose | [Details →](./devops) |
| **Flux** | GitOps tool for Kubernetes | [Details →](./devops) |
| **kustomize** | Kubernetes configuration management | — |
| **Dagu** | DAG-based workflow executor | — |

## Kubernetes Tools

| Tool | Description |
|------|-------------|
| **k9s** | Terminal UI for Kubernetes |
| **kind** | Kubernetes in Docker |
| **k3d** | Lightweight Kubernetes (k3s in Docker) |
| **minikube** | Local Kubernetes cluster |
| **nerdctl** | Container runtime CLI (compatible with Docker CLI) |
| **skaffold** | Local Kubernetes development |

## Cloud CLI

| Tool | Cloud Provider | Documentation |
|------|---------------|---------------|
| **AWS CLI** | Amazon Web Services | [Details →](./cloud) |
| **Azure CLI** | Microsoft Azure | [Details →](./cloud) |
| **Google Cloud CLI** | Google Cloud Platform | [Details →](./cloud) |

## Code Quality

| Tool | Description | Documentation |
|------|-------------|---------------|
| **pre-commit** | Multi-language pre-commit hooks | [Details →](./quality) |
| **ruff** | Python linter and formatter | [Details →](./quality) |
| **ripgrep (rg)** | Fast grep alternative | [Details →](./quality) |
| **fd** | Fast find alternative | [Details →](./quality) |
| **bat** | Better cat with syntax highlighting | [Details →](./quality) |
| **biome** | JavaScript/TypeScript formatter/linter | [Details →](./quality) |
| **oxlint** | JavaScript/linter | [Details →](./quality) |
| **golangci-lint** | Go linter aggregator | [Details →](./quality) |
| **cargo-audit** | Rust security audit | [Details →](./quality) |
| **cargo-deny** | Rust dependency checker | [Details →](./quality) |
| **cargo-nextest** | Rust test runner | [Details →](./quality) |
| **hadolint** | Dockerfile linter | [Details →](./quality) |
| **actionlint** | GitHub Actions linter | [Details →](./quality) |

## Git Tools

| Tool | Description |
|------|-------------|
| **Git** | Version control (MinGit on Windows) |
| **gh** | GitHub CLI |
| **lazygit** | Terminal UI for Git |
| **jj** (jujutsu) | Version control system (Git-compatible) |
| **delta** | Syntax-highlighted git diff |
| **gitleaks** | Git secret/history scanner |
| **lefthook** | Git hooks manager |

## AI/ML Tools

| Tool | Description | Documentation |
|------|-------------|---------------|
| **Ollama** | Run LLMs locally (Llama, Mistral, Gemma) | [Details →](./ai) |
| **usql** | Universal SQL CLI (AI-enhanced) | — |

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
| **dive** | Docker image analysis | — |

## CLI & Terminal Enhancements

| Tool | Description |
|------|-------------|
| **jq** | JSON processor |
| **fzf** | Fuzzy finder |
| **eza** | Modern ls replacement |
| **duf** | Disk usage utility |
| **dust** | Disk usage analyzer |
| **sd** | Better sed (simple find & replace) |
| **zoxide** | Smart cd replacement |
| **atuin** | Shell history sync |
| **starship** | Cross-shell prompt |
| **tealdeer** (tldr) | Simplified man pages |
| **xh** | HTTP client (like curl, but better) |
| **hyperfine** | Command-line benchmarking |
| **gping** | Graphical ping |
| **trippy** | Network diagnostics (traceroute + ping) |

## System & Terminal Tools

| Tool | Description |
|------|-------------|
| **curl** | HTTP client |
| **pwsh** | PowerShell |
| **NASM** | Netwide Assembler |
| **x-cmd** | Command-line toolbox with 100+ modules and AI |
| **7zip** | File archiver |
| **bottom** | System monitor (htop alternative) |
| **watchexec** | File watcher and command runner |
| **systemctl** | Systemd service manager (Linux) |

## Security Tools

| Tool | Description |
|------|-------------|
| **cosign** | Container image signing/verification |
| **grype** | Container vulnerability scanner |
| **syft** | SBOM (Software Bill of Materials) generator |
| **trivy** | Vulnerability scanner for containers/Git repos |
| **gitleaks** | Git secret/history scanner |
| **age** | Simple, modern file encryption (YubiKey support) |
| **sops** | Secrets operations (encrypted files with multi-key support) |

## Windows-Specific

| Tool | Description |
|------|-------------|
| **choco** | Chocolatey package manager |
| **winget** | Windows Package Manager |
| **rcedit** | Windows resource editor |
| **MSVC Build Tools** | cl, link, lib, nmake, ml64, dumpbin, editbin |
| **Wix** | Windows Installer (MSI) creation |
| **vcpkg** | C/C++ package manager |

## Editors & TUI Tools

| Tool | Description |
|------|-------------|
| **VS Code** (code) | Visual Studio Code editor |
| **Helix** | Modal text editor |
| **Zellij** | Terminal multiplexer |
| **Yazi** | Terminal file manager |

## CI/CD Tools

| Tool | Description |
|------|-------------|
| **prek** | Pre-commit hooks manager |
| **release-please** | Release automation (conventional commits) |
| **goreleaser** | Go release automation |
| **buf** | Protocol Buffers tools |
| **ctlptl** | Local Kubernetes for CI |

## Other Tools

| Tool | Description |
|------|-------------|
| **awscli** | AWS CLI |
| **azcli** | Azure CLI |
| **gcloud** | Google Cloud CLI |
| **llvm** | Compiler toolchain |
| **maturin** | Python/Rust package builder |
| **openclaw** | Open source CLAW |
| **wix** | Windows Installer XML (WiX) |
| **xcodebuild** | Xcode build tool (macOS) |
| **yq** | YAML/XML/JSON processor |

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

## Complete Tool List (132 Total)

> **Note**: For detailed documentation, click the links above. For undocumented tools, please refer to the tool's official documentation.

All 132 tools are immediately available with `vx <tool>`. No manual installation required — vx handles everything automatically.

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
