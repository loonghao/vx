# Python Ecosystem

vx provides comprehensive Python support through both standalone Python runtime and the `uv` package manager.

## Supported Tools

| Tool | Description |
|------|-------------|
| `python` | Python interpreter (via python-build-standalone; Python 2.7 uses PyPy2.7 legacy compatibility builds) |
| `uv` | Fast Python package manager |
| `uvx` | Python tool runner (uv tool run) |

## Python Runtime

vx uses [python-build-standalone](https://github.com/astral-sh/python-build-standalone) from Astral for portable Python distributions. For Python 2.7, vx uses official PyPy2.7 portable archives because python-build-standalone does not publish CPython 2.7 builds. Supports **Python 2.7 and Python 3.7 to 3.13+**.

### Version Support Status

| Version | Status | Notes |
|---------|--------|-------|
| Python 3.13+ | Active | Latest features |
| Python 3.12 | Active | Recommended for production |
| Python 3.11 | Active | Stable |
| Python 3.10 | Active | Stable |
| Python 3.9 | EOL | Last build: 20251120 |
| Python 3.8 | EOL | Limited availability |
| Python 3.7 | EOL | Legacy support only |
| Python 2.7 | EOL | Legacy compatibility via PyPy2.7 |

> **Note**: Python versions that have reached End-of-Life (EOL) may have limited availability. Python 2.7 is intended for legacy test and migration workflows; CPython-specific native extensions may require a system CPython 2.7 installation.

### Installation

```bash
# Install latest Python
vx install python@latest

# Install specific version
vx install python@3.12.8
vx install python@3.11.11
vx install python@3.10.16
vx install python@3.9.21
vx install python@3.8.20
vx install python@3.7.9
vx install python@2.7

# List available versions
vx list python
```

### Running Python

```bash
vx python --version
vx python script.py
vx python -m pytest
```

> **Recommendation**: For pure Python development, we recommend using `uv` instead of managing Python directly. `uv` provides faster package installation, built-in virtual environment management, and automatic Python version management.

## uv (Recommended)

[uv](https://github.com/astral-sh/uv) is an extremely fast Python package and project manager. **We strongly recommend using uv for Python development** as it provides:

- 10-100x faster package installation than pip
- Built-in virtual environment management
- Automatic Python version management
- Modern project management with `pyproject.toml`

### Installation

```bash
vx install uv@latest
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
vx uv venv .venv37 --python 3.7
vx uv venv .venv27 --python 2.7
```

When `uv` receives a simple version through `vx uv ... --python <version>`, vx resolves that version with the vx Python provider first and passes the installed interpreter path to uv. Python 2.7 is a special legacy case: uv itself requires Python 3.6+, so `vx uv venv ... --python 2.7` creates the environment with PyPA's Python 2.7 `virtualenv.pyz` while preserving the same vx command shape.

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

### New Python Project (Recommended)

```bash
# Initialize project with uv
vx uv init my-project
cd my-project

# Add dependencies
vx uv add requests pandas

# Run code
vx uv run python main.py
```

### Using Standalone Python

```bash
# Install Python directly
vx install python@3.12.8

# Run Python
vx python --version
vx python script.py
```

### Legacy Multi-Python Testing

Use separate virtual environments per Python line and keep the commands in `justfile`:

```makefile
venv37:
    vx uv venv .venv37 --python 3.7
    vx uv pip install --python .venv37 -r requirements-py37.txt

venv27:
    vx uv venv .venv27 --python 2.7
    .venv27/bin/python -m pip install -r requirements-py27.txt

test37: venv37
    .venv37/bin/python -m pytest

test27: venv27
    .venv27/bin/python -m pytest

test-legacy: test37 test27
```

On Windows, use `.venv37\Scripts\python.exe` and `.venv27\Scripts\python.exe` in the `justfile` commands.

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

1. **Use uv for development**: uv provides the best Python development experience with automatic version management and fast package installation
2. **Use uvx for tools**: Run linters, formatters, etc. with `uvx` instead of installing globally
3. **Pin Python version**: Specify version in `vx.toml` for reproducibility
4. **Use standalone Python for specific needs**: When you need a specific Python version without uv's management
