# RFC 0019: Executable Layout Configuration

Complete guide for adding layout configuration to provider.toml files.

## Overview

RFC 0019 standardizes how VX handles different download formats (binary files, archives) and their internal structure. This eliminates the need for custom `post_extract` hooks in most cases.

## When to Use Layout Configuration

Add `[runtimes.layout]` configuration when:
- **Binary downloads**: Single file needs renaming or specific placement
- **Archives**: Executables are in nested directories or need path mapping
- **Complex structures**: Different platforms have different directory layouts

## Download Types

### 1. Binary Download

Single executable file downloaded directly (not in an archive).

**Use cases:**
- kubectl (single binary per platform)
- ninja (single binary)
- rust (rustup-init needs renaming)

**Configuration:**

```toml
[runtimes.layout]
download_type = "binary"

[runtimes.layout.binary."windows-x86_64"]
source_name = "tool.exe"           # Downloaded file name
target_name = "tool.exe"           # Final executable name
target_dir = "bin"                 # Target directory (relative to install root)

[runtimes.layout.binary."macos-x86_64"]
source_name = "tool"
target_name = "tool"
target_dir = "bin"
target_permissions = "755"         # Unix permissions (chmod)

[runtimes.layout.binary."macos-aarch64"]
source_name = "tool"
target_name = "tool"
target_dir = "bin"
target_permissions = "755"

[runtimes.layout.binary."linux-x86_64"]
source_name = "tool"
target_name = "tool"
target_dir = "bin"
target_permissions = "755"

[runtimes.layout.binary."linux-aarch64"]
source_name = "tool"
target_name = "tool"
target_dir = "bin"
target_permissions = "755"
```

### 2. Archive Download

Compressed archive (`.tar.gz`, `.zip`, `.tar.xz`) containing files.

**Use cases:**
- Standard tools with `bin/` directory (node, go, cmake)
- Root directory executables (terraform, just, deno)
- Platform-specific directories (helm, bun)

**Configuration:**

```toml
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "{pattern}"         # Remove this directory prefix
executable_paths = [               # Paths to find executables (after strip)
    "bin/tool.exe",               # Windows
    "bin/tool"                    # Unix
]
```

## Common Patterns

### Pattern 1: Binary with Version in Name

**Scenario:** Downloaded as `yasm-1.3.0-win64.exe`, need `bin/yasm.exe`

```toml
[runtimes.layout]
download_type = "binary"

[runtimes.layout.binary."windows-x86_64"]
source_name = "yasm-{version}-win64.exe"  # {version} is replaced
target_name = "yasm.exe"
target_dir = "bin"

[runtimes.layout.binary."macos-x86_64"]
source_name = "yasm-{version}-macos"
target_name = "yasm"
target_dir = "bin"
target_permissions = "755"

[runtimes.layout.binary."linux-x86_64"]
source_name = "yasm-{version}-linux"
target_name = "yasm"
target_dir = "bin"
target_permissions = "755"
```

### Pattern 2: Standard Archive with bin/

**Scenario:** Archive contains `node-v20.0.0-linux-x64/bin/node`

```toml
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "node-v{version}-{os}-{arch}"  # Removes top directory
executable_paths = [
    "bin/node.exe",  # Windows
    "bin/node"       # Unix
]
```

**Variables in strip_prefix:**
- `{version}` - Runtime version (e.g., "20.0.0")
- `{os}` - Operating system (windows, linux, darwin)
- `{arch}` - Architecture (x86_64, aarch64, arm64, amd64)

### Pattern 3: Root Directory Executable

**Scenario:** Archive contains `terraform.exe` or `terraform` directly

```toml
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = ""                 # No prefix to remove
executable_paths = [
    "terraform.exe",              # Windows (root)
    "terraform"                   # Unix (root)
]
```

### Pattern 4: Platform-Specific Directories

**Scenario:** Archive contains `linux-amd64/helm`, `darwin-arm64/helm`

```toml
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "{os}-{arch}"      # Remove platform directory
executable_paths = [
    "helm.exe",                   # Windows
    "helm"                        # Unix
]
```

### Pattern 5: Complex Nested Structure

**Scenario:** Different structures per platform (e.g., Go)

```toml
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "go"
executable_paths = [
    "bin/go.exe",                 # All platforms have bin/
    "bin/go"
]
```

### Pattern 6: Python Standalone Builds

**Scenario:** Complex structure with `python/bin/python3`

```toml
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "python"
executable_paths = [
    "bin/python3.exe",            # Windows
    "bin/python3"                 # Unix
]
```

## Platform Identifiers

### Operating Systems
- `windows` - Windows
- `linux` - Linux
- `darwin` / `macos` - macOS

### Architectures
- `x86_64` / `amd64` - 64-bit Intel/AMD
- `aarch64` / `arm64` - 64-bit ARM
- `i686` / `x86` - 32-bit Intel/AMD

### Platform Strings (for binary layout)
- `windows-x86_64`
- `windows-aarch64`
- `macos-x86_64`
- `macos-aarch64`
- `linux-x86_64`
- `linux-aarch64`

## Migration Examples

### Example 1: YASM (Binary)

**Before (custom post_extract):**
```rust
fn post_extract(&self, version: &str, install_path: &PathBuf) -> Result<()> {
    // Manual renaming logic...
}
```

**After (RFC 0019):**
```toml
[runtimes.layout]
download_type = "binary"

[runtimes.layout.binary."windows-x86_64"]
source_name = "yasm-{version}-win64.exe"
target_name = "yasm.exe"
target_dir = "bin"

[runtimes.layout.binary."linux-x86_64"]
source_name = "yasm-{version}-linux"
target_name = "yasm"
target_dir = "bin"
target_permissions = "755"
```

### Example 2: Node.js (Archive)

**Before (custom post_extract):**
```rust
fn post_extract(&self, _version: &str, install_path: &PathBuf) -> Result<()> {
    // Find and flatten nested directory...
}
```

**After (RFC 0019):**
```toml
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "node-v{version}-{os}-{arch}"
executable_paths = [
    "bin/node.exe",
    "bin/node"
]
```

### Example 3: Terraform (Archive, Root)

```toml
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = ""
executable_paths = [
    "terraform.exe",
    "terraform"
]
```

### Example 4: Helm (Archive, Platform Dir)

```toml
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "{os}-{arch}"
executable_paths = [
    "helm.exe",
    "helm"
]
```

## Special Cases

### JavaScript Files (Yarn)

For tools that are JavaScript files (not native executables):

```toml
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "yarn-v{version}"
executable_paths = [
    "bin/yarn.js"  # JavaScript file
]
```

### Bundled Tools (npm, npx)

Tools bundled with another runtime don't need layout configuration:

```toml
[[runtimes]]
name = "npm"
executable = "npm"
bundled_with = "node"  # No layout needed

[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "node", version = "*", reason = "npm is bundled with Node.js" }
]
```

### System Tools (git, docker)

System tools typically use detection only, but can have layout for portable installs:

```toml
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "PortableGit-{version}"
executable_paths = [
    "bin/git.exe",
    "bin/git"
]

# Detection for system-installed versions
[runtimes.detection]
command = "{executable} --version"
pattern = "git version ([\\d.]+)"
system_paths = ["/usr/bin/git", "C:\\Program Files\\Git\\bin\\git.exe"]
```

## Validation Checklist

When adding layout configuration, verify:

- [ ] `download_type` is either "binary" or "archive"
- [ ] For binary: All target platforms have configuration
- [ ] For binary: `target_permissions` set for Unix platforms
- [ ] For archive: `strip_prefix` matches actual archive structure
- [ ] For archive: `executable_paths` includes both Windows and Unix variants
- [ ] Variable placeholders (`{version}`, `{os}`, `{arch}`) are correctly used
- [ ] Test download and installation on all platforms

## Testing Layout Configuration

```bash
# Test installation with layout
vx install {tool}@{version}

# Verify executable location
vx which {tool}

# Test execution
vx {tool} --version
```

## Common Mistakes

### ❌ Incorrect: Using wrong separator in paths
```toml
executable_paths = ["bin\\node.exe"]  # Wrong: backslashes
```

### ✅ Correct: Always use forward slashes
```toml
executable_paths = ["bin/node.exe"]   # Right: forward slashes
```

### ❌ Incorrect: Forgetting Unix permissions
```toml
[runtimes.layout.binary."linux-x86_64"]
source_name = "tool"
target_name = "tool"
target_dir = "bin"
# Missing: target_permissions = "755"
```

### ✅ Correct: Set executable permissions
```toml
[runtimes.layout.binary."linux-x86_64"]
source_name = "tool"
target_name = "tool"
target_dir = "bin"
target_permissions = "755"  # Required for Unix
```

### ❌ Incorrect: Wrong strip_prefix pattern
```toml
strip_prefix = "tool-1.0.0"  # Hardcoded version
```

### ✅ Correct: Use variables
```toml
strip_prefix = "tool-{version}"  # Dynamic version
```

## Benefits of RFC 0019

1. **Declarative**: Configuration in TOML, no Rust code
2. **Consistent**: Same approach for all providers
3. **Maintainable**: Easy to update when download format changes
4. **Testable**: Configuration can be validated without running code
5. **Cross-platform**: Handles platform differences cleanly

## Migration Strategy

1. Add layout configuration to provider.toml
2. Remove custom post_extract implementation
3. Test installation on all platforms
4. Update documentation if needed
