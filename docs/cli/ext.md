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

Install an extension from a remote source.

```bash
vx ext install <SOURCE>
```

**Supported Sources:**

| Format | Example |
|--------|---------|
| GitHub shorthand | `github:user/repo` |
| GitHub shorthand with version | `github:user/repo@v1.0.0` |
| GitHub HTTPS URL | `https://github.com/user/repo` |
| GitHub SSH URL | `git@github.com:user/repo.git` |

**Examples:**

```bash
# Install from GitHub
vx ext install github:user/vx-ext-docker

# Install specific version
vx ext install github:user/vx-ext-docker@v1.0.0

# Install from HTTPS URL
vx ext install https://github.com/user/vx-ext-docker
```

### uninstall

Uninstall an extension.

```bash
vx ext uninstall <NAME>
vx ext uninstall my-extension
```

### update

Update installed extensions.

```bash
vx ext update <NAME>
vx ext update --all
```

**Options:**

| Option | Description |
|--------|-------------|
| `--all` | Update all installed extensions |

**Examples:**

```bash
# Update specific extension
vx ext update docker-compose

# Update all extensions
vx ext update --all
```

### check

Check for extension updates.

```bash
vx ext check <NAME>
vx ext check --all
```

**Options:**

| Option | Description |
|--------|-------------|
| `--all` | Check all installed extensions |

**Examples:**

```bash
# Check specific extension
vx ext check docker-compose

# Check all extensions
vx ext check --all
```

**Output:**

```
Updates Available:
  docker-compose: 1.0.0 -> 1.1.0
  scaffold: 2.0.0 -> 2.1.0

Run 'vx ext update --all' to update all extensions
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
authors = ["Your Name"]
type = "command"  # command, hook, or provider

[runtime]
requires = "python >= 3.10"  # or "node >= 18", "bash", etc.
dependencies = ["requests", "pyyaml"]  # runtime dependencies

[entrypoint]
main = "main.py"  # main entry point
args = ["--config", "config.yaml"]  # default arguments

[commands.hello]
description = "Say hello"
script = "commands/hello.py"

[commands.build]
description = "Build the project"
script = "commands/build.sh"
args = ["--production"]

# Hook extension type
[hooks]
pre-install = "hooks/pre-install.py"
post-install = "hooks/post-install.py"
pre-run = "hooks/pre-run.sh"
post-run = "hooks/post-run.sh"
```

## Extension Types

### Command Extensions

Provide new CLI commands via `vx x <extension>`:

```toml
[extension]
name = "docker-compose"
type = "command"

[commands.up]
description = "Start services"
script = "up.py"
```

### Hook Extensions

Execute at specific lifecycle events:

```toml
[extension]
name = "pre-commit-check"
type = "hook"

[hooks]
pre-install = "check.py"
post-install = "setup.py"
pre-run = "validate.sh"
```

**Available Hook Events:**

| Event | Description |
|-------|-------------|
| `pre-install` | Before installing a runtime |
| `post-install` | After installing a runtime |
| `pre-uninstall` | Before uninstalling a runtime |
| `post-uninstall` | After uninstalling a runtime |
| `pre-run` | Before running a command |
| `post-run` | After running a command |
| `enter-project` | When entering a project directory |
| `leave-project` | When leaving a project directory |

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

**Hook-specific variables:**

| Variable | Description |
|----------|-------------|
| `VX_HOOK_EVENT` | The hook event being triggered |
| `VX_HOOK_RUNTIME` | Runtime name (for install/uninstall hooks) |
| `VX_HOOK_VERSION` | Runtime version (for install/uninstall hooks) |
| `VX_HOOK_COMMAND` | Command being run (for pre/post-run hooks) |
| `VX_HOOK_ARGS` | Command arguments |
| `VX_HOOK_PROJECT_DIR` | Project directory |

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
├── commands/
│   ├── hello.py
│   └── build.sh
└── hooks/
    ├── pre-install.py
    └── post-install.py
```

## Publishing Extensions

To publish your extension:

1. Create a GitHub repository
2. Add `vx-extension.toml` to the root
3. Tag releases with semantic versions (e.g., `v1.0.0`)
4. Users can install with: `vx ext install github:user/repo`

## See Also

- [Extension Development](/advanced/extension-development) - Detailed extension development guide
- [Plugin Development](/advanced/plugin-development) - Creating providers
- [Configuration](/config/vx-toml) - Project configuration
