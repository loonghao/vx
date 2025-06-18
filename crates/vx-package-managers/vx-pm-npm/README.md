# 📦 vx-pm-npm

<div align="center">

**NPM Package Manager Plugin for vx Universal Tool Manager**

[![Crates.io](https://img.shields.io/crates/v/vx-pm-npm.svg)](https://crates.io/crates/vx-pm-npm)
[![Documentation](https://docs.rs/vx-pm-npm/badge.svg)](https://docs.rs/vx-pm-npm)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/loonghao/vx/workflows/CI/badge.svg)](https://github.com/loonghao/vx/actions)

*Complete NPM package management with beautiful installation experience and zero configuration*

</div>

## Overview

`vx-pm-npm` provides NPM (Node Package Manager) support for vx, enabling package management, script execution, and NPX functionality through the vx interface.

## Features

- **NPM Package Manager**: Full npm package management capabilities
- **NPX Integration**: Package runner functionality for one-time tool execution
- **Script Execution**: Run package.json scripts through vx
- **Registry Support**: Support for custom npm registries
- **Cache Management**: Efficient package caching and management
- **Workspace Support**: npm workspaces and monorepo support
- **Security**: Package audit and vulnerability scanning

## Supported Commands

### Package Management
```bash
# Install packages
vx npm install
vx npm install express
vx npm install -g typescript
vx npm install --save-dev jest

# Uninstall packages
vx npm uninstall express
vx npm uninstall -g typescript

# Update packages
vx npm update
vx npm update express
vx npm outdated
```

### Project Management
```bash
# Initialize projects
vx npm init
vx npm init -y
vx npm init @scope/package

# Run scripts
vx npm run dev
vx npm run build
vx npm run test
vx npm start

# Information
vx npm list
vx npm list --depth=0
vx npm info express
vx npm view express versions
```

### NPX Package Runner
```bash
# Run packages without installing
vx npx create-react-app my-app
vx npx typescript --init
vx npx cowsay "Hello from vx!"

# Run specific versions
vx npx typescript@4.9.5 --version
vx npx -p typescript@latest tsc --version

# Execute local binaries
vx npx jest
vx npx eslint src/
```

### Registry and Configuration
```bash
# Registry management
vx npm config set registry https://registry.npmjs.org/
vx npm config get registry
vx npm config list

# Authentication
vx npm login
vx npm logout
vx npm whoami

# Publishing
vx npm publish
vx npm unpublish package@version
```

## Installation

NPM support is automatically available when Node.js is installed through vx:

```bash
# Install Node.js (includes npm)
vx install node@18.17.0

# NPM is automatically available
vx npm --version
```

## Configuration

### Project Configuration (.vx.toml)
```toml
[tools]
node = "18.17.0"          # NPM comes with Node.js

[npm]
registry = "https://registry.npmjs.org/"
cache_dir = "~/.npm"
prefix = "~/.npm-global"
```

### Global Configuration
```toml
[npm.settings]
registry = "https://registry.npmjs.org/"
cache = "~/.npm"
prefix = "~/.npm-global"
audit_level = "moderate"
fund = false
```

### NPM Configuration (.npmrc)
```ini
# Global .npmrc
registry=https://registry.npmjs.org/
cache=~/.npm
prefix=~/.npm-global
audit-level=moderate
fund=false

# Project .npmrc
registry=https://registry.npmjs.org/
save-exact=true
package-lock=true
```

## Package.json Integration

### Basic package.json
```json
{
  "name": "my-project",
  "version": "1.0.0",
  "description": "My awesome project",
  "main": "index.js",
  "scripts": {
    "start": "node index.js",
    "dev": "nodemon index.js",
    "build": "webpack --mode production",
    "test": "jest"
  },
  "dependencies": {
    "express": "^4.18.0"
  },
  "devDependencies": {
    "jest": "^29.0.0",
    "nodemon": "^3.0.0"
  }
}
```

### Script Execution
```bash
# Run package.json scripts
vx npm run start
vx npm run dev
vx npm run build
vx npm run test

# List available scripts
vx npm run

# Run with arguments
vx npm run test -- --watch
vx npm run build -- --env production
```

## NPX Integration

### Package Runner Features
- **Environment Isolation**: Each npx execution runs in isolation
- **Version Specification**: Run specific package versions
- **Temporary Installation**: Packages are installed temporarily
- **Local Binary Execution**: Run locally installed binaries

### Common NPX Use Cases
```bash
# Project scaffolding
vx npx create-react-app my-react-app
vx npx create-next-app my-next-app
vx npx @angular/cli new my-angular-app

# Development tools
vx npx typescript --init
vx npx eslint --init
vx npx prettier --write .

# One-time utilities
vx npx cowsay "Hello World"
vx npx http-server
vx npx json-server db.json
```

## Integration

### With vx-core
```rust
use vx_core::{PackageManager, ToolManager};
use vx_pm_npm::NpmPackageManager;

let npm = NpmPackageManager::new();
let manager = ToolManager::new();

// Install packages
npm.install_package("express", None).await?;

// Run scripts
npm.run_script("dev").await?;
```

### Plugin Registration
```rust
use vx_core::{Plugin, PluginManager};
use vx_pm_npm::NpmPlugin;

let plugin = NpmPlugin::new();
let mut manager = PluginManager::new();

manager.register_plugin(Box::new(plugin))?;
```

## Development

### Building
```bash
cd crates/vx-package-managers/vx-pm-npm
cargo build
```

### Testing
```bash
cargo test
```

### Integration Testing
```bash
# Test with actual npm installation
cargo test --features integration-tests
```

## Implementation Details

### Package Manager Structure
- **NpmPackageManager**: Core npm functionality
- **NpxRunner**: NPX package runner implementation
- **RegistryClient**: npm registry interaction
- **CacheManager**: Package cache management

### Command Mapping
```rust
// npm commands -> vx npm equivalents
"npm install" -> vx npm install
"npm run dev" -> vx npm run dev
"npx create-react-app" -> vx npx create-react-app
```

### Environment Management
- **Node.js Detection**: Automatic Node.js version detection
- **PATH Management**: Proper PATH configuration for npm binaries
- **Registry Configuration**: Support for custom registries
- **Cache Isolation**: Isolated cache per vx environment

## Security Features

### Package Auditing
```bash
# Security audit
vx npm audit
vx npm audit fix
vx npm audit fix --force

# Audit configuration
vx npm config set audit-level moderate
```

### Registry Security
- **HTTPS Only**: Secure package downloads
- **Checksum Verification**: Package integrity verification
- **Signature Validation**: Package signature verification (when available)
- **Vulnerability Scanning**: Automatic vulnerability detection

## Performance Optimization

### Caching Strategy
- **Package Cache**: Shared package cache across projects
- **Metadata Cache**: Registry metadata caching
- **Parallel Downloads**: Concurrent package downloads
- **Incremental Installs**: Only install changed dependencies

### Network Optimization
```bash
# Configure npm for better performance
vx npm config set fetch-retries 5
vx npm config set fetch-retry-factor 2
vx npm config set fetch-retry-mintimeout 10000
vx npm config set fetch-retry-maxtimeout 60000
```

## Troubleshooting

### Common Issues
```bash
# Clear npm cache
vx npm cache clean --force

# Verify cache
vx npm cache verify

# Check npm configuration
vx npm config list
vx npm config get registry

# Reinstall node_modules
rm -rf node_modules package-lock.json
vx npm install
```

### Network Issues
```bash
# Check registry connectivity
vx npm ping

# Use different registry
vx npm config set registry https://registry.npmmirror.com/

# Configure proxy
vx npm config set proxy http://proxy:8080
vx npm config set https-proxy http://proxy:8080
```

### Permission Issues
```bash
# Fix npm permissions (Unix)
sudo chown -R $(whoami) ~/.npm
sudo chown -R $(whoami) /usr/local/lib/node_modules

# Use different prefix
vx npm config set prefix ~/.npm-global
export PATH=~/.npm-global/bin:$PATH
```

## Best Practices

### Package.json Management
- Use exact versions for critical dependencies
- Separate dependencies and devDependencies
- Include engines field for Node.js version
- Use npm scripts for common tasks

### Security
- Regularly run `npm audit`
- Keep dependencies updated
- Use `.npmrc` for project-specific configuration
- Review package permissions before installation

### Performance
- Use `package-lock.json` for reproducible builds
- Configure npm cache appropriately
- Use npm workspaces for monorepos
- Consider using npm ci in CI/CD

## License

This project is licensed under the MIT License - see the [LICENSE](../../../LICENSE) file for details.

## Contributing

Contributions are welcome! Please see the [contributing guidelines](../../../CONTRIBUTING.md) for more information.

## Related Crates

- [`vx-core`](../../vx-core/README.md) - Core functionality
- [`vx-cli`](../../vx-cli/README.md) - Command-line interface
- [`vx-tool-node`](../../vx-tools/vx-tool-node/README.md) - Node.js tool
- [`vx-pm-yarn`](../vx-pm-yarn/README.md) - Yarn package manager
- [`vx-pm-pnpm`](../vx-pm-pnpm/README.md) - PNPM package manager
