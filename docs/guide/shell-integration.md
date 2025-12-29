# Shell Integration

Shell integration provides automatic environment activation and command completion.

## Setting Up Shell Integration

::: code-group

```bash [Bash]
# Add to ~/.bashrc
eval "$(vx shell init bash)"
```

```zsh [Zsh]
# Add to ~/.zshrc
eval "$(vx shell init zsh)"
```

```fish [Fish]
# Add to ~/.config/fish/config.fish
vx shell init fish | source
```

```powershell [PowerShell]
# Add to $PROFILE
Invoke-Expression (& vx shell init powershell | Out-String)
```

:::

## What Shell Integration Provides

### 1. Automatic PATH Setup

Your PATH is configured to find vx-managed tools:

```bash
# Without shell integration
/usr/bin/node  # System node

# With shell integration
~/.local/share/vx/envs/default/node  # vx-managed node
```

### 2. Directory-Based Environment Switching

When you `cd` into a directory with `vx.toml`, the environment automatically activates:

```bash
cd my-project/  # Activates project environment
cd ~            # Returns to default environment
```

### 3. Command Completion

Tab completion for vx commands and options:

```bash
vx ins<TAB>     # Completes to "vx install"
vx install no<TAB>  # Completes to "vx install node"
```

## Shell Completions

Generate completion scripts separately:

::: code-group

```bash [Bash]
vx shell completions bash > ~/.local/share/bash-completion/completions/vx
```

```zsh [Zsh]
vx shell completions zsh > ~/.zfunc/_vx
```

```fish [Fish]
vx shell completions fish > ~/.config/fish/completions/vx.fish
```

```powershell [PowerShell]
vx shell completions powershell > ~\Documents\PowerShell\Completions\vx.ps1
```

:::

## Environment Indicators

With shell integration, your prompt can show the active environment:

::: code-group

```bash [Bash]
PS1='$([ -n "$VX_ENV" ] && echo "[$VX_ENV] ")'"$PS1"
```

```zsh [Zsh]
PROMPT='%F{cyan}${VX_ENV:+[$VX_ENV] }%f'"$PROMPT"
```

```fish [Fish]
function fish_prompt
    if set -q VX_ENV
        echo -n "[$VX_ENV] "
    end
    # ... rest of prompt
end
```

```powershell [PowerShell]
function prompt {
    if ($env:VX_ENV) {
        Write-Host "[$env:VX_ENV] " -NoNewline -ForegroundColor Cyan
    }
    # ... rest of prompt
}
```

:::

## Manual Environment Activation

If you prefer manual control:

```bash
# Activate specific environment
eval "$(vx env activate my-env)"

# Deactivate (restore default)
eval "$(vx env activate default)"
```

## Troubleshooting

### Shell Integration Not Working

1. Verify installation:

   ```bash
   vx shell init bash  # Should output shell script
   ```

2. Check if sourced correctly:

   ```bash
   source ~/.bashrc
   echo $PATH | grep vx  # Should show vx paths
   ```

3. Restart your terminal

### Completions Not Working

1. Verify completion script exists:

   ```bash
   ls ~/.local/share/bash-completion/completions/vx
   ```

2. For Zsh, ensure `compinit` is called:

   ```zsh
   autoload -Uz compinit && compinit
   ```

### Slow Shell Startup

If shell startup is slow:

1. Use lazy loading:

   ```bash
   # Bash - lazy load vx
   vx() {
       unset -f vx
       eval "$(command vx shell init bash)"
       vx "$@"
   }
   ```

2. Cache the init script:

   ```bash
   # Generate once
   vx shell init bash > ~/.vx-init.sh

   # Source cached version
   source ~/.vx-init.sh
   ```

## Advanced Configuration

### Disable Auto-Switching

If you don't want automatic environment switching:

```bash
export VX_AUTO_SWITCH=false
eval "$(vx shell init bash)"
```

### Custom Hook

Add custom behavior when environment changes:

```bash
# Bash
vx_env_changed() {
    echo "Switched to environment: $VX_ENV"
    # Custom logic here
}
```

## Next Steps

- [CLI Reference](/cli/shell) - Shell command reference
- [Environment Management](/guide/environment-management) - Managing environments
