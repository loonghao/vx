# Commands Reference

This page provides a quick reference for all vx commands.

## Tool Management

### install

Install a specific tool version.

```bash
vx install <TOOL> [VERSION]
vx install node 20
vx install uv latest
vx install go 1.21.5 --force
```

### list

List supported tools.

```bash
vx list
vx list --status
vx list --all              # Include unsupported platforms
vx list node
```

### uninstall

Remove a tool version.

```bash
vx uninstall <TOOL> [VERSION]
vx uninstall node 18
vx uninstall node --force
```

### which

Show which tool version is being used.

```bash
vx which <TOOL>
vx which node
vx which node --all
```

### versions

Show available versions for a tool.

```bash
vx versions <TOOL>
vx versions node --latest 10
vx versions node --prerelease
```

### switch

Switch to a different version.

```bash
vx switch <TOOL>@<VERSION>
vx switch node@18
vx switch node@20 --global
```

## Project Management

### init

Initialize project configuration.

```bash
vx init
vx init -i                    # Interactive
vx init --template nodejs
vx init --tools node,uv
```

### setup

Install all project tools.

```bash
vx setup
vx setup --dry-run
vx setup --force
vx setup --verbose
```

### sync

Sync tools with configuration.

```bash
vx sync
vx sync --check
vx sync --force
```

### add

Add a tool to project.

```bash
vx add <TOOL>
vx add node
vx add node --version 20
```

### rm-tool

Remove a tool from project.

```bash
vx rm-tool <TOOL>
vx rm-tool node
```

## Script Execution

### run

Run a script from `.vx.toml`.

```bash
vx run <SCRIPT> [ARGS]
vx run dev
vx run test -- --coverage
```

### dev

Enter development environment.

```bash
vx dev
vx dev --shell zsh
vx dev -c "npm run build"
```

## Environment Management

### env

Manage environments.

```bash
vx env create <NAME>
vx env list
vx env use <NAME>
vx env delete <NAME>
vx env show [NAME]
vx env add <TOOL>@<VERSION>
vx env remove <TOOL>
vx env export [NAME]
vx env import <FILE>
vx env activate [NAME]
```

### global

Manage global tool versions.

```bash
vx global list
vx global set <TOOL> <VERSION>
vx global unset <TOOL>
```

### venv

Python virtual environment management.

```bash
vx venv create [PATH]
vx venv activate [PATH]
vx venv list
```

## Configuration

### config

Manage configuration.

```bash
vx config show
vx config set <KEY> <VALUE>
vx config get <KEY>
vx config reset [KEY]
vx config edit
```

### shell

Shell integration.

```bash
vx shell init [SHELL]
vx shell completions <SHELL>
```

## Maintenance

### clean

Clean up system.

```bash
vx clean
vx clean --cache
vx clean --orphaned
vx clean --all
vx clean --dry-run
```

### stats

Show disk usage statistics.

```bash
vx stats
```

### self-update

Update vx itself.

```bash
vx self-update
vx self-update --check
vx self-update --force
```
