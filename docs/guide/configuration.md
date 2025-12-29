# Configuration

vx uses a simple TOML-based configuration system with two levels:

1. **Project Configuration** (`vx.toml`) - Per-project settings
2. **Global Configuration** (`~/.config/vx/config.toml`) - User-wide defaults

## Project Configuration (vx.toml)

Create a `vx.toml` file in your project root:

```toml
min_version = "0.6.0"

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
package_manager = "uv"

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
depends = ["build"]

[settings]
auto_install = true
parallel_install = true
cache_duration = "7d"

[hooks]
post_setup = "vx run db:migrate"
pre_commit = "vx run lint"

[services.database]
image = "postgres:16"
ports = ["5432:5432"]
env = { POSTGRES_PASSWORD = "dev" }
```

## Sections Explained

### [project]

Project metadata:

```toml
[project]
name = "my-project"
description = "Project description"
version = "1.0.0"
license = "MIT"
repository = "https://github.com/org/repo"
```

### [tools]

Tool versions to use. Supports simple strings or detailed configuration:

```toml
[tools]
node = "20"          # Major version
uv = "latest"        # Latest stable
go = "1.21.5"        # Exact version
rust = "stable"      # Channel

# Detailed configuration
[tools.node]
version = "20"
postinstall = "corepack enable"
os = ["linux", "darwin", "windows"]
```

### [python]

Python environment configuration:

```toml
[python]
version = "3.11"
venv = ".venv"
package_manager = "uv"  # uv | pip | poetry

[python.dependencies]
requirements = ["requirements.txt", "requirements-dev.txt"]
packages = ["pytest", "black", "ruff"]
git = ["https://github.com/user/repo.git"]
dev = ["pytest", "mypy"]
```

### [env]

Environment variables with required/optional declarations:

```toml
[env]
NODE_ENV = "development"
DEBUG = "true"

[env.required]
API_KEY = "Description of required variable"
DATABASE_URL = "Database connection string"

[env.optional]
CACHE_DIR = "Optional cache directory"

[env.secrets]
provider = "auto"  # auto | 1password | vault | aws-secrets
items = ["DATABASE_URL", "API_KEY"]
```

### [scripts]

Runnable scripts with dependencies:

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
depends = ["build"]  # Run build first
```

### [settings]

Behavior settings:

```toml
[settings]
auto_install = true       # Auto-install missing tools
parallel_install = true   # Install tools in parallel
cache_duration = "7d"     # Cache duration
shell = "auto"            # Shell (auto, bash, zsh, fish, pwsh)
log_level = "info"        # Log level

[settings.experimental]
monorepo = false
workspaces = false
```

### [hooks] <Badge type="tip" text="v0.6.0+" />

Lifecycle hooks for automation:

```toml
[hooks]
pre_setup = "echo 'Starting setup...'"
post_setup = ["vx run db:migrate", "vx run seed"]
pre_commit = "vx run lint && vx run test"
enter = "vx sync --check"
```

Available hooks:

- `pre_setup` - Before `vx setup`
- `post_setup` - After `vx setup`
- `pre_commit` - Before git commit
- `enter` - When entering project directory

### [services] <Badge type="tip" text="v0.6.0+" />

Local development services (docker-compose style):

```toml
[services.database]
image = "postgres:16"
ports = ["5432:5432"]
env = { POSTGRES_PASSWORD = "dev" }
healthcheck = "pg_isready"

[services.redis]
image = "redis:7-alpine"
ports = ["6379:6379"]

[services.app]
command = "npm run dev"
depends_on = ["database", "redis"]
ports = ["3000:3000"]
```

### [dependencies] <Badge type="tip" text="v0.6.0+" />

Smart dependency management:

```toml
[dependencies]
lockfile = true
audit = true
auto_update = "minor"

[dependencies.node]
package_manager = "pnpm"
registry = "https://registry.npmmirror.com"

[dependencies.python]
index_url = "https://pypi.tuna.tsinghua.edu.cn/simple"
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

# Validate configuration
vx config validate
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
3. Project config (`vx.toml`)
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

## Migrating from Older Versions

If you have an older `vx.toml`, you can migrate to the new format:

```bash
# Check compatibility
vx config check

# Auto-migrate to v2 format
vx config migrate --to v2

# Validate after migration
vx config validate
```

See the [Migration Guide](/guide/migration) for detailed instructions.

## Next Steps

- [vx.toml Reference](/config/vx-toml) - Complete configuration reference
- [Environment Variables](/config/env-vars) - All environment variables
- [Project Environments](/guide/project-environments) - Working with project environments
- [Best Practices](/guide/best-practices) - Configuration best practices
