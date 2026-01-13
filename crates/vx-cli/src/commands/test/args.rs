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

    /// Full CI test: install + functional tests for all runtimes
    /// This is the complete end-to-end test with real network downloads
    #[arg(long)]
    pub ci: bool,

    /// Specify runtimes to test in CI mode (comma-separated)
    /// Example: --ci-runtimes node,go,uv
    #[arg(long, value_delimiter = ',')]
    pub ci_runtimes: Option<Vec<String>>,

    /// Skip these runtimes in CI mode (comma-separated)
    /// Example: --ci-skip spack,msvc
    #[arg(long, value_delimiter = ',')]
    pub ci_skip: Option<Vec<String>>,

    /// Timeout for each runtime test in seconds (default: 300)
    #[arg(long, default_value = "300")]
    pub timeout: u64,

    /// Continue testing even if some runtimes fail
    #[arg(long)]
    pub keep_going: bool,

    /// Use a custom VX root directory for testing (isolated environment)
    /// If not specified, uses a temporary directory in CI mode
    #[arg(long)]
    pub vx_root: Option<PathBuf>,

    /// Use a temporary directory as VX root (auto-cleaned after test)
    #[arg(long)]
    pub temp_root: bool,

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
