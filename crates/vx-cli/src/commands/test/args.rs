//! Test command arguments

use clap::Args as ClapArgs;
use std::path::PathBuf;

/// Test runtime availability and providers (CI-friendly)
#[derive(ClapArgs, Clone, Debug)]
pub struct Args {
    /// Runtime name to test (e.g., "yarn", "node", "go")
    pub runtime: Option<String>,

    /// Test all registered runtimes
    #[arg(long, conflicts_with_all = &["runtime", "extension", "local"])]
    pub all: bool,

    /// Test a provider from URL (e.g., https://github.com/user/vx-provider-foo)
    #[arg(long, conflicts_with_all = &["runtime", "all", "local"])]
    pub extension: Option<String>,

    /// Test a local provider directory (for development)
    #[arg(long, conflicts_with_all = &["runtime", "all", "extension"])]
    pub local: Option<PathBuf>,

    // === Test Modes ===
    /// Only test platform support (no installation required)
    #[arg(long)]
    pub platform_only: bool,

    /// Run functional tests (execute --version, etc.)
    #[arg(long)]
    pub functional: bool,

    /// Test installation process
    #[arg(long)]
    pub install: bool,

    // === Checks ===
    /// Check if runtime is installed in vx store
    #[arg(long)]
    pub installed: bool,

    /// Check if runtime is available on system PATH
    #[arg(long)]
    pub system: bool,

    // === Output Control ===
    /// Show detailed test information
    #[arg(long)]
    pub detailed: bool,

    /// Silent mode: exit code only, no output
    #[arg(short, long)]
    pub quiet: bool,

    /// JSON output format (for CI integration)
    #[arg(long)]
    pub json: bool,

    /// Verbose output (show all test steps)
    #[arg(short, long)]
    pub verbose: bool,
}
