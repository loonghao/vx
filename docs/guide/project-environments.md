# Project Environments

For team projects, vx supports project-specific tool configurations through `.vx.toml` files.

## Creating a Project Configuration

### Interactive Mode

```bash
vx init -i
```

This guides you through setting up:

- Project metadata
- Tool versions
- Scripts
- Environment variables

### Using Templates

```bash
# List available templates
vx init --list-templates

# Use a template
vx init --template nodejs
vx init --template python
vx init --template fullstack
```

### Manual Creation

Create a `.vx.toml` file:

```toml
[project]
name = "my-project"
description = "A sample project"

[tools]
node = "20"
uv = "latest"

[scripts]
dev = "npm run dev"
test = "npm test"
```

## Setting Up the Environment

After creating `.vx.toml`, run:

```bash
vx setup
```

This will:

1. Install all required tools
2. Set up Python virtual environment (if configured)
3. Install dependencies
4. Verify environment variables

### Setup Options

```bash
# Dry run - show what would be done
vx setup --dry-run

# Force reinstall all tools
vx setup --force

# Verbose output
vx setup --verbose

# Sequential installation (no parallelism)
vx setup --no-parallel
```

## Running Scripts

Define scripts in `.vx.toml`:

```toml
[scripts]
dev = "npm run dev"
test = "pytest"
build = "go build -o app"
lint = "npm run lint && uvx ruff check ."

[scripts.start]
command = "python main.py"
description = "Start the server"
args = ["--host", "0.0.0.0", "--port", "8080"]
env = { DEBUG = "true" }
cwd = "src"
```

Run scripts with:

```bash
vx run dev
vx run test
vx run start
```

Pass additional arguments:

```bash
vx run test -- -v --coverage
```

## Development Environment

Enter a development shell with all tools available:

```bash
vx dev
```

This:

1. Activates the project environment
2. Sets up PATH with project tools
3. Activates Python venv (if configured)
4. Sets environment variables
5. Spawns a new shell

### Running Commands in Dev Environment

```bash
# Run a single command
vx dev -c "npm run build"

# Specify shell
vx dev --shell zsh
```

## Python Projects

For Python projects, configure the virtual environment:

```toml
[python]
version = "3.11"
venv = ".venv"

[python.dependencies]
requirements = ["requirements.txt"]
packages = ["pytest", "black", "ruff"]
git = [
    "https://github.com/user/repo.git",
]
dev = ["pytest", "mypy"]
```

When you run `vx setup`:

1. Creates `.venv` with Python 3.11
2. Installs from `requirements.txt`
3. Installs listed packages
4. Installs git dependencies

## Environment Variables

### Static Variables

```toml
[env]
NODE_ENV = "development"
DEBUG = "true"
```

### Required Variables

```toml
[env.required]
API_KEY = "Your API key for the service"
DATABASE_URL = "Database connection string"
```

Required variables must be set before running scripts. vx will warn if they're missing.

### Optional Variables

```toml
[env.optional]
CACHE_DIR = "Optional cache directory"
LOG_LEVEL = "Logging level (default: info)"
```

## Managing Tools

### Add a Tool

```bash
vx add node
vx add node --version 18
```

### Remove a Tool

```bash
vx rm-tool node
```

### Update Tools

Edit `.vx.toml` and run:

```bash
vx setup
```

## Syncing with Team

When you clone a project with `.vx.toml`:

```bash
git clone https://github.com/team/project
cd project
vx setup  # Installs all required tools
```

Check if environment is in sync:

```bash
vx sync --check
```

## Best Practices

::: tip Commit .vx.toml
Share tool versions with your team by committing `.vx.toml` to version control.
:::

::: tip Use Specific Versions
Avoid "latest" for reproducibility. Use specific versions like `node = "20.10"`.
:::

::: tip Document Required Variables
Use `[env.required]` with descriptions to help team members set up correctly.
:::

## Example: Full-Stack Project

```toml
[project]
name = "fullstack-app"
description = "A full-stack web application"

[tools]
node = "20"
uv = "latest"
go = "1.21"

[python]
version = "3.11"
venv = ".venv"

[python.dependencies]
requirements = ["backend/requirements.txt"]

[env]
NODE_ENV = "development"

[env.required]
DATABASE_URL = "PostgreSQL connection string"
JWT_SECRET = "Secret for JWT tokens"

[scripts]
# Frontend
frontend = "cd frontend && npm run dev"
frontend-build = "cd frontend && npm run build"

# Backend
backend = "cd backend && python main.py"
migrate = "cd backend && python manage.py migrate"

# Full stack
dev = "concurrently 'vx run frontend' 'vx run backend'"
test = "vx run test-frontend && vx run test-backend"
test-frontend = "cd frontend && npm test"
test-backend = "cd backend && pytest"
```

## Next Steps

- [Environment Management](/guide/environment-management) - Managing multiple environments
- [.vx.toml Reference](/config/vx-toml) - Complete configuration reference
