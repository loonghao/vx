# Extension Development Guide

This guide explains how to create extensions for vx. Extensions allow you to add custom commands and functionality using scripting languages like Python, Shell, or Node.js.

## Overview

vx extensions leverage the runtimes that vx already manages. Your extension scripts can use Python, Node.js, or any other runtime that vx supports, without requiring users to install anything manually.

```
┌─────────────────────────────────────────────────────────────┐
│                    vx Extension System                       │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │   scaffold   │  │docker-compose│  │  my-tool     │  ...  │
│  │  (Python)    │  │   (Shell)    │  │  (Node.js)   │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
│         │                 │                 │                │
│         ▼                 ▼                 ▼                │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              vx Managed Runtimes                     │    │
│  │   python 3.12  │  bash  │  node 20  │  ...          │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## Extension Types

### 1. Command Extensions

Add new CLI commands accessible via `vx x <extension> [subcommand]`:

```bash
vx x docker-compose up
vx x scaffold create react-app my-app
vx x my-tool run --verbose
```

### 2. Hook Extensions (Future)

Execute scripts on specific events:

```toml
[extension]
type = "hook"

[hooks]
pre-install = "check.py"
post-install = "setup.py"
```

## Quick Start

### 1. Create Extension Directory

```bash
mkdir -p ~/.vx/extensions/my-extension
cd ~/.vx/extensions/my-extension
```

### 2. Create Configuration File

Create `vx-extension.toml`:

```toml
[extension]
name = "my-extension"
version = "1.0.0"
description = "My custom extension"
type = "command"

[runtime]
requires = "python >= 3.8"

[entrypoint]
main = "main.py"

[commands.hello]
description = "Say hello"
script = "main.py"
args = ["hello"]

[commands.greet]
description = "Greet someone"
script = "main.py"
args = ["greet"]
```

### 3. Create Script

Create `main.py`:

```python
#!/usr/bin/env python3
import sys
import os

def main():
    args = sys.argv[1:]

    if not args:
        print("Usage: vx x my-extension <hello|greet> [args...]")
        sys.exit(1)

    cmd = args[0]

    if cmd == "hello":
        print("Hello from my extension!")
    elif cmd == "greet":
        name = args[1] if len(args) > 1 else "World"
        print(f"Hello, {name}!")
    else:
        print(f"Unknown command: {cmd}")
        sys.exit(1)

if __name__ == "__main__":
    main()
```

### 4. Test Your Extension

```bash
# List extensions
vx ext list

# Run commands
vx x my-extension hello
vx x my-extension greet Alice
```

## Configuration Reference

### vx-extension.toml

```toml
[extension]
name = "extension-name"           # Required: unique identifier
version = "1.0.0"                 # Required: semver version
description = "Description"       # Required: short description
type = "command"                  # Required: command | hook | provider

[runtime]
requires = "python >= 3.8"        # Required: runtime dependency
# Supported formats:
# - "python >= 3.8"
# - "node >= 18"
# - "bash"

[entrypoint]
main = "main.py"                  # Default script to run
args = ["--config", "config.yaml"] # Default arguments

[commands.subcommand]
description = "Subcommand description"
script = "subcommand.py"          # Script for this subcommand
args = ["--flag"]                 # Additional arguments
```

## Extension Locations

Extensions are loaded from these locations (in priority order):

| Priority | Location | Description |
|----------|----------|-------------|
| 1 (highest) | `~/.vx/extensions-dev/` | Development extensions (symlinks) |
| 2 | `.vx/extensions/` | Project-level extensions |
| 3 | `~/.vx/extensions/` | User-level extensions |

### Development Mode

For active development, use `vx ext dev` to link your extension:

```bash
# Link extension from any directory
vx ext dev /path/to/my-extension

# Unlink when done
vx ext dev --unlink my-extension
```

This creates a symlink in `~/.vx/extensions-dev/`, giving it highest priority.

## Environment Variables

vx injects these environment variables when running extension scripts:

| Variable | Description |
|----------|-------------|
| `VX_VERSION` | Current vx version |
| `VX_EXTENSION_DIR` | Path to the extension directory |
| `VX_EXTENSION_NAME` | Extension name |
| `VX_PROJECT_DIR` | Current project directory (if in a project) |
| `VX_RUNTIMES_DIR` | Path to vx runtimes directory |
| `VX_HOME` | vx home directory (`~/.vx`) |

### Using Environment Variables

```python
#!/usr/bin/env python3
import os
from pathlib import Path

# Get extension directory for loading resources
ext_dir = Path(os.environ.get("VX_EXTENSION_DIR", "."))
templates_dir = ext_dir / "templates"

# Get project directory
project_dir = os.environ.get("VX_PROJECT_DIR")
if project_dir:
    print(f"Running in project: {project_dir}")
```

## Example: Project Scaffolding Extension

A complete example of a scaffolding extension:

### Directory Structure

```
~/.vx/extensions/scaffold/
├── vx-extension.toml
├── main.py
└── templates/
    ├── react-app/
    │   ├── package.json
    │   └── src/
    │       └── index.js
    └── python-cli/
        ├── pyproject.toml
        └── src/
            └── main.py
```

### vx-extension.toml

```toml
[extension]
name = "scaffold"
version = "1.0.0"
description = "Project scaffolding tool"
type = "command"

[runtime]
requires = "python >= 3.8"

[entrypoint]
main = "main.py"

[commands.create]
description = "Create a new project from template"
script = "main.py"
args = ["create"]

[commands.list]
description = "List available templates"
script = "main.py"
args = ["list"]
```

### main.py

```python
#!/usr/bin/env python3
"""Project scaffolding extension for vx."""

import sys
import os
import shutil
from pathlib import Path

def get_templates_dir() -> Path:
    """Get the templates directory."""
    ext_dir = Path(os.environ.get("VX_EXTENSION_DIR", "."))
    return ext_dir / "templates"

def list_templates():
    """List all available templates."""
    templates_dir = get_templates_dir()

    if not templates_dir.exists():
        print("No templates directory found")
        return

    print("Available templates:")
    for template in templates_dir.iterdir():
        if template.is_dir():
            print(f"  - {template.name}")

def create_project(template_name: str, project_name: str):
    """Create a new project from a template."""
    templates_dir = get_templates_dir()
    src = templates_dir / template_name

    if not src.exists():
        print(f"Error: Template '{template_name}' not found")
        print("Available templates:")
        list_templates()
        sys.exit(1)

    dst = Path.cwd() / project_name

    if dst.exists():
        print(f"Error: Directory '{project_name}' already exists")
        sys.exit(1)

    shutil.copytree(src, dst)
    print(f"✓ Created '{project_name}' from template '{template_name}'")
    print(f"  cd {project_name}")

def main():
    args = sys.argv[1:]

    if not args:
        print("Usage: vx x scaffold <create|list> [args...]")
        print("\nCommands:")
        print("  list              List available templates")
        print("  create <t> <n>    Create project <n> from template <t>")
        sys.exit(1)

    cmd = args[0]

    if cmd == "list":
        list_templates()
    elif cmd == "create":
        if len(args) < 3:
            print("Usage: vx x scaffold create <template> <project-name>")
            sys.exit(1)
        create_project(args[1], args[2])
    else:
        print(f"Unknown command: {cmd}")
        sys.exit(1)

if __name__ == "__main__":
    main()
```

### Usage

```bash
# List templates
vx x scaffold list

# Create a new project
vx x scaffold create react-app my-app
vx x scaffold create python-cli my-cli
```

## Example: Docker Compose Extension

An extension for managing Docker Compose services:

### vx-extension.toml

```toml
[extension]
name = "docker-compose"
version = "1.0.0"
description = "Manage Docker Compose services"
type = "command"

[runtime]
requires = "python >= 3.8"

[entrypoint]
main = "main.py"

[commands.up]
description = "Start services"
script = "main.py"
args = ["up"]

[commands.down]
description = "Stop services"
script = "main.py"
args = ["down"]

[commands.logs]
description = "View service logs"
script = "main.py"
args = ["logs"]
```

### main.py

```python
#!/usr/bin/env python3
"""Docker Compose management extension."""

import subprocess
import sys

def run_compose(args: list[str]):
    """Run docker compose with arguments."""
    cmd = ["docker", "compose"] + args
    result = subprocess.run(cmd)
    sys.exit(result.returncode)

def main():
    args = sys.argv[1:]

    if not args:
        print("Usage: vx x docker-compose <up|down|logs> [args...]")
        sys.exit(1)

    cmd = args[0]
    extra_args = args[1:]

    if cmd == "up":
        run_compose(["up", "-d"] + extra_args)
    elif cmd == "down":
        run_compose(["down"] + extra_args)
    elif cmd == "logs":
        run_compose(["logs", "-f"] + extra_args)
    else:
        # Pass through to docker compose
        run_compose(args)

if __name__ == "__main__":
    main()
```

## Best Practices

### 1. Handle Errors Gracefully

```python
import sys

def main():
    try:
        # Your code here
        pass
    except FileNotFoundError as e:
        print(f"Error: File not found - {e}")
        sys.exit(1)
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)
```

### 2. Provide Help Messages

```python
def show_help():
    print("""
Usage: vx x my-extension <command> [options]

Commands:
  create    Create a new item
  list      List all items
  delete    Delete an item

Options:
  -h, --help    Show this help message
  -v, --verbose Enable verbose output
""")
```

### 3. Use Structured Output

For scripts that might be parsed by other tools:

```python
import json

def output_json(data):
    """Output data as JSON for machine parsing."""
    print(json.dumps(data, indent=2))

def output_table(headers, rows):
    """Output data as a formatted table."""
    widths = [max(len(str(cell)) for cell in col)
              for col in zip(headers, *rows)]

    # Print header
    print(" | ".join(h.ljust(w) for h, w in zip(headers, widths)))
    print("-+-".join("-" * w for w in widths))

    # Print rows
    for row in rows:
        print(" | ".join(str(c).ljust(w) for c, w in zip(row, widths)))
```

### 4. Support Configuration Files

```python
import os
from pathlib import Path

def load_config():
    """Load configuration from multiple locations."""
    config_locations = [
        Path.cwd() / ".my-extension.toml",
        Path.home() / ".config" / "my-extension" / "config.toml",
    ]

    for config_path in config_locations:
        if config_path.exists():
            # Load and return config
            pass

    return {}  # Default config
```

## CLI Commands

### Managing Extensions

```bash
# List all installed extensions
vx ext list

# Show extension details
vx ext info <extension-name>

# Link local extension for development
vx ext dev /path/to/extension

# Unlink development extension
vx ext dev --unlink <extension-name>

# Install from remote (future)
vx ext install github:user/vx-ext-name

# Uninstall extension
vx ext uninstall <extension-name>
```

### Running Extension Commands

```bash
# Run extension command
vx x <extension> [subcommand] [args...]

# Examples
vx x scaffold list
vx x scaffold create react-app my-app
vx x docker-compose up
vx x docker-compose logs api
```

## Troubleshooting

### Extension Not Found

```bash
# Check if extension is installed
vx ext list

# Verify extension directory exists
ls ~/.vx/extensions/my-extension/

# Check vx-extension.toml syntax
cat ~/.vx/extensions/my-extension/vx-extension.toml
```

When an extension is not found, vx provides detailed diagnostic information:

```
Extension 'my-extension' not found.

Available extensions:
  - docker-compose
  - scaffold

Searched in:
  - /home/user/.vx/extensions-dev/
  - /home/user/.vx/extensions/
  - /project/.vx/extensions/

To install an extension:
  vx ext install <extension-name>

To create a local extension:
  mkdir -p ~/.vx/extensions/my-extension
  # Create vx-extension.toml in that directory
```

### Subcommand Not Found

When you try to run a subcommand that doesn't exist:

```
Subcommand 'invalid' not found in extension 'docker-compose'.

Available commands:
  vx x docker-compose up
  vx x docker-compose down
  vx x docker-compose logs
```

### No Entrypoint Defined

If your extension has no main entrypoint and you don't specify a subcommand:

```
Extension 'my-ext' has no main entrypoint defined.

Use one of the available commands:
  vx x my-ext build
  vx x my-ext test
```

To fix this, add an entrypoint to your `vx-extension.toml`:

```toml
[entrypoint]
main = "main.py"
```

### Script Not Found

When the script file specified in the configuration doesn't exist:

```
Script 'scripts/run.py' not found for extension 'my-ext'.

Expected at: /home/user/.vx/extensions/my-ext/scripts/run.py

Make sure the script file exists and the path in vx-extension.toml is correct.
```

### Runtime Not Available

```bash
# Check if required runtime is installed
vx list python

# Install the runtime
vx install python 3.12
```

When a required runtime is not installed:

```
Runtime 'python >= 3.10' required by extension 'my-ext' is not available.

Install it with:
  vx install python >= 3.10
```

### Configuration Errors

If your `vx-extension.toml` has syntax errors:

```
Invalid configuration in '/home/user/.vx/extensions/my-ext/vx-extension.toml' at position 15

Error: expected `=`

Tip: Validate your TOML syntax at https://www.toml-lint.com/
```

### Permission Denied

On Unix systems, ensure scripts are executable:

```bash
chmod +x ~/.vx/extensions/my-extension/main.py
```

### Development Link Errors

When trying to unlink an extension that isn't a development link:

```
Extension 'my-ext' at '/home/user/.vx/extensions/my-ext' is not a development link.

Only symlinked extensions (created with 'vx ext dev') can be unlinked.
To remove a regular extension, delete its directory manually.
```

## Error Exit Codes

Extensions should use standard exit codes for consistency:

| Exit Code | Meaning |
|-----------|---------|
| 0 | Success |
| 1 | General error |
| 64 | Usage error (invalid command/arguments) |
| 65 | Data error (invalid configuration) |
| 66 | Input error (file not found) |
| 69 | Unavailable (runtime not installed) |
| 73 | Cannot create (link failed) |
| 74 | IO error |
| 77 | Permission denied |
| 78 | Configuration error |

## Advanced Topics

### Multiple Runtime Support

Extensions can work with different runtimes. Here's how to create a Node.js extension:

```toml
[extension]
name = "npm-scripts"
version = "1.0.0"
description = "Run npm scripts with enhancements"
type = "command"

[runtime]
requires = "node >= 18"

[entrypoint]
main = "index.js"
```

```javascript
#!/usr/bin/env node
// index.js
const { execSync } = require('child_process');

const args = process.argv.slice(2);
const command = args[0];

if (command === 'run') {
    const script = args[1];
    console.log(`Running npm script: ${script}`);
    execSync(`npm run ${script}`, { stdio: 'inherit' });
} else {
    console.log('Usage: vx x npm-scripts run <script-name>');
    process.exit(1);
}
```

### Shell Script Extensions

For simple automation tasks, you can use shell scripts:

```toml
[extension]
name = "git-helpers"
version = "1.0.0"
description = "Git workflow helpers"
type = "command"

[runtime]
requires = "bash"

[commands.sync]
description = "Sync with upstream"
script = "sync.sh"

[commands.cleanup]
description = "Clean up merged branches"
script = "cleanup.sh"
```

```bash
#!/bin/bash
# sync.sh
git fetch upstream
git rebase upstream/main
git push origin main
```

### Extension Dependencies

If your extension needs Python packages, document them:

```toml
[extension]
name = "api-client"
version = "1.0.0"
type = "command"

[runtime]
requires = "python >= 3.10"
dependencies = ["requests", "pyyaml", "rich"]

[entrypoint]
main = "main.py"
```

Users should install dependencies before using:

```bash
# Using uv (recommended)
vx uv pip install requests pyyaml rich

# Or using pip
vx pip install requests pyyaml rich
```

### Testing Extensions

Create a test script for your extension:

```python
#!/usr/bin/env python3
# test_extension.py
import subprocess
import sys

def test_list_command():
    result = subprocess.run(
        ["vx", "x", "my-extension", "list"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0
    assert "Available" in result.stdout

def test_invalid_command():
    result = subprocess.run(
        ["vx", "x", "my-extension", "invalid"],
        capture_output=True,
        text=True
    )
    assert result.returncode != 0

if __name__ == "__main__":
    test_list_command()
    test_invalid_command()
    print("All tests passed!")
```

### Publishing Extensions

While vx doesn't yet have a central registry, you can share extensions via Git:

```bash
# Create a repository for your extension
cd ~/.vx/extensions/my-extension
git init
git add .
git commit -m "Initial commit"
git remote add origin https://github.com/user/vx-ext-my-extension
git push -u origin main
```

Others can install by cloning:

```bash
git clone https://github.com/user/vx-ext-my-extension ~/.vx/extensions/my-extension
```

## API Reference

### ExtensionConfig Structure

The complete configuration schema:

```toml
[extension]
name = "string"              # Required: unique identifier (kebab-case)
version = "string"           # Semver version (default: "0.1.0")
description = "string"       # Short description
type = "command|hook|provider"  # Extension type (default: "command")
authors = ["string"]         # List of authors
license = "string"           # SPDX license identifier

[runtime]
requires = "string"          # Runtime requirement (e.g., "python >= 3.10")
dependencies = ["string"]    # Package dependencies

[entrypoint]
main = "string"              # Main script file
args = ["string"]            # Default arguments

[commands.<name>]
description = "string"       # Command description
script = "string"            # Script file to execute
args = ["string"]            # Default arguments for this command

[hooks]
<hook-name> = "string"       # Hook script mappings
```

### Environment Variables Reference

| Variable | Type | Description |
|----------|------|-------------|
| `VX_VERSION` | String | Current vx version (e.g., "0.5.26") |
| `VX_EXTENSION_DIR` | Path | Absolute path to extension directory |
| `VX_EXTENSION_NAME` | String | Extension name from config |
| `VX_PROJECT_DIR` | Path | Current working directory |
| `VX_RUNTIMES_DIR` | Path | Path to `~/.vx/store/` |
| `VX_HOME` | Path | Path to `~/.vx/` |

### Error Types

The extension system provides detailed error diagnostics:

| Error Type | Exit Code | Description |
|------------|-----------|-------------|
| `ConfigNotFound` | 64 | vx-extension.toml not found |
| `ConfigInvalid` | 65 | TOML syntax error |
| `ConfigMissingField` | 65 | Required field missing |
| `ExtensionNotFound` | 66 | Extension not in any search path |
| `DuplicateExtension` | 65 | Same name in multiple locations |
| `SubcommandNotFound` | 64 | Unknown subcommand |
| `NoEntrypoint` | 78 | No main script defined |
| `ScriptNotFound` | 66 | Script file doesn't exist |
| `RuntimeNotAvailable` | 69 | Required runtime not installed |
| `ExecutionFailed` | varies | Script returned non-zero |
| `LinkFailed` | 73 | Failed to create symlink |
| `NotADevLink` | 64 | Cannot unlink non-symlink |
| `Io` | 74 | File system error |
| `PermissionDenied` | 77 | Insufficient permissions |

## See Also

- [CLI Reference: ext command](/cli/ext)
- [Provider Development Guide](./plugin-development)
