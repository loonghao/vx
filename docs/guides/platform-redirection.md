# Platform Redirection Guide

## Overview

vx uses a **platform-agnostic API** with automatic platform-specific storage. This allows:

1. **Unified API**: Access tools using `<provider>/<version>/` paths
2. **Platform-specific Storage**: Files stored in `<provider>/<version>/<platform>/` directories
3. **Automatic Redirection**: PathManager transparently redirects to current platform
4. **Offline Bundle Support**: Single bundle can contain all platforms

## Directory Structure

```
~/.vx/store/
├── node/
│   └── 20.0.0/           # Unified version directory (API)
│       ├── windows-x64/      # Platform-specific (storage)
│       ├── darwin-x64/
│       └── linux-x64/
├── python/
│   └── 3.9.21/
│       ├── windows-x64/
│       └── linux-x64/
└── uv/
    └── 0.5.0/
        ├── windows-x64/
        └── linux-x64/
```

## API Changes

### PathManager

New methods added to `vx_paths::PathManager`:

```rust
/// Get platform directory name for current platform
/// Returns: "windows-x64", "darwin-arm64", "linux-x64", etc.
pub fn platform_dir_name(&self) -> String

/// Get actual platform-specific store directory
/// Returns: ~/.vx/store/<runtime>/<version>/<platform>
pub fn platform_store_dir(&self, runtime_name: &str, version: &str) -> PathBuf

/// Get actual executable path in platform-specific directory
/// Returns: ~/.vx/store/<runtime>/<version>/<platform>/bin/<runtime>.exe
pub fn platform_executable_path(&self, runtime_name: &str, version: &str) -> PathBuf

/// Check if a runtime version is installed (checks platform-specific dir)
pub fn is_version_in_store(&self, runtime_name: &str, version: &str) -> bool

/// List all installed versions (checks platform-specific dirs)
pub fn list_store_versions(&self, runtime_name: &str) -> Result<Vec<String>>
```

### PathResolver

All store lookup methods now automatically use platform-specific directories:

```rust
// These all use platform_store_dir internally:
pub fn find_tool(&self, tool_name: &str) -> Result<Option<ToolLocation>>
pub fn find_in_store(&self, tool_name: &str) -> Result<Option<ToolLocation>>
pub fn find_in_store_with_exe(
    &self,
    tool_name: &str,
    exe_name: &str,
) -> Result<Option<ToolLocation>>
pub fn find_all_in_store(&self, tool_name: &str) -> Result<Vec<ToolLocation>>
pub fn find_all_in_store_with_exe(
    &self,
    tool_name: &str,
    exe_name: &str,
) -> Result<Vec<ToolLocation>>
pub fn find_tool_version(&self, tool_name: &str, version: &str) -> Option<ToolLocation>
pub fn find_tool_version_with_executable(
    &self,
    tool_name: &str,
    version: &str,
    exe_name: &str,
) -> Option<ToolLocation>>
```

## Runtime Installation

When installing a runtime, files are stored in platform-specific directory:

```rust
// In Runtime::install():
let base_install_path = ctx.paths.version_store_dir(store_name, version);
let install_path = base_install_path.join(platform.as_str());
// Files extracted to: ~/.vx/store/<name>/<version>/<platform>/
```

## Offline Bundle Creation

Create offline bundles that work across all platforms:

```bash
# Bundle structure
vx-bundle/
└── store/
    └── node/
        └── 20.0.0/
            ├── windows-x64/    # All platforms in one bundle
            ├── darwin-x64/
            └── linux-x64/
```

When extracting the bundle, vx automatically selects the correct platform
directory for the current system.

## Migration Guide

### For Runtime Implementors

If you implement custom runtimes, ensure installation uses platform directories:

```rust
async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
    let store_name = self.store_name();
    let platform = Platform::current();

    // Use platform-specific directory
    let base_install_path = ctx.paths.version_store_dir(store_name, version);
    let install_path = base_install_path.join(platform.as_str());

    // Download and extract to install_path
    ctx.installer.download_and_extract(&url, &install_path).await?;

    Ok(InstallResult::success(install_path, exe_path, version.to_string()))
}
```

### For Path Usage

When looking up installed tools, use platform-specific directories:

```rust
let resolver = PathResolver::new()?;

// These automatically use platform-specific directories
let tool = resolver.find_tool("node")?;
let latest = resolver.find_latest_tool("node")?;
let versions = resolver.list_store_versions("node")?;

// Find specific version (uses platform directory)
let node_20 = resolver.find_tool_version("node", "20.0.0")?;
```

## Benefits

1. **Cross-Platform Compatibility**: Same bundle works on Windows, macOS, Linux
2. **Disk Efficiency**: Multiple platforms can share same bundle
3. **Transparent API**: Users don't need to know platform details
4. **Easy Distribution**: Single bundle supports all platforms
5. **Future-Proof**: Easy to add new platform support

## Implementation Details

### Platform Detection

`vx_paths` includes a lightweight platform detector:

```rust
pub struct CurrentPlatform {
    pub os: &'static str,   // "windows", "darwin", "linux", etc.
    pub arch: &'static str, // "x64", "arm64", etc.
}

impl CurrentPlatform {
    pub fn current() -> Self { /* compile-time detection */ }
    pub fn as_str(&self) -> String { /* "windows-x64", etc. */ }
}
```

### Path Resolution

1. **Lookup**: Request `<provider>/<version>/`
2. **Redirect**: Check `<provider>/<version>/<platform>/`
3. **Fallback**: Return None if not found

This happens automatically in `PathResolver` methods.

## Testing

Test platform-specific paths:

```rust
#[test]
fn test_platform_redirection() {
    let manager = PathManager::new()?;
    let platform_dir = manager.platform_store_dir("node", "20.0.0");

    assert!(platform_dir.ends_with("node/20.0.0/windows-x64"));

    let versions = manager.list_store_versions("node")?;
    // Only versions with current platform subdirectory are listed
}
```
