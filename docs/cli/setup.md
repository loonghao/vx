# setup

Install all project tools from `.vx.toml`.

## Synopsis

```bash
vx setup [OPTIONS]
```

## Description

The `vx setup` command is the **first command to run** when joining a project or after cloning a repository. It reads the project's `.vx.toml` configuration and:

1. Checks which tools are already installed
2. Installs all missing tools to `~/.vx/store/`
3. Reports installation status

This ensures all team members have the exact same tool versions.

## Options

| Option | Description |
|--------|-------------|
| `-f`, `--force` | Force reinstall all tools |
| `--dry-run` | Preview operations without executing |
| `-v`, `--verbose` | Show verbose output |
| `--no-parallel` | Disable parallel installation |

## Usage Scenarios

### Scenario 1: Initial Project Setup

When you clone a project with `.vx.toml`:

```bash
git clone https://github.com/example/project.git
cd project
vx setup
```

Output:

```
🚀 VX Development Environment Setup

Checking tool status...

Tools:
  ✓ node@20.10.0 (installed)
  ✗ uv@0.5.14 (missing)
  ✗ go@1.21.5 (missing)

Installing 2 tool(s)...
  ✓ uv@0.5.14
  ✓ go@1.21.5

✓ Successfully installed 2 tool(s) in 12.3s

Next steps:
  1. Enter dev environment: vx dev
  2. Or run tools directly: vx <tool> [args]

Available scripts:
  vx run dev -> npm run dev
  vx run test -> pytest
```

### Scenario 2: CI/CD Pipeline

In CI/CD, run `vx setup` to ensure tools are available:

```yaml
# GitHub Actions
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install vx
        run: curl -fsSL https://get.vx.dev | bash

      - name: Setup project tools
        run: vx setup

      - name: Build
        run: vx dev -c "npm run build"
```

### Scenario 3: Force Reinstall

If you suspect tool corruption or want a clean slate:

```bash
vx setup --force
```

This reinstalls all tools even if they appear to be installed.

### Scenario 4: Preview Changes

See what would be installed without actually installing:

```bash
vx setup --dry-run
```

Output:

```
🚀 VX Development Environment Setup

Tools:
  ✓ node@20.10.0 (installed)
  ✗ uv@0.5.14 (missing)
  ✗ go@1.21.5 (missing)

Would install 2 tool(s):
  - uv@0.5.14
  - go@1.21.5
```

## Configuration

Setup reads from `.vx.toml`:

```toml
[tools]
node = "20"
uv = "latest"
go = "1.21"

[settings]
auto_install = true    # Auto-install missing tools in vx dev
parallel_install = true # Install tools in parallel

[scripts]
dev = "npm run dev"
test = "pytest"
build = "npm run build"
```

## Tool Storage

All tools are installed to the global store at `~/.vx/store/`:

```
~/.vx/
├── store/
│   ├── node/
│   │   ├── 20.10.0/
│   │   └── 18.19.0/
│   ├── uv/
│   │   └── 0.5.14/
│   └── go/
│       └── 1.21.5/
└── bin/
```

This content-addressable storage allows:

- Multiple projects to share the same tool versions
- No disk space wasted on duplicates
- Fast switching between versions

## Workflow: setup vs dev

```
┌─────────────────────────────────────────────────────────────────┐
│                        Project Workflow                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   1. Clone project                                               │
│      git clone https://github.com/example/project.git            │
│                                                                  │
│   2. Install tools (one-time)                                    │
│      vx setup                                                    │
│                                                                  │
│   3. Enter dev environment (daily)                               │
│      vx dev                                                      │
│      # or                                                        │
│      eval "$(vx dev --export)"                                   │
│                                                                  │
│   4. Run project scripts                                         │
│      vx run dev                                                  │
│      vx run test                                                 │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

| Command | Purpose | When to Use |
|---------|---------|-------------|
| `vx setup` | Install tools | First time, after `.vx.toml` changes |
| `vx dev` | Enter environment | Daily development |
| `vx dev --export` | Activate in current shell | IDE integration, scripts |
| `vx run <script>` | Run defined scripts | Build, test, deploy |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success - all tools installed |
| 1 | Error - some tools failed to install |

## Tips

1. **Run after git pull**: If `.vx.toml` might have changed:

   ```bash
   git pull && vx setup
   ```

2. **Use in git hooks**: Auto-setup on checkout:

   ```bash
   # .git/hooks/post-checkout
   #!/bin/sh
   vx setup --dry-run | grep -q "missing" && vx setup
   ```

3. **Verbose mode for debugging**:

   ```bash
   vx setup --verbose
   ```

4. **Parallel is faster**: By default, tools install in parallel. Use `--no-parallel` only if you encounter issues.

## See Also

- [init](../cli/commands#init) - Initialize project configuration
- [dev](dev) - Enter development environment
- [sync](../cli/commands#sync) - Sync tools with configuration
