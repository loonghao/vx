# RFC 0025: Cross-Language Global Package Isolation

## Overview

This RFC proposes a comprehensive cross-language global package isolation system for vx, addressing the pollution problem when running commands like `vx npm install -g` outside of projects. The design draws inspiration from pnpm's Content-Addressable Store (CAS), Nix's immutable store, and mise's backend architecture.

## Motivation

### Current Problem

When users run global package installation commands through vx, packages are installed into the runtime's directory:

```
~/.vx/store/node/20.x.x/lib/node_modules/typescript  ← Pollution!
```

This causes several issues:

1. **Cross-project contamination**: Different projects sharing the same node version have conflicting global packages
2. **Version loss on upgrade**: Upgrading node version loses all global packages
3. **No project-level control**: Cannot use different versions of global tools for different projects
4. **Multi-language problem**: Same issue exists for pip, cargo, go install, gem install

### Affected Package Managers

| Language | Install Command | Current Pollution Location |
|----------|----------------|---------------------------|
| Node.js | `npm install -g` | `~/.vx/store/node/{ver}/lib/node_modules/` |
| Python | `pip install` | `~/.vx/store/python/{ver}/lib/python3.x/site-packages/` |
| Rust | `cargo install` | `~/.cargo/bin/` (system) |
| Go | `go install` | `$GOPATH/bin/` or `~/go/bin/` (system) |
| Ruby | `gem install` | `~/.vx/store/ruby/{ver}/lib/ruby/gems/` |

## Design Goals

1. **Complete isolation**: Global packages never pollute runtime installations
2. **Cross-language consistency**: Unified design pattern for all ecosystems
3. **Space efficiency**: Deduplicate identical packages using CAS + symlinks
4. **Project-level control**: Allow `vx.toml` to declare project-scoped "global" tools
5. **Cross-platform support**: Work correctly on Windows, macOS, and Linux
6. **Backward compatibility**: Existing workflows continue to function

## Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    vx Package Isolation Architecture                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  Content-Addressable Store (CAS)                                │   │
│  │  ~/.vx/packages/{ecosystem}/{package}/{version}/                │   │
│  │                                                                 │   │
│  │  ├── npm/                                                       │   │
│  │  │   ├── typescript/5.3.3/                                      │   │
│  │  │   │   ├── node_modules/typescript/                           │   │
│  │  │   │   └── bin/tsc                                            │   │
│  │  │   └── eslint/8.56.0/                                         │   │
│  │  │                                                              │   │
│  │  ├── pip/                                                       │   │
│  │  │   ├── black/24.1.0/                                          │   │
│  │  │   │   ├── venv/  (isolated virtual environment)              │   │
│  │  │   │   └── bin/black                                          │   │
│  │  │   └── nox/2024.1.0/                                          │   │
│  │  │                                                              │   │
│  │  ├── cargo/                                                     │   │
│  │  │   ├── ripgrep/14.0.0/                                        │   │
│  │  │   │   └── bin/rg                                             │   │
│  │  │   └── fd-find/9.0.0/                                         │   │
│  │  │                                                              │   │
│  │  └── go/                                                        │   │
│  │      └── golangci-lint/1.55.0/                                  │   │
│  │          └── bin/golangci-lint                                  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  Global Shims Directory                                         │   │
│  │  ~/.vx/shims/                                                   │   │
│  │                                                                 │   │
│  │  ├── tsc -> ../packages/npm/typescript/5.3.3/bin/tsc            │   │
│  │  ├── black -> ../packages/pip/black/24.1.0/bin/black            │   │
│  │  └── rg -> ../packages/cargo/ripgrep/14.0.0/bin/rg              │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  Project-Level Isolation (Symlinks)                             │   │
│  │                                                                 │   │
│  │  ~/project-a/                      ~/project-b/                 │   │
│  │  ├── vx.toml                       ├── vx.toml                  │   │
│  │  │   [tools.global]                │   [tools.global]           │   │
│  │  │   typescript = "5.3"            │   typescript = "5.4"       │   │
│  │  │                                 │                            │   │
│  │  └── .vx/bin/                      └── .vx/bin/                 │   │
│  │      └── tsc -> ~/.vx/packages/... │     └── tsc -> ...         │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Directory Structure

```
~/.vx/
├── store/                          # Runtime versions (existing)
│   ├── node/20.10.0/
│   ├── python/3.11.0/
│   └── rust/1.75.0/
│
├── packages/                       # 🆕 Global packages CAS
│   ├── npm/                        # npm ecosystem
│   │   ├── typescript/
│   │   │   ├── 5.3.3/
│   │   │   │   ├── package.json    # Metadata
│   │   │   │   ├── node_modules/   # Actual package
│   │   │   │   └── bin/            # Executables
│   │   │   └── 5.4.2/
│   │   └── eslint/
│   │       └── 8.56.0/
│   │
│   ├── pip/                        # pip ecosystem
│   │   ├── black/
│   │   │   └── 24.1.0/
│   │   │       ├── venv/           # Isolated venv
│   │   │       └── bin/
│   │   └── nox/
│   │       └── 2024.1.0/
│   │
│   ├── cargo/                      # cargo ecosystem
│   │   └── ripgrep/
│   │       └── 14.0.0/
│   │           └── bin/rg
│   │
│   ├── go/                         # go ecosystem
│   │   └── golangci-lint/
│   │       └── 1.55.0/
│   │           └── bin/golangci-lint
│   │
│   └── gem/                        # gem ecosystem
│       └── bundler/
│           └── 2.5.0/
│
├── shims/                          # 🆕 Global shims (symlinks)
│   ├── tsc -> ../packages/npm/typescript/5.3.3/bin/tsc
│   ├── black -> ../packages/pip/black/24.1.0/bin/black
│   └── rg -> ../packages/cargo/ripgrep/14.0.0/bin/rg
│
└── config/                         # Global configuration
    └── global-tools.toml           # 🆕 Global tool versions
```

## Environment Variable Redirection

### Per-Ecosystem Configuration

| Ecosystem | Environment Variable | Purpose | Redirect Target |
|-----------|---------------------|---------|-----------------|
| **npm** | `NPM_CONFIG_PREFIX` | Global install prefix | `~/.vx/packages/npm/{pkg}/{ver}` |
| **pip** | `PIP_TARGET` | Package install directory | `~/.vx/packages/pip/{pkg}/{ver}/venv` |
| **pip** | `VIRTUAL_ENV` | Virtual environment path | `~/.vx/packages/pip/{pkg}/{ver}/venv` |
| **cargo** | `CARGO_INSTALL_ROOT` | Binary install root | `~/.vx/packages/cargo/{pkg}/{ver}` |
| **go** | `GOBIN` | Binary install directory | `~/.vx/packages/go/{pkg}/{ver}/bin` |
| **gem** | `GEM_HOME` | Gem install directory | `~/.vx/packages/gem/{pkg}/{ver}` |
| **gem** | `GEM_PATH` | Gem lookup path | `~/.vx/packages/gem/{pkg}/{ver}` |

### Implementation Strategy

```rust
/// Environment variable configuration for package managers
pub struct PackageManagerEnv {
    ecosystem: Ecosystem,
    package: String,
    version: String,
}

impl PackageManagerEnv {
    /// Generate environment variables for isolated installation
    pub fn install_env(&self, paths: &VxPaths) -> HashMap<String, String> {
        let pkg_dir = paths.package_dir(&self.ecosystem, &self.package, &self.version);
        let mut env = HashMap::new();

        match self.ecosystem {
            Ecosystem::Node => {
                // npm global install redirection
                env.insert("NPM_CONFIG_PREFIX".into(), pkg_dir.to_string_lossy().into());
                env.insert("NPM_CONFIG_GLOBAL".into(), "true".into());
            }
            Ecosystem::Python => {
                // pip install to isolated venv
                let venv_dir = pkg_dir.join("venv");
                env.insert("VIRTUAL_ENV".into(), venv_dir.to_string_lossy().into());
                env.insert("PIP_TARGET".into(), venv_dir.join("lib").to_string_lossy().into());
            }
            Ecosystem::Rust => {
                // cargo install redirection
                env.insert("CARGO_INSTALL_ROOT".into(), pkg_dir.to_string_lossy().into());
            }
            Ecosystem::Go => {
                // go install redirection
                let bin_dir = pkg_dir.join("bin");
                env.insert("GOBIN".into(), bin_dir.to_string_lossy().into());
            }
            Ecosystem::Ruby => {
                // gem install redirection
                env.insert("GEM_HOME".into(), pkg_dir.to_string_lossy().into());
                env.insert("GEM_PATH".into(), pkg_dir.to_string_lossy().into());
            }
            _ => {}
        }

        env
    }
}
```

## Platform-Specific Considerations

### Windows

#### Symlink Permissions

Windows requires special permissions to create symbolic links:

| Method | Requirement | Use Case |
|--------|-------------|----------|
| **Developer Mode** | Windows 10+ with Developer Mode enabled | Recommended for developers |
| **Administrator** | Run as Administrator | Not recommended for daily use |
| **Junction Points** | No special permissions required | Directories only, fallback option |

**Implementation Strategy**:

```rust
/// Create a symlink with Windows fallback
pub fn create_symlink(target: &Path, link: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        // Try symbolic link first (requires Developer Mode or Admin)
        if let Err(_) = std::os::windows::fs::symlink_file(target, link) {
            if target.is_dir() {
                // Fall back to junction for directories
                junction::create(target, link)?;
            } else {
                // Fall back to hard link for files
                std::fs::hard_link(target, link)?;
            }
        }
        Ok(())
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link)?;
        Ok(())
    }
}
```

#### Path Length Limits

Windows has a 260 character path limit by default. Solutions:

1. **Enable Long Paths**: Registry key `LongPathsEnabled` (Windows 10 1607+)
2. **Short Base Path**: Use short paths like `C:\vx\` instead of `C:\Users\username\.vx\`
3. **Extended Path Prefix**: Use `\\?\` prefix for paths > 260 chars

```rust
/// Normalize path for Windows long path support
pub fn normalize_path(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        if path.to_string_lossy().len() > 200 {
            // Use extended path prefix for long paths
            let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
            PathBuf::from(format!("\\\\?\\{}", abs.display()))
        } else {
            path.to_path_buf()
        }
    }

    #[cfg(not(windows))]
    {
        path.to_path_buf()
    }
}
```

#### Case Insensitivity

Windows filesystem is case-insensitive. Package lookups must normalize case:

```rust
/// Normalize package name for filesystem lookup
pub fn normalize_package_name(name: &str) -> String {
    #[cfg(windows)]
    {
        name.to_lowercase()
    }

    #[cfg(not(windows))]
    {
        name.to_string()
    }
}
```

### macOS

#### Case Sensitivity

APFS (default since macOS 10.13) is case-insensitive by default. Same normalization as Windows applies.

#### SIP (System Integrity Protection)

SIP restricts access to system directories. vx already uses `~/.vx` which is unaffected.

#### Gatekeeper / Notarization

Downloaded binaries may be quarantined. Solution:

```rust
/// Remove quarantine attribute on macOS
#[cfg(target_os = "macos")]
pub fn remove_quarantine(path: &Path) -> Result<()> {
    use std::process::Command;
    Command::new("xattr")
        .args(["-d", "com.apple.quarantine"])
        .arg(path)
        .output()
        .ok(); // Ignore errors if attribute doesn't exist
    Ok(())
}
```

### Linux

#### Symlink Support

Full symlink support on all common filesystems (ext4, XFS, Btrfs, ZFS).

#### File Permissions

Ensure executables have proper permissions:

```rust
/// Set executable permissions on Unix
#[cfg(unix)]
pub fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(perms.mode() | 0o111); // Add execute bit
    std::fs::set_permissions(path, perms)?;
    Ok(())
}
```

#### Shared Systems

On shared systems (multi-user servers), consider per-user isolation:

```
~/.vx/                   # User-specific
/opt/vx/                 # System-wide (admin managed)
```

## vx.toml Syntax Extensions

### Project-Scoped Global Tools

```toml
# vx.toml

[tools]
node = "20"
python = "3.11"
rust = "1.75"

# 🆕 Project-scoped "global" tools
# These are installed globally but symlinked per-project
[tools.global]
typescript = "5.3"      # Auto-detected as npm:typescript
black = "24.1"          # Auto-detected as pip:black
ripgrep = "14"          # Auto-detected as cargo:ripgrep

# 🆕 Explicit backend specification
[tools.global.npm]
typescript = "5.3"
eslint = "8"
prettier = "3"

[tools.global.pip]
black = "24.1"
nox = "latest"
ruff = "0.1"

[tools.global.cargo]
ripgrep = "14"
fd-find = "9"
bat = "0.24"

[tools.global.go]
golangci-lint = "1.55"

[tools.global.gem]
bundler = "2.5"
```

### Global Configuration File

For tools used across all projects:

```toml
# ~/.vx/config/global-tools.toml

[npm]
typescript = "5.3"
prettier = "3"

[pip]
black = "24.1"
ruff = "0.1"

[cargo]
ripgrep = "14"
```

### Priority Resolution

When the same tool is declared in multiple places:

```
Project vx.toml [tools.global] > Global config > System PATH
```

## Data Structures

### VxPaths Extensions

```rust
impl VxPaths {
    /// Package CAS directory: ~/.vx/packages/{ecosystem}/{package}/{version}
    pub fn package_dir(&self, ecosystem: &Ecosystem, package: &str, version: &str) -> PathBuf {
        self.base_dir
            .join("packages")
            .join(ecosystem.to_string().to_lowercase())
            .join(normalize_package_name(package))
            .join(version)
    }

    /// Package binary directory
    pub fn package_bin_dir(&self, ecosystem: &Ecosystem, package: &str, version: &str) -> PathBuf {
        self.package_dir(ecosystem, package, version).join("bin")
    }

    /// Global shims directory: ~/.vx/shims
    pub fn shims_dir(&self) -> PathBuf {
        self.base_dir.join("shims")
    }

    /// Project-local bin directory: {project}/.vx/bin
    pub fn project_bin_dir(&self, project_root: &Path) -> PathBuf {
        project_root.join(".vx").join("bin")
    }
}
```

### GlobalPackage Structure

```rust
/// A globally installed package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalPackage {
    /// Package name
    pub name: String,
    /// Installed version
    pub version: String,
    /// Ecosystem (npm, pip, cargo, go, gem)
    pub ecosystem: Ecosystem,
    /// Installation timestamp
    pub installed_at: DateTime<Utc>,
    /// Executables provided by this package
    pub executables: Vec<String>,
    /// Runtime dependency (e.g., node@20 for npm packages)
    pub runtime_dependency: Option<RuntimeDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDependency {
    pub runtime: String,      // e.g., "node"
    pub version: String,      // e.g., "20"
}
```

### PackageRegistry

```rust
/// Registry of installed global packages
pub struct PackageRegistry {
    packages: HashMap<(Ecosystem, String), GlobalPackage>,
    path: PathBuf,
}

impl PackageRegistry {
    /// Load registry from disk
    pub fn load(path: &Path) -> Result<Self>;

    /// Save registry to disk
    pub fn save(&self) -> Result<()>;

    /// Register a new package
    pub fn register(&mut self, package: GlobalPackage) -> Result<()>;

    /// Get package by name and ecosystem
    pub fn get(&self, ecosystem: &Ecosystem, name: &str) -> Option<&GlobalPackage>;

    /// List all packages for an ecosystem
    pub fn list_by_ecosystem(&self, ecosystem: &Ecosystem) -> Vec<&GlobalPackage>;

    /// Update shims after package changes
    pub fn update_shims(&self, paths: &VxPaths) -> Result<()>;
}
```

## Tool Invocation: Explicit vs Implicit

vx supports two modes for invoking globally installed tools:

### Explicit Invocation (via `vx` prefix)

Always works, regardless of PATH configuration:

```bash
# Explicit invocation - always works
vx tsc --version
vx black --check .
vx rg "pattern" .

# With version specification
vx tsc@5.3 --version
vx black@24.1 --check .
```

**How it works**:

```
┌─────────────────────────────────────────────────────────────────────────┐
│  vx tsc --version                                                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  1. Executor receives "tsc" as runtime_name                             │
│       │                                                                 │
│       ▼                                                                 │
│  2. Check Provider Registry (static providers)                          │
│     ├── Found? → Use static provider (e.g., vite, release-please)      │
│     └── Not found? → Continue to step 3                                │
│       │                                                                 │
│       ▼                                                                 │
│  3. Check Package Registry (dynamic packages)                           │
│     ├── Found in ~/.vx/packages/npm/typescript/? → Use it              │
│     └── Not found? → Error: "Tool 'tsc' not installed"                 │
│       │                                                                 │
│       ▼                                                                 │
│  4. Resolve executable path                                             │
│     └── ~/.vx/packages/npm/typescript/5.3.3/bin/tsc                    │
│       │                                                                 │
│       ▼                                                                 │
│  5. Execute with proper environment                                     │
│     └── Ensure node is in PATH, run tsc                                │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Implicit Invocation (via shims in PATH)

Works when `~/.vx/shims` is in user's PATH:

```bash
# Implicit invocation - requires PATH setup
tsc --version
black --check .
rg "pattern" .
```

**Setup required**:

```bash
# Install shell integration
vx hook install

# Or manually add to shell config:
# bash/zsh:
export PATH="$HOME/.vx/shims:$PATH"

# PowerShell:
$env:PATH = "$env:USERPROFILE\.vx\shims;$env:PATH"

# fish:
set -gx PATH $HOME/.vx/shims $PATH
```

**How shims work**:

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Shim Structure                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ~/.vx/shims/                                                           │
│  ├── tsc           → Wrapper script or symlink                         │
│  ├── black         → Wrapper script or symlink                         │
│  └── rg            → Wrapper script or symlink                         │
│                                                                         │
│  Unix shim (wrapper script):                                            │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ #!/bin/sh                                                        │   │
│  │ exec "$HOME/.vx/packages/npm/typescript/5.3.3/bin/tsc" "$@"     │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  Windows shim (.cmd):                                                   │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ @echo off                                                        │   │
│  │ "%USERPROFILE%\.vx\packages\npm\typescript\5.3.3\bin\tsc" %*    │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Project-Level Invocation

Within a project with `vx.toml`, tools are available via `vx dev` or `vx shell`:

```toml
# vx.toml
[tools]
node = "20"

[tools.global]
typescript = "5.3"
eslint = "8"
```

```bash
# Enter project environment
vx dev

# Now tools are directly available (project .vx/bin is in PATH)
tsc --version
eslint --check .
```

**Project-level shims**:

```
project/
├── vx.toml
├── .vx/
│   └── bin/                    # Project-local shims
│       ├── tsc -> ~/.vx/packages/npm/typescript/5.3.3/bin/tsc
│       └── eslint -> ~/.vx/packages/npm/eslint/8.56.0/bin/eslint
└── src/
```

### PATH Priority Order

```
┌─────────────────────────────────────────────────────────────────────────┐
│  PATH Priority (highest to lowest)                                      │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  1. Project .vx/bin/          ← Project-specific global tools          │
│  2. vx runtime bin dirs       ← ~/.vx/store/node/20.x.x/bin/           │
│  3. vx global shims           ← ~/.vx/shims/                           │
│  4. User PATH prepend         ← User's custom prepend paths            │
│  5. System PATH               ← Original system PATH                   │
│  6. User PATH append          ← User's custom append paths             │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## Unified Architecture: Static Providers + Dynamic Packages

### Current Problem: Code Duplication

vx currently has two similar but separate systems:

1. **Static PackageRuntime** (e.g., `vite`, `release-please`): Compiled into vx
2. **Global Packages** (RFC-0025): Runtime-installed packages

Both use the same underlying logic (`install_npm_package()`, `install_pip_package()`).

### Solution: DynamicPackageRuntime

Introduce a unified architecture that reuses existing `PackageRuntime` trait:

```rust
/// Dynamic package runtime - no need to create a provider for each package
pub struct DynamicPackageRuntime {
    /// Package name (e.g., "typescript", "black")
    name: String,
    /// Ecosystem (Node, Python, Rust, Go)
    ecosystem: Ecosystem,
    /// Installation method
    install_method: InstallMethod,
    /// Required runtime (e.g., "node" for npm packages)
    required_runtime: String,
    /// Required runtime version constraint (optional)
    required_version: Option<String>,
    /// Executables provided by this package
    executables: Vec<String>,
}

impl DynamicPackageRuntime {
    /// Create from package specification
    ///
    /// Supports formats:
    /// - "npm:typescript@5.3"
    /// - "pip:black@24.1"
    /// - "cargo:ripgrep@14"
    /// - "typescript@5.3" (auto-detect ecosystem)
    pub fn from_spec(spec: &str) -> Result<Self> {
        let (ecosystem, package, version) = parse_package_spec(spec)?;

        Ok(Self {
            name: package.clone(),
            ecosystem,
            install_method: match ecosystem {
                Ecosystem::NodeJs => InstallMethod::npm(&package),
                Ecosystem::Python => InstallMethod::pip(&package),
                Ecosystem::Rust => InstallMethod::cargo(&package),
                Ecosystem::Go => InstallMethod::go(&package),
                _ => return Err(anyhow!("Unsupported ecosystem: {:?}", ecosystem)),
            },
            required_runtime: ecosystem.default_runtime().to_string(),
            required_version: None,
            executables: vec![package], // Default: package name = executable name
        })
    }

    /// Create from GlobalPackage registry entry
    pub fn from_global_package(pkg: &GlobalPackage) -> Self {
        Self {
            name: pkg.name.clone(),
            ecosystem: pkg.ecosystem,
            install_method: match pkg.ecosystem {
                Ecosystem::NodeJs => InstallMethod::npm(&pkg.name),
                Ecosystem::Python => InstallMethod::pip(&pkg.name),
                Ecosystem::Rust => InstallMethod::cargo(&pkg.name),
                Ecosystem::Go => InstallMethod::go(&pkg.name),
                _ => InstallMethod::Binary,
            },
            required_runtime: pkg.runtime_dependency
                .as_ref()
                .map(|d| d.runtime.clone())
                .unwrap_or_else(|| pkg.ecosystem.default_runtime().to_string()),
            required_version: pkg.runtime_dependency
                .as_ref()
                .map(|d| d.version.clone()),
            executables: pkg.executables.clone(),
        }
    }
}

// Implement Runtime trait - reuse existing infrastructure
#[async_trait]
impl Runtime for DynamicPackageRuntime {
    fn name(&self) -> &str {
        &self.name
    }

    fn ecosystem(&self) -> Ecosystem {
        self.ecosystem
    }

    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        // Delegate to PackageRuntime::install_package()
        self.install_package(version, ctx).await
    }

    // ... other trait methods
}

// Implement PackageRuntime trait - reuse install logic
#[async_trait]
impl PackageRuntime for DynamicPackageRuntime {
    fn install_method(&self) -> InstallMethod {
        self.install_method.clone()
    }

    fn required_runtime(&self) -> &str {
        &self.required_runtime
    }

    fn required_runtime_version(&self) -> Option<&str> {
        self.required_version.as_deref()
    }
}
```

### When to Use Static vs Dynamic

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Static Provider (compiled)              Dynamic Package (runtime)      │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Use when:                               Use when:                      │
│  ├── Special install logic               ├── Standard npm/pip install  │
│  │   (e.g., pnpm renames files)          │                             │
│  ├── Special execution hooks             ├── No special logic needed   │
│  │   (e.g., npm pre_run for deps)        │                             │
│  ├── Version constraints                 ├── User-installed packages   │
│  │   (e.g., vite needs node>=18)         │                             │
│  ├── Multiple executables                ├── vx.toml [tools.global]    │
│  │   (e.g., @angular/cli → ng)           │                             │
│  └── Core vx functionality               └── Any npm/pip/cargo package │
│                                                                         │
│  Examples:                               Examples:                      │
│  ├── vite (node>=18 requirement)         ├── typescript                │
│  ├── release-please                      ├── eslint                    │
│  ├── rez (pip package)                   ├── prettier                  │
│  ├── pre-commit                          ├── black                     │
│  └── pnpm (special install)              ├── ripgrep                   │
│                                          └── any user package          │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Executor Resolution Flow

```rust
impl Executor {
    pub async fn execute(&self, runtime_name: &str, args: &[String]) -> Result<i32> {
        // 1. Check static Provider Registry first
        if let Some(runtime) = self.registry.get_runtime(runtime_name) {
            return self.execute_with_runtime(runtime, args).await;
        }

        // 2. Check if it's an alias for a static provider
        if let Some(runtime) = self.registry.get_runtime_by_alias(runtime_name) {
            return self.execute_with_runtime(runtime, args).await;
        }

        // 3. Check Package Registry for dynamic packages
        if let Some(package) = self.package_registry.find_by_executable(runtime_name) {
            let dynamic_runtime = DynamicPackageRuntime::from_global_package(package);
            return self.execute_with_runtime(&dynamic_runtime, args).await;
        }

        // 4. Check vx.toml [tools.global] for project-declared packages
        if let Some(ref project_config) = self.project_config {
            if let Some(pkg_spec) = project_config.get_global_tool(runtime_name) {
                let dynamic_runtime = DynamicPackageRuntime::from_spec(pkg_spec)?;
                // Auto-install if not present
                self.ensure_installed(&dynamic_runtime).await?;
                return self.execute_with_runtime(&dynamic_runtime, args).await;
            }
        }

        // 5. Not found
        Err(anyhow!(
            "Tool '{}' not found. Install with: vx install-global npm:{}",
            runtime_name, runtime_name
        ))
    }
}
```

## CLI Commands

### New Commands

```bash
# Install a global package (isolated)
vx install-global npm:typescript@5.3
vx install-global pip:black@24.1
vx install-global cargo:ripgrep@14

# Shorthand (auto-detect ecosystem from registry)
vx install-global typescript@5.3

# List global packages
vx list-global
vx list-global --ecosystem npm

# Uninstall global package
vx uninstall-global npm:typescript

# Show package info
vx info-global typescript

# Update shims after manual changes
vx shim-update
```

### Modified Behavior (Implicit Interception)

```bash
# Current (polluting):
vx npm install -g typescript
# → Installs to ~/.vx/store/node/20.x.x/lib/node_modules/

# New (isolated with warning):
vx npm install -g typescript
# → Intercepts -g flag
# → Installs to ~/.vx/packages/npm/typescript/5.x.x/
# → Creates shim at ~/.vx/shims/tsc
# → Prints: "Tip: Use 'vx install-global npm:typescript' for explicit global install"
```

### Comparison: Explicit vs Implicit Installation

| Aspect | Explicit (`vx install-global`) | Implicit (`vx npm install -g`) |
|--------|-------------------------------|-------------------------------|
| **Clarity** | Clear intent | Requires interception |
| **Registry** | Always recorded | Recorded after interception |
| **Version** | Explicit in command | Parsed from npm output |
| **Ecosystem** | Explicit in command | Inferred from tool |
| **Recommended** | ✅ Yes | ⚠️ Supported for compatibility |

## Implementation Plan

### Phase 1: Environment Variable Redirection (v0.8.x)

Quick fix to prevent pollution immediately.

1. [ ] Implement `PackageManagerEnv` struct
2. [ ] Intercept `npm install -g` and redirect via `NPM_CONFIG_PREFIX`
3. [ ] Intercept `pip install` and use isolated venv
4. [ ] Intercept `cargo install` and redirect via `CARGO_INSTALL_ROOT`
5. [ ] Intercept `go install` and redirect via `GOBIN`
6. [ ] Add basic shim generation

**Estimated effort**: 2-3 days

### Phase 2: DynamicPackageRuntime + CAS (v0.9.x)

Implement unified architecture with proper CAS.

**Core Infrastructure**:
1. [ ] Implement `DynamicPackageRuntime` struct in `vx-runtime`
2. [ ] Add `InstallMethod::cargo()` and `InstallMethod::go()` variants
3. [ ] Extend `VxPaths` with package-related methods (`package_dir`, `shims_dir`)
4. [ ] Implement `GlobalPackage` and `PackageRegistry` structs
5. [ ] Implement cross-platform symlink creation (with Windows fallbacks)

**Executor Integration**:
6. [ ] Add `PackageRegistry` to `Executor` struct
7. [ ] Implement resolution flow: Static Provider → Package Registry → vx.toml
8. [ ] Add `find_by_executable()` method to locate packages by binary name

**CLI Commands**:
9. [ ] Add `vx install-global` command
10. [ ] Add `vx list-global` command
11. [ ] Add `vx uninstall-global` command
12. [ ] Add `vx info-global` command
13. [ ] Add `vx shim-update` command

**Shim Management**:
14. [ ] Implement shim generation for Unix (shell wrapper)
15. [ ] Implement shim generation for Windows (.cmd wrapper)
16. [ ] Implement shim cleanup on package uninstall

**Estimated effort**: 2-3 weeks

### Phase 3: vx.toml Integration (v1.0.x)

Full project-level control.

1. [ ] Parse `[tools.global]` section in vx.toml
2. [ ] Parse `[tools.global.npm]`, `[tools.global.pip]` subsections
3. [ ] Implement project-local `.vx/bin` symlink generation
4. [ ] Update `vx sync` to install global tools from vx.toml
5. [ ] Update `vx dev` to include project global tools in PATH
6. [ ] Implement lock file support for global tools (`vx.lock` extension)
7. [ ] Add implicit interception for `vx npm install -g` with warning
8. [ ] Add documentation and user guides

**Estimated effort**: 1-2 weeks

### Phase 4: Advanced Features (v1.1.x)

1. [ ] Package version constraints (semver ranges)
2. [ ] Automatic package updates (`vx upgrade-global`)
3. [ ] Package aliases (`vx alias tsc="typescript tsc"`)
4. [ ] Shared cache across users (optional)
5. [ ] Plugin system for additional ecosystems
6. [ ] `vx hook install` for automatic PATH setup

**Estimated effort**: Ongoing

## Migration Path

### From Current vx

1. Existing runtime installations in `~/.vx/store/` are unaffected
2. Global packages already installed in runtimes will continue to work
3. New global installs will go to `~/.vx/packages/`
4. Users can optionally migrate existing packages with `vx migrate-global`

### Migration Command

```bash
# Detect globally installed packages in runtime directories
vx migrate-global --detect

# Migrate specific package
vx migrate-global npm:typescript

# Migrate all detected packages
vx migrate-global --all
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // === VxPaths Tests ===

    #[test]
    fn test_package_dir_generation() {
        let paths = VxPaths::new().unwrap();
        let dir = paths.package_dir(&Ecosystem::NodeJs, "typescript", "5.3.3");
        assert!(dir.ends_with("packages/npm/typescript/5.3.3"));
    }

    #[test]
    fn test_shims_dir() {
        let paths = VxPaths::new().unwrap();
        let shims = paths.shims_dir();
        assert!(shims.ends_with("shims"));
    }

    // === DynamicPackageRuntime Tests ===

    #[test]
    fn test_dynamic_runtime_from_spec_npm() {
        let runtime = DynamicPackageRuntime::from_spec("npm:typescript@5.3").unwrap();
        assert_eq!(runtime.name(), "typescript");
        assert_eq!(runtime.ecosystem(), Ecosystem::NodeJs);
        assert_eq!(runtime.required_runtime(), "node");
        assert!(runtime.install_method().is_npm());
    }

    #[test]
    fn test_dynamic_runtime_from_spec_pip() {
        let runtime = DynamicPackageRuntime::from_spec("pip:black@24.1").unwrap();
        assert_eq!(runtime.name(), "black");
        assert_eq!(runtime.ecosystem(), Ecosystem::Python);
        assert_eq!(runtime.required_runtime(), "python");
        assert!(runtime.install_method().is_pip());
    }

    #[test]
    fn test_dynamic_runtime_from_spec_auto_detect() {
        // Auto-detect ecosystem from package registry
        let runtime = DynamicPackageRuntime::from_spec("typescript@5.3").unwrap();
        assert_eq!(runtime.ecosystem(), Ecosystem::NodeJs);
    }

    // === PackageRegistry Tests ===

    #[test]
    fn test_registry_find_by_executable() {
        let mut registry = PackageRegistry::new();
        registry.register(GlobalPackage {
            name: "typescript".to_string(),
            version: "5.3.3".to_string(),
            ecosystem: Ecosystem::NodeJs,
            executables: vec!["tsc".to_string(), "tsserver".to_string()],
            ..Default::default()
        }).unwrap();

        // Find by executable name
        let pkg = registry.find_by_executable("tsc");
        assert!(pkg.is_some());
        assert_eq!(pkg.unwrap().name, "typescript");

        // Find by package name
        let pkg = registry.find_by_executable("typescript");
        assert!(pkg.is_none()); // "typescript" is not an executable
    }

    // === Environment Redirection Tests ===

    #[test]
    fn test_npm_env_redirection() {
        let env = PackageManagerEnv::new(Ecosystem::NodeJs, "typescript", "5.3.3");
        let vars = env.install_env(&VxPaths::new().unwrap());
        assert!(vars.contains_key("NPM_CONFIG_PREFIX"));
    }

    #[test]
    fn test_pip_env_redirection() {
        let env = PackageManagerEnv::new(Ecosystem::Python, "black", "24.1.0");
        let vars = env.install_env(&VxPaths::new().unwrap());
        assert!(vars.contains_key("VIRTUAL_ENV"));
        assert!(vars.contains_key("PIP_TARGET"));
    }

    #[test]
    fn test_cargo_env_redirection() {
        let env = PackageManagerEnv::new(Ecosystem::Rust, "ripgrep", "14.0.0");
        let vars = env.install_env(&VxPaths::new().unwrap());
        assert!(vars.contains_key("CARGO_INSTALL_ROOT"));
    }

    // === Shim Tests ===

    #[cfg(unix)]
    #[test]
    fn test_unix_shim_generation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let shim_path = temp_dir.path().join("tsc");
        let target = PathBuf::from("/home/user/.vx/packages/npm/typescript/5.3.3/bin/tsc");

        create_shim(&shim_path, &target).unwrap();

        assert!(shim_path.exists());
        let content = std::fs::read_to_string(&shim_path).unwrap();
        assert!(content.contains("#!/bin/sh"));
        assert!(content.contains(&target.display().to_string()));
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_shim_generation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let shim_path = temp_dir.path().join("tsc.cmd");
        let target = PathBuf::from(r"C:\Users\user\.vx\packages\npm\typescript\5.3.3\bin\tsc.cmd");

        create_shim(&shim_path, &target).unwrap();

        assert!(shim_path.exists());
        let content = std::fs::read_to_string(&shim_path).unwrap();
        assert!(content.contains("@echo off"));
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_symlink_fallback() {
        // Test junction fallback when symlink fails
        let temp_dir = tempfile::tempdir().unwrap();
        let target = temp_dir.path().join("target_dir");
        let link = temp_dir.path().join("link_dir");

        std::fs::create_dir(&target).unwrap();

        // This should try symlink first, then fall back to junction
        let result = create_symlink(&target, &link);
        assert!(result.is_ok());
        assert!(link.exists());
    }
}
```

### Integration Tests

1. **Cross-platform CI**: Test on Windows, macOS, and Linux
2. **Permission tests**: Verify behavior without admin rights (Windows)
3. **Path length tests**: Test with very long package names (Windows)
4. **Concurrent access**: Multiple projects using same packages

### Manual Testing Checklist

**Explicit Invocation**:
- [ ] `vx install-global npm:typescript@5.3` installs to `~/.vx/packages/npm/typescript/5.3.x/`
- [ ] `vx tsc --version` works (explicit invocation)
- [ ] `vx tsc@5.2 --version` uses specific version
- [ ] `vx list-global` shows installed packages
- [ ] `vx uninstall-global npm:typescript` removes package and shim

**Implicit Invocation**:
- [ ] After `vx hook install`, `tsc --version` works directly
- [ ] Shim correctly delegates to package executable
- [ ] Windows: `.cmd` shim works in cmd.exe and PowerShell
- [ ] Unix: Shell wrapper has correct permissions (755)

**Project-Level**:
- [ ] `vx.toml` with `[tools.global]` auto-installs on `vx sync`
- [ ] `vx dev` includes project global tools in PATH
- [ ] Project `.vx/bin/` contains correct symlinks

**Isolation**:
- [ ] `vx npm install -g typescript` creates isolated package (not in node store)
- [ ] Multiple projects can use different typescript versions
- [ ] Upgrading node version doesn't affect global packages

**Platform-Specific**:
- [ ] Windows: Works without Developer Mode (junction fallback)
- [ ] Windows: Works with paths > 200 characters
- [ ] macOS: Quarantine attribute handled correctly
- [ ] Linux: File permissions set correctly

## User Documentation

### Quick Start Guide

```markdown
# Using Global Tools with vx

vx provides isolated global package management that prevents pollution across projects.

## Installing Global Tools

```bash
# Install a global package
vx install-global typescript@5.3
vx install-global black@24.1
vx install-global ripgrep@14

# Or use explicit ecosystem prefix
vx install-global npm:typescript@5.3
vx install-global pip:black@24.1
vx install-global cargo:ripgrep@14
```

## Using Global Tools

After installation, tools are available globally via shims:

```bash
tsc --version
black --version
rg --version
```

## Project-Specific Global Tools

Define in your `vx.toml`:

```toml
[tools.global]
typescript = "5.3"
black = "24.1"
```

Run `vx sync` to install and configure:

```bash
vx sync
```
```

### Windows-Specific Guide

```markdown
# vx on Windows

## Recommended Setup

1. **Enable Developer Mode** (Settings → Privacy & security → For developers)
   - This allows vx to create symlinks without admin rights

2. **Or use vx with standard permissions**
   - vx will automatically use junction points for directories
   - Hard links for files as fallback

## Troubleshooting

### "Permission denied" when creating symlinks

This happens when Developer Mode is not enabled. Options:
1. Enable Developer Mode (recommended)
2. Run terminal as Administrator (not recommended for daily use)
3. vx will automatically fall back to junctions/hard links

### Long path issues

If you see "path too long" errors:
1. Enable long paths in Windows (requires admin once):
   ```powershell
   Set-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem' -Name 'LongPathsEnabled' -Value 1
   ```
2. Or configure vx to use a shorter base path:
   ```toml
   # ~/.vxrc
   base_dir = "C:\\vx"
   ```
```

### macOS-Specific Guide

```markdown
# vx on macOS

## First-Time Setup

vx works out of the box on macOS. No special configuration needed.

## Troubleshooting

### "App is damaged" or security warnings

Downloaded binaries may be quarantined. vx handles this automatically,
but if you see issues:

```bash
xattr -d com.apple.quarantine ~/.vx/packages/**/*
```

### Rosetta 2 (Apple Silicon)

vx automatically downloads arm64 binaries when available.
For x86_64-only tools, ensure Rosetta 2 is installed:

```bash
softwareupdate --install-rosetta
```
```

## Security Considerations

### Package Verification

1. **Checksum verification**: Verify downloaded packages against known checksums
2. **Source verification**: Only install from trusted registries (npm, PyPI, crates.io)
3. **No arbitrary code execution**: Package install scripts run in isolated environment

### Symlink Security

1. **Symlink targets validated**: Only create symlinks to known package directories
2. **No symlink following for writes**: Prevent symlink attacks
3. **Permission checks**: Verify directory permissions before creating symlinks

### Windows-Specific Security

1. **No elevation prompts**: vx never requests admin rights
2. **Junction point safety**: Only create junctions to vx-managed directories
3. **PATH injection prevention**: Validate all PATH modifications

## Backward Compatibility

### Existing Workflows

| Scenario | Current Behavior | New Behavior |
|----------|-----------------|--------------|
| `vx npm install -g pkg` | Installs to node's lib | Redirected to CAS |
| `vx pip install pkg` | Installs to python's site-packages | Redirected to CAS |
| `vx cargo install pkg` | Installs to ~/.cargo/bin | Redirected to CAS |
| Existing global packages | In runtime directories | Continue to work |
| New vx.toml projects | Only runtime tools | Supports global tools |

### Configuration Migration

No configuration migration required. New features are opt-in.

## References

- [pnpm - Fast, disk space efficient package manager](https://pnpm.io/)
- [Nix - The purely functional package manager](https://nixos.org/)
- [mise - The front-end to your dev env](https://mise.jdx.dev/)
- [uv - An extremely fast Python package installer](https://github.com/astral-sh/uv)
- [Windows Symbolic Links](https://docs.microsoft.com/en-us/windows/win32/fileio/symbolic-links)
- [Windows Junction Points](https://docs.microsoft.com/en-us/windows/win32/fileio/hard-links-and-junctions)

## Appendix A: Ecosystem-Specific Details

### npm/Node.js

**Environment Variables**:
- `NPM_CONFIG_PREFIX`: Global install prefix
- `NPM_CONFIG_CACHE`: Cache directory (can be shared)
- `NODE_PATH`: Additional module lookup paths

**Package Structure**:
```
~/.vx/packages/npm/typescript/5.3.3/
├── lib/
│   └── node_modules/
│       └── typescript/
├── bin/
│   ├── tsc -> ../lib/node_modules/typescript/bin/tsc
│   └── tsserver -> ../lib/node_modules/typescript/bin/tsserver
└── package.json  # Metadata
```

### pip/Python

**Environment Variables**:
- `VIRTUAL_ENV`: Virtual environment root
- `PIP_TARGET`: Package install directory
- `PYTHONPATH`: Module lookup paths

**Package Structure**:
```
~/.vx/packages/pip/black/24.1.0/
├── venv/
│   ├── bin/
│   │   ├── black
│   │   └── python -> ~/.vx/store/python/3.11.0/bin/python
│   └── lib/
│       └── python3.11/
│           └── site-packages/
│               └── black/
└── package.json  # Metadata
```

### cargo/Rust

**Environment Variables**:
- `CARGO_INSTALL_ROOT`: Binary install root
- `CARGO_HOME`: Cargo home (registry cache, etc.)

**Package Structure**:
```
~/.vx/packages/cargo/ripgrep/14.0.0/
├── bin/
│   └── rg
└── package.json  # Metadata
```

### go/Go

**Environment Variables**:
- `GOBIN`: Binary install directory
- `GOPATH`: Go workspace (can be shared)

**Package Structure**:
```
~/.vx/packages/go/golangci-lint/1.55.0/
├── bin/
│   └── golangci-lint
└── package.json  # Metadata
```

### gem/Ruby

**Environment Variables**:
- `GEM_HOME`: Gem install directory
- `GEM_PATH`: Gem lookup paths

**Package Structure**:
```
~/.vx/packages/gem/bundler/2.5.0/
├── gems/
│   └── bundler-2.5.0/
├── bin/
│   └── bundle
└── package.json  # Metadata
```

## Appendix B: Error Handling

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `SymlinkPermissionDenied` | Windows without Developer Mode | Enable Developer Mode or run as admin |
| `PathTooLong` | Path > 260 chars on Windows | Enable long paths or use shorter base_dir |
| `PackageNotFound` | Package not in registry | Check package name spelling |
| `VersionNotFound` | Requested version unavailable | Use `vx list-remote pkg` to see versions |
| `RuntimeNotInstalled` | npm pkg needs node | Install node first: `vx use node@20` |

### Error Recovery

```rust
/// Attempt package installation with fallback strategies
pub async fn install_with_fallback(
    package: &PackageRequest,
    paths: &VxPaths,
) -> Result<GlobalPackage> {
    // Strategy 1: Normal symlink installation
    match install_with_symlinks(package, paths).await {
        Ok(pkg) => return Ok(pkg),
        Err(e) if e.is_permission_error() => {
            tracing::warn!("Symlink failed, trying fallback: {}", e);
        }
        Err(e) => return Err(e),
    }

    // Strategy 2: Windows junction fallback
    #[cfg(windows)]
    match install_with_junctions(package, paths).await {
        Ok(pkg) => return Ok(pkg),
        Err(e) => {
            tracing::warn!("Junction failed: {}", e);
        }
    }

    // Strategy 3: Copy installation (no deduplication)
    install_with_copy(package, paths).await
}
```

