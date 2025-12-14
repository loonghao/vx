# vx-pm-bun

[![Crates.io](https://img.shields.io/crates/v/vx-pm-bun.svg)](https://crates.io/crates/vx-pm-bun)
[![Documentation](https://docs.rs/vx-pm-bun/badge.svg)](https://docs.rs/vx-pm-bun)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Bun package manager and runtime support for the vx universal tool manager.

## Status

🚧 **Under Development** - This crate is currently under development and not yet implemented.

## Overview

`vx-pm-bun` will provide Bun package manager and runtime support for vx, enabling ultra-fast JavaScript/TypeScript development with built-in bundling, testing, and package management through the vx interface.

## Planned Features

- **Bun Runtime**: Fast JavaScript/TypeScript runtime
- **Package Manager**: Lightning-fast package installation
- **Built-in Bundler**: Zero-configuration bundling
- **Test Runner**: Built-in test runner with Jest compatibility
- **TypeScript Support**: Native TypeScript execution
- **Hot Reloading**: Fast development server with hot reloading
- **Web APIs**: Node.js and Web API compatibility

## Planned Commands

### Runtime
```bash
# Execute JavaScript/TypeScript (planned)
vx bun run script.js
vx bun run script.ts
vx bun --version

# REPL (planned)
vx bun repl
```

### Package Management
```bash
# Install packages (planned)
vx bun install
vx bun add express
vx bun add --dev @types/node
vx bun remove express

# Update packages (planned)
vx bun update
vx bun outdated
```

### Development Server
```bash
# Development server (planned)
vx bun dev
vx bun --hot run server.ts
vx bun --watch run script.ts
```

### Bundling
```bash
# Build and bundle (planned)
vx bun build ./src/index.ts --outdir ./dist
vx bun build --minify --target browser
vx bun build --format esm --splitting
```

### Testing
```bash
# Test runner (planned)
vx bun test
vx bun test --watch
vx bun test --coverage
```

## Current Status

This crate is currently in the planning phase. JavaScript/Node.js development is currently supported through:
- [`vx-pm-npm`](../vx-pm-npm/README.md) - NPM package manager (available now)
- [`vx-tool-node`](../../vx-tools/vx-tool-node/README.md) - Node.js runtime

## Development Roadmap

1. **Phase 1**: Basic Bun runtime and package manager
2. **Phase 2**: Development server and hot reloading
3. **Phase 3**: Built-in bundler and build tools
4. **Phase 4**: Test runner and advanced features

## Bun Advantages (Planned)

### Performance
- **3x faster** than Node.js for many workloads
- **20x faster** package installation than npm
- **Native bundling** without external tools
- **Fast startup** with optimized runtime

### Developer Experience
- **Zero configuration**: Works out of the box
- **TypeScript native**: No compilation step needed
- **Hot reloading**: Instant feedback during development
- **All-in-one**: Runtime, package manager, bundler, and test runner

### Compatibility
- **Node.js APIs**: Drop-in replacement for most Node.js code
- **npm packages**: Compatible with existing npm ecosystem
- **Web APIs**: Fetch, WebSocket, and other Web APIs built-in

## Feature Comparison

| Feature | Node.js + NPM (Available) | Bun (Planned) |
|---------|---------------------------|---------------|
| JavaScript Runtime | ✅ | 🚧 |
| TypeScript Support | ⚠️ (requires compilation) | 🚧 (native) |
| Package Manager | ✅ | 🚧 |
| Bundler | ❌ (external tools) | 🚧 (built-in) |
| Test Runner | ❌ (external tools) | 🚧 (built-in) |
| Hot Reloading | ❌ (external tools) | 🚧 (built-in) |
| Performance | ⚠️ | 🚧 (3x faster) |

## Contributing

This crate is not yet implemented. If you're interested in contributing to Bun support in vx, please:

1. Check the main project [issues](https://github.com/loonghao/vx/issues)
2. Join the discussion about Bun runtime and package manager support
3. See the [contributing guidelines](../../../CONTRIBUTING.md)

## Alternative Solutions

While this crate is under development, consider these alternatives:

### Node.js + NPM (Available Now)
```bash
# Use Node.js and NPM for JavaScript development
vx npm install express
vx node server.js
vx npm run dev
```

### System Bun
```bash
# Use system Bun with vx
vx --use-system-path bun --version
vx --use-system-path bun install
vx --use-system-path bun run dev
```

## Configuration (Planned)

### Project Configuration (.vx.toml)
```toml
# Planned configuration
[tools]
bun = "latest"

[bun]
auto_install = true
prefer_bun = true         # Prefer bun over node for JS execution
```

### Bun Configuration (bunfig.toml)
```toml
# Planned Bun configuration integration
[install]
registry = "https://registry.npmjs.org/"
cache = "~/.bun/install/cache"

[run]
bun = true
hot = true

[test]
coverage = true
```

## Use Cases (Planned)

### Full-Stack Development
```bash
# Frontend + Backend with Bun (planned)
vx bun create next-app frontend
vx bun create hono-app backend

# Development
vx bun dev                # Start dev server
vx bun test              # Run tests
vx bun build             # Build for production
```

### TypeScript Development
```bash
# Native TypeScript execution (planned)
vx bun run server.ts     # No compilation needed
vx bun test *.test.ts    # Test TypeScript directly
vx bun build src/index.ts --outdir dist
```

### Package Development
```bash
# Library development (planned)
vx bun init
vx bun add --dev typescript
vx bun build --format esm,cjs
vx bun test
vx bun publish
```

## Performance Benchmarks (Expected)

### Package Installation
```bash
# npm (current)
time vx npm install      # ~30 seconds

# bun (planned)
time vx bun install      # ~1.5 seconds (20x faster)
```

### Runtime Performance
```bash
# Node.js (current)
time vx node server.js   # baseline

# Bun (planned)
time vx bun run server.js # 3x faster startup
```

## License

This project is licensed under the MIT License - see the [LICENSE](../../../LICENSE) file for details.

## Related Crates

- [`vx-core`](../../vx-core/README.md) - Core functionality
- [`vx-cli`](../../vx-cli/README.md) - Command-line interface
- [`vx-pm-npm`](../vx-pm-npm/README.md) - NPM package manager (available now)
- [`vx-tool-node`](../../vx-tools/vx-tool-node/README.md) - Node.js tool (available now)
- [`vx-pm-yarn`](../vx-pm-yarn/README.md) - Yarn package manager (planned)
- [`vx-pm-pnpm`](../vx-pm-pnpm/README.md) - PNPM package manager (planned)
