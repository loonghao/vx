# env

Manage vx environments.

## Synopsis

```bash
vx env <SUBCOMMAND> [OPTIONS]
```

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `create` | Create a new environment |
| `use` | Activate an environment |
| `list` | List all environments |
| `delete` | Remove an environment |
| `show` | Show environment details |
| `add` | Add a runtime to an environment |
| `remove` | Remove a runtime from an environment |
| `export` | Export environment variables for shell activation |
| `import` | Import environment from a file |
| `activate` | Alias for `export` |

## create

Create a new environment.

```bash
vx env create <NAME> [OPTIONS]
```

Options:

- `--from <ENV>` - Clone from existing environment
- `--set-default` - Set as default after creation

Examples:

```bash
vx env create my-env
vx env create new-env --from existing-env
vx env create my-env --set-default
```

## use

Activate an environment.

```bash
vx env use <NAME> [OPTIONS]
```

Options:

- `--global` - Set as global default

Examples:

```bash
vx env use my-env
vx env use my-env --global
```

## list

List all environments.

```bash
vx env list [OPTIONS]
```

Options:

- `--detailed` - Show detailed information

Examples:

```bash
vx env list
vx env list --detailed
```

## delete

Remove an environment.

```bash
vx env delete <NAME> [OPTIONS]
```

Options:

- `--force` - Force deletion without confirmation

Examples:

```bash
vx env delete my-env
vx env delete my-env --force
```

## show

Show environment details.

```bash
vx env show [NAME]
```

Examples:

```bash
vx env show           # Show current environment
vx env show my-env    # Show specific environment
```

## add

Add a runtime to an environment.

```bash
vx env add <RUNTIME>@<VERSION> [OPTIONS]
```

Options:

- `--env <NAME>` - Target environment (defaults to current)

Examples:

```bash
vx env add node@20
vx env add go@1.21 --env my-env
```

## remove

Remove a runtime from an environment.

```bash
vx env remove <RUNTIME> [OPTIONS]
```

Options:

- `--env <NAME>` - Target environment (defaults to current)

Examples:

```bash
vx env remove node
vx env remove node --env my-env
```

## export

Export environment variables for shell activation. This command reads the `.vx.toml` configuration in the current directory and generates shell scripts that set up PATH to include all configured tools.

```bash
vx env export [OPTIONS]
```

Options:

- `-f`, `--format <FORMAT>` - Output format (auto-detected if not specified):
  - `shell` - Bash/Zsh compatible (default on Unix)
  - `powershell` - PowerShell compatible (default on Windows)
  - `batch` - Windows CMD batch file
  - `github` - GitHub Actions format (appends to `$GITHUB_PATH`)

### Shell Activation

The export command is designed to work like Python's virtual environment activation. Use `eval` to activate the environment in your current shell:

**Bash/Zsh:**

```bash
eval "$(vx env export)"
```

**Fish:**

```fish
vx env export | source
```

**PowerShell:**

```powershell
Invoke-Expression (vx env export --format powershell)
```

**Windows CMD:**

```batch
vx env export --format batch > activate.bat && activate.bat
```

### GitHub Actions Integration

For CI/CD pipelines, use the `github` format to automatically add tool paths to `$GITHUB_PATH`:

```yaml
- name: Setup vx environment
  run: |
    if [ -f ".vx.toml" ]; then
      vx env export --format github >> $GITHUB_PATH
    fi
```

### How It Works

1. Reads `.vx.toml` from the current directory
2. Resolves all configured tools to their installation paths in `~/.vx/store/`
3. Generates shell commands to prepend these paths to `PATH`

### Example Output

For a project with `uv` and `node` configured:

**Shell format:**

```bash
# vx environment activation
# Generated from: /path/to/project/.vx.toml
export PATH="/home/user/.vx/store/uv/0.5.14:/home/user/.vx/store/node/22.12.0/bin:$PATH"
```

**PowerShell format:**

```powershell
# vx environment activation
# Generated from: C:\path\to\project\.vx.toml
$env:PATH = "C:\Users\user\.vx\store\uv\0.5.14;C:\Users\user\.vx\store\node\22.12.0;$env:PATH"
```

**GitHub Actions format:**

```
/home/runner/.vx/store/uv/0.5.14
/home/runner/.vx/store/node/22.12.0/bin
```

### Use Cases

1. **Shell sessions**: Activate tools for interactive development
2. **CI/CD**: Ensure tools are available in subsequent workflow steps
3. **Scripts**: Source the activation before running project scripts
4. **IDE integration**: Configure terminal profiles to auto-activate

Examples:

```bash
# Activate in current shell
eval "$(vx env export)"

# Check which format will be used
vx env export --format shell

# Use in CI
vx env export --format github >> $GITHUB_PATH
```

## import

Import environment from a file.

```bash
vx env import <FILE> [OPTIONS]
```

Options:

- `-n`, `--name <NAME>` - Environment name (defaults to name in file)
- `-f`, `--force` - Force overwrite if exists

Examples:

```bash
vx env import my-env.toml
vx env import my-env.toml --name new-env
vx env import my-env.toml --force
```

## activate

Alias for `export`. Generate shell activation script.

```bash
vx env activate [OPTIONS]
```

This is equivalent to `vx env export`. See [export](#export) for full documentation.

Examples:

```bash
# Activate in current shell
eval "$(vx env activate)"

# With specific format
vx env activate --format powershell
```

## See Also

- [Environment Management](../guide/environment-management) - Guide
- [Shell Integration](../guide/shell-integration) - Shell setup
