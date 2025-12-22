# shell

Shell integration commands.

## Synopsis

```bash
vx shell <SUBCOMMAND> [OPTIONS]
```

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `init` | Generate shell initialization script |
| `completions` | Generate shell completion script |

## init

Generate shell initialization script.

```bash
vx shell init [SHELL]
```

Arguments:

- `SHELL` - Shell type (auto-detected if not specified)

Supported shells:

- `bash`
- `zsh`
- `fish`
- `powershell`

### Setup

**Bash** - Add to `~/.bashrc`:

```bash
eval "$(vx shell init bash)"
```

**Zsh** - Add to `~/.zshrc`:

```zsh
eval "$(vx shell init zsh)"
```

**Fish** - Add to `~/.config/fish/config.fish`:

```fish
vx shell init fish | source
```

**PowerShell** - Add to `$PROFILE`:

```powershell
Invoke-Expression (& vx shell init powershell | Out-String)
```

## completions

Generate shell completion script.

```bash
vx shell completions <SHELL>
```

Arguments:

- `SHELL` - Shell type (required)

### Completions Setup

**Bash**:

```bash
vx shell completions bash > ~/.local/share/bash-completion/completions/vx
```

**Zsh**:

```zsh
vx shell completions zsh > ~/.zfunc/_vx
# Ensure ~/.zfunc is in fpath
```

**Fish**:

```fish
vx shell completions fish > ~/.config/fish/completions/vx.fish
```

**PowerShell**:

```powershell
vx shell completions powershell | Out-File ~\Documents\PowerShell\Completions\vx.ps1
```

## What Shell Integration Provides

1. **PATH Configuration**: vx-managed tools are added to PATH
2. **Auto-Switching**: Environment switches when entering project directories
3. **Completions**: Tab completion for commands and options

## See Also

- [Shell Integration Guide](../guide/shell-integration)
