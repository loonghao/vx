# vx-tool-standard

[![Crates.io](https://img.shields.io/crates/v/vx-tool-standard.svg)](https://crates.io/crates/vx-tool-standard)
[![Documentation](https://docs.rs/vx-tool-standard/badge.svg)](https://docs.rs/vx-tool-standard)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Standard tool runtime interface and utilities for vx tool management.

[English](README.md) | [中文](README_zh.md)

## Overview

`vx-tool-standard` provides a standardized interface and utilities for implementing tool runtimes in the vx ecosystem. It defines common patterns, traits, and helper functions that tool implementations can use to ensure consistency and interoperability.

## Features

- **Standard Tool Runtime Interface**: Common traits for tool runtime operations
- **Platform Abstraction**: Cross-platform tool execution and management
- **Configuration Management**: Standardized tool configuration handling
- **Error Handling**: Consistent error types and handling patterns
- **Utility Functions**: Common operations for tool implementations
- **Testing Utilities**: Helper functions for testing tool implementations

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
vx-tool-standard = "0.2.6"
```

## Usage

### Implementing a Tool Runtime

```rust
use vx_tool_standard::{ToolRuntime, VxResult, Platform};
use async_trait::async_trait;

struct MyToolRuntime {
    name: String,
    version: Option<String>,
}

#[async_trait]
impl ToolRuntime for MyToolRuntime {
    async fn is_available(&self) -> VxResult<bool> {
        // Check if tool is available on the system
        Ok(which::which(&self.name).is_ok())
    }

    async fn get_version(&self) -> VxResult<Option<String>> {
        // Get installed version
        Ok(self.version.clone())
    }

    async fn get_path(&self) -> VxResult<Option<PathBuf>> {
        // Get tool executable path
        which::which(&self.name)
            .map(Some)
            .map_err(|e| e.into())
    }

    async fn execute(&self, args: &[String]) -> VxResult<i32> {
        // Execute tool with arguments
        let status = std::process::Command::new(&self.name)
            .args(args)
            .status()?;
        
        Ok(status.code().unwrap_or(1))
    }
}
```

### Using Standard Configurations

```rust
use vx_tool_standard::{StandardConfig, Platform};

let config = StandardConfig::builder()
    .name("my-tool")
    .version("1.0.0")
    .description("My awesome tool")
    .for_platforms(vec![Platform::Windows, Platform::Linux, Platform::MacOS])
    .build();

println!("Tool: {} v{}", config.name(), config.version());
```

### Platform-Specific Operations

```rust
use vx_tool_standard::Platform;

let current_platform = Platform::current();

match current_platform {
    Platform::Windows => {
        // Windows-specific logic
        println!("Running on Windows");
    }
    Platform::Linux => {
        // Linux-specific logic
        println!("Running on Linux");
    }
    Platform::MacOS => {
        // macOS-specific logic
        println!("Running on macOS");
    }
    Platform::Unknown => {
        println!("Unknown platform");
    }
}
```

### Error Handling

```rust
use vx_tool_standard::{VxResult, VxError};

fn tool_operation() -> VxResult<String> {
    // Operation that might fail
    if some_condition {
        Ok("Success".to_string())
    } else {
        Err(VxError::ToolNotFound("my-tool".to_string()))
    }
}

match tool_operation() {
    Ok(result) => println!("Success: {}", result),
    Err(VxError::ToolNotFound(tool)) => {
        eprintln!("Tool not found: {}", tool);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Architecture

### Core Traits

- **ToolRuntime**: Main interface for tool runtime operations
- **Configurable**: Interface for tools with configuration
- **Executable**: Interface for executable tools
- **Versionable**: Interface for tools with version information

### Utility Types

- **Platform**: Platform detection and abstraction
- **StandardConfig**: Standard configuration structure
- **VxResult**: Standard result type with error handling
- **VxError**: Standard error types for tool operations

## Advanced Usage

### Custom Tool Configuration

```rust
use vx_tool_standard::{StandardConfig, Configurable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct MyToolConfig {
    #[serde(flatten)]
    standard: StandardConfig,
    custom_option: String,
    debug_mode: bool,
}

impl Configurable for MyToolConfig {
    type Config = Self;
    
    fn config(&self) -> &Self::Config {
        self
    }
}
```

### Testing Utilities

```rust
use vx_tool_standard::testing::{MockToolRuntime, TestEnvironment};

#[tokio::test]
async fn test_tool_runtime() {
    let mut mock = MockToolRuntime::new("test-tool");
    mock.expect_is_available().returning(|| Ok(true));
    mock.expect_get_version().returning(|| Ok(Some("1.0.0".to_string())));
    
    assert!(mock.is_available().await.unwrap());
    assert_eq!(mock.get_version().await.unwrap(), Some("1.0.0".to_string()));
}

#[test]
fn test_with_environment() {
    let env = TestEnvironment::new()
        .with_tool("node", "18.17.0")
        .with_tool("npm", "9.6.7");
    
    env.run(|| {
        // Test code that depends on tools being available
        assert!(which::which("node").is_ok());
        assert!(which::which("npm").is_ok());
    });
}
```

## Integration with vx Ecosystem

### Plugin Integration

```rust
use vx_tool_standard::{ToolRuntime, StandardConfig};
use vx_plugin::VxTool;

struct StandardTool {
    config: StandardConfig,
    runtime: Box<dyn ToolRuntime>,
}

impl VxTool for StandardTool {
    fn name(&self) -> &str {
        self.config.name()
    }
    
    fn description(&self) -> &str {
        self.config.description()
    }
    
    // Implement other VxTool methods using runtime
}
```

### Configuration Integration

```rust
use vx_tool_standard::StandardConfig;
use vx_config::VxConfig;

let vx_config = VxConfig::load()?;
let tool_config = StandardConfig::from_vx_config(&vx_config, "my-tool")?;
```

## Examples

Check the `examples/` directory for complete examples:

- `basic_tool.rs` - Basic tool runtime implementation
- `configurable_tool.rs` - Tool with custom configuration
- `platform_specific.rs` - Platform-specific tool behavior
- `testing_example.rs` - Testing tool implementations

## Testing

```bash
cargo test
```

Run with logging:

```bash
RUST_LOG=vx_tool_standard=debug cargo test
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.