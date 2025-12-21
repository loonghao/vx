# vx Installation Guide

This guide covers all the ways to install vx on different platforms and package managers.

## 🚀 Quick Install

### One-line Install Scripts

**Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
```

### Advanced Installation Options

**Install specific version:**
```bash
# Linux/macOS
VX_VERSION="0.1.0" curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash

# Windows
$env:VX_VERSION="0.1.0"; powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
```

**Install to custom directory:**
```bash
# Linux/macOS
VX_INSTALL_DIR="$HOME/bin" curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash

# Windows
$env:VX_INSTALL_DIR="C:\tools\vx"; powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
```

**Build from source:**
```bash
# Linux/macOS
BUILD_FROM_SOURCE=true curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash

# Windows (in vx repository directory)
powershell -c ".\install.ps1 -BuildFromSource"
```

## 📦 Package Managers

### Windows

#### Winget (Recommended)
```powershell
winget install Hal.vx
```

#### Chocolatey
```powershell
choco install vx
```

#### Scoop
```powershell
scoop bucket add loonghao https://github.com/loonghao/scoop-bucket
scoop install vx
```

### macOS

#### Homebrew (Recommended)
```bash
brew tap loonghao/tap
brew install vx
```

### Linux

#### Arch Linux (AUR)
```bash
# Using yay
yay -S vx-bin

# Using paru
paru -S vx-bin
```

#### Debian/Ubuntu (APT)
```bash
# Add repository (coming soon)
curl -fsSL https://github.com/loonghao/vx/releases/latest/download/vx_Linux_x86_64.deb -o vx.deb
sudo dpkg -i vx.deb
```

#### RedHat/CentOS/Fedora (RPM)
```bash
# Download and install RPM (coming soon)
curl -fsSL https://github.com/loonghao/vx/releases/latest/download/vx_Linux_x86_64.rpm -o vx.rpm
sudo rpm -i vx.rpm
```

#### Alpine Linux (APK)
```bash
# Coming soon to Alpine repositories
```

## 🐳 Container Images

### Docker
```bash
# Run vx in a container
docker run --rm -it ghcr.io/loonghao/vx:latest vx --help

# Use as base image
FROM ghcr.io/loonghao/vx:latest
```

### Podman
```bash
podman run --rm -it ghcr.io/loonghao/vx:latest vx --help
```

## 📥 Direct Download

### GitHub Releases

Download the latest release for your platform from:
https://github.com/loonghao/vx/releases/latest

#### Available Platforms:
- **Linux**: x86_64, aarch64 (ARM64)
- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon)
- **Windows**: x86_64
- **FreeBSD**: x86_64

#### PGO Optimized Builds
For better performance, choose files ending with `_pgo` (Profile-Guided Optimization):
- `vx_Linux_x86_64_pgo.tar.gz` (Recommended for Linux)
- `vx_Darwin_x86_64_pgo.tar.gz` (Recommended for macOS Intel)
- `vx_Windows_x86_64_pgo.zip` (Recommended for Windows)

### Manual Installation

1. Download the appropriate archive for your platform
2. Extract the archive:
   ```bash
   # Linux/macOS
   tar -xzf vx_*.tar.gz
   
   # Windows
   # Extract using Windows Explorer or PowerShell
   Expand-Archive vx_*.zip
   ```
3. Move the binary to a directory in your PATH:
   ```bash
   # Linux/macOS
   sudo mv vx /usr/local/bin/
   
   # Windows (PowerShell as Administrator)
   Move-Item vx.exe "C:\Program Files\vx\vx.exe"
   ```

## 🔧 Build from Source

### Prerequisites
- Rust 1.83.0 or later
- Git

### Build Steps
```bash
# Clone the repository
git clone https://github.com/loonghao/vx.git
cd vx

# Build with optimizations
cargo build --release

# Install to ~/.cargo/bin (make sure it's in your PATH)
cargo install --path .
```

### Development Build
```bash
# Quick development build
cargo build

# Run tests
cargo test

# Run with cargo
cargo run -- --help
```

## 🌐 CI/CD Integration

### GitHub Actions
```yaml
- name: Install vx
  run: |
    curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
    echo "$HOME/.vx/bin" >> $GITHUB_PATH
```

### GitLab CI
```yaml
before_script:
  - curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
  - export PATH="$HOME/.vx/bin:$PATH"
```

### Azure Pipelines
```yaml
- script: |
    curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
    echo "##vso[task.prependpath]$(HOME)/.vx/bin"
  displayName: 'Install vx'
```

## 🔍 Verification

After installation, verify that vx is working:

```bash
# Check version
vx --version

# List available tools
vx list

# Get help
vx --help
```

## 🆘 Troubleshooting

### Command not found
If you get "command not found" after installation:

1. **Restart your terminal** or source your shell profile:
   ```bash
   # Bash
   source ~/.bashrc
   
   # Zsh
   source ~/.zshrc
   
   # Fish
   source ~/.config/fish/config.fish
   ```

2. **Check PATH**: Ensure the installation directory is in your PATH:
   ```bash
   echo $PATH
   ```

3. **Manual PATH update**:
   ```bash
   # Add to your shell profile
   echo 'export PATH="$HOME/.vx/bin:$PATH"' >> ~/.bashrc
   ```

### Permission Issues
If you encounter permission issues:

```bash
# Linux/macOS: Use sudo for system-wide installation
sudo install.sh --install-dir /usr/local/bin

# Or install to user directory (no sudo required)
install.sh --install-dir ~/.local/bin
```

### Package Manager Issues
- **Winget**: Ensure you have the latest version of App Installer
- **Homebrew**: Run `brew update` before installing
- **Chocolatey**: Run PowerShell as Administrator
- **AUR**: Ensure you have an AUR helper installed

## 📚 Next Steps

After successful installation:

1. **Configure vx**: Run `vx config` to set up your preferences
2. **Install tools**: Use `vx install <tool>` to install development tools
3. **Explore features**: Check out the [User Guide](USER_GUIDE.md)
4. **Join the community**: Visit our [GitHub Discussions](https://github.com/loonghao/vx/discussions)

## 🤝 Contributing

Found an issue with installation? Please:
1. Check existing [issues](https://github.com/loonghao/vx/issues)
2. Create a new issue with your platform details
3. Consider contributing a fix via pull request

---

**Need help?** Join our community:
- 💬 [GitHub Discussions](https://github.com/loonghao/vx/discussions)
- 🐛 [Report Issues](https://github.com/loonghao/vx/issues)
- 📖 [Documentation](https://github.com/loonghao/vx)
