# Environment Variables

vx respects various environment variables for configuration and behavior.

## vx Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `VX_HOME` | Override vx data directory | Platform-specific |
| `VX_CONFIG_DIR` | Override config directory | Platform-specific |
| `VX_CACHE_DIR` | Override cache directory | Platform-specific |
| `VX_AUTO_INSTALL` | Enable/disable auto-install | `true` |
| `VX_VERBOSE` | Enable verbose output | `false` |
| `VX_DEBUG` | Enable debug output | `false` |
| `VX_ENV` | Current environment name | `default` |

## Data Directories

### Default Locations

| Platform | Data | Config | Cache |
|----------|------|--------|-------|
| Linux | `~/.local/share/vx` | `~/.config/vx` | `~/.cache/vx` |
| macOS | `~/Library/Application Support/vx` | `~/.config/vx` | `~/Library/Caches/vx` |
| Windows | `%LOCALAPPDATA%\vx` | `%APPDATA%\vx` | `%LOCALAPPDATA%\vx\cache` |

### Override Directories

```bash
# Override all vx directories
export VX_HOME=/custom/path/vx

# Override specific directories
export VX_CONFIG_DIR=/custom/config
export VX_CACHE_DIR=/custom/cache
```

## Behavior Control

### Auto-Install

```bash
# Disable auto-install
export VX_AUTO_INSTALL=false

# Enable auto-install (default)
export VX_AUTO_INSTALL=true
```

### Verbose Output

```bash
# Enable verbose output
export VX_VERBOSE=true

# Or use flag
vx --verbose node --version
```

### Debug Output

```bash
# Enable debug output
export VX_DEBUG=true

# Or use flag
vx --debug node --version
```

## Environment Management

### Current Environment

```bash
# Set current environment
export VX_ENV=my-env

# Check current environment
echo $VX_ENV
```

### Environment Directory

When an environment is active:

```bash
VX_ENV=my-env
VX_ENV_DIR=/home/user/.local/share/vx/envs/my-env
```

## Tool-Specific Variables

### Node.js

```bash
NODE_ENV=production vx node server.js
```

### Go

```bash
GOPROXY=direct vx go get github.com/user/repo
CGO_ENABLED=0 vx go build
```

### Rust

```bash
CARGO_HOME=/custom/cargo vx cargo build
RUSTFLAGS="-C target-cpu=native" vx cargo build --release
```

### Python/UV

```bash
UV_CACHE_DIR=/custom/cache vx uv pip install requests
```

## Shell Integration

When shell integration is enabled, these variables are set automatically:

```bash
# Added to PATH
PATH="/home/user/.local/share/vx/envs/default:$PATH"

# Environment tracking
VX_ENV="default"
VX_ENV_DIR="/home/user/.local/share/vx/envs/default"
```

## CI/CD Usage

### GitHub Actions

```yaml
env:
  VX_AUTO_INSTALL: true
  VX_VERBOSE: true

steps:
  - run: vx setup
  - run: vx run test
```

### GitLab CI

```yaml
variables:
  VX_AUTO_INSTALL: "true"
  VX_HOME: "$CI_PROJECT_DIR/.vx"

script:
  - vx setup
  - vx run test
```

## Troubleshooting

### Check Current Settings

```bash
# Show all vx-related environment variables
env | grep VX_

# Show effective configuration
vx config show
```

### Reset Environment

```bash
# Unset all vx variables
unset VX_HOME VX_ENV VX_AUTO_INSTALL VX_VERBOSE VX_DEBUG
```
