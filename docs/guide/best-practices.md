# Best Practices

This guide covers recommended patterns and best practices for configuring vx projects.

## Project Setup

### Always Specify min_version

Ensure your configuration works with the expected vx version:

```toml
min_version = "0.6.0"

[project]
name = "my-project"
```

### Use Specific Versions in Production

For reproducibility, use exact versions in production projects:

```toml
# Development - flexible
[tools]
node = "20"        # Latest 20.x
uv = "latest"      # Latest stable

# Production - pinned
[tools]
node = "20.10.0"   # Exact version
uv = "0.5.14"      # Exact version
```

### Document Your Project

Add metadata for discoverability:

```toml
[project]
name = "my-fullstack-app"
description = "AI-powered fullstack application"
version = "1.0.0"
license = "MIT"
repository = "https://github.com/org/repo"
```

## Tool Configuration

### Use Detailed Config for Complex Tools

When tools need post-install steps or OS restrictions:

```toml
[tools.node]
version = "20"
postinstall = "corepack enable"
os = ["linux", "darwin"]  # Skip on Windows

[tools.rust]
version = "stable"
postinstall = "rustup component add clippy rustfmt"
```

### Group Related Tools

Organize tools by ecosystem:

```toml
# Frontend
[tools]
node = "20"
bun = "latest"

# Backend
[python]
version = "3.12"
package_manager = "uv"

# Infrastructure
[tools]
terraform = "1.6"
kubectl = "1.28"
```

## Scripts

### Use Descriptive Names

```toml
[scripts]
# Good - clear intent
dev = "npm run dev"
test:unit = "pytest tests/unit"
test:integration = "pytest tests/integration"
build:prod = "npm run build -- --mode production"

# Avoid - unclear
run = "npm start"
t = "pytest"
```

### Add Descriptions for Complex Scripts

```toml
[scripts.deploy]
command = "npm run build && aws s3 sync dist/ s3://bucket"
description = "Build and deploy to production S3 bucket"

[scripts.db:reset]
command = "prisma migrate reset --force"
description = "Reset database (WARNING: destroys all data)"
```

### Use Dependencies for Multi-Step Tasks

```toml
[scripts]
lint = "eslint . && prettier --check ."
typecheck = "tsc --noEmit"
test = "vitest run"

[scripts.ci]
command = "echo 'All checks passed!'"
description = "Run all CI checks"
depends = ["lint", "typecheck", "test"]
```

### Avoid Hardcoded Paths

```toml
# Bad - hardcoded paths
[scripts]
build = "cd /home/user/project && npm run build"

# Good - relative paths
[scripts]
build = "npm run build"

[scripts.backend]
command = "python main.py"
cwd = "backend"
```

## Environment Variables

### Document Required Variables

```toml
[env.required]
DATABASE_URL = "PostgreSQL connection string (postgres://user:pass@host:5432/db)"
API_KEY = "External API key from https://api.example.com/keys"
JWT_SECRET = "Secret for JWT signing (min 32 characters)"
```

### Use Secrets for Sensitive Data

```toml
[env.secrets]
provider = "auto"  # Detects 1password, vault, etc.
items = ["DATABASE_URL", "API_KEY", "JWT_SECRET"]
```

### Provide Sensible Defaults

```toml
[env]
NODE_ENV = "development"
LOG_LEVEL = "info"
PORT = "3000"

[env.optional]
DEBUG = "Enable debug mode (true/false)"
CACHE_TTL = "Cache TTL in seconds (default: 3600)"
```

## Hooks

### Use pre_setup for Validation

```toml
[hooks]
pre_setup = "node -e \"if(process.version.slice(1,3)<'18'){process.exit(1)}\""
```

### Use post_setup for Initialization

```toml
[hooks]
post_setup = [
  "npm install",
  "vx run db:migrate",
  "vx run seed",
  "echo '✓ Setup complete!'"
]
```

### Use pre_commit for Quality Gates

```toml
[hooks]
pre_commit = "vx run lint && vx run typecheck && vx run test:unit"
```

### Keep Hooks Fast

```toml
# Good - fast checks
[hooks]
pre_commit = "vx run lint:staged"  # Only lint changed files

# Avoid - slow full checks
[hooks]
pre_commit = "vx run lint && vx run test:all"  # Too slow for commits
```

## Services

### Define Health Checks

```toml
[services.database]
image = "postgres:16"
ports = ["5432:5432"]
env = { POSTGRES_PASSWORD = "dev" }
healthcheck = "pg_isready -U postgres"

[services.redis]
image = "redis:7-alpine"
ports = ["6379:6379"]
healthcheck = "redis-cli ping"
```

### Use depends_on for Ordering

```toml
[services.api]
command = "npm run dev"
depends_on = ["database", "redis"]
ports = ["3000:3000"]

[services.worker]
command = "npm run worker"
depends_on = ["database", "redis"]
```

### Separate Dev and Test Services

```toml
# Development database
[services.database]
image = "postgres:16"
ports = ["5432:5432"]
env = { POSTGRES_DB = "myapp_dev" }

# Test database (different port)
[services.database-test]
image = "postgres:16"
ports = ["5433:5432"]
env = { POSTGRES_DB = "myapp_test" }
```

## Dependencies

### Enable Auditing

```toml
[dependencies]
audit = true
lockfile = true
```

### Configure Package Managers

```toml
[dependencies.node]
package_manager = "pnpm"  # Faster, saves disk space
registry = "https://registry.npmmirror.com"  # Mirror for China

[dependencies.python]
index_url = "https://pypi.tuna.tsinghua.edu.cn/simple"
```

### Set License Constraints

```toml
[dependencies.constraints]
"*" = { licenses = ["MIT", "Apache-2.0", "BSD-3-Clause", "ISC"] }
```

## Team Collaboration

### Commit .vx.toml to Git

```bash
# .gitignore - do NOT ignore .vx.toml
# .vx.toml  # Don't add this!

# DO ignore local overrides
.vx.local.toml
```

### Use Consistent Settings

```toml
[settings]
auto_install = true
parallel_install = true
shell = "auto"  # Let vx detect the shell
```

### Document Setup in README

```markdown
## Development Setup

1. Install vx: `curl -fsSL https://get.vx.dev | bash`
2. Setup project: `vx setup`
3. Start development: `vx run dev`
```

## Performance

### Enable Parallel Installation

```toml
[settings]
parallel_install = true
```

### Use Caching

```toml
[settings]
cache_duration = "7d"  # Cache version lookups
```

### Minimize Hook Execution Time

```toml
[hooks]
# Fast - only check what changed
pre_commit = "lint-staged"

# Slow - checks everything
# pre_commit = "npm run lint && npm run test"
```

## Security

### Never Commit Secrets

```toml
# Good - reference env vars
[env.required]
API_KEY = "API key (set in environment)"

# Bad - hardcoded secrets
[env]
API_KEY = "sk-1234567890abcdef"  # NEVER DO THIS
```

### Use Secret Providers

```toml
[env.secrets]
provider = "1password"  # or "vault", "aws-secrets"
items = ["DATABASE_URL", "API_KEY"]
```

### Enable Security Auditing

```toml
[dependencies]
audit = true

[dependencies.constraints]
"lodash" = ">=4.17.21"  # Known security fix
```

## Debugging

### Use Verbose Mode

```bash
vx setup --verbose
vx run dev --verbose
```

### Validate Configuration

```bash
vx config validate
vx config show
```

### Test Without Changes

```bash
vx setup --dry-run
```

## Common Patterns

### Monorepo Setup

```toml
[project]
name = "monorepo"

[settings.experimental]
monorepo = true
workspaces = true

[scripts]
dev:api = { command = "npm run dev", cwd = "packages/api" }
dev:web = { command = "npm run dev", cwd = "packages/web" }
dev:all = "concurrently 'vx run dev:api' 'vx run dev:web'"
```

### Full-Stack Application

```toml
[tools]
node = "20"

[python]
version = "3.12"
venv = ".venv"

[services.database]
image = "postgres:16"
ports = ["5432:5432"]

[services.backend]
command = "python -m uvicorn main:app --reload"
depends_on = ["database"]
ports = ["8000:8000"]
cwd = "backend"

[services.frontend]
command = "npm run dev"
ports = ["3000:3000"]
cwd = "frontend"

[scripts]
dev = "vx services up"
```

### CI/CD Integration

```toml
[scripts]
ci:lint = "npm run lint"
ci:test = "npm run test -- --coverage"
ci:build = "npm run build"

[scripts.ci]
command = "echo 'CI passed'"
depends = ["ci:lint", "ci:test", "ci:build"]
```

## See Also

- [Configuration Reference](/config/vx-toml) - Complete field reference
- [Migration Guide](/guide/migration) - Upgrading from older versions
- [CLI Reference](/cli/overview) - All available commands
