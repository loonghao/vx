# vx-installer

[![Crates.io](https://img.shields.io/crates/v/vx-installer.svg)](https://crates.io/crates/vx-installer)
[![Documentation](https://docs.rs/vx-installer/badge.svg)](https://docs.rs/vx-installer)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Installation utilities and helpers for the vx universal tool manager.

## Status

🚧 **Under Development** - This crate is currently under development and not yet implemented.

## Overview

`vx-installer` will provide installation utilities and helpers for the vx ecosystem, including installation scripts, system integration, and deployment tools.

## Planned Features

- **Installation Scripts**: Cross-platform installation scripts
- **System Integration**: Shell integration and PATH management
- **Auto-updater**: Automatic vx updates and maintenance
- **Deployment Tools**: Tools for deploying vx in different environments
- **Configuration Migration**: Migrate from other tool managers
- **Uninstallation**: Clean removal of vx and its components

## Planned Components

### Installation Scripts
```bash
# Cross-platform installation (planned)
curl -fsSL https://vx.sh/install.sh | bash    # Unix
iwr -useb https://vx.sh/install.ps1 | iex     # Windows
```

### System Integration
```bash
# Shell integration setup (planned)
vx-installer setup-shell bash
vx-installer setup-shell zsh
vx-installer setup-shell fish
vx-installer setup-shell powershell
```

### Migration Tools
```bash
# Migrate from other tool managers (planned)
vx-installer migrate --from nvm
vx-installer migrate --from rustup
vx-installer migrate --from pyenv
```

### Maintenance
```bash
# Update and maintenance (planned)
vx-installer update
vx-installer check-health
vx-installer repair
vx-installer uninstall
```

## Current Status

This crate is currently in the planning phase. Installation is currently handled through:
- Manual installation scripts in the main repository
- Cargo installation for development builds
- Package managers (planned)

## Development Roadmap

1. **Phase 1**: Basic installation scripts and system integration
2. **Phase 2**: Auto-updater and maintenance tools
3. **Phase 3**: Migration tools from other tool managers
4. **Phase 4**: Advanced deployment and enterprise features

## Installation Methods (Planned)

### Script Installation
```bash
# One-line installation (planned)
curl -fsSL https://get.vx.sh | bash

# With options
curl -fsSL https://get.vx.sh | bash -s -- --version latest
curl -fsSL https://get.vx.sh | bash -s -- --prefix /usr/local
```

### Package Managers
```bash
# Package manager installation (planned)
brew install vx                    # macOS
choco install vx                   # Windows
apt install vx                     # Ubuntu/Debian
yum install vx                     # RHEL/CentOS
```

### Manual Installation
```bash
# Manual installation (current)
cargo install vx-cli
```

## System Integration (Planned)

### Shell Setup
```bash
# Automatic shell integration (planned)
vx-installer setup-shell --auto

# Manual shell setup
echo 'eval "$(vx shell-init)"' >> ~/.bashrc
echo 'eval "$(vx shell-init)"' >> ~/.zshrc
```

### PATH Management
```bash
# PATH configuration (planned)
vx-installer configure-path
vx-installer verify-path
vx-installer fix-path
```

### Environment Variables
```bash
# Environment setup (planned)
export VX_HOME="$HOME/.vx"
export PATH="$VX_HOME/bin:$PATH"
```

## Migration Support (Planned)

### From NVM
```bash
# Migrate Node.js versions from nvm (planned)
vx-installer migrate --from nvm
# Detects ~/.nvm and migrates installed Node.js versions
```

### From Rustup
```bash
# Migrate Rust toolchains from rustup (planned)
vx-installer migrate --from rustup
# Detects ~/.rustup and migrates installed toolchains
```

### From Pyenv
```bash
# Migrate Python versions from pyenv (planned)
vx-installer migrate --from pyenv
# Detects ~/.pyenv and migrates installed Python versions
```

## Configuration (Planned)

### Installation Configuration
```toml
# ~/.vx/installer.toml (planned)
[installation]
prefix = "/usr/local"
create_symlinks = true
update_shell_config = true

[migration]
backup_existing = true
preserve_configs = true
```

### Update Configuration
```toml
# Auto-update settings (planned)
[updates]
auto_check = true
auto_install = false
channel = "stable"
check_interval = "24h"
```

## Contributing

This crate is not yet implemented. If you're interested in contributing to installation and deployment tools for vx, please:

1. Check the main project [issues](https://github.com/loonghao/vx/issues)
2. Join the discussion about installation and deployment
3. See the [contributing guidelines](../../CONTRIBUTING.md)

## Current Installation

While this crate is under development, you can install vx using:

### From Source
```bash
git clone https://github.com/loonghao/vx
cd vx
cargo install --path .
```

### From Crates.io
```bash
cargo install vx-cli
```

### Manual Setup
```bash
# Add to shell configuration
echo 'eval "$(vx shell-init)"' >> ~/.bashrc
source ~/.bashrc
```

## Planned Architecture

### Installation Flow
1. **Download**: Fetch vx binary for target platform
2. **Verification**: Verify checksums and signatures
3. **Installation**: Install to appropriate directory
4. **Integration**: Set up shell integration
5. **Verification**: Verify installation success

### Update Flow
1. **Check**: Check for available updates
2. **Download**: Download new version
3. **Backup**: Backup current installation
4. **Replace**: Replace with new version
5. **Verify**: Verify update success

## Security (Planned)

### Download Security
- **HTTPS Only**: Secure downloads from official sources
- **Checksum Verification**: SHA256 verification of downloads
- **Signature Validation**: GPG signature verification
- **Reproducible Builds**: Verifiable build process

### Installation Security
- **Permission Checks**: Verify installation permissions
- **Path Validation**: Validate installation paths
- **Backup Creation**: Backup existing installations
- **Rollback Support**: Rollback failed installations

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## Related Crates

- [`vx-core`](../vx-core/README.md) - Core functionality
- [`vx-cli`](../vx-cli/README.md) - Command-line interface
