# Contributing to vx

Thank you for your interest in contributing to vx! 🎉

## Quick Start

```bash
# Clone and build
git clone https://github.com/loonghao/vx.git
cd vx
vx cargo build

# Run tests
vx just test

# Quick pre-commit cycle (format → lint → test → build)
vx just quick
```

## Prerequisites

- **Rust 1.93+** (managed by vx itself: `vx cargo --version`)
- **Git**
- **just** (task runner, managed by vx: `vx just --list`)

## Development Workflow

1. **Fork and clone** the repository
2. **Create a branch**: `vx git checkout -b feature/my-feature`
3. **Set up pre-commit hooks**: `vx prek install`
4. **Make changes** and add tests
5. **Run quality checks**: `vx just quick`
6. **Submit a pull request**

## Key Rules

- **Tests go in `tests/` directories** — Never write inline `#[cfg(test)]` modules
- **Use `rstest`** for parameterized tests
- **Use correct terminology**: Runtime (not Tool), Provider (not Plugin)
- **New providers use Starlark DSL only** — Create `crates/vx-providers/<name>/provider.star`
- **Layer dependencies go downward only** — See [architecture overview](docs/architecture/OVERVIEW.md)

## Adding a New Provider

Most new tools only need a `provider.star` file (no Rust code):

```starlark
# crates/vx-providers/mytool/provider.star
load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:provider_templates.star", "github_rust_provider")

name        = "mytool"
description = "My awesome tool"
ecosystem   = "custom"
runtimes    = [runtime_def("mytool")]
permissions = github_permissions()

_p = github_rust_provider("owner", "repo",
    asset = "mytool-{vversion}-{triple}.{ext}")
fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]
```

Test it: `vx mytool --version`

## Detailed Guide

For the complete contributing guide including CI pipeline details, dependency constraints, and code style guidelines, see:

📖 **[Full Contributing Guide](docs/advanced/contributing.md)**

## Community

- [GitHub Issues](https://github.com/loonghao/vx/issues) — Bug reports
- [GitHub Discussions](https://github.com/loonghao/vx/discussions) — Feature requests and questions
- Contact: <hal.long@outlook.com>

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
