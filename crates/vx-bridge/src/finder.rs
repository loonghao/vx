//! Executable finder — searches for the target command in vx store and system paths.

use std::path::{Path, PathBuf};

/// Strategy for finding executables, tried in order.
#[derive(Debug, Clone)]
pub enum SearchStrategy {
    /// Search in the vx store for a managed runtime.
    /// Uses `vx_paths::get_latest_runtime_root(name)` to find the latest installed version.
    VxRuntime(String),
    /// Search at specific absolute paths (e.g., well-known install locations).
    AbsolutePaths(Vec<PathBuf>),
    /// Search for a command name in the system PATH environment variable.
    SystemPath(String),
}

/// Finds executables according to a sequence of search strategies.
pub struct ExecutableFinder {
    strategies: Vec<SearchStrategy>,
}

impl ExecutableFinder {
    pub fn new(strategies: Vec<SearchStrategy>) -> Self {
        Self { strategies }
    }

    /// Find the first matching executable.
    pub fn find(&self) -> Option<PathBuf> {
        for strategy in &self.strategies {
            if let Some(path) = self.try_strategy(strategy) {
                return Some(path);
            }
        }
        None
    }

    fn try_strategy(&self, strategy: &SearchStrategy) -> Option<PathBuf> {
        match strategy {
            SearchStrategy::VxRuntime(name) => Self::find_in_vx_store(name),
            SearchStrategy::AbsolutePaths(paths) => Self::find_in_absolute_paths(paths),
            SearchStrategy::SystemPath(name) => Self::find_in_system_path(name),
        }
    }

    /// Search the vx store for the latest installed version of a runtime.
    fn find_in_vx_store(runtime_name: &str) -> Option<PathBuf> {
        match vx_paths::get_latest_runtime_root(runtime_name) {
            Ok(Some(root)) => {
                let exe = root.executable_path().to_path_buf();
                if exe.exists() { Some(exe) } else { None }
            }
            _ => None,
        }
    }

    /// Check a list of absolute paths for existence.
    fn find_in_absolute_paths(paths: &[PathBuf]) -> Option<PathBuf> {
        paths.iter().find(|p| p.exists()).cloned()
    }

    /// Search the system PATH for a command.
    fn find_in_system_path(command_name: &str) -> Option<PathBuf> {
        let exe_name = if cfg!(windows) && !command_name.contains('.') {
            format!("{}.exe", command_name)
        } else {
            command_name.to_string()
        };

        let path_var = std::env::var("PATH").ok()?;
        let separator = if cfg!(windows) { ';' } else { ':' };

        for dir in path_var.split(separator) {
            let candidate = Path::new(dir).join(&exe_name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
        None
    }
}
