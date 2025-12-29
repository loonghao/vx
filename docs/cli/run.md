# run

Run a script defined in `.vx.toml`.

## Synopsis

```bash
vx run <SCRIPT> [ARGS...]
vx run <SCRIPT> --help
vx run --list
```

## Description

Executes a script defined in the `[scripts]` section of `.vx.toml`. The script runs with:

- Project environment variables (from `[env]` and `.env` files)
- Variable interpolation support (`\{\{var\}\}` syntax)
- Python venv activated (if configured)
- Tool paths configured

## Arguments

| Argument | Description |
|----------|-------------|
| `SCRIPT` | Name of the script to run |
| `ARGS` | Additional arguments passed to the script |

## Options

| Option | Description |
|--------|-------------|
| `--help`, `-h` | Show help for the script |
| `--list` | List all available scripts |

## Variable Interpolation

Scripts support variable interpolation using `\{\{var\}\}` syntax:

```toml
[scripts]
build = "cargo build -p \{\{project.name\}\}"
tag = "git tag v\{\{arg1\}\}"
info = "echo 'Building on \{\{os.name\}\} (\{\{os.arch\}\})'"
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

Define scripts in `.vx.toml`:

```toml
[scripts]
# Simple command
dev = "npm run dev"
test = "pytest"
build = "go build -o app"

# With variable interpolation
deploy = "kubectl apply -f k8s/\{\{arg1\}\}.yaml"
```

## Examples

### Run Simple Script

```bash
vx run dev
```

### Pass Arguments

```bash
# Arguments are interpolated if script uses \{\{arg1\}\}, \{\{arg2\}\}, etc.
vx run deploy production

# Or passed directly
vx run test -- --coverage --verbose
```

### View Script Help

```bash
vx run deploy --help
```

Output:

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
  dev = "npm run dev"
  test = "pytest"
  build = "go build -o app"
```

### Using Environment Variables

```bash
# From .env file
echo "API_KEY=secret123" > .env
vx run deploy  # API_KEY is available

# Inline override
API_KEY=newkey vx run deploy
```

## See Also

- [dev](./dev) - Enter development environment
- [setup](./setup) - Install project tools
- [x](./x) - Run extensions
- [Configuration](/config/vx-toml) - .vx.toml reference
