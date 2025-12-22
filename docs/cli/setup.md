# setup

Install all project tools from `.vx.toml`.

## Synopsis

```bash
vx setup [OPTIONS]
```

## Description

Reads the project's `.vx.toml` configuration and:

1. Installs all required tools
2. Sets up Python virtual environment
3. Installs Python dependencies
4. Verifies environment variables

## Options

| Option | Description |
|--------|-------------|
| `-f`, `--force` | Force reinstall all tools |
| `--dry-run` | Preview operations without executing |
| `-v`, `--verbose` | Show verbose output |
| `--no-parallel` | Disable parallel installation |

## Examples

### Basic Setup

```bash
vx setup
```

Output:

```
🚀 VX Development Environment Setup

Project: my-project
  A sample project

📦 Phase 1: Installing tools...

Tools:
  �?node@20 (installed)
  �?uv@latest (missing)
  �?go@1.21 (installed)

Installing 1 tool(s)...
  �?uv@0.1.24

🐍 Phase 2: Setting up Python environment...

  �?Created .venv
  �?Installed from requirements.txt
  �?Installed pytest, black

🔐 Phase 3: Environment variables...

Missing required environment variables:
  API_KEY - Your API key

Hint: Set these variables in your shell or create a .env file

🎉 Setup completed in 12.3s

Next steps:
  1. Activate Python environment:
     source .venv/bin/activate
  2. Or use vx dev to enter the full dev environment

Available scripts:
  vx run dev -> npm run dev
  vx run test -> pytest
```

### Dry Run

```bash
vx setup --dry-run
```

Shows what would be done without making changes.

### Force Reinstall

```bash
vx setup --force
```

Reinstalls all tools even if already installed.

### Verbose Output

```bash
vx setup --verbose
```

Shows detailed progress and debugging information.

## Configuration

Setup reads from `.vx.toml`:

```toml
[project]
name = "my-project"
description = "A sample project"

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
git = ["https://github.com/user/repo.git"]

[env.required]
API_KEY = "Your API key"

[settings]
auto_install = true
parallel_install = true
```

## Phases

### Phase 1: Tool Installation

- Checks which tools are installed
- Installs missing tools
- Supports parallel installation

### Phase 2: Python Environment

- Creates virtual environment
- Installs from requirements files
- Installs listed packages
- Installs git dependencies

### Phase 3: Environment Variables

- Checks required variables
- Reports missing variables
- Provides hints for setup

## See Also

- [init](../cli/commands#init) - Initialize project configuration
- [sync](../cli/commands#sync) - Sync tools with configuration
- [dev](dev) - Enter development environment
