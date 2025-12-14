# vx-pm-pnpm

[![Crates.io](https://img.shields.io/crates/v/vx-pm-pnpm.svg)](https://crates.io/crates/vx-pm-pnpm)
[![Documentation](https://docs.rs/vx-pm-pnpm/badge.svg)](https://docs.rs/vx-pm-pnpm)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

PNPM package manager support for the vx universal tool manager.

## Status

🚧 **Under Development** - This crate is currently under development and not yet implemented.

## Overview

`vx-pm-pnpm` will provide PNPM (Performant NPM) package manager support for vx, enabling fast, disk space efficient package management for JavaScript/Node.js projects through the vx interface.

## Planned Features

- **PNPM Package Manager**: Fast, disk space efficient package management
- **Symlink-based**: Content-addressable storage with symlinks
- **Workspace Support**: PNPM workspaces and monorepo support
- **Strict Mode**: Strict dependency resolution
- **Script Execution**: Run package.json scripts through vx
- **Cache Management**: Global package cache with deduplication
- **Lock File**: pnpm-lock.yaml file management

## Planned Commands

### Package Management
```bash
# Install packages (planned)
vx pnpm install
vx pnpm add express
vx pnpm add --save-dev jest
vx pnpm remove express

# Update packages (planned)
vx pnpm update
vx pnpm update express
vx pnpm outdated
```

### Project Management
```bash
# Initialize projects (planned)
vx pnpm init

# Run scripts (planned)
vx pnpm run dev
vx pnpm run build
vx pnpm run test
vx pnpm start

# Information (planned)
vx pnpm list
vx pnpm list --depth=0
vx pnpm why express
```

### Workspace Management
```bash
# Workspace commands (planned)
vx pnpm -r install          # Install in all workspaces
vx pnpm -r run build        # Run build in all workspaces
vx pnpm --filter <pkg> add express
vx pnpm --filter <pkg> run test
```

### Store Management
```bash
# Store commands (planned)
vx pnpm store status
vx pnpm store prune
vx pnpm store path
```

## Current Status

This crate is currently in the planning phase. JavaScript/Node.js package management is currently provided through the [`vx-pm-npm`](../vx-pm-npm/README.md) crate.

For immediate JavaScript development needs, please use:
- [`vx-pm-npm`](../vx-pm-npm/README.md) - NPM package manager (available now)
- [`vx-tool-node`](../../vx-tools/vx-tool-node/README.md) - Node.js runtime with npm

## Development Roadmap

1. **Phase 1**: Basic PNPM package management
2. **Phase 2**: Workspace and monorepo features
3. **Phase 3**: Store management and optimization
4. **Phase 4**: Advanced features (strict mode, filtering)

## PNPM Advantages (Planned)

### Disk Space Efficiency
- **Content-addressable storage**: Packages stored once globally
- **Symlink structure**: Projects link to global store
- **Deduplication**: Automatic deduplication of dependencies

### Performance Benefits
- **Faster installs**: Parallel installation and linking
- **Incremental installs**: Only install changed dependencies
- **Efficient updates**: Minimal file operations

### Strict Dependency Management
- **Flat node_modules**: No phantom dependencies
- **Strict resolution**: Only declared dependencies accessible
- **Reproducible builds**: Consistent dependency resolution

## Feature Comparison

| Feature | NPM (Available) | PNPM (Planned) | Yarn (Planned) |
|---------|----------------|----------------|----------------|
| Package Installation | ✅ | 🚧 | 🚧 |
| Disk Space Efficiency | ❌ | 🚧 | ❌ |
| Symlink Structure | ❌ | 🚧 | ❌ |
| Workspaces | ✅ | 🚧 | 🚧 |
| Strict Mode | ❌ | 🚧 | ❌ |
| Global Store | ❌ | 🚧 | ❌ |

## Contributing

This crate is not yet implemented. If you're interested in contributing to PNPM support in vx, please:

1. Check the main project [issues](https://github.com/loonghao/vx/issues)
2. Join the discussion about PNPM package manager support
3. See the [contributing guidelines](../../../CONTRIBUTING.md)

## Alternative Solutions

While this crate is under development, consider these alternatives:

### NPM (Available Now)
```bash
# Use NPM for JavaScript package management
vx npm install express
vx npm run dev
vx npm test
```

### System PNPM
```bash
# Use system PNPM with vx
vx --use-system-path pnpm --version
vx --use-system-path pnpm install
```

## Configuration (Planned)

### Project Configuration (.vx.toml)
```toml
# Planned configuration
[package_managers]
pnpm = "latest"

[pnpm]
store_dir = "~/.pnpm-store"
strict_peer_dependencies = true
auto_install_peers = true
```

### PNPM Configuration (.npmrc)
```ini
# Planned PNPM configuration integration
store-dir=~/.pnpm-store
strict-peer-dependencies=true
auto-install-peers=true
shamefully-hoist=false
```

### Workspace Configuration (pnpm-workspace.yaml)
```yaml
# Planned workspace configuration
packages:
  - 'packages/*'
  - 'apps/*'
  - '!**/test/**'
```

## Architecture (Planned)

### Store Structure
```
~/.pnpm-store/
├── v3/
│   └── files/
│       ├── 00/
│       ├── 01/
│       └── ...
└── tmp/
```

### Project Structure
```
my-project/
├── node_modules/
│   ├── .pnpm/          # Real packages
│   ├── express/        # Symlink to .pnpm
│   └── ...
├── package.json
└── pnpm-lock.yaml
```

## Performance Benefits (Planned)

### Installation Speed
- **Parallel processing**: Concurrent package resolution and installation
- **Incremental installs**: Only process changed dependencies
- **Link-based**: Fast symlink creation vs file copying

### Disk Usage
- **Global deduplication**: Single copy of each package version
- **Content addressing**: Efficient storage of package contents
- **Shared cache**: Packages shared across all projects

## License

This project is licensed under the MIT License - see the [LICENSE](../../../LICENSE) file for details.

## Related Crates

- [`vx-core`](../../vx-core/README.md) - Core functionality
- [`vx-cli`](../../vx-cli/README.md) - Command-line interface
- [`vx-pm-npm`](../vx-pm-npm/README.md) - NPM package manager (available now)
- [`vx-pm-yarn`](../vx-pm-yarn/README.md) - Yarn package manager (planned)
- [`vx-tool-node`](../../vx-tools/vx-tool-node/README.md) - Node.js tool
