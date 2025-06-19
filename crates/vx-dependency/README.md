# vx-dependency

[![Crates.io](https://img.shields.io/crates/v/vx-dependency.svg)](https://crates.io/crates/vx-dependency)
[![Documentation](https://docs.rs/vx-dependency/badge.svg)](https://docs.rs/vx-dependency)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Dependency resolution engine for vx tool management system.

[English](README.md) | [中文](README_zh.md)

## Overview

`vx-dependency` provides a sophisticated dependency resolution engine for managing tool dependencies in the vx ecosystem. It handles complex dependency graphs, version constraints, and automatic dependency installation.

## Features

- **Multi-layer Dependency Support**: Handle tool-to-tool dependencies automatically
- **Dependency Graph Management**: Build and analyze complex dependency relationships
- **Circular Dependency Detection**: Prevent and detect circular dependencies
- **Version Constraint Resolution**: Resolve version conflicts across dependencies
- **Automatic Installation**: Install dependencies in the correct order
- **Conflict Resolution**: Handle and resolve dependency conflicts intelligently

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
vx-dependency = "0.2.6"
```

## Usage

### Basic Dependency Resolution

```rust
use vx_dependency::{DependencyResolver, DependencyGraph};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resolver = DependencyResolver::new();
    
    // Add tool dependencies
    resolver.add_dependency("yarn", "node")?;
    resolver.add_dependency("npm", "node")?;
    
    // Resolve installation order
    let install_order = resolver.resolve_install_order("yarn").await?;
    println!("Install order: {:?}", install_order);
    
    Ok(())
}
```

### Dependency Graph Analysis

```rust
use vx_dependency::DependencyGraph;

let mut graph = DependencyGraph::new();

// Build dependency relationships
graph.add_dependency("tool-a", "tool-b", "^1.0.0")?;
graph.add_dependency("tool-b", "tool-c", ">=2.0.0")?;

// Check for circular dependencies
if let Some(cycle) = graph.detect_cycles() {
    println!("Circular dependency detected: {:?}", cycle);
}

// Get topological order
let order = graph.topological_sort()?;
println!("Installation order: {:?}", order);
```

### Version Constraint Resolution

```rust
use vx_dependency::{VersionConstraint, DependencyResolver};

let resolver = DependencyResolver::new();

// Add version constraints
resolver.add_constraint("node", ">=18.0.0")?;
resolver.add_constraint("node", "<20.0.0")?;

// Resolve compatible version
let version = resolver.resolve_version("node").await?;
println!("Resolved version: {}", version);
```

## Architecture

### Core Components

- **DependencyResolver**: Main engine for dependency resolution
- **DependencyGraph**: Graph structure for dependency relationships
- **VersionConstraint**: Version constraint handling and resolution
- **InstallationPlanner**: Optimizes installation order

### Integration Points

- **vx-plugin**: Tool plugin integration for dependency information
- **vx-version**: Version parsing and constraint resolution
- **vx-core**: Core tool management integration

## Advanced Features

### Dependency Caching

```rust
use vx_dependency::{DependencyResolver, CacheConfig};

let cache_config = CacheConfig::new()
    .with_ttl(Duration::from_secs(3600))
    .with_max_entries(1000);

let resolver = DependencyResolver::with_cache(cache_config);
```

### Custom Dependency Sources

```rust
use vx_dependency::{DependencySource, DependencyResolver};

struct CustomSource;

impl DependencySource for CustomSource {
    async fn get_dependencies(&self, tool: &str) -> Result<Vec<String>> {
        // Custom dependency resolution logic
        Ok(vec!["custom-dep".to_string()])
    }
}

let resolver = DependencyResolver::with_source(Box::new(CustomSource));
```

## Examples

### Package Manager Dependencies

```rust
// npm depends on node
resolver.add_dependency("npm", "node")?;

// yarn depends on node
resolver.add_dependency("yarn", "node")?;

// pnpm depends on node
resolver.add_dependency("pnpm", "node")?;

// When installing yarn, node will be installed first
let order = resolver.resolve_install_order("yarn").await?;
// Result: ["node", "yarn"]
```

### Complex Dependency Chains

```rust
// Build tool chain: app -> bundler -> compiler -> runtime
resolver.add_dependency("my-app", "webpack")?;
resolver.add_dependency("webpack", "typescript")?;
resolver.add_dependency("typescript", "node")?;

let order = resolver.resolve_install_order("my-app").await?;
// Result: ["node", "typescript", "webpack", "my-app"]
```

## Error Handling

```rust
use vx_dependency::{DependencyError, DependencyResolver};

match resolver.resolve_install_order("tool").await {
    Ok(order) => println!("Install order: {:?}", order),
    Err(DependencyError::CircularDependency(cycle)) => {
        eprintln!("Circular dependency: {:?}", cycle);
    }
    Err(DependencyError::VersionConflict { tool, constraints }) => {
        eprintln!("Version conflict for {}: {:?}", tool, constraints);
    }
    Err(e) => eprintln!("Dependency error: {}", e),
}
```

## Testing

```bash
cargo test
```

Run with dependency resolution logging:

```bash
RUST_LOG=vx_dependency=debug cargo test
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.