# .vx.toml Reference

Complete reference for the `.vx.toml` project configuration file.

## File Location

Place `.vx.toml` in your project root. vx searches for it in the current directory and parent directories.

## Complete Example

```toml
[project]
name = "my-project"
description = "A sample project"
version = "1.0.0"

[tools]
node = "20"
uv = "latest"
go = "1.21"
rust = "stable"

[python]
version = "3.11"
venv = ".venv"

[python.dependencies]
requirements = ["requirements.txt", "requirements-dev.txt"]
packages = ["pytest", "black", "ruff"]
git = ["https://github.com/user/repo.git"]
dev = ["pytest", "mypy"]

[env]
NODE_ENV = "development"
DEBUG = "true"

[env.required]
API_KEY = "Your API key"
DATABASE_URL = "Database connection string"

[env.optional]
CACHE_DIR = "Optional cache directory"

[scripts]
dev = "npm run dev"
test = "pytest"
build = "go build -o app"

[scripts.start]
command = "python main.py"
description = "Start the server"
args = ["--host", "0.0.0.0", "--port", "8080"]
cwd = "src"
env = { DEBUG = "true" }

[settings]
auto_install = true
parallel_install = true
cache_duration = "7d"
```

## Sections

### [project]

Project metadata.

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Project name |
| `description` | string | Project description |
| `version` | string | Project version |

```toml
[project]
name = "my-project"
description = "A sample project"
version = "1.0.0"
```

### [tools]

Tool versions to use.

```toml
[tools]
node = "20"          # Major version
uv = "latest"        # Latest stable
go = "1.21.5"        # Exact version
rust = "stable"      # Channel
```

#### Version Specifiers

| Format | Example | Description |
|--------|---------|-------------|
| Major | `"20"` | Latest 20.x.x |
| Minor | `"20.10"` | Latest 20.10.x |
| Exact | `"20.10.0"` | Exact version |
| Latest | `"latest"` | Latest stable |
| LTS | `"lts"` | Latest LTS (Node.js) |
| Channel | `"stable"` | Channel (Rust) |

### [python]

Python environment configuration.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `version` | string | - | Python version |
| `venv` | string | `".venv"` | Virtual environment path |

```toml
[python]
version = "3.11"
venv = ".venv"
```

### [python.dependencies]

Python dependencies.

| Field | Type | Description |
|-------|------|-------------|
| `requirements` | string[] | Requirements files |
| `packages` | string[] | Direct packages |
| `git` | string[] | Git URLs |
| `dev` | string[] | Dev dependencies |

```toml
[python.dependencies]
requirements = ["requirements.txt"]
packages = ["requests", "pandas"]
git = ["https://github.com/user/repo.git"]
dev = ["pytest", "black"]
```

### [env]

Environment variables.

```toml
[env]
NODE_ENV = "development"
DEBUG = "true"
```

### [env.required]

Required environment variables (must be set).

```toml
[env.required]
API_KEY = "Description of the variable"
DATABASE_URL = "Database connection string"
```

### [env.optional]

Optional environment variables (with descriptions).

```toml
[env.optional]
CACHE_DIR = "Optional cache directory"
LOG_LEVEL = "Logging level (default: info)"
```

### [scripts]

Runnable scripts.

#### Simple Scripts

```toml
[scripts]
dev = "npm run dev"
test = "pytest"
build = "go build -o app"
```

#### Complex Scripts

```toml
[scripts.start]
command = "python main.py"
description = "Start the server"
args = ["--host", "0.0.0.0"]
cwd = "src"
env = { DEBUG = "true" }
```

| Field | Type | Description |
|-------|------|-------------|
| `command` | string | Command to run |
| `description` | string | Human-readable description |
| `args` | string[] | Default arguments |
| `cwd` | string | Working directory |
| `env` | table | Environment variables |

### [settings]

Behavior settings.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `auto_install` | bool | `true` | Auto-install missing tools |
| `parallel_install` | bool | `true` | Install in parallel |
| `cache_duration` | string | `"7d"` | Cache duration |

```toml
[settings]
auto_install = true
parallel_install = true
cache_duration = "7d"
```

## Validation

vx validates `.vx.toml` on load. Common errors:

- Invalid TOML syntax
- Unknown tool names
- Invalid version specifiers
- Missing required fields

## Tips

1. **Commit to git**: Share configuration with team
2. **Use specific versions**: For reproducibility
3. **Document env vars**: Use descriptions in `[env.required]`
4. **Define scripts**: Make common tasks easy
