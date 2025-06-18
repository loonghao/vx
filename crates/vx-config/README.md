# ⚙️ vx-config

<div align="center">

**Advanced Configuration Management for the vx Universal Tool Manager**

[![Crates.io](https://img.shields.io/crates/v/vx-config.svg)](https://crates.io/crates/vx-config)
[![Documentation](https://docs.rs/vx-config/badge.svg)](https://docs.rs/vx-config)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/loonghao/vx/workflows/CI/badge.svg)](https://github.com/loonghao/vx/actions)

*Intelligent, layered configuration system with automatic project detection*

</div>

---

## 🎯 Overview

`vx-config` provides the comprehensive configuration management system for vx, enabling intelligent tool version management, automatic project detection, and layered configuration from multiple sources. It's designed to work seamlessly with zero configuration while offering powerful customization when needed.

## ✨ Features

### 🏗️ Layered Configuration System
- **Built-in Defaults**: Sensible defaults that work out of the box
- **User Configuration**: Global user preferences and tool versions
- **Project Configuration**: Project-specific tool requirements
- **Environment Variables**: Runtime configuration overrides
- **Priority Resolution**: Intelligent merging with proper precedence

### 🔍 Intelligent Project Detection
- **Automatic Detection**: Recognizes Python, Rust, Node.js, and Go projects
- **Multi-Language Support**: Handles polyglot projects with multiple ecosystems
- **Configuration Inference**: Automatically suggests tool versions based on project files
- **Lock File Analysis**: Reads package-lock.json, Cargo.lock, poetry.lock, etc.

### 📄 Multiple Format Support
- **TOML**: Primary configuration format (`.vx.toml`)
- **JSON**: Alternative format support
- **Environment Variables**: `VX_*` prefixed variables
- **Legacy Support**: Reads existing tool configuration files

### 🎯 Smart Tool Management
- **Version Constraints**: Semantic version ranges and constraints
- **Tool Dependencies**: Automatic dependency resolution
- **Conflict Detection**: Identifies and resolves version conflicts
- **Inheritance**: Project configs inherit from user configs

## 🚀 Quick Start

### Basic Usage

```rust
use vx_config::ConfigManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create configuration manager
    let config_manager = ConfigManager::new().await?;
    
    // Get tool version for current project
    let node_version = config_manager.get_tool_version("node");
    println!("Node.js version: {:?}", node_version);
    
    // Check if auto-install is enabled
    if config_manager.is_auto_install_enabled() {
        println!("Auto-install is enabled");
    }
    
    // Get project-specific configuration
    let project_config = config_manager.get_project_config().await?;
    println!("Project tools: {:?}", project_config.tools);
    
    Ok(())
}
```

### Configuration Files

#### Global Configuration (`~/.config/vx/config.toml`)

```toml
[defaults]
auto_install = true
check_updates = true
update_interval = "24h"

[tools]
node = "20.11.0"
uv = "0.5.26"
go = "1.21.6"

[settings]
cache_duration = "7d"
parallel_downloads = 4
use_system_path = false

[ui]
show_progress = true
use_colors = true
progress_style = "default"
```

#### Project Configuration (`.vx.toml`)

```toml
[tools]
node = "18.17.0"        # Specific version
uv = "latest"           # Latest version
go = "^1.21.0"          # Version constraint

[settings]
auto_install = true
cache_duration = "7d"

[scripts]
dev = "npm run dev"
build = "npm run build"
test = "npm test"
lint = "uvx ruff check ."

[project]
name = "my-awesome-project"
type = ["node", "python"]
```

## 💡 Advanced Usage

### Project Detection

```rust
use vx_config::{ConfigManager, ProjectType};

let config_manager = ConfigManager::new().await?;

// Detect project types in current directory
let project_types = config_manager.detect_project_types(".").await?;
for project_type in project_types {
    match project_type {
        ProjectType::Node => println!("Node.js project detected"),
        ProjectType::Python => println!("Python project detected"),
        ProjectType::Rust => println!("Rust project detected"),
        ProjectType::Go => println!("Go project detected"),
    }
}

// Get recommended tool versions for detected projects
let recommendations = config_manager.get_tool_recommendations().await?;
println!("Recommended tools: {:?}", recommendations);
```

### Custom Configuration Sources

```rust
use vx_config::{ConfigManager, ConfigSource};

let config_manager = ConfigManager::builder()
    .add_source(ConfigSource::file("/custom/path/config.toml"))
    .add_source(ConfigSource::env_vars("MYAPP_"))
    .build()
    .await?;
```

### Version Constraint Resolution

```rust
use vx_config::{VersionConstraint, VersionResolver};

// Parse version constraints
let constraint = VersionConstraint::parse("^18.0.0")?;
let resolver = VersionResolver::new();

// Find best matching version
let available_versions = vec!["18.17.0", "18.19.0", "20.11.0"];
let best_match = resolver.resolve(&constraint, &available_versions)?;
println!("Best match: {}", best_match);
```

## 🏗️ Architecture

### Configuration Hierarchy

```
Environment Variables (highest priority)
    ↓
Project Configuration (.vx.toml)
    ↓
User Configuration (~/.config/vx/config.toml)
    ↓
Built-in Defaults (lowest priority)
```

### Core Components

- **ConfigManager**: Main interface for configuration operations
- **ProjectDetector**: Automatic project type detection
- **VersionResolver**: Semantic version constraint resolution
- **ConfigParser**: Multi-format configuration parsing
- **SourceMerger**: Intelligent configuration merging

### Project Detection Logic

```
1. Check for language-specific files:
   - package.json (Node.js)
   - pyproject.toml, requirements.txt (Python)
   - Cargo.toml (Rust)
   - go.mod (Go)

2. Analyze lock files for version hints:
   - package-lock.json, yarn.lock
   - poetry.lock, Pipfile.lock
   - Cargo.lock
   - go.sum

3. Read existing tool configurations:
   - .nvmrc, .node-version
   - .python-version, .tool-versions
   - rust-toolchain.toml
```

## 🔧 Configuration Reference

### Tool Configuration

```toml
[tools]
# Exact version
node = "18.17.0"

# Version constraint
go = "^1.21.0"          # >= 1.21.0, < 1.22.0
uv = "~0.5.26"          # >= 0.5.26, < 0.6.0

# Special values
rust = "latest"         # Latest stable version
python = "system"       # Use system installation
```

### Settings Configuration

```toml
[settings]
auto_install = true              # Auto-install missing tools
check_updates = true             # Check for tool updates
update_interval = "24h"          # Update check frequency
cache_duration = "7d"            # Cache duration
parallel_downloads = 4           # Concurrent downloads
use_system_path = false          # Use system PATH
```

### UI Configuration

```toml
[ui]
show_progress = true             # Show progress bars
use_colors = true                # Use colored output
progress_style = "default"       # Progress bar style
log_level = "info"               # Logging level
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with specific features
cargo test --features extended-detection

# Run integration tests
cargo test --test integration_tests

# Test configuration parsing
cargo test config_parsing
```

## 🔗 Related Crates

- [`vx-installer`](../vx-installer/README.md) - Universal installation engine
- [`vx-core`](../vx-core/README.md) - Core functionality and utilities
- [`vx-cli`](../vx-cli/README.md) - Command-line interface
- [`vx-plugin`](../vx-plugin/README.md) - Plugin system and trait definitions

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

---

<div align="center">

**Intelligent configuration for the modern developer**

[🚀 Get Started](../../README.md) | [📖 Documentation](https://docs.rs/vx-config) | [🤝 Contributing](../../CONTRIBUTING.md)

</div>