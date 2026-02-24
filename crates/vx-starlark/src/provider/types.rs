//! Type definitions for Starlark provider descriptors and metadata.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Resolved install layout from a Starlark `install_layout()` descriptor
///
/// The Starlark script returns a descriptor dict (e.g. from `msi_install()`,
/// `archive_install()`, or `binary_install()` in install.star). The Rust layer
/// resolves the descriptor into this typed struct and performs the actual I/O.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallLayout {
    /// MSI package installation (Windows only)
    Msi {
        url: String,
        executable_paths: Vec<String>,
        strip_prefix: Option<String>,
        extra_args: Vec<String>,
    },
    /// Archive installation (ZIP, TAR.GZ, TAR.XZ, etc.)
    Archive {
        url: String,
        strip_prefix: Option<String>,
        executable_paths: Vec<String>,
    },
    /// Single binary installation
    Binary {
        url: String,
        executable_name: Option<String>,
        /// Unix file permissions (e.g. "755")
        permissions: String,
    },
    /// System tool finder (for prepare_execution)
    ///
    /// Instructs the Rust runtime to search for an already-installed system tool
    /// via PATH lookup and optional known system paths, before falling back to
    /// the vx-managed installation.
    SystemFind {
        executable: String,
        system_paths: Vec<String>,
        hint: Option<String>,
    },
}

/// Actions returned by `post_extract()` hook in Starlark provider scripts
///
/// The `post_extract()` function returns a list of these action descriptors.
/// The Rust runtime executes them in order after archive extraction.
#[derive(Debug, Clone)]
pub enum PostExtractAction {
    /// Create a shim script that wraps another executable
    ///
    /// Starlark: `create_shim("bunx", "bun", args=["x"])`
    CreateShim {
        name: String,
        target: String,
        args: Vec<String>,
        shim_dir: Option<String>,
    },
    /// Set Unix file permissions on an extracted file
    ///
    /// Starlark: `set_permissions("bin/mytool", "755")`
    SetPermissions { path: String, mode: String },
    /// Run an arbitrary command as part of the post-extract hook
    ///
    /// Starlark: `run_command("install_name_tool", ["-add_rpath", "..."])`
    RunCommand {
        executable: String,
        args: Vec<String>,
        working_dir: Option<String>,
        env: HashMap<String, String>,
        /// How to handle command failure: "warn", "error", "ignore"
        on_failure: String,
    },
    /// Flatten a nested subdirectory into the install root
    ///
    /// Starlark: `flatten_dir(pattern = "jdk-*")`
    ///
    /// Many archives extract to a single top-level subdirectory
    /// (e.g. `jdk-21.0.1+12/`, `ffmpeg-7.1-essentials_build/`).
    /// This action moves all contents one level up and removes the
    /// now-empty subdirectory.
    FlattenDir {
        /// Optional glob pattern to match the subdirectory name (e.g. "jdk-*").
        /// If None, flattens the single subdirectory if exactly one exists.
        pattern: Option<String>,
        keep_subdirs: Vec<String>,
    },
}

/// Actions returned by `pre_run()` hook in Starlark provider scripts
///
/// The `pre_run()` function returns a list of these action descriptors.
/// The Rust runtime executes them in order before running the tool.
#[derive(Debug, Clone)]
pub enum PreRunAction {
    /// Ensure project dependencies are installed before running
    ///
    /// Starlark: `ensure_dependencies("bun")`
    EnsureDependencies {
        package_manager: String,
        check_file: String,
        lock_file: Option<String>,
        install_dir: String,
    },
    /// Run an arbitrary command before the tool executes
    ///
    /// Starlark: `run_command("git", ["submodule", "update"])`
    RunCommand {
        executable: String,
        args: Vec<String>,
        working_dir: Option<String>,
        env: HashMap<String, String>,
        on_failure: String,
    },
}

/// A single environment variable operation, returned by `environment()` in provider.star.
///
/// Provider scripts return a list of `EnvOp` values (created via `env_set()`,
/// `env_prepend()`, `env_append()`, `env_unset()` from `@vx//stdlib:env.star`).
/// The Rust runtime applies them in order, enabling rez-style layered env composition.
///
/// # Example (provider.star)
/// ```python
/// load("@vx//stdlib:env.star", "env_set", "env_prepend")
///
/// def environment(ctx, version):
///     return [
///         env_set("GOROOT", ctx.install_dir),
///         env_prepend("PATH", ctx.install_dir + "/bin"),
///     ]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum EnvOp {
    /// Set an environment variable to a fixed value (overwrite)
    Set { key: String, value: String },
    /// Prepend a value to an environment variable (PATH-style, with separator)
    Prepend {
        key: String,
        value: String,
        #[serde(default = "default_path_sep")]
        sep: String,
    },
    /// Append a value to an environment variable (PATH-style, with separator)
    Append {
        key: String,
        value: String,
        #[serde(default = "default_path_sep")]
        sep: String,
    },
    /// Remove an environment variable
    Unset { key: String },
}

fn default_path_sep() -> String {
    if cfg!(windows) {
        ";".to_string()
    } else {
        ":".to_string()
    }
}

impl EnvOp {
    /// Apply this operation to a mutable environment map.
    ///
    /// `system_env` is the base environment (e.g. `std::env::vars()`).
    /// When prepending/appending, the existing value in `env` is used first,
    /// falling back to `system_env`, then to an empty string.
    pub fn apply(&self, env: &mut std::collections::HashMap<String, String>) {
        match self {
            EnvOp::Set { key, value } => {
                env.insert(key.clone(), value.clone());
            }
            EnvOp::Prepend { key, value, sep } => {
                let existing = env
                    .get(key)
                    .cloned()
                    .or_else(|| std::env::var(key).ok())
                    .unwrap_or_default();
                let new_val = if existing.is_empty() {
                    value.clone()
                } else {
                    format!("{}{}{}", value, sep, existing)
                };
                env.insert(key.clone(), new_val);
            }
            EnvOp::Append { key, value, sep } => {
                let existing = env
                    .get(key)
                    .cloned()
                    .or_else(|| std::env::var(key).ok())
                    .unwrap_or_default();
                let new_val = if existing.is_empty() {
                    value.clone()
                } else {
                    format!("{}{}{}", existing, sep, value)
                };
                env.insert(key.clone(), new_val);
            }
            EnvOp::Unset { key } => {
                env.remove(key);
            }
        }
    }
}

/// Apply a sequence of EnvOps to build a final environment map.
///
/// Operations are applied in order, so later ops can override earlier ones.
/// This enables rez-style layered composition when multiple providers contribute.
pub fn apply_env_ops(
    ops: &[EnvOp],
    base: Option<&std::collections::HashMap<String, String>>,
) -> std::collections::HashMap<String, String> {
    let mut env = base.cloned().unwrap_or_default();
    for op in ops {
        op.apply(&mut env);
    }
    env
}

/// Package alias configuration (RFC 0033)
///
/// When set on a provider, `vx <name>` is automatically routed to
/// `vx <ecosystem>:<package>`, unifying the execution path with
/// the RFC 0027 package request mechanism.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageAlias {
    /// Target ecosystem (e.g., "npm", "pip", "cargo")
    pub ecosystem: String,
    /// Package name in that ecosystem
    pub package: String,
    /// Executable name override (defaults to package name)
    #[serde(default)]
    pub executable: Option<String>,
}

/// Provider metadata parsed from the script
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderMeta {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    /// Platform constraints (os: ["windows", "linux"])
    #[serde(default)]
    pub platforms: Option<HashMap<String, Vec<String>>>,
    /// Package alias: routes `vx <name>` to `vx <ecosystem>:<package>` (RFC 0033)
    #[serde(default)]
    pub package_alias: Option<PackageAlias>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Check type for a test command entry
///
/// Determines how the test framework interprets the `command` field:
///
/// - `"command"` (default) — run a shell command and check exit code / output
/// - `"check_path"`        — assert that a file or directory exists at the given path
/// - `"check_env"`         — assert that an environment variable is set (and optionally matches)
/// - `"check_file"`        — assert that a file exists and optionally contains a pattern
/// - `"check_not_path"`    — assert that a path does NOT exist
/// - `"check_not_env"`     — assert that an environment variable is NOT set
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TestCheckType {
    /// Run a shell command (default)
    #[default]
    Command,
    /// Assert a path (file or directory) exists
    CheckPath,
    /// Assert an environment variable is set
    CheckEnv,
    /// Assert a file exists and optionally contains a pattern
    CheckFile,
    /// Assert a path does NOT exist
    CheckNotPath,
    /// Assert an environment variable is NOT set
    CheckNotEnv,
}

/// A single test command definition for a runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCommandMeta {
    /// Command template (supports {executable}, {version}, {install_dir})
    ///
    /// For `check_type = "command"`: the shell command to run.
    /// For `check_type = "check_path"` / `"check_not_path"`: the path to check.
    /// For `check_type = "check_env"` / `"check_not_env"`: the env var name.
    /// For `check_type = "check_file"`: the file path to check.
    pub command: String,
    /// Check type — determines how `command` is interpreted
    #[serde(default)]
    pub check_type: TestCheckType,
    /// Expect the command to succeed (exit code 0) — only for `command` type
    #[serde(default = "default_true")]
    pub expect_success: bool,
    /// Expected output pattern (regex) — for `command` type: matches stdout/stderr;
    /// for `check_env`: the env var value must match; for `check_file`: file content must match
    #[serde(default)]
    pub expected_output: Option<String>,
    /// Test name/description
    #[serde(default)]
    pub name: Option<String>,
    /// Timeout for this specific command (ms) — only for `command` type
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

fn default_true() -> bool {
    true
}

/// Runtime metadata parsed from the script
#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeMeta {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub executable: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default = "default_priority")]
    pub priority: u32,
    /// Command prefix to prepend before user args (e.g., ["x"] for bunx -> bun x)
    #[serde(default)]
    pub command_prefix: Vec<String>,
    /// Known system paths (glob patterns) for system-installed tools
    ///
    /// Populated from the `system_paths` field in the `runtimes` list of provider.star.
    /// Used by `where_cmd` to locate system-installed tools (e.g. MSVC cl.exe).
    #[serde(default)]
    pub system_paths: Vec<String>,
    /// Functional test commands for this runtime
    #[serde(default)]
    pub test_commands: Vec<TestCommandMeta>,
    /// Install dependencies (vx-managed runtimes that must be installed first)
    /// Format: ["7zip", "node>=18", ...]
    #[serde(default)]
    pub install_deps: Vec<String>,
    /// The parent runtime that bundles this runtime (e.g., "uv" for uvx)
    /// When set, this runtime's executable is found in the parent's store directory.
    #[serde(default)]
    pub bundled_with: Option<String>,
}

fn default_priority() -> u32 {
    100
}

/// Detect if a path is a Starlark provider
pub fn is_starlark_provider(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e == "star")
        .unwrap_or(false)
}

/// Check if a directory contains a Starlark provider
pub fn has_starlark_provider(dir: &std::path::Path) -> bool {
    dir.join("provider.star").exists()
}
