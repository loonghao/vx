# RFC 0028: Proxy-Managed and Bundled Runtimes

- **Status**: Implemented (Yarn 2.x+ support)
- **Created**: 2026-02-03
- **Updated**: 2026-02-03
- **Implemented**: 2026-02-03
- **Related**: 
  - Yarn 2.x+ (Berry) corepack integration
  - .NET SDK tool chain (dotnet, msbuild, nuget)
  - Bundled tools pattern (npm with node, cargo with rust)

## Implementation Status

### Yarn 2.x+ (Berry) via Corepack - ✅ Implemented

The following features are now working:

```bash
# All Yarn versions are now supported
$ vx yarn@1.22.22 --version  # Classic - direct download
1.22.22

$ vx yarn@2.4.3 --version    # Berry 2.x - via corepack
2.4.3

$ vx yarn@3.6.0 --version    # Berry 3.x - via corepack
3.6.0

$ vx yarn@4.0.0 --version    # Berry 4.x - via corepack
4.0.0

$ vx yarn@4.12.0 --version   # Latest Berry - via corepack
4.12.0
```

### Implementation Details

1. **`ExecutionPrep` struct** added to `vx-runtime` for proxy execution configuration
2. **`is_version_installable()`** method added to `Runtime` trait
3. **`prepare_execution()`** method added to `Runtime` trait
4. **Yarn provider** updated to:
   - Fetch versions from both `yarn` (1.x/2.x) and `@yarnpkg/cli-dist` (3.x/4.x) npm packages
   - Return `false` from `is_version_installable()` for Yarn 2.x+
   - Enable corepack and prepare specific version in `prepare_execution()`
5. **Executor** modified to:
   - Skip direct installation for proxy-managed versions
   - Call `prepare_execution()` before command execution
   - Handle proxy execution preparation in the execution flow

### Not Yet Implemented

- .NET SDK tool chain support (planned for future RFC)

## Summary

Introduce support for "proxy-managed runtimes" and enhanced "bundled runtimes" — tools that are not directly installed by vx but managed through:
1. **Proxy mechanism** (e.g., Node.js corepack managing Yarn 2.x+)
2. **Bundled with another runtime** (e.g., msbuild bundled with dotnet SDK)
3. **Runtime-specific package managers** (e.g., `dotnet tool` managed tools)

This RFC provides a unified architecture for handling these complex scenarios, enabling vx to seamlessly work with:
- Yarn 2.x+ (Berry) via corepack
- .NET tool chain (dotnet, msbuild, nuget, dotnet tools)
- Future similar ecosystems

## Motivation

### Current Problem

Yarn 2.x+ (Berry) cannot be directly downloaded and installed like Yarn 1.x (Classic):

```bash
# Yarn 1.x - works fine
$ vx yarn@1.22.19 --version
1.22.19

# Yarn 2.x+ - fails
$ vx yarn@4.0.0 --version
✗ Failed to resolve version: No version found for yarn matching '4.0.0'
```

The current implementation:
1. `fetch_versions()` only returns Yarn 1.x versions (filtering out 2.x+)
2. `download_url()` returns `None` for 2.x+ versions
3. Users cannot use Yarn 2.x+ through vx

### Yarn 2.x+ Installation Method

Yarn 2.x+ is designed to be managed via **corepack** (bundled with Node.js 16.10+):

```bash
# Standard Yarn 2.x+ installation
$ corepack enable           # One-time setup
$ yarn --version            # Corepack manages the version
```

Or per-project:
```bash
# In package.json
{
  "packageManager": "yarn@4.0.0"
}

$ corepack enable
$ yarn --version           # Automatically uses 4.0.0
```

### Goals

1. Support `vx yarn@4.0.0 --version` seamlessly
2. Auto-enable corepack when needed
3. Provide a general mechanism for "proxy-managed" tools
4. Maintain backward compatibility

## Design

### Key Concepts

| Concept | Description | Example |
|---------|-------------|---------|
| **Directly Installable** | vx downloads and installs the tool | Yarn 1.x, Node.js, Go |
| **Proxy-Managed** | Tool is managed by another tool (proxy) | Yarn 2.x+ (corepack), rustup-managed cargo |
| **Proxy** | The tool that manages other tools | corepack (part of Node.js), rustup |

### Architecture Changes

#### 1. New Runtime Trait Methods

Add two methods to the `Runtime` trait:

```rust
/// Whether this version needs to be downloaded and installed by vx
/// 
/// Returns `true` (default): vx will download and install
/// Returns `false`: vx will use proxy mechanism instead
fn is_version_installable(&self, version: &str) -> bool {
    true
}

/// Prepare execution for proxy-managed versions
/// 
/// Called before executing a version that returns `false` from
/// `is_version_installable()`. This is where proxy setup happens.
async fn prepare_execution(
    &self,
    version: &str,
    ctx: &ExecutionContext,
) -> Result<ExecutionPrep> {
    Ok(ExecutionPrep::default())
}

/// Execution preparation result
pub struct ExecutionPrep {
    /// Use system PATH instead of vx-managed path
    pub use_system_path: bool,
    
    /// Override the executable path directly
    /// Used when the executable is discovered dynamically (e.g., bundled tools)
    pub executable_override: Option<PathBuf>,
    
    /// Additional environment variables
    pub env_vars: HashMap<String, String>,
    
    /// Command prefix to add before user arguments
    /// e.g., ["dotnet", "msbuild"] for running msbuild via dotnet
    pub command_prefix: Vec<String>,
    
    /// Whether the proxy/bundled tool is ready
    pub proxy_ready: bool,
    
    /// Additional PATH entries to prepend
    pub path_prepend: Vec<PathBuf>,
}

impl Default for ExecutionPrep {
    fn default() -> Self {
        Self {
            use_system_path: false,
            executable_override: None,
            env_vars: HashMap::new(),
            command_prefix: Vec::new(),
            proxy_ready: false,
            path_prepend: Vec::new(),
        }
    }
}
```

#### 2. Modified Execution Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        vx yarn@4.0.0 --version                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  1. Resolve version                                                         │
│     └── fetch_versions() returns ALL versions (1.x, 2.x, 3.x, 4.x)         │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  2. Check installable                                                       │
│     └── runtime.is_version_installable("4.0.0") → false                    │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  3. Ensure proxy (Node.js) installed                                        │
│     └── Check Node.js >= 16.10.0 is installed                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  4. Prepare execution                                                       │
│     └── runtime.prepare_execution("4.0.0", ctx)                            │
│         └── Enable corepack if not already enabled                         │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  5. Execute via proxy                                                       │
│     └── Run: yarn --version (corepack manages the actual version)          │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### 3. YarnRuntime Implementation

```rust
#[async_trait]
impl Runtime for YarnRuntime {
    // ... existing methods ...
    
    /// Yarn 1.x: directly installable
    /// Yarn 2.x+: managed by corepack
    fn is_version_installable(&self, version: &str) -> bool {
        version.starts_with('1')
    }
    
    /// Return ALL versions (not just 1.x)
    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        VersionFetcherBuilder::npm("yarn")
            .skip_prereleases()
            .limit(100)
            .build()
            .fetch(ctx).await
    }
    
    /// Prepare execution for Yarn 2.x+
    async fn prepare_execution(
        &self,
        version: &str,
        ctx: &ExecutionContext,
    ) -> Result<ExecutionPrep> {
        if self.is_version_installable(version) {
            return Ok(ExecutionPrep::default());
        }
        
        // Check/enable corepack
        if !Self::is_corepack_enabled().await {
            let node_exe = ctx.resolve_executable("node")
                .ok_or_else(|| anyhow!("Node.js is required for Yarn {}"))?
                .await?;
            
            Self::enable_corepack(&node_exe).await?;
        }
        
        // Set packageManager field if needed for version pinning
        // This ensures corepack uses the exact requested version
        if let Some(project_root) = &ctx.working_dir {
            Self::ensure_package_manager_field(project_root, version).await?;
        }
        
        Ok(ExecutionPrep {
            use_system_path: true,  // Use corepack's yarn from PATH
            proxy_ready: true,
            ..Default::default()
        })
    }
}

impl YarnRuntime {
    /// Check if corepack is enabled
    pub async fn is_corepack_enabled() -> bool {
        // Check if 'yarn' command works via corepack
        Command::new("yarn")
            .arg("--version")
            .output().await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    
    /// Enable corepack using the provided Node.js executable
    pub async fn enable_corepack(node_exe: &Path) -> Result<()> {
        info!("Enabling corepack for Yarn 2.x+ support...");
        
        let output = Command::new(node_exe)
            .args(["--eval", "require('child_process').execSync('corepack enable', {stdio: 'inherit'})"])
            .output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to enable corepack: {}", stderr));
        }
        
        info!("Corepack enabled successfully");
        Ok(())
    }
    
    /// Ensure package.json has the correct packageManager field
    async fn ensure_package_manager_field(
        project_root: &Path,
        version: &str,
    ) -> Result<()> {
        let package_json = project_root.join("package.json");
        
        if !package_json.exists() {
            // Create minimal package.json
            let content = json!({
                "name": "vx-project",
                "packageManager": format!("yarn@{}", version)
            });
            tokio::fs::write(&package_json, serde_json::to_string_pretty(&content)?).await?;
            return Ok(());
        }
        
        // Read and update existing package.json
        let content = tokio::fs::read_to_string(&package_json).await?;
        let mut package: serde_json::Value = serde_json::from_str(&content)?;
        
        let pm_field = format!("yarn@{}", version);
        if package.get("packageManager").and_then(|v| v.as_str()) != Some(&pm_field) {
            package["packageManager"] = json!(pm_field);
            tokio::fs::write(&package_json, serde_json::to_string_pretty(&package)?).await?;
        }
        
        Ok(())
    }
}
```

### Comparison: Before vs After

| Scenario | Before | After |
|----------|--------|-------|
| `vx yarn@1.22.19 --version` | ✅ Works (direct download) | ✅ Works (unchanged) |
| `vx yarn@4.0.0 --version` | ❌ "No version found" | ✅ Works (corepack) |
| `vx yarn --version` (no version) | ✅ Uses latest 1.x | ✅ Uses latest from npm (via corepack) |
| `vx install yarn@4.0.0` | ❌ Fails | ⚠️ Warns: "Yarn 2.x+ uses corepack, run 'vx yarn@4.0.0' instead" |

### Business Scenarios

#### Scenario 1: Proxy-Managed (Yarn 2.x+ via Corepack)

```
User Request: vx yarn@4.0.0 --version

Decision Flow:
  is_version_installable("4.0.0")? 
    → false (Yarn 2.x+ uses corepack)
  
  Action:
    1. Ensure Node.js >= 16.10.0 installed
    2. Run prepare_execution() → enable corepack
    3. Execute via system PATH (corepack-managed yarn)
```

#### Scenario 2: Bundled Tools (.NET SDK with msbuild)

.NET SDK bundles msbuild, nuget, and other tools:

```
User Request: vx msbuild MyProject.csproj

Decision Flow:
  is_msbuild_directly_installable?
    → false (msbuild is bundled with dotnet SDK)
  
  Action:
    1. Check if dotnet SDK is installed
    2. Run prepare_execution():
       - Find dotnet installation
       - Set environment for bundled tools
    3. Execute: dotnet msbuild MyProject.csproj
       OR use bundled msbuild directly if available
```

**Key Difference**: Bundled tools are physically present in the runtime's installation, while proxy-managed tools are dynamically resolved by the proxy.

#### Scenario 3: Runtime Package Manager (.NET Global Tools)

.NET has its own package manager for tools:

```bash
# Install .NET tool globally
$ dotnet tool install -g dotnetsay

# The tool becomes available in PATH
$ dotnetsay "Hello World"
```

vx integration:

```
User Request: vx dotnetsay "Hello World"

Decision Flow:
  is_dotnetsay_directly_installable?
    → false (it's a dotnet global tool)
  
  Action:
    1. Check if dotnet SDK is installed
    2. Run prepare_execution():
       - Check if dotnetsay is installed via `dotnet tool list -g`
       - If not: auto-install via `dotnet tool install -g dotnetsay`
       - Add dotnet tool path to PATH
    3. Execute: dotnetsay "Hello World"
```

#### Scenario 4: Multi-Level Tool Chain (Visual Studio + .NET)

Visual Studio includes multiple tools with complex relationships:

```
Visual Studio
├── MSBuild (bundled)
├── devenv.exe (IDE)
└── dotnet SDK (optional component)
    ├── dotnet CLI
    ├── msbuild (alternative to VS version)
    ├── nuget
    └── dotnet tools
```

vx should handle:

```bash
# If VS is installed, use VS-bundled msbuild
$ vx msbuild MyProject.csproj

# If only dotnet SDK is installed
$ vx msbuild MyProject.csproj  # → uses dotnet msbuild

# Explicit version selection
$ vx dotnet@8.0 msbuild MyProject.csproj
```

### General Applicability

| Tool | Type | Proxy/Bundle | is_version_installable() |
|------|------|--------------|--------------------------|
| Yarn 2.x+ | Proxy-Managed | corepack (Node.js) | `!version.starts_with('1')` |
| Yarn 1.x | Direct | - | `true` |
| pnpm (corepack) | Proxy-Managed | corepack (Node.js) | `false` (if corepack mode) |
| pnpm (direct) | Direct | - | `true` |
| msbuild | Bundled | dotnet SDK / VS | `false` |
| nuget | Bundled | dotnet SDK | `false` |
| dotnet tool | Runtime PM | dotnet SDK | `false` |
| cargo | Proxy-Managed | rustup | `false` (always rustup-managed) |
| npm | Bundled | Node.js | `false` (bundled with node) |
| npx | Bundled | Node.js | `false` (bundled with node) |

## Implementation Plan

### Phase 1: Core Architecture

- [ ] Add `is_version_installable()` method to `Runtime` trait with default `true`
- [ ] Add `prepare_execution()` method to `Runtime` trait with default no-op
- [ ] Add `ExecutionPrep` struct with `use_system_path`, `env_vars`, `command_prefix`, `proxy_ready`

### Phase 2: Executor Integration

- [ ] Modify `ensure_version_installed()` to check `is_version_installable()`
- [ ] If not installable: ensure proxy dependency (Node.js for corepack)
- [ ] Modify `execute_with_version()` to call `prepare_execution()`
- [ ] Handle `use_system_path` flag in execution path

### Phase 3: Yarn Provider Update

- [ ] Implement `is_version_installable()` for YarnRuntime
- [ ] Implement `prepare_execution()` for YarnRuntime
- [ ] Update `fetch_versions()` to return all versions
- [ ] Add corepack helper methods (`is_corepack_enabled()`, `enable_corepack()`)
- [ ] Add `ensure_package_manager_field()` for version pinning

### Phase 4: .NET Provider Update

- [ ] Implement `MsbuildRuntime` as bundled tool of dotnet
- [ ] Implement `NugetRuntime` as bundled tool of dotnet
- [ ] Add `DotnetToolRuntime` for runtime-managed tools
- [ ] Create `BundledToolExecutionPrep` helper for common bundled tool patterns
- [ ] Add version detection for bundled tools (msbuild version vs dotnet SDK version)

### Phase 5: Testing

- [ ] Unit tests for `is_version_installable()`
- [ ] Integration test for Yarn 1.x (unchanged behavior)
- [ ] Integration test for Yarn 2.x+ with corepack
- [ ] Test corepack auto-enable flow
- [ ] Integration test for dotnet bundled tools (msbuild, nuget)
- [ ] Integration test for dotnet global tools

### Phase 6: Documentation

- [ ] Update Node.js/Yarn documentation
- [ ] Add examples for Yarn 1.x vs 2.x+ usage
- [ ] Document `is_version_installable()` for provider authors
- [ ] Add .NET tool chain documentation
- [ ] Create "Bundled Tools" best practices guide

## Alternative Approaches Considered

### Option A: Separate Runtimes for Yarn 1.x and 2.x+

Create `yarn-classic` and `yarn-berry` as separate runtimes:

```rust
// yarn-classic: directly installable
vx yarn-classic@1.22.19 --version

// yarn-berry: corepack-managed
vx yarn-berry@4.0.0 --version
```

**Rejected**: Breaks user expectations. Users expect `vx yarn@4.0.0` to work.

### Option B: Post-Install Hook on Node.js

Auto-enable corepack when installing Node.js:

```rust
impl Runtime for NodeRuntime {
    async fn post_install(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        // Parse version, if >= 16.10.0, enable corepack
        if Self::version_meets_requirement(version, ">=16.10.0") {
            Self::enable_corepack().await?;
        }
        Ok(())
    }
}
```

**Rejected**: Corepack enablement is a system-wide operation that affects PATH and should be explicit or on-demand, not automatic on every Node.js install.

### Option C: Manifest-Driven Proxy Management

Add proxy information to `provider.toml`:

```toml
[[runtimes]]
name = "yarn"
# ...

[runtimes.proxy_management]
enabled = true
proxy_runtime = "node"
min_proxy_version = "16.10.0"
is_installable = "version.startsWith('1')"
enable_command = "corepack enable"
```

**Rejected**: Too complex for the initial implementation. Code-based approach is more flexible and easier to test. Can be migrated to manifest later.

## Design Considerations

### Version Matching Complexity

Different tools have different version relationships:

| Scenario | Version Source | Example |
|----------|---------------|---------|
| Direct | Tool's own version | Yarn 1.x: `1.22.19` |
| Proxy-managed | Proxy resolves version | Yarn 2.x+: corepack uses `packageManager` field |
| Bundled | Parent runtime version | MSBuild version = .NET SDK version |
| Runtime PM | Package manager resolves | `dotnet-ef` version from NuGet |

**Solution**: `fetch_versions()` should return versions meaningful to the user, while `prepare_execution()` handles the mapping to actual execution.

### Cross-Platform Complexity

| Tool | Windows | macOS | Linux |
|------|---------|-------|-------|
| Yarn (corepack) | ✅ | ✅ | ✅ |
| MSBuild (VS) | ✅ | ❌ | ❌ |
| MSBuild (dotnet) | ✅ | ✅ | ✅ |
| devenv | ✅ | ❌ | ❌ |

**Solution**: Bundled tools inherit platform constraints from their parent runtime. Provider manifests should declare platform support.

### Version Pinning Strategies

Different tools use different version pinning:

```bash
# Node.js: packageManager field in package.json
{ "packageManager": "yarn@4.0.0" }

# .NET: global.json for SDK version
{ "sdk": { "version": "8.0.100" } }

# Rust: rust-toolchain.toml
[toolchain]
channel = "1.75.0"
```

**Solution**: `prepare_execution()` should detect and respect existing version pinning files, only modifying them when explicitly requested.

### Execution Context Propagation

```rust
/// Enhanced ExecutionContext with proxy/bundled tool support
pub struct ExecutionContext {
    // ... existing fields ...
    
    /// Parent runtime chain for bundled tools
    /// e.g., ["dotnet", "msbuild"] means msbuild bundled with dotnet
    pub parent_chain: Vec<String>,
    
    /// Proxy runtime if using proxy management
    /// e.g., Some("corepack") for Yarn 2.x+
    pub proxy_runtime: Option<String>,
    
    /// Whether this is a system-installed tool
    pub is_system_tool: bool,
}
```

## Backward Compatibility

### Fully Compatible

- All existing `Runtime` implementations continue to work
- Default `is_version_installable()` returns `true` (existing behavior)
- Default `prepare_execution()` is a no-op (existing behavior)

### Yarn-Specific Changes

- `fetch_versions()` will return more versions (2.x+)
- `download_url()` behavior unchanged (returns `None` for 2.x+)
- New behavior only activates when explicitly requesting 2.x+

## Open Questions

### Corepack/Yarn Questions

1. **Version pinning**: Should we always write `packageManager` field to package.json, or respect user's existing configuration?

2. **Global vs Project scope**: Corepack can be enabled globally (system PATH) or per-project. Which should vx use?

3. **Other corepack tools**: Should pnpm also support corepack-managed mode as an option?

4. **Error handling**: If corepack enablement fails, should we provide fallback instructions?

### .NET Questions

5. **MSBuild version mapping**: Should `vx msbuild@17.0` mean:
   - MSBuild version 17.0?
   - .NET SDK version that bundles MSBuild 17.0?
   - Or both with smart mapping?

6. **Visual Studio detection**: Should vx detect and use VS-bundled tools when available, or always prefer dotnet SDK?

7. **dotnet tool versioning**: How should version constraints work for dotnet global tools?
   - `vx dotnetsay@1.0.0` (exact version)
   - `vx dotnetsay@latest` (always latest)
   - Support both?

8. **Tool manifest support**: Should vx respect `.config/dotnet-tools.json` for local tools?

### General Questions

9. **Performance**: `prepare_execution()` may involve spawning subprocesses. How do we minimize overhead?

10. **Caching**: Should we cache `prepare_execution()` results to avoid repeated setup?

11. **Discoverability**: How do users discover that `vx msbuild` works without explicit installation?

## References

- [Yarn Berry Documentation](https://yarnpkg.com/getting-started/install)
- [Node.js Corepack Documentation](https://nodejs.org/api/corepack.html)
- [packageManager field in package.json](https://nodejs.org/api/packages.html#packagemanager)

## Appendix A: .NET Implementation Example

### A1. Bundled Tool Pattern (MSBuild)

```rust
/// MSBuild bundled with .NET SDK
#[derive(Debug, Clone)]
pub struct MsbuildRuntime {
    /// Reference to parent runtime (dotnet)
    parent_runtime: String, // "dotnet"
}

#[async_trait]
impl Runtime for MsbuildRuntime {
    fn name(&self) -> &str {
        "msbuild"
    }
    
    fn description(&self) -> &str {
        "Microsoft Build Engine (bundled with .NET SDK)"
    }
    
    /// MSBuild is bundled with dotnet - not directly installable
    fn is_version_installable(&self, _version: &str) -> bool {
        false
    }
    
    /// Fetch versions from dotnet SDK releases
    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // MSBuild versions track dotnet SDK versions
        // Use dotnet runtime to fetch available SDK versions
        let dotnet = ctx.registry.get_runtime("dotnet").ok_or(...)?;
        dotnet.fetch_versions(ctx).await
    }
    
    /// Prepare execution by finding bundled msbuild
    async fn prepare_execution(
        &self,
        version: &str,
        ctx: &ExecutionContext,
    ) -> Result<ExecutionPrep> {
        // Find dotnet SDK installation
        let dotnet_exe = ctx.resolve_executable("dotnet")
            .ok_or_else(|| anyhow!(".NET SDK is required for msbuild"))?
            .await?;
        
        // Check if specific SDK version is installed
        let output = Command::new(&dotnet_exe)
            .args(["--list-sdks"])
            .output().await?;
        
        let sdk_output = String::from_utf8_lossy(&output.stdout);
        let sdk_available = sdk_output.lines()
            .any(|line| line.starts_with(version));
        
        if !sdk_available {
            return Err(anyhow!(
                ".NET SDK {} is not installed. Run 'vx install dotnet@{}'",
                version, version
            ));
        }
        
        // Find msbuild location within dotnet SDK
        let sdk_path = self.find_sdk_path(&dotnet_exe, version).await?;
        let msbuild_path = sdk_path.join("MSBuild.dll");
        
        Ok(ExecutionPrep {
            use_system_path: false,
            executable_override: Some(msbuild_path),
            command_prefix: vec![sdk_path.join("dotnet").display().to_string()],
            proxy_ready: true,
            ..Default::default()
        })
    }
}
```

### A2. Runtime Package Manager Pattern (.NET Global Tools)

```rust
/// .NET Global Tool runtime
#[derive(Debug, Clone)]
pub struct DotnetToolRuntime {
    tool_name: String,
}

#[async_trait]
impl Runtime for DotnetToolRuntime {
    fn name(&self) -> &str {
        &self.tool_name
    }
    
    fn is_version_installable(&self, _version: &str) -> bool {
        // .NET tools are installed via `dotnet tool install`, not downloaded directly
        false
    }
    
    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Query NuGet for available versions of this tool
        // Use NuGet API or `dotnet tool search`
        let dotnet = ctx.registry.get_runtime("dotnet").ok_or(...)?;
        
        let output = Command::new("dotnet")
            .args(["tool", "search", &self.tool_name, "--prerelease"])
            .output().await?;
        
        // Parse search output for versions
        // ...
    }
    
    async fn prepare_execution(
        &self,
        version: &str,
        ctx: &ExecutionContext,
    ) -> Result<ExecutionPrep> {
        // Check if tool is installed
        let tool_list = Command::new("dotnet")
            .args(["tool", "list", "-g"])
            .output().await?;
        
        let tool_output = String::from_utf8_lossy(&tool_list.stdout);
        let is_installed = tool_output.lines()
            .any(|line| line.starts_with(&self.tool_name));
        
        if !is_installed {
            info!("Installing .NET tool: {}@{}", self.tool_name, version);
            
            let install_cmd = if version == "latest" {
                vec!["tool", "install", "-g", &self.tool_name]
            } else {
                vec!["tool", "install", "-g", &self.tool_name, "--version", version]
            };
            
            let status = Command::new("dotnet")
                .args(&install_cmd)
                .status().await?;
            
            if !status.success() {
                return Err(anyhow!("Failed to install {}@{}", self.tool_name, version));
            }
        }
        
        // Get dotnet tool path
        let tool_path = self.get_tool_path(&self.tool_name).await?;
        
        Ok(ExecutionPrep {
            use_system_path: true,  // Tool is in PATH after install
            proxy_ready: true,
            ..Default::default()
        })
    }
}
```

### A3. Provider Registration

```rust
/// .NET provider with bundled tools
pub struct DotnetProvider;

impl Provider for DotnetProvider {
    fn name(&self) -> &str {
        "dotnet"
    }
    
    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![
            // Main SDK
            Arc::new(DotnetRuntime::new()),
            
            // Bundled tools
            Arc::new(MsbuildRuntime { parent_runtime: "dotnet".to_string() }),
            Arc::new(NugetRuntime { parent_runtime: "dotnet".to_string() }),
            
            // Common global tools (lazy registration)
            Arc::new(DotnetToolRuntime { tool_name: "dotnet-ef".to_string() }),
        ]
    }
}
```

## Appendix B: Corepack Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Corepack Architecture                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────────────────────────┐│
│  │   User      │────▶│    yarn     │────▶│  corepack shim (from PATH)     ││
│  │   Command   │     │   (shell)   │     │                                 ││
│  └─────────────┘     └─────────────┘     └─────────────────────────────────┘│
│                                                     │                       │
│                                                     ▼                       │
│                                          ┌─────────────────┐               │
│                                          │  package.json   │               │
│                                          │  packageManager │               │
│                                          │  = "yarn@4.0.0" │               │
│                                          └─────────────────┘               │
│                                                     │                       │
│                                                     ▼                       │
│                                          ┌─────────────────┐               │
│                                          │  Download yarn  │               │
│                                          │  4.0.0 to cache │               │
│                                          │  (~/.cache/...) │               │
│                                          └─────────────────┘               │
│                                                     │                       │
│                                                     ▼                       │
│                                          ┌─────────────────┐               │
│                                          │  Execute yarn   │               │
│                                          │  4.0.0 binary   │               │
│                                          └─────────────────┘               │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```
