//! Bridge configuration — declarative builder for bridge executables.

use std::path::PathBuf;
use std::process::ExitCode;

use crate::finder::{ExecutableFinder, SearchStrategy};
use crate::runner::run_bridge;

/// Declarative configuration for a bridge executable.
///
/// A bridge is a stub executable that delegates to another tool managed by vx.
/// For example, `MSBuild.exe` delegates to `dotnet msbuild`.
///
/// # Example
///
/// ```rust,no_run
/// use vx_bridge::BridgeConfig;
///
/// fn main() -> std::process::ExitCode {
///     BridgeConfig::new("MSBuild")
///         .target_vx_runtime("dotnet")
///         .prefix_args(&["msbuild"])
///         .run()
/// }
/// ```
pub struct BridgeConfig {
    /// Human-readable name for error messages (e.g., "MSBuild").
    name: String,
    /// Search strategies for finding the target executable, tried in order.
    strategies: Vec<SearchStrategy>,
    /// Arguments to prepend before the caller's arguments.
    prefix: Vec<String>,
    /// Hint message shown when the target executable is not found.
    not_found_hint: Option<String>,
}

impl BridgeConfig {
    /// Create a new bridge configuration with a name (used in error messages).
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            strategies: Vec::new(),
            prefix: Vec::new(),
            not_found_hint: None,
        }
    }

    /// Add a vx-managed runtime as a search target.
    ///
    /// This searches `~/.vx/store/{runtime_name}/` for the latest installed version.
    pub fn target_vx_runtime(mut self, runtime_name: &str) -> Self {
        self.strategies
            .push(SearchStrategy::VxRuntime(runtime_name.to_string()));
        self
    }

    /// Add specific absolute paths to search.
    ///
    /// These are checked after vx store but before PATH. Useful for well-known
    /// installation locations (e.g., `C:\Program Files\dotnet\dotnet.exe`).
    pub fn system_search_paths(mut self, paths: &[&str]) -> Self {
        self.strategies.push(SearchStrategy::AbsolutePaths(
            paths.iter().map(PathBuf::from).collect(),
        ));
        self
    }

    /// Add a system PATH search for a command name.
    ///
    /// On Windows, `.exe` is automatically appended if not present.
    pub fn system_path_search(mut self, command_name: &str) -> Self {
        self.strategies
            .push(SearchStrategy::SystemPath(command_name.to_string()));
        self
    }

    /// Set arguments to prepend before the caller's arguments.
    ///
    /// For example, `prefix_args(&["msbuild"])` turns `MSBuild.exe /t:Build`
    /// into `dotnet msbuild /t:Build`.
    pub fn prefix_args(mut self, args: &[&str]) -> Self {
        self.prefix = args.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set a custom hint message shown when the target executable is not found.
    pub fn not_found_hint(mut self, hint: &str) -> Self {
        self.not_found_hint = Some(hint.to_string());
        self
    }

    /// Execute the bridge: find the target, forward args, return exit code.
    ///
    /// This is the main entry point — call this from `fn main()`.
    pub fn run(self) -> ExitCode {
        let caller_args: Vec<String> = std::env::args().skip(1).collect();

        let finder = ExecutableFinder::new(self.strategies);

        let executable = match finder.find() {
            Some(path) => path,
            None => {
                eprintln!("vx {} bridge: target executable not found.", self.name);
                if let Some(hint) = &self.not_found_hint {
                    eprintln!("{}", hint);
                }
                return ExitCode::from(1);
            }
        };

        let prefix_refs: Vec<&str> = self.prefix.iter().map(|s| s.as_str()).collect();
        run_bridge(&executable, &prefix_refs, &caller_args)
    }
}
