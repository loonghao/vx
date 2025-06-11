# Testing New Features

This document outlines how to test the new features added to vx.

## Virtual Environment Support

### 1. Create a virtual environment
```bash
vx venv create my-rust-env --tools cargo@latest,rustc@latest
```

### 2. List virtual environments
```bash
vx venv list
```

### 3. Activate virtual environment
```bash
# This will show the commands to run
vx venv activate my-rust-env

# To actually activate (copy and paste the output):
export VX_VENV=my-rust-env
export PATH=/path/to/vx/venvs/my-rust-env/bin:$PATH
export PS1="(vx:my-rust-env) $PS1"
```

### 4. Check current environment
```bash
vx venv current
```

### 5. Remove virtual environment
```bash
vx venv remove my-rust-env
```

## Separate Rust Tools

### 1. Check cargo tool
```bash
vx where cargo
```

### 2. Check rustc tool
```bash
vx where rustc
```

### 3. List all tools
```bash
vx list
```

## Environment Isolation Testing

### 1. Test without system path (default)
```bash
vx go version  # Should fail or use vx-managed version only
```

### 2. Test with system path
```bash
vx --use-system-path go version  # Should use system Go
```

## Expected Behavior

- Virtual environments should be created in `~/.config/vx/venvs/` (Linux/macOS) or `%APPDATA%\vx\venvs\` (Windows)
- Each virtual environment should have its own `bin/` and `config/` directories
- Activation should provide shell commands to modify PATH and environment variables
- Cargo and rustc should be separate tools in the tool registry
- Environment isolation should prevent fallback to system tools by default
