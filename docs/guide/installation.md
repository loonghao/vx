# Installation

vx can be installed on Windows, macOS, and Linux using various methods.

## Quick Installation

::: code-group

```bash [Linux/macOS]
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
```

```powershell [Windows]
irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex
```

:::

## Package Managers

### Homebrew (macOS/Linux)

```bash
brew tap loonghao/vx
brew install vx
```

Or install directly:

```bash
brew install loonghao/vx/vx
```

### Scoop (Windows)

```powershell
scoop bucket add loonghao https://github.com/loonghao/scoop-bucket
scoop install vx
```

### Cargo (From Source)

If you have Rust installed:

```bash
cargo install vx
```

## Manual Installation

### Download Binary

1. Go to the [Releases page](https://github.com/loonghao/vx/releases)
2. Download the appropriate binary for your platform:
   - `vx-x86_64-unknown-linux-gnu.tar.gz` - Linux x64
   - `vx-aarch64-unknown-linux-gnu.tar.gz` - Linux ARM64
   - `vx-x86_64-apple-darwin.tar.gz` - macOS x64
   - `vx-aarch64-apple-darwin.tar.gz` - macOS ARM64 (Apple Silicon)
   - `vx-x86_64-pc-windows-msvc.zip` - Windows x64

3. Extract and add to PATH:

::: code-group

```bash [Linux/macOS]
tar -xzf vx-*.tar.gz
sudo mv vx /usr/local/bin/
```

```powershell [Windows]
# Extract to a directory in your PATH
Expand-Archive vx-*.zip -DestinationPath $env:USERPROFILE\.vx\bin
# Add to PATH (run in elevated PowerShell)
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$env:USERPROFILE\.vx\bin", "User")
```

:::

## Shell Integration

For the best experience, add shell integration to your profile:

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

## Verify Installation

```bash
vx --version
```

You should see output like:

```
vx 0.5.11
```

## Updating vx

To update vx to the latest version:

```bash
vx self-update
```

Or check for updates without installing:

```bash
vx self-update --check
```

## Uninstalling

::: code-group

```bash [Linux/macOS]
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash -s -- --uninstall
```

```powershell [Windows]
irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex -Uninstall
```

:::

### Manual Uninstall

1. Remove the vx binary from your PATH
2. Remove the vx data directory:
   - Linux/macOS: `~/.local/share/vx`
   - Windows: `%LOCALAPPDATA%\vx`
3. Remove shell integration from your profile
