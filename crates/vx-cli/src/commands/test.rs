//! Test command - Universal Provider Testing Framework
//!
//! This command provides a comprehensive testing framework for vx providers:
//! 1. **Built-in Providers**: Test all registered providers (E2E)
//! 2. **Local Providers**: Test providers during development
//! 3. **Remote Providers**: Test external providers from URLs
//!
//! Use cases:
//! - CI/CD: `vx test --all --json`
//! - Development: `vx test ./my-provider --functional`
//! - Extension: `vx test --extension https://github.com/user/provider`

use crate::commands::{CommandContext, CommandHandler};
use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Clone)]
pub struct TestCommand {
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

#[async_trait]
impl CommandHandler for TestCommand {
    async fn execute(&self, ctx: &CommandContext) -> Result<()> {
        // Determine test mode
        if self.all {
            handle_test_all_providers(ctx, self).await
        } else if let Some(ref url) = self.extension {
            handle_test_extension(ctx, url, self).await
        } else if let Some(ref path) = self.local {
            handle_test_local_provider(ctx, path, self).await
        } else if let Some(ref runtime) = self.runtime {
            handle_test_runtime(ctx, runtime, self).await
        } else {
            anyhow::bail!(
                "Please specify:\n  \
                 - A runtime: vx test <runtime>\n  \
                 - --all: test all providers\n  \
                 - --local <path>: test local provider\n  \
                 - --extension <url>: test remote provider"
            );
        }
    }
}

// ============================================================================
// Test Modes
// ============================================================================

/// Test a single runtime
async fn handle_test_runtime(
    ctx: &CommandContext,
    runtime_name: &str,
    opts: &TestCommand,
) -> Result<()> {
    let tester = RuntimeTester::new(ctx, opts);
    let result = tester.test_runtime(runtime_name).await?;

    output_single_result(&result, opts);
    exit_with_result(&result);
}

/// Test all registered providers
async fn handle_test_all_providers(ctx: &CommandContext, opts: &TestCommand) -> Result<()> {
    let tester = ProviderTester::new(ctx, opts);
    let summary = tester.test_all().await?;

    output_summary(&summary, opts);
    exit_with_summary(&summary);
}

/// Test a local provider (development mode)
async fn handle_test_local_provider(
    ctx: &CommandContext,
    path: &PathBuf,
    opts: &TestCommand,
) -> Result<()> {
    if !opts.quiet {
        println!("🧪 Testing local provider: {}", path.display());
    }

    // Load provider.toml
    let provider_toml = path.join("provider.toml");
    if !provider_toml.exists() {
        anyhow::bail!(
            "provider.toml not found in {}. Not a valid provider directory.",
            path.display()
        );
    }

    let tester = LocalProviderTester::new(ctx, path, opts);
    let summary = tester.test_provider().await?;

    output_summary(&summary, opts);
    exit_with_summary(&summary);
}

/// Test a remote provider extension
async fn handle_test_extension(
    ctx: &CommandContext,
    url: &str,
    opts: &TestCommand,
) -> Result<()> {
    if !opts.quiet {
        println!("🌐 Testing extension from: {}", url);
    }

    // Download and cache extension
    let cache_dir = ctx.runtime_context.paths.cache_dir().join("extensions");
    std::fs::create_dir_all(&cache_dir)?;

    let tester = ExtensionTester::new(ctx, url, opts);
    let summary = tester.test_extension().await?;

    output_summary(&summary, opts);
    exit_with_summary(&summary);
}

// ============================================================================
// Runtime Tester - Test a single runtime
// ============================================================================

struct RuntimeTester<'a> {
    ctx: &'a CommandContext,
    opts: &'a TestCommand,
}

impl<'a> RuntimeTester<'a> {
    fn new(ctx: &'a CommandContext, opts: &'a TestCommand) -> Self {
        Self { ctx, opts }
    }

    async fn test_runtime(&self, runtime_name: &str) -> Result<TestResult> {
        let mut result = TestResult::new(runtime_name);

        // Step 1: Find runtime (search by runtime name, not provider name)
        let runtime = self
            .ctx
            .registry()
            .get_runtime(runtime_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown runtime: {}", runtime_name))?;

        // Step 2: Platform support
        result.platform_supported = self.test_platform_support(&*runtime)?;
        if !result.platform_supported {
            return Ok(result);
        }

        if self.opts.platform_only {
            result.passed = true;
            return Ok(result);
        }

        // Step 3: Installation status
        if self.opts.installed || !self.opts.system {
            result.vx_installed = self.check_vx_installed(runtime_name)?;
            result.installed_versions = self.list_installed_versions(runtime_name)?;
        }

        // Step 4: System availability
        if self.opts.system || !self.opts.installed {
            result.system_available = self.check_system_available(runtime_name)?;
            if result.system_available {
                result.system_path = which::which(runtime_name).ok().map(|p| p.display().to_string());
            }
        }

        // Step 5: Functional tests
        if self.opts.functional {
            result.functional_test = self.run_functional_test(runtime_name).await;
        }

        // Step 6: Installation test
        if self.opts.install {
            result.install_test = self.test_installation(runtime_name).await;
        }

        result.available = result.vx_installed || result.system_available;
        result.passed = result.available || result.functional_test.unwrap_or(false);

        Ok(result)
    }

    fn test_platform_support(&self, runtime: &dyn vx_runtime::Runtime) -> Result<bool> {
        let current_platform = vx_runtime::Platform::current();
        Ok(runtime.is_platform_supported(&current_platform))
    }

    fn check_vx_installed(&self, runtime_name: &str) -> Result<bool> {
        let path_manager = vx_paths::PathManager::new()?;
        let versions = path_manager.list_store_versions(runtime_name)?;
        Ok(!versions.is_empty())
    }

    fn list_installed_versions(&self, runtime_name: &str) -> Result<Vec<String>> {
        let path_manager = vx_paths::PathManager::new()?;
        path_manager.list_store_versions(runtime_name)
    }

    fn check_system_available(&self, runtime_name: &str) -> Result<bool> {
        Ok(which::which(runtime_name).is_ok())
    }

    async fn run_functional_test(&self, runtime_name: &str) -> Option<bool> {
        // Try to execute --version
        let output = std::process::Command::new("vx")
            .arg(runtime_name)
            .arg("--version")
            .output()
            .ok()?;

        Some(output.status.success())
    }

    async fn test_installation(&self, _runtime_name: &str) -> Option<bool> {
        // TODO: Test installation process
        // This would install, verify, and cleanup
        None
    }
}

// ============================================================================
// Provider Tester - Test all providers
// ============================================================================

struct ProviderTester<'a> {
    ctx: &'a CommandContext,
    opts: &'a TestCommand,
}

impl<'a> ProviderTester<'a> {
    fn new(ctx: &'a CommandContext, opts: &'a TestCommand) -> Self {
        Self { ctx, opts }
    }

    async fn test_all(&self) -> Result<TestSummary> {
        let mut summary = TestSummary::default();
        let registry = self.ctx.registry();
        let all_providers = registry.providers();

        for provider in all_providers {
            if self.opts.verbose && !self.opts.quiet {
                println!("\n=== Testing Provider: {} ===", provider.name());
            }

            for runtime in provider.runtimes() {
                let runtime_name = runtime.name();
                let tester = RuntimeTester::new(self.ctx, self.opts);
                
                match tester.test_runtime(runtime_name).await {
                    Ok(result) => {
                        if !self.opts.quiet && !self.opts.json {
                            self.print_result_line(&result);
                        }
                        summary.add_result(result);
                    }
                    Err(e) => {
                        if self.opts.verbose && !self.opts.quiet {
                            eprintln!("  ✗ {} - error: {}", runtime_name, e);
                        }
                        summary.errors.push((runtime_name.to_string(), e.to_string()));
                    }
                }
            }
        }

        Ok(summary)
    }

    fn print_result_line(&self, result: &TestResult) {
        if !result.platform_supported {
            println!("  ⚠ {} - platform not supported", result.runtime);
        } else if result.passed {
            println!("  ✓ {} - passed", result.runtime);
        } else {
            println!("  ✗ {} - failed", result.runtime);
        }
    }
}

// ============================================================================
// Local Provider Tester - Test a provider directory
// ============================================================================

struct LocalProviderTester<'a> {
    _ctx: &'a CommandContext,
    path: &'a PathBuf,
    opts: &'a TestCommand,
}

impl<'a> LocalProviderTester<'a> {
    fn new(ctx: &'a CommandContext, path: &'a PathBuf, opts: &'a TestCommand) -> Self {
        Self { _ctx: ctx, path, opts }
    }

    async fn test_provider(&self) -> Result<TestSummary> {
        let mut summary = TestSummary::default();

        // 1. Validate provider.toml
        if !self.opts.quiet {
            println!("📋 Validating provider.toml...");
        }
        let config = self.load_provider_config()?;
        
        if !self.opts.quiet {
            println!("✓ Provider: {} ({})", config.name, config.description);
            println!("✓ Runtimes: {}", config.runtimes.len());
        }

        // 2. Test each runtime
        for runtime_config in &config.runtimes {
            if self.opts.verbose && !self.opts.quiet {
                println!("\n--- Testing Runtime: {} ---", runtime_config.name);
            }

            let result = self.test_local_runtime(runtime_config).await?;
            
            if !self.opts.quiet && !self.opts.json {
                self.print_runtime_result(&result);
            }
            
            summary.add_result(result);
        }

        Ok(summary)
    }

    fn load_provider_config(&self) -> Result<ProviderConfig> {
        let toml_path = self.path.join("provider.toml");
        let content = std::fs::read_to_string(&toml_path)
            .context("Failed to read provider.toml")?;
        
        toml::from_str(&content).context("Failed to parse provider.toml")
    }

    async fn test_local_runtime(&self, config: &RuntimeConfig) -> Result<TestResult> {
        let mut result = TestResult::new(&config.name);

        // Test platform support
        result.platform_supported = self.check_platform_support(config)?;
        
        if !result.platform_supported {
            return Ok(result);
        }

        // Test download URLs
        if let Some(ref urls) = config.download_urls {
            result.download_url_valid = self.validate_download_urls(urls)?;
        }

        // Test version detection
        result.version_detection = self.test_version_detection(config).await;

        result.passed = result.platform_supported 
            && result.download_url_valid.unwrap_or(true)
            && result.version_detection.unwrap_or(true);

        Ok(result)
    }

    fn check_platform_support(&self, config: &RuntimeConfig) -> Result<bool> {
        let current_platform = vx_runtime::Platform::current();
        
        if let Some(ref platforms) = config.platforms {
            Ok(platforms.iter().any(|p| {
                format!("{}-{}", p.os, p.arch) == format!("{}-{}", current_platform.os, current_platform.arch)
            }))
        } else {
            Ok(true) // No platform restrictions
        }
    }

    fn validate_download_urls(&self, _urls: &serde_json::Value) -> Result<Option<bool>> {
        // TODO: Validate URL templates
        Ok(Some(true))
    }

    async fn test_version_detection(&self, _config: &RuntimeConfig) -> Option<bool> {
        // TODO: Test version fetching
        None
    }

    fn print_runtime_result(&self, result: &TestResult) {
        if !result.platform_supported {
            println!("  ⚠ {} - platform not supported", result.runtime);
        } else if result.passed {
            println!("  ✓ {} - passed", result.runtime);
        } else {
            println!("  ✗ {} - failed", result.runtime);
        }
    }
}

// ============================================================================
// Extension Tester - Test a remote provider
// ============================================================================

struct ExtensionTester<'a> {
    ctx: &'a CommandContext,
    url: &'a str,
    opts: &'a TestCommand,
}

impl<'a> ExtensionTester<'a> {
    fn new(ctx: &'a CommandContext, url: &'a str, opts: &'a TestCommand) -> Self {
        Self { ctx, url, opts }
    }

    async fn test_extension(&self) -> Result<TestSummary> {
        // 1. Download extension
        if !self.opts.quiet {
            println!("📥 Downloading extension from {}...", self.url);
        }
        let local_path = self.download_extension().await?;

        // 2. Test as local provider
        let tester = LocalProviderTester::new(self.ctx, &local_path, self.opts);
        tester.test_provider().await
    }

    async fn download_extension(&self) -> Result<PathBuf> {
        // Get cache directory from context
        let cache_dir = vx_paths::PathManager::new()?
            .cache_dir()
            .join("extensions");
        std::fs::create_dir_all(&cache_dir)?;
        
        // TODO: Implement extension download
        // - Clone git repo
        // - Or download tarball
        // - Extract to cache
        anyhow::bail!("Extension download not yet implemented")
    }
}

// ============================================================================
// Data Structures
// ============================================================================

#[derive(Debug, Clone, serde::Serialize)]
struct TestResult {
    runtime: String,
    platform_supported: bool,
    vx_installed: bool,
    system_available: bool,
    available: bool,
    passed: bool,
    
    #[serde(skip_serializing_if = "Vec::is_empty")]
    installed_versions: Vec<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    system_path: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    functional_test: Option<bool>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    install_test: Option<bool>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    download_url_valid: Option<bool>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    version_detection: Option<bool>,
}

impl TestResult {
    fn new(runtime: &str) -> Self {
        Self {
            runtime: runtime.to_string(),
            platform_supported: false,
            vx_installed: false,
            system_available: false,
            available: false,
            passed: false,
            installed_versions: Vec::new(),
            system_path: None,
            functional_test: None,
            install_test: None,
            download_url_valid: None,
            version_detection: None,
        }
    }
}

#[derive(Debug, Default, serde::Serialize)]
struct TestSummary {
    total: usize,
    passed: usize,
    failed: usize,
    skipped: usize,
    errors: Vec<(String, String)>,
    results: Vec<TestResult>,
}

impl TestSummary {
    fn add_result(&mut self, result: TestResult) {
        self.total += 1;
        if !result.platform_supported {
            self.skipped += 1;
        } else if result.passed {
            self.passed += 1;
        } else {
            self.failed += 1;
        }
        self.results.push(result);
    }
}

// Temporary structures for provider.toml parsing
#[derive(Debug, serde::Deserialize)]
struct ProviderConfig {
    name: String,
    description: String,
    runtimes: Vec<RuntimeConfig>,
}

#[derive(Debug, serde::Deserialize)]
struct RuntimeConfig {
    name: String,
    platforms: Option<Vec<PlatformConfig>>,
    download_urls: Option<serde_json::Value>,
}

#[derive(Debug, serde::Deserialize)]
struct PlatformConfig {
    os: String,
    arch: String,
}

// ============================================================================
// Output Functions
// ============================================================================

fn output_single_result(result: &TestResult, opts: &TestCommand) {
    if opts.json {
        println!("{}", serde_json::to_string_pretty(result).unwrap());
    } else if !opts.quiet {
        println!();
        if result.platform_supported {
            if result.passed {
                println!("✓ Runtime '{}' - PASSED", result.runtime);
            } else {
                println!("✗ Runtime '{}' - FAILED", result.runtime);
            }
            
            if opts.detailed {
                if result.vx_installed {
                    println!("  ✓ Installed in vx store");
                    if !result.installed_versions.is_empty() {
                        println!("    Versions: {}", result.installed_versions.join(", "));
                    }
                }
                if result.system_available {
                    println!("  ✓ Available on system PATH");
                    if let Some(ref path) = result.system_path {
                        println!("    Path: {}", path);
                    }
                }
                if let Some(functional) = result.functional_test {
                    if functional {
                        println!("  ✓ Functional test passed");
                    } else {
                        println!("  ✗ Functional test failed");
                    }
                }
            }
        } else {
            println!("⚠ Runtime '{}' - Platform not supported", result.runtime);
        }
    }
}

fn output_summary(summary: &TestSummary, opts: &TestCommand) {
    if opts.json {
        println!("{}", serde_json::to_string_pretty(summary).unwrap());
    } else if !opts.quiet {
        println!();
        println!("=== Test Summary ===");
        println!("Total:   {}", summary.total);
        println!("Passed:  {}", summary.passed);
        println!("Failed:  {}", summary.failed);
        println!("Skipped: {}", summary.skipped);
        
        if opts.detailed && !summary.results.is_empty() {
            println!("\nDetailed Results:");
            for result in &summary.results {
                println!("  - {}: {}", result.runtime, if result.passed { "✓" } else { "✗" });
            }
        }
        
        if !summary.errors.is_empty() {
            println!("\nErrors:");
            for (runtime, error) in &summary.errors {
                println!("  - {}: {}", runtime, error);
            }
        }
    }
}

fn exit_with_result(result: &TestResult) -> ! {
    if result.passed || (result.platform_supported && result.available) {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

fn exit_with_summary(summary: &TestSummary) -> ! {
    if summary.failed == 0 {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
