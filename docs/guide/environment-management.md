# Environment Management

vx allows you to create and manage multiple isolated environments for different projects or purposes.

## Understanding Environments

An environment is a collection of tool versions that work together. Each environment is isolated, so you can have:

- Node.js 18 in one environment
- Node.js 20 in another
- Different Go versions for different projects

## Listing Environments

```bash
vx env list
```

Output:

```
Environments:

* default (active)
  project-a
  project-b
```

For detailed information:

```bash
vx env list --detailed
```

## Creating Environments

### Basic Creation

```bash
vx env create my-env
```

### Clone from Existing

```bash
vx env create new-env --from existing-env
```

### Set as Default

```bash
vx env create my-env --set-default
```

## Adding Tools to Environments

```bash
# Add to current environment
vx env add node@20

# Add to specific environment
vx env add node@20 --env my-env
vx env add go@1.21 --env my-env
vx env add uv@latest --env my-env
```

## Removing Tools from Environments

```bash
vx env remove node
vx env remove node --env my-env
```

## Switching Environments

### Temporary Switch

```bash
vx env use my-env
```

This prints activation instructions for your shell.

### Set as Global Default

```bash
vx env use my-env --global
```

## Showing Environment Details

```bash
# Show current environment
vx env show

# Show specific environment
vx env show my-env
```

Output:

```
Environment: my-env
Path: /home/user/.local/share/vx/envs/my-env
Active: yes

Runtimes:
  node -> /home/user/.local/share/vx/store/node/20.10.0
  go -> /home/user/.local/share/vx/store/go/1.21.5
```

## Exporting Environments

Export an environment configuration for sharing:

```bash
# Export to TOML (default)
vx env export my-env -o my-env.toml

# Export to JSON
vx env export my-env -o my-env.json --format json

# Export to YAML
vx env export my-env -o my-env.yaml --format yaml

# Export shell script
vx env export my-env --format shell
```

### Export Format

```toml
name = "my-env"
exported_at = "2024-01-15T10:30:00Z"

[runtimes]
node = "20.10.0"
go = "1.21.5"
uv = "0.1.24"
```

## Importing Environments

Import an environment from a file:

```bash
# Import with same name
vx env import my-env.toml

# Import with different name
vx env import my-env.toml --name new-env

# Force overwrite existing
vx env import my-env.toml --force
```

This will:

1. Create the environment
2. Install any missing tools
3. Add tools to the environment

## Activating Environments

Generate shell activation scripts:

::: code-group

```bash [Bash/Zsh]
eval "$(vx env activate my-env)"
```

```fish [Fish]
vx env activate my-env --shell fish | source
```

```powershell [PowerShell]
Invoke-Expression (vx env activate my-env --shell powershell)
```

:::

## Deleting Environments

```bash
# With confirmation
vx env delete my-env

# Force delete
vx env delete my-env --force
```

::: warning
You cannot delete the `default` environment.
:::

## Environment Variables

When an environment is active, these variables are set:

| Variable | Description |
|----------|-------------|
| `VX_ENV` | Current environment name |
| `VX_ENV_DIR` | Path to environment directory |
| `PATH` | Updated to include environment tools |

## Use Cases

### Per-Project Environments

```bash
# Create environment for each project
vx env create project-a
vx env add node@18 --env project-a

vx env create project-b
vx env add node@20 --env project-b

# Switch when working on different projects
cd project-a && vx env use project-a
cd project-b && vx env use project-b
```

### Testing Different Versions

```bash
# Test code with different Node versions
vx env create test-node18
vx env add node@18 --env test-node18

vx env create test-node20
vx env add node@20 --env test-node20

# Run tests in each
eval "$(vx env activate test-node18)"
npm test

eval "$(vx env activate test-node20)"
npm test
```

### Sharing Team Environments

```bash
# Export team environment
vx env export team-env -o team-env.toml

# Share the file (git, email, etc.)

# Team members import
vx env import team-env.toml
```

## Best Practices

1. **Use descriptive names**: `project-name` or `purpose-version`
2. **Export before major changes**: Backup your environment
3. **Use project configs**: For project-specific environments, prefer `vx.toml`
4. **Clean up unused environments**: `vx env delete old-env`

## Next Steps

- [Shell Integration](/guide/shell-integration) - Automatic environment activation
- [CLI Reference](/cli/env) - Complete env command reference
