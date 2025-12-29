# vx.toml Reference

Complete reference for the `vx.toml` project configuration file.

## File Location

Place `vx.toml` in your project root. vx searches for it in the current directory and parent directories.

## Version Requirement

Use `min_version` to specify the minimum vx version required:

```toml
min_version = "0.6.0"
```

## Complete Example

```toml
min_version = "0.6.0"

[project]
name = "my-fullstack-app"
description = "AI-powered fullstack application"
version = "1.0.0"
license = "MIT"
repository = "https://github.com/org/repo"

[tools]
node = "20"
uv = "latest"
go = "1.22"
rust = "stable"

[tools.node]
version = "20"
postinstall = "corepack enable"
os = ["linux", "darwin", "windows"]

[python]
version = "3.12"
venv = ".venv"
package_manager = "uv"

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

[env.secrets]
provider = "auto"
items = ["DATABASE_URL", "API_KEY"]

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
depends = ["build"]

[settings]
auto_install = true
parallel_install = true
cache_duration = "7d"
shell = "auto"
log_level = "info"

[hooks]
pre_setup = "echo 'Starting setup...'"
post_setup = ["vx run db:migrate", "vx run seed"]
pre_commit = "vx run lint && vx run test:unit"
enter = "vx sync --check"

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
env_file = ".env.local"

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

## Sections

### [project]

Project metadata.

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Project name |
| `description` | string | Project description |
| `version` | string | Project version |
| `license` | string | License identifier (e.g., MIT, Apache-2.0) |
| `repository` | string | Repository URL |

```toml
[project]
name = "my-project"
description = "A sample project"
version = "1.0.0"
license = "MIT"
repository = "https://github.com/org/repo"
```

### [tools]

Tool versions to use. Supports simple version strings or detailed configuration.

#### Simple Version

```toml
[tools]
node = "20"          # Major version
uv = "latest"        # Latest stable
go = "1.21.5"        # Exact version
rust = "stable"      # Channel
```

#### Detailed Configuration

```toml
[tools.node]
version = "20"
postinstall = "corepack enable"
os = ["linux", "darwin", "windows"]
install_env = { NODE_OPTIONS = "--max-old-space-size=4096" }
```

| Field | Type | Description |
|-------|------|-------------|
| `version` | string | Version string |
| `postinstall` | string | Command to run after installation |
| `os` | string[] | Limit to specific operating systems |
| `install_env` | table | Environment variables for installation |

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
| `package_manager` | string | `"uv"` | Package manager (uv, pip, poetry) |

```toml
[python]
version = "3.12"
venv = ".venv"
package_manager = "uv"
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

Environment variables with support for required/optional declarations and secrets.

#### Static Variables

```toml
[env]
NODE_ENV = "development"
DEBUG = "true"
```

#### Required Variables

Variables that must be set (vx will warn if missing):

```toml
[env.required]
API_KEY = "Description of the variable"
DATABASE_URL = "Database connection string"
```

#### Optional Variables

Optional variables with descriptions:

```toml
[env.optional]
CACHE_DIR = "Optional cache directory"
LOG_LEVEL = "Logging level (default: info)"
```

#### Secrets

Load secrets from secure storage:

```toml
[env.secrets]
provider = "auto"  # auto | 1password | vault | aws-secrets
items = ["DATABASE_URL", "API_KEY"]
```

### [scripts]

Runnable scripts with support for simple commands and detailed configuration.

#### Simple Scripts

```toml
[scripts]
dev = "npm run dev"
test = "pytest"
build = "go build -o app"
```

#### Detailed Scripts

```toml
[scripts.start]
command = "python main.py"
description = "Start the server"
args = ["--host", "0.0.0.0"]
cwd = "src"
env = { DEBUG = "true" }
depends = ["build"]
```

| Field | Type | Description |
|-------|------|-------------|
| `command` | string | Command to run |
| `description` | string | Human-readable description |
| `args` | string[] | Default arguments |
| `cwd` | string | Working directory |
| `env` | table | Environment variables |
| `depends` | string[] | Scripts to run first |

### [settings]

Behavior settings.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `auto_install` | bool | `true` | Auto-install missing tools |
| `parallel_install` | bool | `true` | Install in parallel |
| `cache_duration` | string | `"7d"` | Cache duration |
| `shell` | string | `"auto"` | Shell to use (auto, bash, zsh, fish, pwsh) |
| `log_level` | string | `"info"` | Log level |

```toml
[settings]
auto_install = true
parallel_install = true
cache_duration = "7d"
shell = "auto"
log_level = "info"

[settings.experimental]
monorepo = false
workspaces = false
```

### [hooks] <Badge type="tip" text="v0.6.0+" />

Lifecycle hooks for automation.

| Hook | When it runs |
|------|--------------|
| `pre_setup` | Before `vx setup` |
| `post_setup` | After `vx setup` |
| `pre_commit` | Before git commit (requires git hooks setup) |
| `enter` | When entering project directory |

```toml
[hooks]
pre_setup = "echo 'Starting setup...'"
post_setup = ["vx run db:migrate", "vx run seed"]
pre_commit = "vx run lint && vx run test:unit"
enter = "vx sync --check"

[hooks.custom]
deploy = "vx run build && vx run deploy"
```

Hooks can be a single command string or an array of commands.

### [services] <Badge type="tip" text="v0.6.0+" />

Service definitions for local development (similar to docker-compose).

```toml
[services.database]
image = "postgres:16"
ports = ["5432:5432"]
env = { POSTGRES_PASSWORD = "dev" }
volumes = ["./data:/var/lib/postgresql/data"]
healthcheck = "pg_isready"

[services.redis]
image = "redis:7-alpine"
ports = ["6379:6379"]

[services.app]
command = "npm run dev"
depends_on = ["database", "redis"]
ports = ["3000:3000"]
env_file = ".env.local"
working_dir = "./frontend"
```

| Field | Type | Description |
|-------|------|-------------|
| `image` | string | Docker image |
| `command` | string | Command to run (for non-container services) |
| `ports` | string[] | Port mappings (host:container) |
| `env` | table | Environment variables |
| `env_file` | string | Environment file path |
| `volumes` | string[] | Volume mounts |
| `depends_on` | string[] | Service dependencies |
| `healthcheck` | string | Health check command |
| `working_dir` | string | Working directory |

### [dependencies] <Badge type="tip" text="v0.6.0+" />

Smart dependency management configuration.

```toml
[dependencies]
lockfile = true
audit = true
auto_update = "minor"  # none | patch | minor | major

[dependencies.node]
package_manager = "pnpm"  # npm | yarn | pnpm | bun
registry = "https://registry.npmmirror.com"

[dependencies.python]
index_url = "https://pypi.tuna.tsinghua.edu.cn/simple"
extra_index_urls = []

[dependencies.constraints]
"lodash" = ">=4.17.21"  # Security constraint
"*" = { licenses = ["MIT", "Apache-2.0", "BSD-3-Clause"] }
```

| Field | Type | Description |
|-------|------|-------------|
| `lockfile` | bool | Generate lockfile |
| `audit` | bool | Run security audit |
| `auto_update` | string | Auto-update strategy |
| `node` | table | Node.js specific settings |
| `python` | table | Python specific settings |
| `constraints` | table | Dependency constraints |

## Future Sections (Planned)

The following sections are planned for future releases:

| Section | Version | Description |
|---------|---------|-------------|
| `[ai]` | v0.7.0 | AI code generation integration |
| `[docs]` | v0.7.0 | Documentation auto-generation |
| `[team]` | v0.8.0 | Team collaboration rules |
| `[remote]` | v0.8.0 | Remote development environments |
| `[security]` | v0.9.0 | Security scanning rules |
| `[test]` | v0.9.0 | Test pipeline configuration |
| `[telemetry]` | v0.9.0 | Performance monitoring |
| `[container]` | v1.0.0 | Container deployment |
| `[versioning]` | v1.0.0 | Version control strategy |

## Validation

vx validates `vx.toml` on load. Common errors:

- Invalid TOML syntax
- Unknown tool names
- Invalid version specifiers
- Missing required fields
- Circular script dependencies

Run validation manually:

```bash
vx config validate
```

## Tips

1. **Commit to git**: Share configuration with team
2. **Use specific versions**: For reproducibility in production
3. **Document env vars**: Use descriptions in `[env.required]`
4. **Define scripts**: Make common tasks easy to discover
5. **Use hooks**: Automate repetitive setup tasks
6. **Service dependencies**: Ensure services start in correct order

## See Also

- [Configuration Guide](/guide/configuration) - Getting started with configuration
- [Migration Guide](/guide/migration) - Migrating from older versions
- [Best Practices](/guide/best-practices) - Configuration best practices
