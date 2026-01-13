# Manifest-Driven Providers

vx uses a **declarative manifest system** to define tools and their behaviors. Instead of writing Rust code for each tool, you can simply create a `provider.toml` file that describes everything vx needs to know about a tool.

## Overview

A manifest-driven provider is defined by a single `provider.toml` file that contains:

- **Provider metadata**: Name, description, homepage, ecosystem
- **Runtime definitions**: Executables, aliases, version sources
- **Platform configurations**: OS-specific settings, download URLs
- **Dependencies**: What other tools are required or recommended
- **Detection rules**: How to find existing installations

This approach makes it easy to:
- Add new tools without writing code
- Customize tool behavior through configuration
- Share tool definitions across teams
- Maintain consistent tool management

## Quick Start

### Using Built-in Providers

vx comes with 40+ built-in providers for popular tools. Just use them directly:

```bash
# These tools are already defined in vx
vx node --version      # Node.js
vx go version          # Go
vx jq --help           # jq JSON processor
vx ffmpeg -version     # FFmpeg media toolkit
```

### Creating a Custom Provider

To add a new tool, create a `provider.toml` file:

```bash
# Create provider directory
mkdir -p ~/.vx/providers/mytool

# Create the manifest
cat > ~/.vx/providers/mytool/provider.toml << 'EOF'
[provider]
name = "mytool"
description = "My awesome tool"
homepage = "https://mytool.example.com"
ecosystem = "devtools"

[[runtimes]]
name = "mytool"
description = "My tool runtime"
executable = "mytool"

[runtimes.versions]
source = "github-releases"
owner = "myorg"
repo = "mytool"

[runtimes.platforms.windows]
executable_extensions = [".exe"]

[runtimes.platforms.unix]
executable_extensions = []
EOF

# Now use it!
vx mytool --version
```

## Provider Manifest Schema

### Provider Section

The `[provider]` section defines basic metadata:

```toml
[provider]
name = "jq"                                          # Required: Provider name
description = "Lightweight command-line JSON processor"  # Required: Description
homepage = "https://jqlang.github.io/jq/"            # Optional: Homepage URL
repository = "https://github.com/jqlang/jq"          # Optional: Source repository
ecosystem = "devtools"                               # Optional: Ecosystem category
```

**Ecosystem values:**
- `nodejs` - Node.js ecosystem (npm, yarn, etc.)
- `python` - Python ecosystem (pip, uv, etc.)
- `rust` - Rust ecosystem (cargo, etc.)
- `go` - Go ecosystem
- `system` - System tools (ffmpeg, git, etc.)
- `devtools` - Development tools (jq, fzf, etc.)
- `cloud` - Cloud CLI tools (aws, gcloud, etc.)

### Runtime Section

Each `[[runtimes]]` entry defines an executable:

```toml
[[runtimes]]
name = "jq"                              # Required: Runtime name
description = "jq JSON processor"        # Required: Description
executable = "jq"                        # Required: Executable name
aliases = ["jqp"]                        # Optional: Alternative names
priority = 100                           # Optional: Priority (higher = preferred)
auto_installable = true                  # Optional: Can be auto-installed (default: true)
bundled_with = "node"                    # Optional: Bundled with another runtime
```

### Version Sources

The `[runtimes.versions]` section defines where to get version information:

#### GitHub Releases

```toml
[runtimes.versions]
source = "github-releases"
owner = "jqlang"
repo = "jq"
strip_v_prefix = true                    # Remove 'v' from version tags
```

#### Node.js Official

```toml
[runtimes.versions]
source = "nodejs-org"
lts_pattern = "lts/*"
```

#### System Detection

```toml
[runtimes.versions]
source = "system"                        # Detect from system installation
```

### Platform Configuration

Define platform-specific settings:

```toml
[runtimes.platforms.windows]
executable_extensions = [".exe", ".cmd"]
search_paths = ["C:\\Program Files\\tool\\bin"]

[runtimes.platforms.unix]
executable_extensions = []
search_paths = ["/usr/bin", "/usr/local/bin"]

[runtimes.platforms.linux]
executable_extensions = []

[runtimes.platforms.macos]
executable_extensions = []
```

### Download Layout

Configure how downloads are structured:

#### Binary Downloads

For single-file executables:

```toml
[runtimes.layout]
download_type = "binary"

[runtimes.layout.binary."windows-x86_64"]
source_name = "jq-windows-amd64.exe"
target_name = "jq.exe"
target_dir = "bin"

[runtimes.layout.binary."linux-x86_64"]
source_name = "jq-linux-amd64"
target_name = "jq"
target_dir = "bin"
target_permissions = "755"
```

#### Archive Downloads

For tools distributed as archives:

```toml
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "node-v{version}-{platform}-{arch}"
executable_paths = [
    "bin/node.exe",   # Windows
    "bin/node"        # Unix
]
```

### Detection Configuration

Define how to detect existing installations:

```toml
[runtimes.detection]
command = "{executable} --version"
pattern = "v?(\\d+\\.\\d+\\.\\d+)"
system_paths = [
    "/usr/bin/node",
    "/usr/local/bin/node",
    "C:\\Program Files\\nodejs\\node.exe"
]
env_hints = ["NODE_HOME", "NVM_DIR"]
```

### Dependencies and Constraints

Define relationships between tools:

```toml
# Recommend npm when using node
[[runtimes.constraints]]
when = "*"
recommends = [
    { runtime = "npm", version = "*", reason = "Default package manager" }
]

# Require node for npm
[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "node", version = ">=14", recommended = "20", reason = "npm requires Node.js" }
]

# Version-specific constraints
[[runtimes.constraints]]
when = ">=9"
requires = [
    { runtime = "node", version = ">=14", reason = "npm 9.x+ requires Node.js 14+" }
]
```

### Environment Variables

Configure environment variables:

```toml
[runtimes.env]
vars = { PATH = "{install_dir}/bin" }

# Version-conditional environment
[runtimes.env.conditional]
">=18" = { NODE_OPTIONS = "--experimental-vm-modules" }
```

### Mirror Configuration

Define download mirrors for different regions:

```toml
[[runtimes.mirrors]]
name = "taobao"
region = "cn"
url = "https://npmmirror.com/mirrors/node"
priority = 100

[[runtimes.mirrors]]
name = "ustc"
region = "cn"
url = "https://mirrors.ustc.edu.cn/node"
priority = 90
```

### Health Checks

Define health check commands:

```toml
[runtimes.health]
check_command = "{executable} --version"
expected_pattern = "v\\d+\\.\\d+\\.\\d+"
exit_code = 0
timeout_ms = 5000
check_on = ["install", "activate"]
```

### Download Configuration

Configure download behavior:

```toml
[runtimes.download]
timeout_ms = 900000           # 15 minutes for large files
max_retries = 5
resume_enabled = true
execution_timeout_ms = 60000  # 1 minute execution timeout
```

### Test Configuration

Define automated tests for the provider:

```toml
[runtimes.test]
timeout_ms = 30000
functional_commands = [
    { command = "{executable} --version", expect_success = true, expected_output = "v\\d+", name = "version_check" },
    { command = "{executable} -e \"console.log('test')\"", expect_success = true, expected_output = "test", name = "eval_test" }
]
install_verification = [
    { command = "{executable} --version", expect_success = true }
]
```

## Real-World Examples

### Simple Binary Tool (jq)

```toml
[provider]
name = "jq"
description = "Lightweight and flexible command-line JSON processor"
homepage = "https://jqlang.github.io/jq/"
repository = "https://github.com/jqlang/jq"
ecosystem = "devtools"

[[runtimes]]
name = "jq"
description = "jq - command-line JSON processor"
executable = "jq"

[runtimes.versions]
source = "github-releases"
owner = "jqlang"
repo = "jq"
strip_v_prefix = true

[runtimes.layout]
download_type = "binary"

[runtimes.layout.binary."windows-x86_64"]
source_name = "jq-windows-amd64.exe"
target_name = "jq.exe"
target_dir = "bin"

[runtimes.layout.binary."linux-x86_64"]
source_name = "jq-linux-amd64"
target_name = "jq"
target_dir = "bin"
target_permissions = "755"

[runtimes.layout.binary."macos-x86_64"]
source_name = "jq-macos-amd64"
target_name = "jq"
target_dir = "bin"
target_permissions = "755"

[runtimes.platforms.windows]
executable_extensions = [".exe"]

[runtimes.platforms.unix]
executable_extensions = []

[[runtimes.constraints]]
when = "*"
recommends = []
```

### Multi-Runtime Provider (Node.js)

```toml
[provider]
name = "node"
description = "JavaScript runtime built on Chrome's V8 engine"
homepage = "https://nodejs.org"
ecosystem = "nodejs"

# Main runtime
[[runtimes]]
name = "node"
description = "Node.js runtime"
executable = "node"
aliases = ["nodejs"]
priority = 100
auto_installable = true

[runtimes.versions]
source = "nodejs-org"
lts_pattern = "lts/*"

[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "node-v{version}-{platform}-{arch}"
executable_paths = ["bin/node.exe", "bin/node"]

[[runtimes.constraints]]
when = "*"
recommends = [
    { runtime = "npm", version = "*", reason = "Default package manager" }
]

# Bundled npm
[[runtimes]]
name = "npm"
description = "Node Package Manager"
executable = "npm"
bundled_with = "node"

[runtimes.platforms.windows]
executable_extensions = [".cmd", ".exe"]

[runtimes.platforms.unix]
executable_extensions = []

[[runtimes.constraints]]
when = ">=9"
requires = [
    { runtime = "node", version = ">=14", reason = "npm 9.x+ requires Node.js 14+" }
]

# Bundled npx
[[runtimes]]
name = "npx"
description = "Node Package Execute"
executable = "npx"
bundled_with = "node"

[runtimes.platforms.windows]
executable_extensions = [".cmd", ".exe"]

[runtimes.platforms.unix]
executable_extensions = []
```

### System Tool (systemctl)

```toml
[provider]
name = "systemctl"
description = "systemd system and service manager"
homepage = "https://systemd.io"
ecosystem = "system"

# Platform restriction
[provider.platforms]
os = ["linux"]

[[runtimes]]
name = "systemctl"
description = "Control systemd services and units"
executable = "systemctl"
auto_installable = false  # Cannot be auto-installed

[runtimes.versions]
source = "system"

[runtimes.detection]
command = "{executable} --version"
pattern = "systemd ([\\d.]+)"
system_paths = ["/usr/bin/systemctl", "/bin/systemctl"]

[runtimes.platforms.linux]
executable_extensions = []
search_paths = ["/usr/bin", "/bin"]

# Bundled tools
[[runtimes]]
name = "journalctl"
description = "View systemd journal logs"
executable = "journalctl"
bundled_with = "systemctl"
auto_installable = false

[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "systemctl", version = "*", reason = "journalctl is part of systemd" }
]
```

## Provider Directory Structure

vx loads providers from multiple locations:

```
~/.vx/providers/          # User-defined providers (highest priority)
├── mytool/
│   └── provider.toml
└── custom-node/
    └── provider.toml

$VX_PROVIDERS_PATH/       # Environment variable path
└── team-tools/
    └── provider.toml

Built-in providers        # Lowest priority
```

**Loading Priority:**
1. `~/.vx/providers/*/provider.toml` (user local, highest)
2. `$VX_PROVIDERS_PATH/*/provider.toml` (environment variable)
3. Built-in providers (lowest)

## Best Practices

### 1. Use Descriptive Names

```toml
# Good
name = "ripgrep"
description = "Fast line-oriented search tool, recursively searches directories"

# Avoid
name = "rg"
description = "Search tool"
```

### 2. Define All Platforms

```toml
# Support all major platforms
[runtimes.layout.binary."windows-x86_64"]
source_name = "tool-windows-amd64.exe"

[runtimes.layout.binary."linux-x86_64"]
source_name = "tool-linux-amd64"

[runtimes.layout.binary."linux-aarch64"]
source_name = "tool-linux-arm64"

[runtimes.layout.binary."macos-x86_64"]
source_name = "tool-darwin-amd64"

[runtimes.layout.binary."macos-aarch64"]
source_name = "tool-darwin-arm64"
```

### 3. Set Appropriate Timeouts

```toml
# Small tools (< 10MB)
[runtimes.download]
timeout_ms = 60000  # 1 minute

# Large tools (> 100MB like FFmpeg)
[runtimes.download]
timeout_ms = 900000  # 15 minutes
resume_enabled = true
```

### 4. Document Dependencies

```toml
[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "node", version = ">=14", reason = "Requires Node.js 14+ for ES modules support" }
]
recommends = [
    { runtime = "npm", version = "*", reason = "Recommended for package management" }
]
```

### 5. Add Health Checks

```toml
[runtimes.health]
check_command = "{executable} --version"
expected_pattern = "\\d+\\.\\d+\\.\\d+"
timeout_ms = 5000
```

## Troubleshooting

### Provider Not Found

```bash
# Check if provider is loaded
vx list

# Verify provider.toml location
ls ~/.vx/providers/mytool/provider.toml
```

### Version Detection Fails

```bash
# Test detection pattern manually
mytool --version

# Check detection configuration
cat ~/.vx/providers/mytool/provider.toml | grep -A5 "\[runtimes.detection\]"
```

### Download Fails

1. Check network connectivity
2. Verify download URL format in manifest
3. Try increasing timeout:
   ```toml
   [runtimes.download]
   timeout_ms = 300000
   max_retries = 5
   ```

## See Also

- [Provider Development Guide](../advanced/extension-development.md) - For Rust-based providers
- [Configuration Reference](../config/vx-toml.md) - Project configuration
- [CLI Commands](../cli/overview.md) - Command reference
