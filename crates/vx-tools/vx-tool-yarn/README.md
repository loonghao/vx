# vx-pm-yarn

[![Crates.io](https://img.shields.io/crates/v/vx-pm-yarn.svg)](https://crates.io/crates/vx-pm-yarn)
[![Documentation](https://docs.rs/vx-pm-yarn/badge.svg)](https://docs.rs/vx-pm-yarn)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Yarn package manager support for the vx universal tool manager.

## Status

🚧 **Under Development** - This crate is currently under development and not yet implemented.

## Overview

`vx-pm-yarn` will provide Yarn package manager support for vx, enabling fast and reliable package management for JavaScript/Node.js projects through the vx interface.

## Planned Features

- **Yarn Package Manager**: Full Yarn package management capabilities
- **Workspace Support**: Yarn workspaces and monorepo support
- **PnP (Plug'n'Play)**: Zero-installs and Plug'n'Play support
- **Berry Support**: Yarn 2+ (Berry) modern features
- **Script Execution**: Run package.json scripts through vx
- **Cache Management**: Efficient package caching and management
- **Lock File**: yarn.lock file management and integrity

## Planned Commands

### Package Management
```bash
# Install packages (planned)
vx yarn install
vx yarn add express
vx yarn add --dev jest
vx yarn remove express

# Update packages (planned)
vx yarn upgrade
vx yarn upgrade express
vx yarn outdated
```

### Project Management
```bash
# Initialize projects (planned)
vx yarn init
vx yarn init -y

# Run scripts (planned)
vx yarn run dev
vx yarn run build
vx yarn run test
vx yarn start

# Information (planned)
vx yarn list
vx yarn info express
```

### Workspace Management
```bash
# Workspace commands (planned)
vx yarn workspaces list
vx yarn workspace <workspace-name> add express
vx yarn workspace <workspace-name> run build
```

## Current Status

This crate is currently in the planning phase. JavaScript/Node.js package management is currently provided through the [`vx-pm-npm`](../vx-pm-npm/README.md) crate.

For immediate JavaScript development needs, please use:
- [`vx-pm-npm`](../vx-pm-npm/README.md) - NPM package manager (available now)
- [`vx-tool-node`](../../vx-tools/vx-tool-node/README.md) - Node.js runtime with npm

## Development Roadmap

1. **Phase 1**: Basic Yarn Classic (v1) support
2. **Phase 2**: Yarn Berry (v2+) support
3. **Phase 3**: Workspace and monorepo features
4. **Phase 4**: Advanced features (PnP, Zero-installs)

## Yarn vs NPM

### Yarn Advantages (Planned)
- **Faster**: Parallel package installation
- **Deterministic**: Lockfile ensures consistent installs
- **Workspaces**: Built-in monorepo support
- **Offline**: Offline package installation
- **Security**: Enhanced security features

### Feature Comparison
| Feature | NPM (Available) | Yarn (Planned) |
|---------|----------------|----------------|
| Package Installation | ✅ | 🚧 |
| Script Running | ✅ | 🚧 |
| Workspaces | ✅ | 🚧 |
| Lock Files | ✅ | 🚧 |
| Offline Mode | ❌ | 🚧 |
| PnP Support | ❌ | 🚧 |

## Contributing

This crate is not yet implemented. If you're interested in contributing to Yarn support in vx, please:

1. Check the main project [issues](https://github.com/loonghao/vx/issues)
2. Join the discussion about Yarn package manager support
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

### System Yarn
```bash
# Use system Yarn with vx
vx --use-system-path yarn --version
vx --use-system-path yarn install
```

## Configuration (Planned)

### Project Configuration (.vx.toml)
```toml
# Planned configuration
[package_managers]
yarn = "latest"

[yarn]
version = "berry"         # or "classic"
enable_pnp = true
workspace_support = true
```

### Yarn Configuration (.yarnrc.yml)
```yaml
# Planned Yarn configuration integration
nodeLinker: pnp
enableGlobalCache: true
compressionLevel: mixed
```

## License

This project is licensed under the MIT License - see the [LICENSE](../../../LICENSE) file for details.

## Related Crates

- [`vx-core`](../../vx-core/README.md) - Core functionality
- [`vx-cli`](../../vx-cli/README.md) - Command-line interface
- [`vx-pm-npm`](../vx-pm-npm/README.md) - NPM package manager (available now)
- [`vx-pm-pnpm`](../vx-pm-pnpm/README.md) - PNPM package manager (planned)
- [`vx-tool-node`](../../vx-tools/vx-tool-node/README.md) - Node.js tool
