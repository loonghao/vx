# dev

Enter development environment with all project tools.

## Synopsis

```bash
vx dev [OPTIONS]
vx dev -c <COMMAND>
vx dev --export [--format <FORMAT>]
```

## Description

The `vx dev` command is the primary way to work with vx-managed tools. It provides three modes of operation:

1. **Interactive Shell Mode** (default): Spawns a new shell with all project tools available in PATH
2. **Command Mode** (`-c`): Runs a single command in the dev environment
3. **Export Mode** (`--export`): Outputs shell script for environment activation

## Options

| Option | Description |
|--------|-------------|
| `--shell <SHELL>` | Shell to use (auto-detected if not specified) |
| `-c`, `--command <CMD>` | Run a command instead of spawning a shell |
| `--no-install` | Don't install missing tools |
| `-v`, `--verbose` | Show verbose output |
| `-e`, `--export` | Export environment variables for shell activation |
| `-f`, `--format <FORMAT>` | Output format for `--export`: shell, powershell, batch, github |

## Usage Scenarios

### Scenario 1: Interactive Development

Enter a development shell with all tools available:

```bash
vx dev
```

Output:

```
✓ Entering vx development environment
ℹ Tools: node, uv, go

💡 Type 'exit' to leave the dev environment.

(vx) $ node --version
v20.10.0
(vx) $ exit
ℹ Left vx development environment
```

**When to use**: Daily development work, exploring the codebase, running ad-hoc commands.

### Scenario 2: Run Single Command

Execute a command in the dev environment without entering a shell:

```bash
vx dev -c "npm run build"
vx dev -c "node scripts/deploy.js"
vx dev -c "go test ./..."
```

**When to use**: CI/CD pipelines, scripts, one-off tasks.

### Scenario 3: Shell Activation (Export Mode)

Export environment variables to activate tools in your current shell:

**Bash/Zsh:**

```bash
eval "$(vx dev --export)"
```

**Fish:**

```fish
vx dev --export | source
```

**PowerShell:**

```powershell
Invoke-Expression (vx dev --export --format powershell)
```

**Windows CMD:**

```batch
vx dev --export --format batch > activate.bat && activate.bat
```

**When to use**: IDE integration, shell profiles, custom scripts.

### Scenario 4: CI/CD Integration

Use export mode with GitHub Actions format:

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install vx
        run: curl -fsSL https://get.vx.dev | bash

      - name: Setup tools
        run: vx setup

      - name: Activate environment
        run: |
          eval "$(vx dev --export --format github)"
          # Tools are now available
          node --version
          npm run build
```

**When to use**: GitHub Actions, GitLab CI, Jenkins, etc.

## Export Formats

| Format | Description | Default On |
|--------|-------------|------------|
| `shell` | Bash/Zsh compatible | Unix |
| `powershell` | PowerShell compatible | Windows (PowerShell) |
| `batch` | Windows CMD batch | Windows (CMD) |
| `github` | GitHub Actions format | GitHub Actions |

### Example Output

For a project with `node` and `uv` configured:

**Shell format:**

```bash
export PATH="/home/user/.vx/bin:/home/user/.vx/store/node/20.0.0/bin:/home/user/.vx/store/uv/0.5.14:$PATH"
```

**PowerShell format:**

```powershell
$env:PATH = "C:\Users\user\.vx\bin;C:\Users\user\.vx\store\node\20.0.0;C:\Users\user\.vx\store\uv\0.5.14;$env:PATH"
```

**GitHub Actions format:**

```bash
echo "/home/runner/.vx/bin" >> $GITHUB_PATH
echo "/home/runner/.vx/store/node/20.0.0/bin" >> $GITHUB_PATH
echo "/home/runner/.vx/store/uv/0.5.14" >> $GITHUB_PATH
export PATH="/home/runner/.vx/bin:/home/runner/.vx/store/node/20.0.0/bin:/home/runner/.vx/store/uv/0.5.14:$PATH"
```

## Environment Setup

When you enter the dev environment (interactive or command mode):

1. **PATH is updated** with project tool versions from `~/.vx/store/`
2. **VX_DEV=1** is set to indicate active environment
3. **VX_PROJECT_ROOT** is set to the project directory
4. **Custom environment variables** from `.vx.toml` `[env]` section are set
5. **Missing tools are auto-installed** (unless `--no-install`)

## Configuration

The dev environment is configured by `.vx.toml`:

```toml
[tools]
node = "20"
uv = "latest"
go = "1.21"

[env]
NODE_ENV = "development"
DEBUG = "true"

[settings]
auto_install = true
```

## Comparison with vx run

| Feature | `vx dev` | `vx run` |
|---------|----------|----------|
| Purpose | Development environment | Run defined scripts |
| Scope | All tools in PATH | Script-specific |
| Interactive | Yes (shell mode) | No |
| Scripts | Any command | Only from `.vx.toml` |

**Use `vx dev`** for:

- Interactive development
- Ad-hoc commands
- Shell activation

**Use `vx run`** for:

- Predefined project scripts
- Consistent task execution
- Team workflows

## Tips

1. **Add to shell profile**: For auto-activation in new terminals:

   ```bash
   # ~/.bashrc or ~/.zshrc
   if [ -f ".vx.toml" ]; then
     eval "$(vx dev --export)"
   fi
   ```

2. **IDE Integration**: Configure your IDE's terminal to run `eval "$(vx dev --export)"` on startup.

3. **Check environment**: Run `echo $PATH` to verify tool paths are included.

4. **Specify shell**: If auto-detection fails, use `--shell`:

   ```bash
   vx dev --shell zsh
   vx dev --shell fish
   ```

## See Also

- [setup](setup) - Install project tools
- [run](../cli/commands#run) - Run defined scripts
- [env](env) - Environment management
