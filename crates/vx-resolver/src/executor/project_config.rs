//! Project configuration for vx.toml
//!
//! This module handles loading and querying project-level tool version
//! configurations from vx.toml files.

use std::collections::HashMap;
use tracing::debug;
use vx_config::parse_config;
use vx_paths::find_config_file_upward;

/// Project tools configuration extracted from vx.toml
#[derive(Debug, Clone)]
pub struct ProjectToolsConfig {
    /// Tool versions from vx.toml (tool_name -> version)
    tools: HashMap<String, String>,
}

impl ProjectToolsConfig {
    /// Create a ProjectToolsConfig from a tools map (for testing)
    pub fn from_tools(tools: HashMap<String, String>) -> Self {
        Self { tools }
    }

    /// Load project configuration from vx.toml in current directory or parent directories
    pub fn load() -> Option<Self> {
        let cwd = std::env::current_dir().ok()?;
        let config_path = find_config_file_upward(&cwd)?;
        let config = parse_config(&config_path).ok()?;
        let tools = config.tools_as_hashmap();

        if tools.is_empty() {
            debug!("No tools defined in vx.toml at {}", config_path.display());
            None
        } else {
            debug!(
                "Loaded {} tool(s) from vx.toml at {}",
                tools.len(),
                config_path.display()
            );
            Some(Self { tools })
        }
    }

    /// Get the version for a specific tool
    pub fn get_version(&self, tool: &str) -> Option<&str> {
        self.tools.get(tool).map(|s| s.as_str())
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
        // First, try direct lookup
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
    /// When `vx.toml` specifies tools like `[tools.msvc]`, running `vx node` should
    /// also inject MSVC's environment variables (VCINSTALLDIR, VCToolsInstallDir, etc.)
    /// so that tools like node-gyp can discover the compiler.
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
