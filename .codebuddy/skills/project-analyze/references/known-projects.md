# Known Test Projects

A curated list of open source projects useful for testing vx-project-analyzer.

## Multi-Language Projects

### codex (OpenAI)
- **URL**: https://github.com/openai/codex
- **Languages**: Rust + Node.js (pnpm workspace)
- **Structure**: Monorepo with `codex-rs/` subdirectory
- **Tests**: Monorepo detection, justfile parsing, multi-ecosystem
- **Notes**: Rust code is in `codex-rs/` subdirectory, not root

### auroraview
- **URL**: https://github.com/loonghao/auroraview
- **Languages**: Rust + Python + Node.js
- **Tests**: Multi-ecosystem detection, workspace dependencies

## Rust Projects

### deno (Denoland)
- **URL**: https://github.com/denoland/deno
- **Languages**: Rust (large workspace)
- **Structure**: Workspace with 50+ member crates
- **Tests**: Large workspace dependency parsing, monorepo support
- **Notes**: Dependencies in workspace members, not root Cargo.toml

### ripgrep (BurntSushi)
- **URL**: https://github.com/BurntSushi/ripgrep
- **Languages**: Rust
- **Tests**: Standard Rust project analysis

### uv (Astral)
- **URL**: https://github.com/astral-sh/uv
- **Languages**: Rust + Python bindings
- **Tests**: Rust workspace, justfile parsing

## Go Projects

### kubectl (Kubernetes)
- **URL**: https://github.com/kubernetes/kubectl
- **Languages**: Go
- **Tests**: Go module parsing, large dependency count

### docker-cli
- **URL**: https://github.com/docker/cli
- **Languages**: Go
- **Tests**: Go project with Makefile

## Node.js Projects

### next.js (Vercel)
- **URL**: https://github.com/vercel/next.js
- **Languages**: Node.js + Rust (turbopack)
- **Structure**: Large pnpm monorepo
- **Tests**: Package.json scripts, internal script reference filtering
- **Notes**: Has many npm scripts that should NOT be detected as external tools

### vite (Vitejs)
- **URL**: https://github.com/vitejs/vite
- **Languages**: Node.js
- **Tests**: pnpm workspace, TypeScript project

## Python Projects

### ruff (Astral)
- **URL**: https://github.com/astral-sh/ruff
- **Languages**: Rust + Python
- **Tests**: pyproject.toml parsing, Rust workspace

### httpx (Encode)
- **URL**: https://github.com/encode/httpx
- **Languages**: Python
- **Tests**: Standard Python project, pyproject.toml

### rez (AcademySoftwareFoundation)
- **URL**: https://github.com/AcademySoftwareFoundation/rez
- **Languages**: Python
- **Structure**: setup.py based, zero runtime dependencies
- **Tests**: setup.py detection, tox.ini, zero-dependency project
- **Notes**: Uses setup.py with `install_requires=[]`, docs dependencies in `docs/requirements.txt`

## C++ Projects

### mrv2 (ggarra13)
- **URL**: https://github.com/ggarra13/mrv2
- **Languages**: C++ (CMake)
- **Structure**: Large CMake project with 88+ CMakeLists.txt files
- **Tests**: CMake dependency parsing, recursive CMakeLists.txt scanning
- **Notes**: Uses find_package extensively, has Python scripts but not a Python project

## Project Selection Guidelines

When choosing a test project, consider:

1. **Diversity** - Test different languages and project structures
2. **Complexity** - Include both simple and complex projects
3. **Popularity** - Well-maintained projects have standard structures
4. **Size** - Mix of small and large projects for performance testing

## Adding New Projects

When adding a new test project:

1. Clone and analyze the project
2. Document any issues discovered
3. Add to this list with relevant metadata
4. Include notes about what the project tests
