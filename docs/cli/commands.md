# Commands Reference

Complete quick reference for all vx commands. Click any command name for detailed documentation.

## Tool Management

### install

Install tool versions. Alias: `i`

```bash
vx install <TOOL>[@VERSION] [OPTIONS]
vx install node@22                    # Install Node.js 22
vx install python@3.12 uv@latest     # Install multiple tools
vx install "node@^22"                # Semver range
vx install node@lts                  # LTS version
vx install go@1.23 --force           # Force reinstall
```

[Full documentation →](./install)

### list

List installed tools and available runtimes. Alias: `ls`

```bash
vx list                    # List all known runtimes
vx list --installed        # Only show installed tools
vx list --status           # Show installation status
vx list node               # Show versions for a specific tool
```

[Full documentation →](./list)

### uninstall

Remove an installed tool version.

```bash
vx uninstall node@18       # Remove specific version
vx uninstall node --all    # Remove all versions
```

### which / where

Show path of the currently active tool version.

```bash
vx which node              # /home/user/.vx/store/node/22.11.0/bin/node
vx which python            # Show active Python path
```

### versions

Show available versions for a tool.

```bash
vx versions node           # List all available Node.js versions
vx versions python         # List available Python versions
```

### switch

Switch to a different installed version.

```bash
vx switch node 20          # Switch to Node.js 20
vx switch python 3.11      # Switch to Python 3.11
```

### search

Search for available tools.

```bash
vx search lint             # Search for linting tools
vx search python           # Search for Python-related tools
```

### test

Test runtime availability and provider functionality. CI-friendly.

```bash
vx test node               # Test Node.js availability
vx test --all              # Test all providers
vx test --all --json       # JSON output for CI
```

[Full documentation →](./test)

### global

Manage globally installed packages with full ecosystem isolation. Alias: `g`

```bash
vx global install typescript       # Install globally
vx global install pip:httpie       # Install with ecosystem prefix
vx global list                     # List global packages
vx global uninstall typescript     # Uninstall
```

[Full documentation →](./global)

---

## Project Management

### init

Initialize a new `vx.toml` configuration for the current project.

```bash
vx init                    # Interactive initialization
vx init --detect           # Auto-detect project tools
```

### add

Add a tool requirement to `vx.toml`.

```bash
vx add node@22             # Add Node.js 22
vx add python@3.12         # Add Python 3.12
```

### remove

Remove a tool from `vx.toml`. Alias: `rm`

```bash
vx remove node             # Remove Node.js requirement
```

### sync

Sync installed tools with `vx.toml` requirements.

```bash
vx sync                    # Install missing, remove extra tools
```

### lock

Generate or update `vx.lock` for reproducible environments.

```bash
vx lock                    # Generate lock file
vx lock --update           # Update lock file
```

### check

Check version constraints and tool availability.

```bash
vx check                   # Verify all tools meet constraints
```

### bundle

Offline development environment packaging.

```bash
vx bundle create           # Create offline bundle from vx.lock
vx bundle export           # Export as portable archive
vx bundle import pkg.tar.gz # Import from archive
vx bundle status           # Show bundle status
```

### analyze

Analyze project dependencies, scripts, and required tools.

```bash
vx analyze                 # Analyze current project
```

---

## Scripts & Environment

### run

Run scripts defined in `vx.toml` with enhanced argument passing and variable interpolation.

```bash
vx run dev                 # Run 'dev' script
vx run test -- --coverage  # Pass args to script
vx run --list              # List available scripts
vx run test -H             # Show script help
```

[Full documentation →](./run)

### dev

Enter an isolated development environment with all project tools.

```bash
vx dev                     # Interactive shell
vx dev -c "node -v"       # Run single command
vx dev --export --format github  # Export for CI
vx dev --info              # Show environment info
```

[Full documentation →](./dev)

### setup

Install all project tools and run setup hooks.

```bash
vx setup                   # Install all project tools
vx setup --force           # Force reinstall
vx setup --dry-run         # Preview without installing
```

[Full documentation →](./setup)

### env

Manage project and global virtual environments.

```bash
vx env create my-env --node=22 --python=3.12
vx env use my-env          # Activate environment
vx env list                # List all environments
vx env show                # Show current environment
vx env delete my-env       # Delete environment
vx env sync                # Sync with vx.toml
```

[Full documentation →](./env)

---

## Configuration & Shell

### config

Manage global and project configuration. Alias: `cfg`

```bash
vx config show             # Show current config
vx config init             # Initialize vx.toml
vx config set key value    # Set config value
vx config get key          # Get config value
vx config validate         # Validate vx.toml
vx config edit             # Open config in editor
vx config schema           # Generate JSON Schema
```

[Full documentation →](./config)

### shell

Shell integration for auto-switching and completions.

```bash
vx shell init bash         # Generate bash init script
vx shell init zsh          # Generate zsh init script
vx shell completions bash  # Generate completions
```

[Full documentation →](./shell)

---

## Extensions & Plugins

### ext

Manage vx extensions. Alias: `extension`

```bash
vx ext list                # List installed extensions
vx ext install <URL>       # Install from repository
vx ext dev <PATH>          # Link local extension for dev
vx ext info <NAME>         # Show extension details
vx ext update              # Update all extensions
vx ext uninstall <NAME>    # Remove extension
```

[Full documentation →](./ext)

### x

Execute extension commands.

```bash
vx x my-extension          # Run extension default command
vx x my-ext cmd --arg      # Run specific subcommand
```

### plugin

Manage provider plugins.

```bash
vx plugin list             # List plugins
vx plugin info <NAME>      # Show plugin details
vx plugin enable <NAME>    # Enable a plugin
vx plugin disable <NAME>   # Disable a plugin
vx plugin search <QUERY>   # Search for plugins
vx plugin stats            # Plugin statistics
```

[Full documentation →](./plugin)

---

## System & Maintenance

### info

Show system information, capabilities, and diagnostics.

```bash
vx info                    # Human-readable info
vx info --json             # JSON output (for scripts/AI)
vx info --warnings         # Show build diagnostics
```

[Full documentation →](./info)

### metrics

View execution performance metrics and reports.

```bash
vx metrics                 # Show performance metrics
vx metrics --json          # JSON format
```

[Full documentation →](./metrics)

### cache

Manage the download and version cache.

```bash
vx cache info              # Show cache statistics
vx cache list              # List cached entries
vx cache prune             # Safe cleanup of expired entries
vx cache purge             # Remove all cache (destructive)
vx cache dir               # Show cache directory path
```

### self-update

Update vx to the latest version.

```bash
vx self-update             # Update to latest
vx self-update --check     # Check for updates
```

### version

Show vx version information.

```bash
vx version                 # Show version
vx --version               # Short form
```

### hook

Manage lifecycle hooks.

```bash
vx hook status             # Show hook status
vx hook run pre-commit     # Run specific hook
vx hook install            # Install hooks
```

### services

Manage development services.

```bash
vx services start          # Start all services
vx services stop           # Stop all services
vx services status         # Service status
vx services logs           # View logs
```

### container

Container and Dockerfile management.

```bash
vx container generate      # Generate Dockerfile
vx container build         # Build container
vx container push          # Push to registry
```

### auth

Authentication management.

```bash
vx auth login              # Authenticate
vx auth logout             # Logout
vx auth status             # Show auth status
```

### migrate

Migrate configuration and data from older formats.

```bash
vx migrate                 # Run migration
```

---

## Implicit Package Execution

For detailed information on running packages without explicit installation, see [Implicit Package Execution](./implicit-package-execution).
