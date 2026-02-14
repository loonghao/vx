# Provider Update Templates

Complete templates for updating provider.toml files to RFC 0018 + RFC 0019 standards.

## Template Selection Guide

| Tool Type | Download Format | Layout Type | Template |
|-----------|-----------------|-------------|----------|
| Single binary | Direct download | `binary` | Template 1 |
| Standard tool | Archive with bin/ | `archive` | Template 2 |
| Compact tool | Archive, root exec | `archive` | Template 3 |
| Platform dirs | Archive with {os}-{arch}/ | `archive` | Template 4 |
| Complex | Archive, nested | `archive` | Template 5 |
| npm package | npm install | None | Template 6 |
| pip package | pip install | None | Template 7 |
| System tool | OS package | None | Template 8 |

## Template 1: Single Binary Download

**Use for**: kubectl, ninja, rustup-init, yasm, rcedit

**Characteristics**:
- Single executable file downloaded directly
- May need renaming (e.g., `tool-1.0.0.exe` → `tool.exe`)
- No archive extraction needed

```toml
[[runtimes]]
name = "tool-name"
description = "Tool description"
executable = "tool-name"

[runtimes.versions]
source = "github-releases"
owner = "owner-name"
repo = "repo-name"
strip_v_prefix = true

# RFC 0019: Binary Layout Configuration
[runtimes.layout]
download_type = "binary"

[runtimes.layout.binary."windows-x86_64"]
source_name = "tool-name.exe"  # Or "tool-name-{version}-win64.exe"
target_name = "tool-name.exe"
target_dir = "bin"

[runtimes.layout.binary."macos-x86_64"]
source_name = "tool-name"  # Or "tool-name-{version}-darwin-amd64"
target_name = "tool-name"
target_dir = "bin"
target_permissions = "755"

[runtimes.layout.binary."macos-aarch64"]
source_name = "tool-name"  # Or "tool-name-{version}-darwin-arm64"
target_name = "tool-name"
target_dir = "bin"
target_permissions = "755"

[runtimes.layout.binary."linux-x86_64"]
source_name = "tool-name"  # Or "tool-name-{version}-linux-amd64"
target_name = "tool-name"
target_dir = "bin"
target_permissions = "755"

[runtimes.layout.binary."linux-aarch64"]
source_name = "tool-name"  # Or "tool-name-{version}-linux-arm64"
target_name = "tool-name"
target_dir = "bin"
target_permissions = "755"

[runtimes.platforms.windows]
executable_extensions = [".exe"]

[runtimes.platforms.unix]
executable_extensions = []
```

## Template 2: Standard Archive (bin/ directory)

**Use for**: node, go, python, cmake, nasm, protoc, ffmpeg

**Characteristics**:
- Archive contains versioned top directory
- Executables in `bin/` subdirectory
- Standard layout across platforms

```toml
[[runtimes]]
name = "tool-name"
description = "Tool description"
executable = "tool-name"

[runtimes.versions]
source = "github-releases"  # or "nodejs-org", "go-dev", etc.
owner = "owner-name"
repo = "repo-name"
strip_v_prefix = true

# RFC 0019: Archive Layout Configuration
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "tool-name-{version}"  # Adjust pattern as needed
executable_paths = [
    "bin/tool-name.exe",  # Windows
    "bin/tool-name"       # Unix
]

[runtimes.platforms.windows]
executable_extensions = [".exe"]

[runtimes.platforms.unix]
executable_extensions = []
```

**Common strip_prefix patterns**:
- Node.js: `"node-v{version}-{os}-{arch}"`
- Go: `"go"`
- CMake: `"cmake-{version}-{os}-{arch}"`
- Python: `"python"`
- NASM: `"nasm-{version}"`

## Template 3: Root Directory Executable

**Use for**: terraform, just, task, deno, zig, pnpm, uv

**Characteristics**:
- Archive with executable in root directory
- No nested directory structure
- Simple layout

```toml
[[runtimes]]
name = "tool-name"
description = "Tool description"
executable = "tool-name"

[runtimes.versions]
source = "github-releases"
owner = "owner-name"
repo = "repo-name"
strip_v_prefix = true

# RFC 0019: Archive Layout Configuration
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = ""  # No prefix to remove
executable_paths = [
    "tool-name.exe",  # Windows (root directory)
    "tool-name"       # Unix (root directory)
]

[runtimes.platforms.windows]
executable_extensions = [".exe"]

[runtimes.platforms.unix]
executable_extensions = []
```

## Template 4: Platform-Specific Directories

**Use for**: helm, bun, kubectl (some versions)

**Characteristics**:
- Archive organized by platform (`linux-amd64/`, `darwin-arm64/`)
- Platform directory needs stripping
- Executables may be in root or bin/ after strip

```toml
[[runtimes]]
name = "tool-name"
description = "Tool description"
executable = "tool-name"

[runtimes.versions]
source = "github-releases"
owner = "owner-name"
repo = "repo-name"
strip_v_prefix = true

# RFC 0019: Archive Layout Configuration
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "{os}-{arch}"  # Or "tool-name-{os}-{arch}"
executable_paths = [
    "tool-name.exe",  # Windows
    "tool-name"       # Unix
]

[runtimes.platforms.windows]
executable_extensions = [".exe"]

[runtimes.platforms.unix]
executable_extensions = []
```

**Common variations**:
- Helm: `"{os}-{arch}"`
- Bun: `"bun-{os}-{arch}"`
- Zig: `"zig-{os}-{arch}-{version}"`

## Template 5: Complex Nested Structure

**Use for**: java, rust (full toolchain), complex SDKs

**Characteristics**:
- Deep directory nesting
- Version or build info in paths
- May vary by platform

```toml
[[runtimes]]
name = "tool-name"
description = "Tool description"
executable = "tool-name"

[runtimes.versions]
source = "custom"  # Or github-releases with complex parsing

# RFC 0019: Archive Layout Configuration
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "tool-{version}+{build}"  # Adjust as needed
executable_paths = [
    "bin/tool.exe",  # Windows
    "bin/tool"       # Unix
]

[runtimes.platforms.windows]
executable_extensions = [".exe"]

[runtimes.platforms.unix]
executable_extensions = []
```

**Examples**:
- Java JDK: `strip_prefix = "jdk-{version}+{build}"`
- Node.js: `strip_prefix = "node-v{version}-{os}-{arch}"`

## Template 6: npm Package

**Use for**: vite, release-please, typescript, webpack

**Characteristics**:
- Installed via npm
- Managed by Node.js package manager
- No layout configuration needed

```toml
[[runtimes]]
name = "tool-name"
description = "Tool description"
executable = "tool-name"

[runtimes.versions]
source = "npm"
package = "package-name"  # npm package name

# Note: npm packages don't need layout configuration
# They are installed via npm and have standard locations

[runtimes.platforms.windows]
executable_extensions = [".cmd", ".exe"]

[runtimes.platforms.unix]
executable_extensions = []

# Optional: npm packages typically require Node.js
[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "node", version = ">=14", reason = "npm packages require Node.js" }
]
```

## Template 7: pip/uv Package

**Use for**: pre-commit, poetry, black, ruff

**Characteristics**:
- Installed via pip or uv
- Python package ecosystem
- No layout configuration needed

```toml
[[runtimes]]
name = "tool-name"
description = "Tool description"
executable = "tool-name"

[runtimes.versions]
source = "pypi"
package = "package-name"  # PyPI package name

# Note: pip/uv packages don't need layout configuration
# They are installed via package manager

[runtimes.platforms.windows]
executable_extensions = [".exe"]

[runtimes.platforms.unix]
executable_extensions = []

# Optional: may require Python/uv
[[runtimes.constraints]]
when = "*"
recommends = [
    { runtime = "uv", version = "*", reason = "Fast Python package installer" }
]
```

## Template 8: System Tool (Detection Only)

**Use for**: git, docker, curl, openssl, systemctl, msbuild

**Characteristics**:
- Typically installed by OS package manager
- System-wide installation
- Use detection to find existing installations

```toml
[[runtimes]]
name = "tool-name"
description = "Tool description"
executable = "tool-name"

# Note: System tools typically installed via OS package manager
# No versions or layout configuration needed
# Use detection to find system installations

# RFC 0018: Detection Configuration
[runtimes.detection]
command = "{executable} --version"
pattern = "tool-name version ([\\d.]+)"
system_paths = [
    "/usr/bin/{executable}",
    "/usr/local/bin/{executable}",
    "C:\\Program Files\\Tool\\bin\\{executable}.exe"
]
env_hints = ["TOOL_HOME", "TOOL_PATH"]

[runtimes.platforms.windows]
executable_extensions = [".exe"]

[runtimes.platforms.unix]
executable_extensions = []
```

## Special Cases

### Bundled Tools (npm with Node.js)

```toml
[[runtimes]]
name = "npm"
description = "Node package manager"
executable = "npm"
bundled_with = "node"  # Key: bundled with Node.js

# No versions or layout needed
# npm comes with Node.js

[runtimes.platforms.windows]
executable_extensions = [".cmd", ".ps1"]

[runtimes.platforms.unix]
executable_extensions = []

[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "node", version = "*", reason = "npm is bundled with Node.js" }
]
```

### JavaScript Executables (Yarn)

```toml
[[runtimes]]
name = "yarn"
description = "JavaScript package manager"
executable = "yarn"

[runtimes.versions]
source = "github-releases"
owner = "yarnpkg"
repo = "yarn"
strip_v_prefix = true

# RFC 0019: Archive Layout (JavaScript file)
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "yarn-v{version}"
executable_paths = [
    "bin/yarn.js"  # JavaScript file, not native binary
]

[runtimes.platforms.windows]
executable_extensions = [".cmd", ".ps1"]

[runtimes.platforms.unix]
executable_extensions = []
```

### Windows-Only Tools (rcedit)

```toml
[[runtimes]]
name = "rcedit"
description = "Windows resource editor"
executable = "rcedit"

[runtimes.versions]
source = "github-releases"
owner = "electron"
repo = "rcedit"
strip_v_prefix = false

# RFC 0019: Binary Layout (Windows only)
[runtimes.layout]
download_type = "binary"

[runtimes.layout.binary."windows-x86_64"]
source_name = "rcedit-x64.exe"
target_name = "rcedit.exe"
target_dir = "bin"

# Only Windows platform
[runtimes.platforms.windows]
executable_extensions = [".exe"]
```

### Installer-Based Tools (Not Yet Supported)

```toml
[[runtimes]]
name = "awscli"
description = "AWS Command Line Interface"
executable = "aws"

[runtimes.versions]
source = "github-releases"
owner = "aws"
repo = "aws-cli"
strip_v_prefix = false

# Note: AWS CLI v2 uses platform-specific installers (.msi, .pkg, etc.)
# Layout configuration will be added when installer support is implemented

[runtimes.platforms.windows]
executable_extensions = [".exe", ".cmd"]

[runtimes.platforms.unix]
executable_extensions = []
```

## Variable Reference

Variables that can be used in layout configuration:

| Variable | Description | Example Values |
|----------|-------------|----------------|
| `{version}` | Runtime version | `20.0.0`, `1.21.0` |
| `{os}` | Operating system | `windows`, `linux`, `darwin` |
| `{arch}` | Architecture | `x86_64`, `aarch64`, `arm64`, `amd64` |
| `{name}` | Runtime name | `node`, `go`, `python` |

**Note**: Variables are replaced automatically by the installer.

## Platform Identifiers

### For Binary Layout

Full platform specifiers:
- `windows-x86_64`
- `windows-aarch64`
- `macos-x86_64`
- `macos-aarch64`
- `linux-x86_64`
- `linux-aarch64`

### For Archive strip_prefix

OS names (use in `{os}`):
- `windows`
- `linux`
- `darwin` (macOS)

Arch names (use in `{arch}`):
- `x86_64` or `amd64` (64-bit Intel/AMD)
- `aarch64` or `arm64` (64-bit ARM)
- `i686` or `x86` (32-bit Intel/AMD)

## Best Practices

1. **Use variables** instead of hardcoding versions/platforms
2. **Set Unix permissions** (`target_permissions = "755"`) for all Unix platforms
3. **Use forward slashes** in paths (works on all platforms)
4. **Test all platforms** before committing
5. **Document special cases** with comments
6. **Keep it simple** - use the simplest template that works

## Quick Decision Tree

```
What's the download format?
├─ Single binary file
│  └─ Use Template 1 (Binary)
├─ Archive (.tar.gz, .zip)
│  ├─ Has bin/ directory?
│  │  └─ Use Template 2 (Standard Archive)
│  ├─ Executable in root?
│  │  └─ Use Template 3 (Root Directory)
│  ├─ Platform subdirectories?
│  │  └─ Use Template 4 (Platform Directories)
│  └─ Complex nesting?
│     └─ Use Template 5 (Complex)
├─ npm package?
│  └─ Use Template 6 (npm)
├─ pip/PyPI package?
│  └─ Use Template 7 (pip)
└─ System installation?
   └─ Use Template 8 (System Tool)
```
