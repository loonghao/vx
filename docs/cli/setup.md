# setup

Install all project tools and run setup hooks from `.vx.toml`.

## Synopsis

```bash
vx setup [OPTIONS]
```

## Description

The `vx setup` command is the **first command to run** when joining a project or after cloning a repository. It reads the project's `.vx.toml` configuration and:

1. Runs `pre_setup` hooks (if defined)
2. Checks which tools are already installed
3. Installs all missing tools to `~/.vx/store/`
4. Runs `post_setup` hooks (if defined)
5. Reports installation status

This ensures all team members have the exact same tool versions.

## Options

| Option | Description |
|--------|-------------|
| `-f`, `--force` | Force reinstall all tools |
| `--dry-run` | Preview operations without executing |
| `-v`, `--verbose` | Show verbose output |
| `--no-parallel` | Disable parallel installation |
| `--no-hooks` | Skip pre/post setup hooks |

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

Running pre-setup hooks...
  ✓ echo 'Starting setup...'

Checking tool status...

Tools:
  ✓ node@20.10.0 (installed)
  ✗ uv@0.5.14 (missing)
  ✗ go@1.21.5 (missing)

Installing 2 tool(s)...
  ✓ uv@0.5.14
  ✓ go@1.21.5

Running post-setup hooks...
  ✓ vx run db:migrate
  ✓ vx run seed

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
        run: vx setup --no-hooks  # Skip hooks in CI if needed

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

Would run post-setup hooks:
  - vx run db:migrate
  - vx run seed
```

### Scenario 5: Skip Hooks

If you only want to install tools without running hooks:

```bash
vx setup --no-hooks
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

[hooks]
pre_setup = "echo 'Preparing environment...'"
post_setup = ["vx run db:migrate", "vx run seed"]

[scripts]
dev = "npm run dev"
test = "pytest"
build = "npm run build"
db:migrate = "prisma migrate dev"
seed = "prisma db seed"
```

### Hooks Configuration

Setup supports lifecycle hooks:

| Hook | When it runs |
|------|--------------|
| `pre_setup` | Before installing tools |
| `post_setup` | After all tools are installed |

Hooks can be a single command or an array:

```toml
[hooks]
# Single command
pre_setup = "echo 'Starting...'"

# Multiple commands (run in order)
post_setup = [
  "vx run db:migrate",
  "vx run seed",
  "vx run build"
]
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
| `vx setup` | Install tools + run hooks | First time, after `.vx.toml` changes |
| `vx dev` | Enter environment | Daily development |
| `vx dev --export` | Activate in current shell | IDE integration, scripts |
| `vx run <script>` | Run defined scripts | Build, test, deploy |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success - all tools installed |
| 1 | Error - some tools failed to install or hook execution failed |

## Tips

1. **Run after git pull**: If `.vx.toml` might have changed:

   ```bash
   git pull && vx setup
   ```

2. **Use git hooks for auto-setup**: Configure `enter` hook:

   ```toml
   [hooks]
   enter = "vx sync --check"
   ```

3. **Verbose mode for debugging**:

   ```bash
   vx setup --verbose
   ```

4. **Parallel is faster**: By default, tools install in parallel. Use `--no-parallel` only if you encounter issues.

5. **Use post_setup for database migrations**:

   ```toml
   [hooks]
   post_setup = ["vx run db:migrate", "vx run seed"]
   ```

## See Also

- [init](../cli/commands#init) - Initialize project configuration
- [dev](dev) - Enter development environment
- [sync](../cli/commands#sync) - Sync tools with configuration
- [hook](../cli/commands#hook) - Manage git hooks
- [services](../cli/commands#services) - Manage development services
