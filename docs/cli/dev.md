# dev

Enter development environment with all project tools.

## Synopsis

```bash
vx dev [OPTIONS]
```

## Description

Spawns a new shell with the project's development environment fully configured:

- All tools from `.vx.toml` available in PATH
- Python virtual environment activated
- Environment variables set
- Working directory preserved

## Options

| Option | Description |
|--------|-------------|
| `--shell <SHELL>` | Shell to use (auto-detected if not specified) |
| `-c`, `--command <CMD>` | Run a command instead of spawning a shell |
| `--no-install` | Don't install missing tools |
| `-v`, `--verbose` | Show verbose output |

## Examples

### Enter Dev Environment

```bash
vx dev
```

Output:

```
🚀 Entering development environment...

Tools available:
  �?node@20.10.0
  �?uv@0.1.24
  �?go@1.21.5

Python venv: .venv (activated)

Type 'exit' to leave the development environment.
```

### Run Single Command

```bash
vx dev -c "npm run build"
```

### Specify Shell

```bash
vx dev --shell zsh
vx dev --shell fish
```

### Skip Tool Installation

```bash
vx dev --no-install
```

## Environment Setup

When you enter the dev environment:

1. **PATH is updated** with project tool versions
2. **Python venv is activated** if configured
3. **Environment variables** from `.vx.toml` are set
4. **Shell prompt** may indicate active environment

## Configuration

The dev environment is configured by `.vx.toml`:

```toml
[tools]
node = "20"
uv = "latest"

[python]
version = "3.11"
venv = ".venv"

[env]
NODE_ENV = "development"
DEBUG = "true"
```

## Exiting

To leave the development environment:

```bash
exit
# or press Ctrl+D
```

## Tips

1. **Use for interactive work**: Best for exploring, debugging, running ad-hoc commands
2. **Use `vx run` for scripts**: For defined tasks, use `vx run` instead
3. **Check environment**: Run `echo $PATH` to verify tool paths

## See Also

- [run](run) - Run defined scripts
- [setup](setup) - Install project tools
- [env](env) - Environment management
