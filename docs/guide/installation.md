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

### Troubleshooting: GitHub API Rate Limit

If you encounter a rate limit error during installation, you have several options:

**Option 1: Use a GitHub token**
```bash
# Linux/macOS
GITHUB_TOKEN='your_token' curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash

# Windows
$env:GITHUB_TOKEN='your_token'; irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex
```

**Option 2: Specify version explicitly**
```bash
# Linux/macOS
VX_VERSION='0.6.7' curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash

# Windows
$env:VX_VERSION='0.6.7'; irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex
```

**Option 3: Use package managers** (see below)

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

### WinGet (Windows)

```powershell
winget install loonghao.vx
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
   - `vx-x86_64-unknown-linux-musl.tar.gz` - Linux x64 (static)
   - `vx-aarch64-unknown-linux-musl.tar.gz` - Linux ARM64 (static)
   - `vx-x86_64-apple-darwin.tar.gz` - macOS x64
   - `vx-aarch64-apple-darwin.tar.gz` - macOS ARM64 (Apple Silicon)
   - `vx-x86_64-pc-windows-msvc.zip` - Windows x64

   > **Note:** Starting from v0.7.0, vx uses [cargo-dist](https://opensource.axo.dev/cargo-dist/) for releases. Binary names no longer include the version number (e.g., `vx-x86_64-pc-windows-msvc.zip` instead of `vx-0.7.0-x86_64-pc-windows-msvc.zip`). Both naming formats are supported during download for backward compatibility.

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
vx 0.6.27
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

Install a specific version:

```bash
vx self-update 0.7.7
```

Force reinstall (useful for recovering from corrupted installations):

```bash
vx self-update --force
```

The self-update command features:
- **Automatic update method detection**: Uses cargo-dist install receipts when available for zero-config updates, with multi-channel CDN fallback for legacy installations
- Multi-channel download with automatic fallback (GitHub Releases → jsDelivr CDN → Fastly CDN)
- Download progress bar with speed and ETA
- SHA256 checksum verification (when available)
- Safe binary replacement on Windows (handles exe locking)
- Cross-version compatibility: can update from any older version (v0.5.x, v0.6.x) to latest

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
