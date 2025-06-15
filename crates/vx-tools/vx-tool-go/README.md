# vx-tool-go

[![Crates.io](https://img.shields.io/crates/v/vx-tool-go.svg)](https://crates.io/crates/vx-tool-go)
[![Documentation](https://docs.rs/vx-tool-go/badge.svg)](https://docs.rs/vx-tool-go)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Go programming language tool support for the vx universal tool manager.

## Overview

`vx-tool-go` provides Go programming language support for vx, enabling automatic installation, version management, and execution of Go commands through the vx interface.

## Features

- **Go Compiler**: Full Go toolchain with compiler and tools
- **Module Management**: Built-in Go module support
- **Auto-Installation**: Automatic download and installation of Go versions
- **Cross-Platform**: Windows, macOS, and Linux support
- **Version Management**: Install and switch between multiple Go versions
- **Build Tools**: Complete build and development toolchain
- **Cross-Compilation**: Support for building across platforms

## Supported Commands

### Go Compiler and Tools
```bash
# Build and run
vx go build
vx go run main.go
vx go run .

# Module management
vx go mod init mymodule
vx go mod tidy
vx go mod download
vx go mod verify

# Testing
vx go test
vx go test ./...
vx go test -v -race

# Code formatting and tools
vx go fmt ./...
vx go vet ./...
vx go clean
```

### Package Management
```bash
# Get packages
vx go get github.com/gin-gonic/gin
vx go get -u github.com/gin-gonic/gin@latest
vx go get ./...

# List modules
vx go list -m all
vx go list -m -versions github.com/gin-gonic/gin

# Install binaries
vx go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
```

### Build and Development
```bash
# Build for different platforms
vx go build -o myapp
GOOS=linux GOARCH=amd64 vx go build -o myapp-linux
GOOS=windows GOARCH=amd64 vx go build -o myapp.exe

# Generate code
vx go generate ./...

# Documentation
vx go doc fmt.Println
vx go doc -http=:6060
```

## Installation

### Through vx CLI
```bash
# Install latest version
vx install go

# Install specific version
vx install go@1.21.6
vx install go@1.20.12

# Install latest version
vx install go@latest
```

### Version Constraints
```bash
# Semantic version ranges
vx install go@^1.21.0     # Latest 1.21.x
vx install go@~1.21.6     # Latest 1.21.6.x
vx install go@>=1.20.0    # 1.20.0 or higher
```

## Configuration

### Project Configuration (.vx.toml)
```toml
[tools]
go = "1.21.6"             # Specific version
# go = "latest"           # Latest stable
# go = "^1.21.0"          # Version range

[tools.go]
auto_install = true
```

### Global Configuration
```toml
[tools.go]
default_version = "latest"
auto_install = true
install_timeout = 300

[go.settings]
GOPATH = "~/go"
GOPROXY = "https://proxy.golang.org,direct"
GOSUMDB = "sum.golang.org"
GOPRIVATE = ""
```

## Environment Variables

### Go-specific Variables
```bash
# Set through vx configuration or environment
export GOPATH=$HOME/go
export GOPROXY=https://proxy.golang.org,direct
export GOSUMDB=sum.golang.org
export GOPRIVATE=example.com/private

# Build-specific
export CGO_ENABLED=1
export GOOS=linux
export GOARCH=amd64
```

### vx Integration
```bash
# vx automatically sets GOROOT
vx go env GOROOT  # Points to vx-managed Go installation

# Other Go environment variables
vx go env GOPATH
vx go env GOPROXY
vx go env GOMODCACHE
```

## Platform Support

### Windows
- **x64**: Full support
- **x86**: Legacy support
- **ARM64**: Windows 11 ARM support

### macOS
- **x64**: Intel Mac support
- **ARM64**: Apple Silicon (M1/M2) support

### Linux
- **x64**: All major distributions
- **ARM64**: ARM-based systems
- **ARMv6**: Raspberry Pi support

## Cross-Compilation

### Supported Targets
```bash
# List all supported platforms
vx go tool dist list

# Common cross-compilation examples
GOOS=windows GOARCH=amd64 vx go build -o app.exe
GOOS=darwin GOARCH=arm64 vx go build -o app-mac-arm64
GOOS=linux GOARCH=arm64 vx go build -o app-linux-arm64
```

### Build Scripts
```bash
#!/bin/bash
# build-all.sh
platforms=("windows/amd64" "darwin/amd64" "darwin/arm64" "linux/amd64" "linux/arm64")

for platform in "${platforms[@]}"
do
    platform_split=(${platform//\// })
    GOOS=${platform_split[0]}
    GOARCH=${platform_split[1]}
    output_name='myapp-'$GOOS'-'$GOARCH
    if [ $GOOS = "windows" ]; then
        output_name+='.exe'
    fi
    
    env GOOS=$GOOS GOARCH=$GOARCH vx go build -o $output_name
done
```

## Integration

### With vx-core
```rust
use vx_core::{Tool, ToolManager};
use vx_tool_go::GoTool;

let go_tool = GoTool::new();
let manager = ToolManager::new();

// Install Go
manager.install_tool(&go_tool, "1.21.6").await?;

// Execute Go commands
manager.execute_tool(&go_tool, &["version"]).await?;
```

### Plugin Registration
```rust
use vx_core::{Plugin, PluginManager};
use vx_tool_go::GoPlugin;

let plugin = GoPlugin::new();
let mut manager = PluginManager::new();

manager.register_plugin(Box::new(plugin))?;
```

## Development

### Building
```bash
cd crates/vx-tool-go
cargo build
```

### Testing
```bash
cargo test
```

### Integration Testing
```bash
# Test with actual Go installation
cargo test --features integration-tests
```

## Implementation Details

### Tool Structure
- **GoTool**: Main Go compiler and toolchain
- **GoModTool**: Go module management
- **GoBuildTool**: Build and compilation tools

### Version Resolution
1. **Project Config**: Check `.vx.toml` for version specification
2. **Global Config**: Fall back to global default
3. **Latest Stable**: Use latest stable if no version specified
4. **Auto-Install**: Download and install if not available

### Installation Process
1. **Version Lookup**: Query Go release API
2. **Download**: Fetch appropriate binary/archive
3. **Extraction**: Extract to vx tools directory
4. **Verification**: Verify installation integrity
5. **Environment Setup**: Configure GOROOT and PATH

## Project Templates

### Basic Application
```bash
# Initialize new Go module
vx go mod init myapp

# Create main.go
cat > main.go << EOF
package main

import "fmt"

func main() {
    fmt.Println("Hello, World!")
}
EOF

# Build and run
vx go build
./myapp
```

### Web Service
```bash
# Initialize module
vx go mod init mywebapp

# Add dependencies
vx go get github.com/gin-gonic/gin

# Create server
cat > main.go << EOF
package main

import "github.com/gin-gonic/gin"

func main() {
    r := gin.Default()
    r.GET("/", func(c *gin.Context) {
        c.JSON(200, gin.H{"message": "Hello, World!"})
    })
    r.Run(":8080")
}
EOF

# Run
vx go run main.go
```

## Error Handling

### Common Errors
- **Network Issues**: Download failures, proxy issues
- **Permission Errors**: Installation directory access
- **Module Errors**: Dependency resolution failures
- **Build Errors**: Compilation failures

### Recovery
```bash
# Reinstall Go
vx install go@1.21.6 --force

# Clear module cache
vx go clean -modcache

# Reset modules
vx go mod tidy

# Use system Go as fallback
vx --use-system-path go version
```

## Performance

- **Fast Compilation**: Go's fast compilation times
- **Efficient Downloads**: Parallel downloading with progress tracking
- **Module Caching**: Shared module cache across projects
- **Quick Execution**: Minimal overhead for tool execution

## Security

- **Checksum Verification**: SHA256 verification of downloads
- **HTTPS Only**: Secure downloads from official sources
- **Module Verification**: Go module checksum verification
- **Sandboxed Builds**: Isolated build environments

## Troubleshooting

### Installation Issues
```bash
# Check available versions
vx search go

# Verify installation
vx go version
vx go env

# Check GOROOT
vx go env GOROOT

# Force reinstall
vx remove go@1.21.6
vx install go@1.21.6
```

### Build Issues
```bash
# Check Go environment
vx go env

# Verify module
vx go mod verify
vx go mod tidy

# Clean build cache
vx go clean -cache
vx go clean -modcache

# Debug build
vx go build -x -v
```

## License

This project is licensed under the MIT License - see the [LICENSE](../../../LICENSE) file for details.

## Contributing

Contributions are welcome! Please see the [contributing guidelines](../../../CONTRIBUTING.md) for more information.

## Related Crates

- [`vx-core`](../../vx-core/README.md) - Core functionality
- [`vx-cli`](../../vx-cli/README.md) - Command-line interface
- [`vx-tool-node`](../vx-tool-node/README.md) - Node.js tool
- [`vx-tool-rust`](../vx-tool-rust/README.md) - Rust tool
