# Configuration

vx uses a simple TOML-based configuration system with two levels:

1. **Project Configuration** (`.vx.toml`) - Per-project settings
2. **Global Configuration** (`~/.config/vx/config.toml`) - User-wide defaults

## Project Configuration (.vx.toml)

Create a `.vx.toml` file in your project root:

```toml
[project]
name = "my-project"
description = "A sample project"
version = "1.0.0"

[tools]
node = "20"
uv = "latest"
go = "1.21"

[python]
version = "3.11"
venv = ".venv"

[python.dependencies]
requirements = ["requirements.txt"]
packages = ["pytest", "black"]

[env]
NODE_ENV = "development"

[env.required]
API_KEY = "Your API key"

[scripts]
dev = "npm run dev"
test = "pytest"
build = "go build -o app"

[scripts.start]
command = "python main.py"
description = "Start the application"
args = ["--port", "8080"]
env = { DEBUG = "true" }

[settings]
auto_install = true
parallel_install = true
cache_duration = "7d"
```

## Sections Explained

### [project]

Project metadata:

```toml
[project]
name = "my-project"
description = "Project description"
version = "1.0.0"
```

### [tools]

Tool versions to use:

```toml
[tools]
node = "20"          # Major version
uv = "latest"        # Latest stable
go = "1.21.5"        # Exact version
rust = "stable"      # Channel
```

### [python]

Python environment configuration:

```toml
[python]
version = "3.11"
venv = ".venv"

[python.dependencies]
requirements = ["requirements.txt", "requirements-dev.txt"]
packages = ["pytest", "black", "ruff"]
git = ["https://github.com/user/repo.git"]
dev = ["pytest", "mypy"]
```

### [env]

Environment variables:

```toml
[env]
NODE_ENV = "development"
DEBUG = "true"

[env.required]
API_KEY = "Description of required variable"

[env.optional]
CACHE_DIR = "Optional cache directory"
```

### [scripts]

Runnable scripts:

```toml
[scripts]
# Simple command
dev = "npm run dev"
test = "pytest"

# Complex script with options
[scripts.start]
command = "python main.py"
description = "Start the server"
args = ["--host", "0.0.0.0"]
cwd = "src"
env = { PORT = "8080" }
```

### [settings]

Behavior settings:

```toml
[settings]
auto_install = true       # Auto-install missing tools
parallel_install = true   # Install tools in parallel
cache_duration = "7d"     # Cache duration
```

## Global Configuration

Located at `~/.config/vx/config.toml`:

```toml
[defaults]
auto_install = true
parallel_install = true
cache_duration = "7d"

[tools]
# Default versions for tools
node = "lts"
python = "3.11"
```

### Managing Global Config

```bash
# Show current config
vx config show

# Set a value
vx config set defaults.auto_install true

# Get a value
vx config get defaults.auto_install

# Reset to defaults
vx config reset

# Edit config file
vx config edit
```

## Environment Variables

vx respects these environment variables:

| Variable | Description |
|----------|-------------|
| `VX_HOME` | Override vx data directory |
| `VX_ENV` | Current environment name |
| `VX_AUTO_INSTALL` | Enable/disable auto-install |
| `VX_VERBOSE` | Enable verbose output |
| `VX_DEBUG` | Enable debug output |

## Configuration Precedence

Configuration is resolved in this order (later overrides earlier):

1. Built-in defaults
2. Global config (`~/.config/vx/config.toml`)
3. Project config (`.vx.toml`)
4. Environment variables
5. Command-line flags

## Initializing a Project

Use `vx init` to create a configuration interactively:

```bash
# Interactive mode
vx init -i

# Use a template
vx init --template nodejs
vx init --template python
vx init --template fullstack

# Specify tools
vx init --tools node,uv,go
```

## Next Steps

- [.vx.toml Reference](/config/vx-toml) - Complete configuration reference
- [Environment Variables](/config/env-vars) - All environment variables
- [Project Environments](/guide/project-environments) - Working with project environments
