# vx-tool-python

[![Crates.io](https://img.shields.io/crates/v/vx-tool-python.svg)](https://crates.io/crates/vx-tool-python)
[![Documentation](https://docs.rs/vx-tool-python/badge.svg)](https://docs.rs/vx-tool-python)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Python programming language tool support for the vx universal tool manager.

## Status

🚧 **Under Development** - This crate is currently under development and not yet implemented.

## Overview

`vx-tool-python` will provide Python programming language support for vx, enabling automatic installation, version management, and execution of Python commands through the vx interface.

## Planned Features

- **Python Runtime**: Full Python interpreter support with version management
- **Pip Integration**: Built-in pip package manager support
- **Virtual Environment**: Python virtual environment management
- **Auto-Installation**: Automatic download and installation of Python versions
- **Cross-Platform**: Windows, macOS, and Linux support
- **Version Management**: Install and switch between multiple Python versions
- **Package Management**: Integration with pip and other Python package managers

## Planned Commands

### Python Interpreter
```bash
# Execute Python scripts (planned)
vx python script.py
vx python --version
vx python -c "print('Hello, World!')"

# Interactive REPL (planned)
vx python
```

### Pip Package Manager
```bash
# Package management (planned)
vx pip install requests
vx pip install -r requirements.txt
vx pip uninstall requests
vx pip list
vx pip freeze
```

### Virtual Environment
```bash
# Virtual environment management (planned)
vx python -m venv myenv
vx python -m venv --system-site-packages myenv
```

## Current Status

This crate is currently in the planning phase. Python support is currently provided through the [`vx-tool-uv`](../vx-tool-uv/README.md) crate, which offers UV package manager functionality.

For immediate Python development needs, please use:
- [`vx-tool-uv`](../vx-tool-uv/README.md) - UV Python package manager (fast pip replacement)

## Development Roadmap

1. **Phase 1**: Basic Python interpreter installation and execution
2. **Phase 2**: Pip package manager integration
3. **Phase 3**: Virtual environment management
4. **Phase 4**: Advanced features (pyenv compatibility, conda support)

## Contributing

This crate is not yet implemented. If you're interested in contributing to Python support in vx, please:

1. Check the main project [issues](https://github.com/loonghao/vx/issues)
2. Join the discussion about Python tool support
3. See the [contributing guidelines](../../../CONTRIBUTING.md)

## Alternative Solutions

While this crate is under development, consider these alternatives:

### UV (Recommended)
```bash
# Use UV for fast Python package management
vx uv pip install requests
vx uvx black .
vx uvx pytest
```

### System Python
```bash
# Use system Python with vx
vx --use-system-path python --version
vx --use-system-path pip install requests
```

## License

This project is licensed under the MIT License - see the [LICENSE](../../../LICENSE) file for details.

## Related Crates

- [`vx-core`](../../vx-core/README.md) - Core functionality
- [`vx-cli`](../../vx-cli/README.md) - Command-line interface
- [`vx-tool-uv`](../vx-tool-uv/README.md) - UV Python tool (available now)
- [`vx-tool-node`](../vx-tool-node/README.md) - Node.js tool
