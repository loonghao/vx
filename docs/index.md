---
layout: home

hero:
  name: vx
  text: Universal Development Tool Manager
  tagline: One command to rule them all - Zero setup, Zero learning curve
  image:
    src: /logo.svg
    alt: vx
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: View on GitHub
      link: https://github.com/loonghao/vx

features:
  - icon: "\U0001F680"
    title: Zero Configuration
    details: Works out of the box, no setup required. Just prefix your commands with vx.
  - icon: "\U0001F527"
    title: Auto-Installation
    details: Tools are installed automatically on first use. No manual installation needed.
  - icon: "\U0001F4E6"
    title: Version Management
    details: Pin specific versions per project with vx.toml configuration.
  - icon: "\U0001F310"
    title: Cross-Platform
    details: Works seamlessly on Windows, macOS, and Linux.
  - icon: "\u26A1"
    title: Blazing Fast
    details: Written in Rust for maximum performance and minimal overhead.
  - icon: "\U0001F529"
    title: Extensible
    details: Plugin system for adding custom tools and workflows.
---

## The Problem We Solve

Every time we start a new development project, we face the same frustrating cycle:

- Install Node.js and npm for frontend tools
- Set up Python and pip/uv for scripts and automation
- Configure Go for backend services
- Manage Rust toolchain for system tools
- Deal with version conflicts and PATH issues

**With the rise of MCP (Model Context Protocol)**, this problem has become even more pronounced. Many MCP servers require `uvx` for Python tools and `npx` for Node.js packages.

## Our Solution

```bash
# Instead of learning and managing multiple tools:
npx create-react-app my-app     # Requires Node.js setup
uvx ruff check .                # Requires Python/UV setup
go run main.go                  # Requires Go installation

# Just use vx with the same commands you already know:
vx npx create-react-app my-app  # Auto-installs Node.js if needed
vx uvx ruff check .             # Auto-installs UV if needed
vx go run main.go               # Auto-installs Go if needed
```

## Quick Install

::: code-group

```bash [Linux/macOS]
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
```

```powershell [Windows]
irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex
```

:::
