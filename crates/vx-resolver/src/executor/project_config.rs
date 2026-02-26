//! Project configuration for vx.toml and vx.lock
//!
//! This module handles loading and querying project-level tool version
//! configurations from vx.toml and vx.lock files.
//!
//! # Version Priority
//!
//! When resolving a tool version, the following priority is used:
//! 1. **Explicit** - Command-line specified (e.g., `vx node@20`)
//! 2. **vx.lock** - Locked version from vx.lock (highest priority in config)
//! 3. **vx.toml** - Project configuration version
//! 4. **Latest** - Default to the latest available version
//!
//! This ensures reproducible builds: once a version is locked in vx.lock,
//! it will be used consistently until the lock file is updated.

use std::collections::HashMap;
use tracing::debug;
use vx_config::parse_config;
use vx_paths::find_config_file_upward;

use crate::version::LockFile;

/// Install options for a specific tool (key-value env-style pairs)
type InstallEnvVars = HashMap<String, String>;

/// Project tools configuration extracted from vx.toml and vx.lock
#[derive(Debug, Clone)]
pub struct ProjectToolsConfig {
    /// Tool versions from vx.toml (tool_name -> version)
    tools: HashMap<String, String>,
    /// Locked tool versions from vx.lock (higher priority than vx.toml)
    locked_tools: HashMap<String, String>,
    /// Per-tool install options extracted from detailed ToolConfig
    /// (e.g., msvc -> {"VX_MSVC_COMPONENTS": "spectre", "VX_MSVC_EXCLUDE_PATTERNS": "..."})
    tool_install_options: HashMap<String, InstallEnvVars>,
}

impl ProjectToolsConfig {
    /// Create a ProjectToolsConfig from a tools map (for testing)
    pub fn from_tools(tools: HashMap<String, String>) -> Self {
        Self {
            tools,
            locked_tools: HashMap::new(),
            tool_install_options: HashMap::new(),
        }
    }

    /// Create a ProjectToolsConfig with both tools and locked versions (for testing)
    pub fn from_tools_with_locked(
        tools: HashMap<String, String>,
        locked_tools: HashMap<String, String>,
    ) -> Self {
        Self {
            tools,
            locked_tools,
            tool_install_options: HashMap::new(),
        }
    }

    /// Load project configuration from vx.toml and vx.lock in current directory or parent directories
    ///
    /// This loads both files from the same directory where vx.toml is found.
    /// The vx.lock has higher priority than vx.toml for version resolution.
    pub fn load() -> Option<Self> {
        let cwd = std::env::current_dir().ok()?;
        let config_path = find_config_file_upward(&cwd)?;
        let config = parse_config(&config_path).ok()?;
        let tools = config.tools_as_hashmap();

        // Load locked versions from vx.lock (same directory as vx.toml)
        let locked_tools = Self::load_locked_versions(&config_path);

        if tools.is_empty() && locked_tools.is_empty() {
            debug!(
                "No tools defined in vx.toml or vx.lock at {}",
                config_path.display()
            );
            None
        } else {
            debug!(
                "Loaded {} tool(s) from vx.toml, {} locked version(s) from vx.lock at {}",
                tools.len(),
                locked_tools.len(),
                config_path.display()
            );

            // Extract install options from detailed tool configs
            let tool_install_options = Self::extract_install_options(&config);

            Some(Self {
                tools,
                locked_tools,
                tool_install_options,
            })
        }
    }

    /// Load locked versions from vx.lock file
    ///
    /// The lock file is expected to be in the same directory as vx.toml.
    fn load_locked_versions(config_path: &std::path::Path) -> HashMap<String, String> {
        let lock_path = config_path
            .parent()
            .map(|p| p.join("vx.lock"))
            .unwrap_or_else(|| config_path.with_file_name("vx.lock"));

        if !lock_path.exists() {
            debug!("No vx.lock found at {}", lock_path.display());
            return HashMap::new();
        }

        match LockFile::load(&lock_path) {
            Ok(lockfile) => {
                let locked: HashMap<String, String> = lockfile
                    .tools
                    .into_iter()
                    .map(|(name, locked_tool)| (name, locked_tool.version))
                    .collect();

                debug!(
                    "Loaded {} locked version(s) from {}",
                    locked.len(),
                    lock_path.display()
                );
                locked
            }
            Err(e) => {
                debug!("Failed to load vx.lock: {}", e);
                HashMap::new()
            }
        }
    }

    /// Get the version for a specific tool
    ///
    /// Priority: vx.lock > vx.toml
    ///
    /// This ensures reproducible builds - once a version is locked,
    /// it will be used consistently until the lock file is updated.
    pub fn get_version(&self, tool: &str) -> Option<&str> {
        // First, check vx.lock (highest priority)
        if let Some(locked) = self.locked_tools.get(tool) {
            return Some(locked);
        }
        // Then, check vx.toml
        self.tools.get(tool).map(|s| s.as_str())
    }

    /// Check if a tool has a locked version in vx.lock
    pub fn is_locked(&self, tool: &str) -> bool {
        self.locked_tools.contains_key(tool)
    }

    /// Get all locked tool names
    pub fn locked_tool_names(&self) -> Vec<&str> {
        self.locked_tools.keys().map(|s| s.as_str()).collect()
    }

    /// Get the version for a tool with ecosystem fallback
    ///
    /// First tries to find the tool directly. If not found, it checks if the tool
    /// belongs to a known ecosystem and tries to use the primary runtime's version.
    ///
    /// **Important**: Only "bundled tools" (tools that are part of the primary runtime)
    /// should fall back to the primary runtime's version. Independent tools like pnpm,
    /// yarn, and bun have their own version schemes and should NOT inherit the Node.js
    /// version.
    ///
    /// Priority: vx.lock > vx.toml (for both direct and fallback lookups)
    ///
    /// Examples of valid fallbacks:
    /// - `cargo` -> checks `cargo` then `rust` (cargo is bundled with Rust)
    /// - `rustc` -> checks `rustc` then `rust` (rustc is bundled with Rust)
    /// - `npm` -> checks `npm` then `node` (npm is bundled with Node.js)
    /// - `pip` -> checks `pip` then `python` (pip is often bundled with Python)
    ///
    /// Examples that should NOT fall back:
    /// - `rustup` -> only checks `rustup` (rustup has its own version scheme: 1.27.x, 1.28.x)
    /// - `pnpm` -> only checks `pnpm` (pnpm has its own version scheme: 9.x, 10.x)
    /// - `yarn` -> only checks `yarn` (yarn has its own version scheme: 1.x, 2.x, 3.x, 4.x)
    /// - `bun` -> only checks `bun` (bun has its own version scheme)
    pub fn get_version_with_fallback(&self, tool: &str) -> Option<&str> {
        // First, try direct lookup (respects lock > config priority)
        if let Some(version) = self.get_version(tool) {
            return Some(version);
        }

        // Fallback to primary runtime for the ecosystem (only for bundled tools)
        let primary = self.bundled_tool_runtime(tool)?;
        self.get_version(primary)
    }

    /// Get companion tools from vx.toml that should have their `prepare_environment()`
    /// called when executing any other tool.
    ///
    /// When `vx.toml` specifies tools like `[tools.msvc]`, running ANY tool
    /// (`vx node`, `vx cmake`, `vx cargo`, `vx dotnet`, etc.) will inject MSVC's
    /// environment variables (VCINSTALLDIR, VCToolsInstallDir, etc.) so that any
    /// tool needing a C/C++ compiler can discover the vx-managed installation.
    ///
    /// Returns a list of (tool_name, version) pairs, excluding the primary runtime
    /// and its bundled tools.
    pub fn get_companion_tools(&self, primary_runtime: &str) -> Vec<(&str, &str)> {
        let primary_ecosystem = self.bundled_tool_runtime(primary_runtime);

        self.tools
            .iter()
            .filter(|(tool_name, _)| {
                let name = tool_name.as_str();
                // Skip the primary runtime itself
                if name == primary_runtime {
                    return false;
                }
                // Skip bundled tools of the primary runtime
                // e.g., if running node, skip npm/npx since they share the same ecosystem
                if let Some(runtime) = self.bundled_tool_runtime(name)
                    && runtime == primary_runtime
                {
                    return false;
                }
                // Skip if the primary runtime is a bundled tool and this is its parent
                if let Some(ecosystem) = primary_ecosystem
                    && name == ecosystem
                {
                    return false;
                }
                true
            })
            .map(|(name, version)| (name.as_str(), version.as_str()))
            .collect()
    }

    /// Get install options for a specific tool.
    ///
    /// Returns `None` if the tool has no detailed configuration (i.e., it uses `Simple` version).
    /// The returned HashMap contains env-style key-value pairs that should be passed to
    /// `RuntimeContext.install_options` before calling `runtime.install()`.
    pub fn get_install_options(&self, tool: &str) -> Option<&InstallEnvVars> {
        self.tool_install_options.get(tool)
    }

    /// Extract install options from all detailed ToolConfig entries in VxConfig.
    ///
    /// This mirrors the logic in `sync.rs::build_install_env_vars()` but stores
    /// the result in `ProjectToolsConfig` for use by the Executor path.
    fn extract_install_options(config: &vx_config::VxConfig) -> HashMap<String, InstallEnvVars> {
        let mut result = HashMap::new();

        // Check tools section
        for name in config.tools.keys() {
            if let Some(tool_config) = config.get_tool_config(name)
                && let Some(env_vars) = Self::build_env_vars_from_tool_config(name, tool_config)
            {
                result.insert(name.to_string(), env_vars);
            }
        }

        // Check runtimes section (tools takes precedence)
        for name in config.runtimes.keys() {
            if result.contains_key(name) {
                continue;
            }
            if let Some(tool_config) = config.get_tool_config(name)
                && let Some(env_vars) = Self::build_env_vars_from_tool_config(name, tool_config)
            {
                result.insert(name.to_string(), env_vars);
            }
        }

        result
    }

    /// Build environment variable map from a single ToolConfig.
    ///
    /// Returns `None` if the tool config has no install-relevant options.
    fn build_env_vars_from_tool_config(
        name: &str,
        tool_config: &vx_config::ToolConfig,
    ) -> Option<InstallEnvVars> {
        let mut env_vars = HashMap::new();

        if let Some(components) = &tool_config.components
            && !components.is_empty()
        {
            env_vars.insert("VX_MSVC_COMPONENTS".to_string(), components.join(","));
        }

        if let Some(patterns) = &tool_config.exclude_patterns
            && !patterns.is_empty()
        {
            env_vars.insert("VX_MSVC_EXCLUDE_PATTERNS".to_string(), patterns.join(","));
        }

        if let Some(install_env) = &tool_config.install_env {
            env_vars.extend(install_env.clone());
        }

        if !env_vars.is_empty() {
            debug!(
                "Extracted {} install option(s) for tool '{}'",
                env_vars.len(),
                name
            );
            Some(env_vars)
        } else {
            None
        }
    }

    /// Get the primary runtime name for a bundled tool
    ///
    /// Returns Some(runtime) only for tools that are **bundled with** their primary runtime
    /// and share the same version. Independent package managers (pnpm, yarn, bun) are NOT
    /// included because they have their own independent version schemes.
    ///
    /// # Bundled vs Independent Tools
    ///
    /// **Bundled tools** (should fall back):
    /// - `cargo`, `rustc` -> bundled with `rust`
    /// - `npm`, `npx` -> bundled with `node`
    /// - `pip`, `pip3` -> often bundled with `python`
    /// - `gofmt` -> bundled with `go`
    ///
    /// **Independent tools** (should NOT fall back):
    /// - `rustup` -> has its own version scheme (1.27.x, 1.28.x), NOT Rust compiler versions
    /// - `pnpm` -> has versions like 9.0.1, 10.28.2 (NOT node versions)
    /// - `yarn` -> has versions like 1.22.0, 2.4.3, 4.0.0 (NOT node versions)
    /// - `bun` -> has versions like 1.0.0, 1.1.0 (NOT node versions)
    fn bundled_tool_runtime(&self, tool: &str) -> Option<&'static str> {
        match tool {
            // Rust ecosystem - only rustc and cargo are bundled with rust toolchain
            // rustup is the installer/manager itself with its own independent version scheme
            "rustc" | "cargo" => Some("rust"),

            // Node.js ecosystem - ONLY npm/npx are bundled with Node.js
            // pnpm, yarn, bun are INDEPENDENT tools with their own version schemes
            "npm" | "npx" => Some("node"),

            // Python ecosystem - pip is often bundled with Python
            // uv is independent and should not fall back
            "pip" | "pip3" => Some("python"),

            // Go ecosystem - gofmt is bundled with Go
            "gofmt" => Some("go"),

            // Everything else (including pnpm, yarn, bun, uv, etc.) should NOT fall back
            _ => None,
        }
    }
}
