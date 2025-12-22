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
| `export` | Export environment configuration |
| `import` | Import environment from a file |
| `activate` | Generate shell activation script |

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

Export environment configuration.

```bash
vx env export [NAME] [OPTIONS]
```

Options:

- `-o`, `--output <FILE>` - Output file (defaults to stdout)
- `-f`, `--format <FORMAT>` - Format: toml, json, yaml, shell

Examples:

```bash
vx env export my-env -o my-env.toml
vx env export my-env --format json
vx env export --format shell
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

Generate shell activation script.

```bash
vx env activate [NAME] [OPTIONS]
```

Options:

- `-s`, `--shell <SHELL>` - Shell type: bash, zsh, fish, powershell

Examples:

```bash
vx env activate my-env
vx env activate my-env --shell fish

# To activate:
eval "$(vx env activate my-env)"
```

## See Also

- [Environment Management](../guide/environment-management) - Guide
- [Shell Integration](../guide/shell-integration) - Shell setup
