# Real-World Use Cases

This guide showcases practical use cases for vx in various development scenarios.

## Windows C++ Development with MSVC

vx provides portable MSVC Build Tools support, enabling C++ compilation without installing Visual Studio.

### Basic MSVC Usage

```bash
# Install MSVC Build Tools
vx install msvc

# Compile a simple C++ program
vx cl main.cpp /Fe:main.exe

# Use nmake for build automation
vx nmake /f Makefile
```

### CMake with MSVC

vx automatically sets up the required environment variables (INCLUDE, LIB, PATH) for MSVC compilation:

```bash
# Install multiple tools at once
vx install msvc cmake ninja

# Configure with CMake using MSVC
vx cmake -B build -G "Ninja" -DCMAKE_C_COMPILER=cl -DCMAKE_CXX_COMPILER=cl

# Build the project
vx cmake --build build
```

### Environment Variables

When using MSVC through vx, the following environment variables are automatically configured:

| Variable | Description |
|----------|-------------|
| `INCLUDE` | Paths to MSVC and Windows SDK header files |
| `LIB` | Paths to MSVC and Windows SDK library files |
| `PATH` | Paths to MSVC compiler binaries |

This means you can use `vx cl` just like you would use `cl.exe` from a Visual Studio Developer Command Prompt.

## DCC (Digital Content Creation) Development

vx is particularly useful for DCC tool development, where you often need to compile plugins for applications like Maya, Houdini, and Unreal Engine.

### Maya Plugin Development

Maya plugins require compilation with specific MSVC versions. vx makes this easy:

```bash
# Install the required MSVC version
vx install msvc@14.29  # VS 2019 for Maya 2024

# Set up your project
vx cmake -B build -G "Ninja" \
  -DCMAKE_C_COMPILER=cl \
  -DCMAKE_CXX_COMPILER=cl \
  -DMAYA_ROOT="C:/Program Files/Autodesk/Maya2024"

# Build the plugin
vx cmake --build build
```

### Houdini Plugin Development

Houdini HDK development also benefits from vx's portable MSVC:

```bash
# Install MSVC and CMake
vx install msvc cmake

# Configure Houdini plugin build
vx cmake -B build -G "Ninja" \
  -DCMAKE_C_COMPILER=cl \
  -DCMAKE_CXX_COMPILER=cl \
  -DHOUDINI_ROOT="C:/Program Files/Side Effects Software/Houdini 20.0"

# Build
vx cmake --build build
```

### Unreal Engine Plugin Development

For Unreal Engine C++ development:

```bash
# Install MSVC Build Tools
vx install msvc

# Use Unreal Build Tool with vx-managed MSVC
# The UBT will automatically detect the compiler
```

## Python Development with UV

vx integrates seamlessly with uv for Python development:

### Project Setup

```bash
# Install uv
vx install uv

# Create a new Python project
vx uv init my-project
cd my-project

# Add dependencies
vx uv add requests numpy pandas

# Run your script
vx uv run python main.py
```

### Virtual Environment Management

```bash
# Create a virtual environment
vx uv venv

# Sync dependencies from pyproject.toml
vx uv sync

# Run tests
vx uv run pytest
```

## Task Automation with Just

vx works great with just for task automation:

```bash
# Install just
vx install just

# Run a task
vx just build

# List available tasks
vx just --list
```

### Example Justfile

```just
# Build the project
build:
    vx cmake --build build

# Run tests
test:
    vx uv run pytest

# Clean build artifacts
clean:
    rm -rf build/

# Full CI pipeline
ci: build test
```

## Node.js Development

### Project Setup

```bash
# Install Node.js and pnpm
vx install node pnpm

# Create a new project
vx pnpm init

# Add dependencies
vx pnpm add express

# Run the development server
vx pnpm run dev
```

### Using npx

```bash
# Create a React app
vx npx create-react-app my-app

# Run a one-off command
vx npx prettier --write .
```

## Go Development

```bash
# Install Go
vx install go

# Initialize a module
vx go mod init myproject

# Build the project
vx go build ./...

# Run tests
vx go test ./...
```

## Multi-Language Projects

vx shines in projects that use multiple languages:

```bash
# Install all required tools
vx install node uv go rust

# Or use vx.toml for declarative setup
cat > vx.toml << 'EOF'
[tools]
node = "22"
uv = "latest"
go = "1.22"
rust = "stable"
EOF

# Set up all tools
vx setup
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Build
on: [push, pull_request]

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v6
      
      - name: Setup vx
        uses: loonghao/vx@main
        
      - name: Install tools
        run: vx install msvc cmake ninja
        
      - name: Build
        run: |
          vx cmake -B build -G Ninja
          vx cmake --build build
```

### GitLab CI

```yaml
build:
  image: ubuntu:latest
  script:
    - curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | sh
    - vx install node uv
    - vx pnpm install
    - vx pnpm run build
```

## Tips and Best Practices

1. **Use vx.toml for reproducibility**: Define your tool versions in `vx.toml` for consistent environments across team members.

2. **Leverage auto-install**: vx automatically installs missing tools when you try to use them.

3. **Combine with task runners**: Use vx with just or make for powerful build automation.

4. **Environment isolation**: Each vx-managed tool is isolated, preventing version conflicts.

5. **Cross-platform scripts**: Write scripts using vx commands that work on Windows, macOS, and Linux.
