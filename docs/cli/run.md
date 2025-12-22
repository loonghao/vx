# run

Run a script defined in `.vx.toml`.

## Synopsis

```bash
vx run <SCRIPT> [ARGS...]
```

## Description

Executes a script defined in the `[scripts]` section of `.vx.toml`. The script runs with:

- Project environment variables
- Python venv activated (if configured)
- Tool paths configured

## Arguments

| Argument | Description |
|----------|-------------|
| `SCRIPT` | Name of the script to run |
| `ARGS` | Additional arguments passed to the script |

## Configuration

Define scripts in `.vx.toml`:

```toml
[scripts]
# Simple command
dev = "npm run dev"
test = "pytest"
build = "go build -o app"

# Complex script
[scripts.start]
command = "python main.py"
description = "Start the server"
args = ["--host", "0.0.0.0"]
cwd = "src"
env = { DEBUG = "true" }
```

## Examples

### Run Simple Script

```bash
vx run dev
```

### Pass Additional Arguments

```bash
vx run test -- --coverage --verbose
```

### Run Script with Description

```bash
vx run start
```

Output:

```
Running 'start': Start the server
python main.py --host 0.0.0.0
```

### List Available Scripts

If you try to run a non-existent script:

```bash
vx run unknown
```

Output:

```
Error: Script 'unknown' not found. Available scripts: dev, test, build, start
```

## Script Configuration

### Simple Scripts

```toml
[scripts]
dev = "npm run dev"
```

### Complex Scripts

```toml
[scripts.deploy]
command = "kubectl apply"
description = "Deploy to Kubernetes"
args = ["-f", "k8s/"]
cwd = "infrastructure"
env = { KUBECONFIG = "~/.kube/config" }
```

### Script Properties

| Property | Description |
|----------|-------------|
| `command` | Command to execute |
| `description` | Human-readable description |
| `args` | Default arguments |
| `cwd` | Working directory |
| `env` | Environment variables |

## Environment

Scripts run with:

1. **Project environment variables** from `[env]` section
2. **Script-specific variables** from script's `env` property
3. **Python venv** activated if `[python]` is configured
4. **PATH** updated with project tool versions

## See Also

- [dev](./dev) - Enter development environment
- [setup](./setup) - Install project tools
- [Configuration](/config/vx-toml) - .vx.toml reference
