# vx-tool-uv

[![Crates.io](https://img.shields.io/crates/v/vx-tool-uv.svg)](https://crates.io/crates/vx-tool-uv)
[![Documentation](https://docs.rs/vx-tool-uv/badge.svg)](https://docs.rs/vx-tool-uv)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

UV Python tool support for the vx universal tool manager.

## Overview

`vx-tool-uv` provides UV (Extremely fast Python package installer and resolver) support for vx, enabling automatic installation, version management, and execution of UV commands through the vx interface.

## Features

- **UV Package Manager**: Lightning-fast Python package installation
- **UVX Support**: Python application runner with environment isolation
- **Auto-Installation**: Automatic download and installation of UV
- **Cross-Platform**: Windows, macOS, and Linux support
- **Virtual Environment**: Seamless integration with UV's venv management
- **Pip Compatibility**: Drop-in replacement for pip commands
- **Performance**: 10-100x faster than traditional pip

## Supported Commands

### UV Package Manager
```bash
# Package installation
vx uv pip install requests
vx uv pip install -r requirements.txt
vx uv pip install -e .

# Package management
vx uv pip uninstall requests
vx uv pip list
vx uv pip show requests
vx uv pip freeze

# Virtual environments
vx uv venv myenv
vx uv venv --python 3.11
vx uv pip install --system requests
```

### UVX Application Runner
```bash
# Run Python applications
vx uvx ruff check .
vx uvx black .
vx uvx pytest

# Run with specific versions
vx uvx --python 3.11 black .
vx uvx --from black==23.0.0 black .

# Install and run
vx uvx --install-only ruff
vx uvx ruff check .
```

### UV Project Management
```bash
# Initialize projects
vx uv init myproject
vx uv init --lib mylib

# Add dependencies
vx uv add requests
vx uv add --dev pytest
vx uv add "django>=4.0"

# Run commands
vx uv run python script.py
vx uv run pytest
vx uv run --python 3.11 python script.py
```

## Installation

### Through vx CLI
```bash
# Install latest version
vx install uv

# Install specific version
vx install uv@0.1.5
vx install uv@latest
```

### Version Constraints
```bash
# Semantic version ranges
vx install uv@^0.1.0      # Latest 0.1.x
vx install uv@~0.1.5      # Latest 0.1.5.x
vx install uv@>=0.1.0     # 0.1.0 or higher
```

## Configuration

### Project Configuration (.vx.toml)
```toml
[tools]
uv = "latest"             # Latest stable version
# uv = "0.1.5"            # Specific version
# uv = "^0.1.0"           # Version range

[tools.uv]
auto_install = true
python_version = "3.11"   # Default Python version
```

### Global Configuration
```toml
[tools.uv]
default_version = "latest"
auto_install = true
install_timeout = 300

[uv.settings]
index_url = "https://pypi.org/simple/"
extra_index_url = []
trusted_host = []
cache_dir = "~/.cache/uv"
```

## Python Version Management

### Python Discovery
UV automatically discovers Python installations:

```bash
# Use system Python
vx uv --python python3.11 pip install requests

# Use specific Python path
vx uv --python /usr/bin/python3.11 pip install requests

# Use Python from PATH
vx uv --python 3.11 pip install requests
```

### Virtual Environment Integration
```bash
# Create virtual environment
vx uv venv .venv

# Activate and use
source .venv/bin/activate  # Unix
.venv\Scripts\activate     # Windows

# Or use uv run directly
vx uv run python script.py
```

## Platform Support

### Windows
- **x64**: Full support
- **x86**: Limited support
- **ARM64**: Windows 11 ARM support

### macOS
- **x64**: Intel Mac support
- **ARM64**: Apple Silicon (M1/M2) support
- **Universal**: Automatic architecture detection

### Linux
- **x64**: All major distributions
- **ARM64**: ARM-based systems
- **musl**: Alpine Linux support

## Performance Benefits

### Speed Comparison
- **10-100x faster** than pip for package installation
- **Parallel downloads** and installations
- **Efficient dependency resolution**
- **Smart caching** of packages and metadata

### Benchmarks
```bash
# Traditional pip
time pip install django flask fastapi  # ~30 seconds

# UV
time vx uv pip install django flask fastapi  # ~3 seconds
```

## Integration

### With vx-core
```rust
use vx_core::{Tool, ToolManager};
use vx_tool_uv::UvTool;

let uv_tool = UvTool::new();
let manager = ToolManager::new();

// Install UV
manager.install_tool(&uv_tool, "latest").await?;

// Execute UV commands
manager.execute_tool(&uv_tool, &["pip", "install", "requests"]).await?;
```

### Plugin Registration
```rust
use vx_core::{Plugin, PluginManager};
use vx_tool_uv::UvPlugin;

let plugin = UvPlugin::new();
let mut manager = PluginManager::new();

manager.register_plugin(Box::new(plugin))?;
```

## Development

### Building
```bash
cd crates/vx-tool-uv
cargo build
```

### Testing
```bash
cargo test
```

### Integration Testing
```bash
# Test with actual UV installation
cargo test --features integration-tests
```

## Implementation Details

### Tool Structure
- **UvTool**: Main UV package manager tool
- **UvxTool**: UVX application runner support
- **UvProjectTool**: UV project management commands

### Version Resolution
1. **Project Config**: Check `.vx.toml` for version specification
2. **Global Config**: Fall back to global default
3. **Latest Stable**: Use latest stable if no version specified
4. **Auto-Install**: Download and install if not available

### Installation Process
1. **Version Lookup**: Query UV release API
2. **Download**: Fetch appropriate binary for platform
3. **Extraction**: Extract to vx tools directory
4. **Verification**: Verify installation integrity
5. **Configuration**: Set up UV configuration

## Migration from Pip

### Command Mapping
```bash
# pip commands -> uv equivalents
pip install requests        → vx uv pip install requests
pip install -r requirements.txt → vx uv pip install -r requirements.txt
pip uninstall requests      → vx uv pip uninstall requests
pip list                    → vx uv pip list
pip freeze                  → vx uv pip freeze
pip show requests           → vx uv pip show requests
```

### Configuration Migration
```bash
# pip.conf -> uv configuration
[global]
index-url = https://pypi.org/simple/
extra-index-url = https://test.pypi.org/simple/

# Equivalent UV configuration
[uv.settings]
index_url = "https://pypi.org/simple/"
extra_index_url = ["https://test.pypi.org/simple/"]
```

## Error Handling

### Common Errors
- **Network Issues**: Download failures, index timeouts
- **Permission Errors**: Installation directory access
- **Python Not Found**: Missing Python installation
- **Package Conflicts**: Dependency resolution failures

### Recovery
```bash
# Reinstall UV
vx install uv --force

# Clear UV cache
vx uv cache clean

# Use system pip as fallback
vx --use-system-path pip install requests
```

## Security

- **Checksum Verification**: SHA256 verification of downloads
- **HTTPS Only**: Secure downloads from official sources
- **Package Verification**: Verification of package integrity
- **Isolated Execution**: Sandboxed package installation

## Troubleshooting

### Installation Issues
```bash
# Check UV installation
vx uv --version

# Verify Python availability
vx uv --python python3 --version

# Check UV cache
vx uv cache info

# Force reinstall
vx remove uv
vx install uv
```

### Runtime Issues
```bash
# Debug UV commands
vx uv --verbose pip install requests

# Check UV configuration
vx uv pip config list

# Test with system Python
vx uv --python /usr/bin/python3 pip install requests
```

## License

This project is licensed under the MIT License - see the [LICENSE](../../../LICENSE) file for details.

## Contributing

Contributions are welcome! Please see the [contributing guidelines](../../../CONTRIBUTING.md) for more information.

## Related Crates

- [`vx-core`](../../vx-core/README.md) - Core functionality
- [`vx-cli`](../../vx-cli/README.md) - Command-line interface
- [`vx-tool-node`](../vx-tool-node/README.md) - Node.js tool
- [`vx-tool-python`](../vx-tool-python/README.md) - Python tool
