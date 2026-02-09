# run

Run a script defined in `vx.toml`.

## Synopsis

```bash
vx run <SCRIPT> [ARGS...]
vx run <SCRIPT> -H
vx run --list
vx run --help
```

## Description

Executes a script defined in the `[scripts]` section of `vx.toml`. The enhanced run command now supports:

- **Flexible argument passing**: Pass arguments directly without conflicts
- **Script-specific help**: Use `-H` to get help for individual scripts
- **Script listing**: Use `--list` to see all available scripts
- **Advanced argument handling**: Support for `-p`, `--lib`, and other tool-specific flags
- Project environment variables (from `[env]` and `.env` files)
- Variable interpolation support (`\{\{var\}\}` syntax)
- Python venv activated (if configured)
- Tool paths configured

## Arguments

| Argument | Description |
|----------|-------------|
| `SCRIPT` | Name of the script to run (optional when using `--list` or `--help`) |
| `ARGS` | Additional arguments passed to the script (supports hyphenated flags like `-p`, `--lib`) |

## Options

| Option | Description |
|--------|-------------|
| `-h`, `--help` | Show help for the run command |
| `-l`, `--list` | List all available scripts |
| `-H`, `--script-help` | Show script-specific help (when script name is provided) |

## Enhanced Argument Handling

The run command now supports passing complex arguments directly to scripts without conflicts:

```bash
# Pass tool-specific flags directly
vx run test-pkgs -p vx-runtime --lib
vx run lint --fix --verbose
vx run build --release --target x86_64-pc-windows-msvc

# Use -- separator for explicit argument separation (optional)
vx run test-pkgs -- -p vx-runtime --lib
```

## Variable Interpolation

Scripts support variable interpolation using `\{\{var\}\}` syntax:

```toml
[scripts]
build = "cargo build -p \{\{project.name\}\}"
tag = "git tag v\{\{arg1\}\}"
info = "echo 'Building on \{\{os.name\}\} (\{\{os.arch\}\})'"
test-pkgs = "cargo test \{\{args\}\}"  # Use {{args}} for all arguments
```

### Built-in Variables

| Variable | Description |
|----------|-------------|
| `\{\{vx.version\}\}` | vx version |
| `\{\{vx.home\}\}` | vx home directory (~/.vx) |
| `\{\{vx.runtimes\}\}` | Runtimes directory |
| `\{\{project.root\}\}` | Project root directory |
| `\{\{project.name\}\}` | Project name (directory name) |
| `\{\{os.name\}\}` | Operating system (linux, macos, windows) |
| `\{\{os.arch\}\}` | CPU architecture (x86_64, aarch64) |
| `\{\{home\}\}` | User home directory |
| `\{\{timestamp\}\}` | Current Unix timestamp |

### Argument Variables

| Variable | Description |
|----------|-------------|
| `\{\{arg1\}\}`, `\{\{arg2\}\}`, ... | Positional arguments |
| `\{\{@\}\}` | All arguments as a string |
| `\{\{#\}\}` | Number of arguments |
| `\{\{args\}\}` | **Recommended**: All arguments (supports complex flags like `-p`, `--lib`) |

### Environment Variables

| Variable | Description |
|----------|-------------|
| `\{\{env.VAR\}\}` | Environment variable VAR |

### Command Interpolation

Use backticks for command output:

```toml
[scripts]
info = "echo 'Commit: `git rev-parse --short HEAD`'"
```

## Environment Variables

### .env File Support

Scripts automatically load environment variables from:

1. `.env` - Base environment file
2. `.env.local` - Local overrides (higher priority)

### Priority Order

1. Script-specific `env` property
2. Global `[env]` section in vx.toml
3. `.env.local` file
4. `.env` file
5. System environment variables

### Configuration

```toml
[env]
NODE_ENV = "development"
API_URL = "http://localhost:3000"

[scripts.dev]
run = "npm run dev"
env = { PORT = "3000" }  # Script-specific
```

## Configuration

Define scripts in `vx.toml`:

```toml
[scripts]
# Simple command
dev = "npm run dev"
test = "pytest"
build = "go build -o app"

# With variable interpolation
deploy = "kubectl apply -f k8s/\{\{arg1\}\}.yaml"

# Modern approach: use {{args}} for complex arguments
test-pkgs = "cargo test \{\{args\}\}"
lint = "eslint \{\{args\}\}"
format = "prettier --write \{\{args\}\}"

# Detailed script with DAG dependencies
[scripts.ci]
command = "echo 'All checks passed!'"
description = "Run all CI checks"
depends = ["lint", "test", "build"]

[scripts.start]
command = "python main.py"
description = "Start the server"
args = ["--host", "0.0.0.0"]
cwd = "src"
env = { DEBUG = "true" }
depends = ["build"]
```

## DAG Dependency Execution

Scripts can declare dependencies on other scripts using the `depends` field. vx uses **topological sorting** to determine the correct execution order.

### How It Works

1. **Build dependency graph** — collect all transitive dependencies
2. **Detect cycles** — report error if circular dependencies exist
3. **Topological sort** — determine execution order
4. **Execute sequentially** — each script runs at most once
5. **Fail fast** — if any dependency fails, the chain stops

### Example

```toml
[scripts]
lint = "eslint ."
typecheck = "tsc --noEmit"
test = "vitest run"
build = "npm run build"

[scripts.ci]
command = "echo '✅ CI passed'"
description = "Run full CI pipeline"
depends = ["lint", "typecheck", "test", "build"]
```

```bash
vx run ci
# Execution: lint → typecheck → test → build → ci
```

### Multi-Level Dependencies

```toml
[scripts]
generate = "protoc --go_out=. *.proto"

[scripts.build]
command = "go build -o app"
depends = ["generate"]

[scripts.test]
command = "go test ./..."
depends = ["generate"]

[scripts.deploy]
command = "kubectl apply -f k8s/"
depends = ["build", "test"]
```

```bash
vx run deploy
# Resolved: generate → build → test → deploy
# (generate runs only once)
```

### Circular Dependency Detection

```toml
[scripts.a]
command = "echo a"
depends = ["b"]

[scripts.b]
command = "echo b"
depends = ["a"]    # Circular!
```

```bash
vx run a
# Error: Circular dependency detected: a -> b -> a
```

## Examples

### Basic Usage

```bash
# Run a simple script
vx run dev

# List all available scripts
vx run --list
```

### Enhanced Argument Passing

```bash
# Pass complex arguments directly (NEW!)
vx run test-pkgs -p vx-runtime --lib
vx run test-pkgs -p vx-provider-python -p vx-runtime

# Traditional approach with -- separator (still supported)
vx run test-pkgs -- -p vx-runtime --lib

# Multiple flags and options
vx run lint --fix --ext .js,.ts src/
vx run build --release --target x86_64-pc-windows-msvc
```

### Script-Specific Help

```bash
# Get help for a specific script (NEW!)
vx run test-pkgs -H
vx run deploy --script-help

# General run command help
vx run --help
```

### Variable Interpolation Examples

```bash
# Arguments are interpolated if script uses \{\{arg1\}\}, \{\{arg2\}\}, etc.
vx run deploy production

# Using {{args}} (recommended for complex arguments)
vx run test-pkgs -p vx-runtime --lib  # Passed as {{args}}
```

### Script Help Output

When you run `vx run deploy -H`, you'll see:

```text
Script: deploy
Command: kubectl apply -f k8s/\{\{arg1\}\}.yaml

Usage: vx run deploy [args...]

Arguments are passed directly to the script.

Variable Interpolation:
  \{\{arg1\}\}          First argument
  \{\{arg2\}\}          Second argument
  \{\{@\}\}             All arguments
  \{\{#\}\}             Number of arguments
  \{\{args\}\}          All arguments (recommended)
  \{\{env.VAR\}\}       Environment variable VAR
  \{\{project.root\}\}  Project root directory
  \{\{project.name\}\}  Project name
  \{\{os.name\}\}       Operating system
  \{\{vx.version\}\}    VX version
```

### List Available Scripts

```bash
vx run --list
```

Output:

```text
Available scripts:
  dev             npm run dev
  test            pytest
  build           go build -o app
  test-pkgs       cargo test {{args}}
  lint            eslint {{args}}
```

## Best Practices

### Use `\{\{args\}\}` for Modern Scripts

For maximum flexibility, use `\{\{args\}\}` in your script definitions:

```toml
[scripts]
# ✅ Recommended: Flexible argument handling
test-pkgs = "cargo test \{\{args\}\}"
lint = "eslint \{\{args\}\}"
build = "cargo build \{\{args\}\}"

# ❌ Old style: Limited to simple arguments
test-old = "cargo test"
```

### Complex Tool Integration

Perfect for tools that require specific flags:

```toml
[scripts]
# Cargo testing with package selection
test-pkgs = "cargo test \{\{args\}\}"
# Usage: vx run test-pkgs -p vx-runtime --lib

# ESLint with flexible options
lint = "eslint \{\{args\}\}"
# Usage: vx run lint --fix --ext .js,.ts src/

# Docker build with platform selection
docker-build = "docker build \{\{args\}\}"
# Usage: vx run docker-build --platform linux/amd64 -t myapp .
```

### Migration from Old Style

If you have existing scripts without `\{\{args\}\}`, they still work but with limitations:

```toml
[scripts]
# This works but only for simple arguments
test = "cargo test"

# This is better for complex arguments
test-new = "cargo test \{\{args\}\}"
```

## Troubleshooting

### Arguments Not Passed Correctly

If your script doesn't receive arguments as expected:

1. **Check if your script uses `\{\{args\}\}`**:
   ```toml
   # Add {{args}} to receive all arguments
   test = "cargo test \{\{args\}\}"
   ```

2. **Use the `--` separator for complex cases**:
   ```bash
   vx run test -- -p vx-runtime --lib
   ```

3. **Check script help**:
   ```bash
   vx run test -H  # Shows how arguments are handled
   ```

### Script Not Found

If you get "Script not found" error:

```bash
# List available scripts
vx run --list

# Check your vx.toml file
cat vx.toml
```

## See Also

- [dev](./dev) - Enter development environment
- [setup](./setup) - Install project tools
- [ext](./ext) - Run extensions
- [Configuration](/config/vx-toml) - vx.toml reference
