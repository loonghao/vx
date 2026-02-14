# /project-analyze

Analyze a real-world project to test vx-project-analyzer.

## Usage

```
/project-analyze <project-name-or-url>
```

## Examples

```
/project-analyze codex
/project-analyze kubectl
/project-analyze https://github.com/denoland/deno
```

## Workflow

This command triggers the `project-analyze` skill which:

1. Resolves the project name to a GitHub URL
2. Clones the project to a temporary location
3. Runs vx-project-analyzer on the project
4. Reports analysis results
5. If issues are found, fixes the analyzer code
6. Cleans up the temporary project

## Known Projects

- `codex` - OpenAI Codex (Rust + Node.js monorepo)
- `kubectl` - Kubernetes kubectl (Go)
- `deno` - Deno runtime (Rust workspace)
- `ripgrep` - ripgrep search tool (Rust)
- `uv` - Astral uv (Rust + Python)
- `nextjs` - Next.js framework (Node.js monorepo)
- `vite` - Vite build tool (Node.js)
- `ruff` - Ruff linter (Rust + Python)
- `httpx` - HTTPX library (Python)
- `auroraview` - AuroraView (Rust + Python + Node.js)
