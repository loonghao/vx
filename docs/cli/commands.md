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

Remove a tool version from global store.

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

Add a tool to project configuration (vx.toml).

```bash
vx add <TOOL>
vx add node
vx add node --version 20
```

### remove

Remove a tool from project configuration (vx.toml).

```bash
vx remove <TOOL>
vx remove node
vx rm node           # rm is an alias for remove
```

## Script Execution

### run

Run a script from `vx.toml`.

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

### cache

Show cache and disk usage statistics.

```bash
vx cache info
```

### self-update

Update vx itself to the latest version with enhanced features:

- **Multi-channel download**: Automatic fallback between GitHub Releases, jsDelivr CDN, and Fastly CDN
- **Progress bar**: Real-time download progress with speed and ETA display
- **Checksum verification**: SHA256 verification for downloaded binaries (when available)
- **Specific version**: Install a specific version instead of latest
- **Safe replacement**: Uses `self_replace` for reliable binary replacement on Windows
- **Backward compatible**: Supports both legacy (v0.5.x) and versioned (v0.6.0+) artifact naming formats
- **Smart version comparison**: Correctly handles version formats like `vx-v0.6.27`, `v0.6.27`, and `0.6.27`

```bash
# Update to latest version
vx self-update

# Check for updates without installing
vx self-update --check

# Force update even if already up to date
vx self-update --force

# Install a specific version
vx self-update 0.5.28

# Include pre-release versions
vx self-update --prerelease

# Use GitHub token to avoid API rate limits
vx self-update --token <GITHUB_TOKEN>
```

Options:

| Option | Description |
|--------|-------------|
| `--check` | Only check for updates, don't install |
| `--force`, `-f` | Force update even if already up to date |
| `--prerelease` | Include pre-release versions |
| `--token <TOKEN>` | GitHub token for authenticated API requests |
| `<VERSION>` | Specific version to install (e.g., `0.5.28`) |

## Extension Management

### ext list

List installed extensions.

```bash
vx ext list
vx ext ls
vx ext list --verbose
```

### ext info

Show extension information.

```bash
vx ext info <NAME>
vx ext info docker-compose
```

### ext dev

Link a local extension for development.

```bash
vx ext dev <PATH>
vx ext dev /path/to/my-extension
vx ext dev --unlink my-extension
```

### ext install

Install an extension from a remote source.

```bash
vx ext install <SOURCE>
vx ext install github:user/repo
```

### ext uninstall

Uninstall an extension.

```bash
vx ext uninstall <NAME>
vx ext uninstall my-extension
```

### x

Execute an extension command.

```bash
vx x <EXTENSION> [COMMAND] [ARGS...]
vx x docker-compose up -d
vx x scaffold create react-app my-app
```
