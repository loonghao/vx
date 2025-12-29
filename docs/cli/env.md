# env

Manage vx environments.

## Overview

vx supports two types of environments:

- **Project Environment**: Created in `.vx/env/` under the project directory. This is the default when `vx.toml` exists.
- **Global Environment**: Created in `~/.vx/envs/` for cross-project use.

All tools are stored globally in `~/.vx/store/` (content-addressable storage). Environments contain symlinks to the global store, saving disk space while allowing per-project tool configurations.

## Synopsis

```bash
vx env <SUBCOMMAND> [OPTIONS]
```

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `create` | Create a new environment (project or global) |
| `use` | Activate an environment |
| `list` | List all environments |
| `delete` | Remove an environment |
| `show` | Show environment details |
| `add` | Add a tool to an environment |
| `remove` | Remove a tool from an environment |
| `sync` | Sync project environment from vx.toml |

> **Note**: For shell activation (exporting PATH), use `vx dev --export` instead. See [dev](dev) for details.

## create

Create a new environment.

```bash
vx env create [NAME] [OPTIONS]
```

Options:

- `-g`, `--global` - Create a global environment (requires NAME)
- `--from <ENV>` - Clone from existing environment
- `--set-default` - Set as default after creation

**Project Environment (default):**

When `vx.toml` exists, creates a project-local environment in `.vx/env/`:

```bash
# In a project with vx.toml
vx env create              # Creates .vx/env/
vx env create --from dev   # Clone from global 'dev' environment
```

**Global Environment:**

```bash
vx env create --global my-env
vx env create -g dev --from default
vx env create -g production --set-default
```

## sync

Sync project environment from `vx.toml`. Creates symlinks in `.vx/env/` for all tools defined in the config.

```bash
vx env sync
```

This command:

1. Reads tool versions from `vx.toml`
2. Creates/updates symlinks in `.vx/env/` pointing to `~/.vx/store/`
3. Reports any missing tools that need to be installed

Example:

```bash
# After running 'vx setup' to install tools
vx env sync

# Output:
# Synced 3 tool(s) to project environment
```

## use

Activate an environment.

```bash
vx env use [NAME] [OPTIONS]
```

Options:

- `--global` - Use a global environment

Examples:

```bash
vx env use                  # Use project environment
vx env use --global dev     # Use global 'dev' environment
vx env use my-env           # Use global environment by name
```

## list

List all environments.

```bash
vx env list [OPTIONS]
```

Options:

- `--detailed` - Show detailed information
- `--global` - Show only global environments

Examples:

```bash
vx env list
vx env list --detailed
vx env list --global
```

Output:

```
Project Environment:

* project (active)

Global Environments:

* default (default)
  dev
  production
```

## delete

Remove an environment.

```bash
vx env delete [NAME] [OPTIONS]
```

Options:

- `-g`, `--global` - Delete a global environment
- `--force` - Force deletion without confirmation

Examples:

```bash
vx env delete                    # Delete project environment
vx env delete --global dev       # Delete global 'dev' environment
vx env delete -g old-env --force
```

## show

Show environment details.

```bash
vx env show [NAME]
```

Examples:

```bash
vx env show           # Show project or default environment
vx env show dev       # Show global 'dev' environment
```

Output:

```
Environment: project
Type: project
Path: /path/to/project/.vx/env

Tools:
  node -> /home/user/.vx/store/node/20.0.0
  uv -> /home/user/.vx/store/uv/0.5.14
```

## add

Add a tool to an environment.

```bash
vx env add <TOOL>@<VERSION> [OPTIONS]
```

Options:

- `-g`, `--global` - Add to global environment (requires `--env`)
- `--env <NAME>` - Target global environment name

Examples:

```bash
# Add to project environment (default)
vx env add node@20.0.0
vx env add uv@0.5.14

# Add to global environment
vx env add node@20 --global --env dev
vx env add go@1.21 --env production
```

## remove

Remove a tool from an environment.

```bash
vx env remove <TOOL> [OPTIONS]
```

Options:

- `-g`, `--global` - Remove from global environment
- `--env <NAME>` - Target global environment name

Examples:

```bash
vx env remove node              # Remove from project environment
vx env remove node --global --env dev
```

## Directory Structure

```
~/.vx/
├── store/                    # Global tool storage (content-addressable)
│   ├── node/20.0.0/
│   ├── uv/0.5.14/
│   └── go/1.21.0/
├── envs/                     # Global environments
│   ├── default/
│   │   └── node -> ../../store/node/20.0.0
│   └── dev/
│       ├── node -> ../../store/node/20.0.0
│       └── go -> ../../store/go/1.21.0
└── ...

/path/to/project/
├── vx.toml                  # Project configuration
├── .vx/
│   └── env/                  # Project environment (symlinks)
│       ├── node -> ~/.vx/store/node/20.0.0
│       └── uv -> ~/.vx/store/uv/0.5.14
└── src/
```

## See Also

- [dev](dev) - Enter development environment (includes `--export` for shell activation)
- [setup](setup) - Install project tools
