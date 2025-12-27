# ext - Extension Management

Manage vx extensions.

## Synopsis

```bash
vx ext <SUBCOMMAND>
vx extension <SUBCOMMAND>  # alias
```

## Subcommands

### list

List installed extensions.

```bash
vx ext list
vx ext ls              # alias
vx ext list --verbose  # show detailed info
```

**Options:**

| Option | Description |
|--------|-------------|
| `-v, --verbose` | Show detailed extension information |

**Output:**

```
Extensions:
  docker-compose (v1.0.0) - Docker Compose wrapper [dev]
  scaffold (v2.1.0) - Project scaffolding tool [user]
  lint-all (v1.0.0) - Run all linters [project]
```

### info

Show detailed information about an extension.

```bash
vx ext info <NAME>
vx ext info docker-compose
```

**Output:**

```
Extension: docker-compose
  Version: 1.0.0
  Description: Docker Compose wrapper with vx integration
  Author: Your Name
  Source: dev (~/.vx/extensions-dev/docker-compose)
  Runtime: python >= 3.10
  Commands:
    - up: Start services
    - down: Stop services
    - logs: View logs
```

### dev

Link a local extension for development.

```bash
vx ext dev <PATH>
vx ext dev /path/to/my-extension
vx ext dev . # link current directory
```

**Options:**

| Option | Description |
|--------|-------------|
| `--unlink` | Unlink instead of link |

**Examples:**

```bash
# Link extension for development
vx ext dev ~/projects/my-extension

# Unlink extension
vx ext dev --unlink my-extension
```

### install

Install an extension from a remote source (future feature).

```bash
vx ext install <SOURCE>
vx ext install github:user/repo
vx ext install https://github.com/user/repo
```

### uninstall

Uninstall an extension.

```bash
vx ext uninstall <NAME>
vx ext uninstall my-extension
```

## Extension Execution

Use `vx x` to execute extension commands:

```bash
vx x <EXTENSION> [COMMAND] [ARGS...]
```

**Examples:**

```bash
# Run extension's main entrypoint
vx x docker-compose

# Run a specific command
vx x docker-compose up -d
vx x scaffold create react-app my-app
vx x lint-all --fix
```

## Extension Configuration

Extensions are configured via `vx-extension.toml`:

```toml
[extension]
name = "my-extension"
version = "1.0.0"
description = "My custom extension"
author = "Your Name"
type = "command"  # command, hook, or provider

[runtime]
requires = "python >= 3.10"  # or "node >= 18", "bash", etc.

[entrypoint]
main = "main.py"  # main entry point

[commands.hello]
description = "Say hello"
script = "commands/hello.py"

[commands.build]
description = "Build the project"
script = "commands/build.sh"
```

## Extension Locations

Extensions are discovered from multiple locations with priority:

1. **Dev extensions** (`~/.vx/extensions-dev/`) - Highest priority
2. **Project extensions** (`.vx/extensions/`) - Project-specific
3. **User extensions** (`~/.vx/extensions/`) - User-installed
4. **Builtin extensions** - Shipped with vx

## Environment Variables

Extensions receive these environment variables:

| Variable | Description |
|----------|-------------|
| `VX_VERSION` | Current vx version |
| `VX_EXTENSION_DIR` | Extension's directory |
| `VX_EXTENSION_NAME` | Extension name |
| `VX_PROJECT_DIR` | Current working directory |
| `VX_RUNTIMES_DIR` | vx runtimes directory |
| `VX_HOME` | vx home directory |

## Creating an Extension

1. Create a directory with `vx-extension.toml`
2. Add your scripts
3. Link for development: `vx ext dev /path/to/extension`
4. Test: `vx x my-extension`

**Example structure:**

```
my-extension/
├── vx-extension.toml
├── main.py           # main entrypoint
└── commands/
    ├── hello.py
    └── build.sh
```

## See Also

- [Plugin Development](/advanced/plugin-development) - Creating providers
- [Configuration](/config/vx-toml) - Project configuration
