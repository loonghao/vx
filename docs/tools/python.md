# Python Ecosystem

vx provides Python support through the `uv` package manager.

## Supported Tools

| Tool | Description |
|------|-------------|
| `uv` | Fast Python package manager |
| `uvx` | Python tool runner (uv tool run) |
| `python` | Python interpreter (via uv) |

## uv

[uv](https://github.com/astral-sh/uv) is an extremely fast Python package and project manager.

### Installation

```bash
vx install uv latest
```

### Package Management

```bash
vx uv pip install requests
vx uv pip install -r requirements.txt
vx uv pip list
```

### Virtual Environments

```bash
vx uv venv .venv
vx uv venv .venv --python 3.11
```

### Project Management

```bash
vx uv init
vx uv add requests
vx uv sync
vx uv run python script.py
```

## uvx

uvx runs Python tools without installing them globally.

```bash
vx uvx ruff check .
vx uvx black .
vx uvx mypy src/
vx uvx pytest
vx uvx jupyter notebook
```

## Python

Run Python through uv:

```bash
vx python --version
vx python script.py
vx python -m pytest
```

## Project Configuration

```toml
[tools]
uv = "latest"

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

[scripts]
test = "pytest"
lint = "uvx ruff check ."
format = "uvx black ."
typecheck = "uvx mypy src/"
```

## Common Workflows

### New Python Project

```bash
# Initialize project
vx uv init my-project
cd my-project

# Add dependencies
vx uv add requests pandas

# Run code
vx uv run python main.py
```

### Data Science

```bash
# Start Jupyter
vx uvx jupyter notebook

# Or JupyterLab
vx uvx jupyter lab
```

### Code Quality

```bash
# Lint with ruff
vx uvx ruff check .

# Format with black
vx uvx black .

# Type check with mypy
vx uvx mypy src/
```

### Testing

```bash
# Run pytest
vx uvx pytest

# With coverage
vx uvx pytest --cov=src
```

## Virtual Environment Setup

When `[python]` is configured in `vx.toml`, `vx setup` will:

1. Create the virtual environment
2. Install from requirements files
3. Install listed packages
4. Install git dependencies

```bash
vx setup
# Creates .venv, installs dependencies
```

## Tips

1. **Use uvx for tools**: Run linters, formatters, etc. with `uvx` instead of installing globally
2. **Pin Python version**: Specify version in `vx.toml` for reproducibility
3. **Use uv for speed**: uv is significantly faster than pip
