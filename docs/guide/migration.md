# Migration Guide

This guide helps you migrate your `.vx.toml` configuration from older versions to the latest format.

## Migration Framework

vx includes a built-in migration framework (`vx-migration`) that automatically handles configuration upgrades. The framework supports:

- **Automatic version detection** - Detects your current config format
- **Plugin-based migrations** - Extensible migration system
- **Dry-run mode** - Preview changes before applying
- **Rollback support** - Revert changes if needed
- **History tracking** - Track all migration operations

### Quick Migration Commands

```bash
# Check what migrations are needed
vx migrate --check

# Preview changes (dry-run)
vx migrate --dry-run

# Execute migrations
vx migrate

# Execute with backup
vx migrate --backup

# Rollback to previous version
vx migrate --rollback v1.0.0
```

### Built-in Migrations

| Migration ID | Description | Version Range |
|-------------|-------------|---------------|
| `file-rename` | Renames `.vx.toml` to `vx.toml` | any |
| `config-v1-to-v2` | Converts `[tools]` to `[runtimes]` | 1.x → 2.0 |

## Version History

| Version | Release | Key Changes |
|---------|---------|-------------|
| v0.5.x | Current | Basic tools, scripts, env |
| v0.6.0 | Upcoming | Hooks, services, dependencies |
| v0.7.0 | Planned | AI integration, docs generation |
| v0.8.0 | Planned | Team collaboration, remote dev |

## Quick Migration

```bash
# Check current configuration compatibility
vx config check

# Auto-migrate to latest format
vx config migrate --to v2

# Validate the result
vx config validate
```

## Migrating from v0.5.x to v0.6.0

### Step 1: Add Version Requirement

Add `min_version` to ensure compatibility:

```toml
# Before
[project]
name = "my-project"

# After
min_version = "0.6.0"

[project]
name = "my-project"
```

### Step 2: Migrate Scripts with Dependencies

If you have scripts that depend on other scripts, use the new `depends` field:

```toml
# Before (manual ordering)
[scripts]
build = "npm run build"
deploy = "npm run build && npm run deploy"

# After (explicit dependencies)
[scripts]
build = "npm run build"

[scripts.deploy]
command = "npm run deploy"
depends = ["build"]
```

### Step 3: Add Lifecycle Hooks

Move setup commands to hooks:

```toml
# Before (in README or manual steps)
# 1. Run npm install
# 2. Run db:migrate
# 3. Run seed

# After (automated)
[hooks]
post_setup = ["npm install", "vx run db:migrate", "vx run seed"]
```

### Step 4: Define Services

If you were using docker-compose separately, integrate it:

```toml
# Before (docker-compose.yml)
# services:
#   db:
#     image: postgres:16
#     ports:
#       - "5432:5432"

# After (.vx.toml)
[services.database]
image = "postgres:16"
ports = ["5432:5432"]
env = { POSTGRES_PASSWORD = "dev" }
healthcheck = "pg_isready"
```

### Step 5: Configure Dependencies

Add dependency management settings:

```toml
[dependencies]
lockfile = true
audit = true
auto_update = "minor"

[dependencies.node]
package_manager = "pnpm"
registry = "https://registry.npmmirror.com"
```

## Complete Migration Example

### Before (v0.5.x)

```toml
[project]
name = "my-app"
version = "1.0.0"

[tools]
node = "20"
uv = "latest"

[python]
version = "3.11"
venv = ".venv"

[python.dependencies]
requirements = ["requirements.txt"]

[env]
NODE_ENV = "development"

[env.required]
DATABASE_URL = "Database connection"

[scripts]
dev = "npm run dev"
test = "pytest"
build = "npm run build"
lint = "eslint . && ruff check ."

[settings]
auto_install = true
```

### After (v0.6.0)

```toml
min_version = "0.6.0"

[project]
name = "my-app"
version = "1.0.0"
license = "MIT"
repository = "https://github.com/org/my-app"

[tools]
node = "20"
uv = "latest"

[tools.node]
version = "20"
postinstall = "corepack enable"

[python]
version = "3.11"
venv = ".venv"
package_manager = "uv"

[python.dependencies]
requirements = ["requirements.txt"]

[env]
NODE_ENV = "development"

[env.required]
DATABASE_URL = "Database connection"

[env.secrets]
provider = "auto"
items = ["DATABASE_URL"]

[scripts]
dev = "npm run dev"
test = "pytest"
build = "npm run build"

[scripts.lint]
command = "eslint . && ruff check ."
description = "Run all linters"

[settings]
auto_install = true
parallel_install = true
cache_duration = "7d"

# New in v0.6.0
[hooks]
post_setup = ["npm install", "vx run db:migrate"]
pre_commit = "vx run lint"

[services.database]
image = "postgres:16"
ports = ["5432:5432"]
env = { POSTGRES_PASSWORD = "dev" }
healthcheck = "pg_isready"

[services.redis]
image = "redis:7-alpine"
ports = ["6379:6379"]

[dependencies]
lockfile = true
audit = true
auto_update = "minor"

[dependencies.node]
package_manager = "pnpm"
```

## Migrating from Other Tools

### From mise (.mise.toml)

```toml
# mise
[tools]
node = "20"
python = "3.11"

[tasks]
dev = "npm run dev"
test = "pytest"

# vx equivalent
[tools]
node = "20"

[python]
version = "3.11"

[scripts]
dev = "npm run dev"
test = "pytest"
```

### From devbox (devbox.json)

```json
// devbox.json
{
  "packages": ["nodejs@20", "python@3.11"],
  "shell": {
    "scripts": {
      "dev": "npm run dev"
    }
  }
}
```

```toml
# .vx.toml equivalent
[tools]
node = "20"

[python]
version = "3.11"

[scripts]
dev = "npm run dev"
```

### From asdf (.tool-versions)

```
# .tool-versions
nodejs 20.10.0
python 3.11.0
golang 1.21.5
```

```toml
# .vx.toml equivalent
[tools]
node = "20.10.0"
go = "1.21.5"

[python]
version = "3.11.0"
```

## Backward Compatibility

vx maintains backward compatibility:

1. **All v0.5.x configs work**: No changes required for basic usage
2. **New fields are optional**: Add them gradually as needed
3. **Warnings, not errors**: Unknown fields generate warnings
4. **Graceful degradation**: Missing features fall back to defaults

## Validation

After migration, validate your configuration:

```bash
# Check for errors
vx config validate

# Show parsed configuration
vx config show

# Test setup without changes
vx setup --dry-run
```

## Common Migration Issues

### Issue: Scripts not running in order

**Problem**: Scripts depend on each other but run in wrong order.

**Solution**: Use `depends` field:

```toml
[scripts.deploy]
command = "npm run deploy"
depends = ["build", "test"]
```

### Issue: Environment variables not loading

**Problem**: Required env vars not being validated.

**Solution**: Move to `[env.required]`:

```toml
[env.required]
API_KEY = "API key for external service"
DATABASE_URL = "PostgreSQL connection string"
```

### Issue: Services not starting

**Problem**: Services start before dependencies are ready.

**Solution**: Use `depends_on` and `healthcheck`:

```toml
[services.app]
command = "npm run dev"
depends_on = ["database"]

[services.database]
image = "postgres:16"
healthcheck = "pg_isready"
```

## Getting Help

- [Configuration Reference](/config/vx-toml) - Complete field reference
- [Best Practices](/guide/best-practices) - Recommended patterns
- [GitHub Issues](https://github.com/vx-dev/vx/issues) - Report problems
