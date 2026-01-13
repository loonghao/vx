//! Test command handler
//!
//! Universal Provider Testing Framework (RFC 0020)

use super::Args;
use crate::commands::CommandContext;
use anyhow::{Context, Result};
use std::path::PathBuf;
use vx_runtime::{RuntimeTestResult, RuntimeTester, TestCaseResult};

/// Handle test command with Args
pub async fn handle(ctx: &CommandContext, args: &Args) -> Result<()> {
    // Determine test mode
    if args.all {
        handle_test_all_providers(ctx, args).await
    } else if let Some(ref url) = args.extension {
        handle_test_extension(ctx, url, args).await
    } else if let Some(ref path) = args.local {
        handle_test_local_provider(ctx, path, args).await
    } else if let Some(ref runtime) = args.runtime {
        handle_test_runtime(ctx, runtime, args).await
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

// ============================================================================
// Test Handlers
// ============================================================================

/// Test a single runtime using manifest-based configuration (RFC 0020)
async fn handle_test_runtime(ctx: &CommandContext, runtime_name: &str, opts: &Args) -> Result<()> {
    // Find runtime in registry
    let runtime = ctx
        .registry()
        .get_runtime(runtime_name)
        .ok_or_else(|| anyhow::anyhow!("Unknown runtime: {}", runtime_name))?;

    // Check platform support
    let current_platform = vx_runtime::Platform::current();
    if !runtime.is_platform_supported(&current_platform) {
        let result = TestResult::new(runtime_name).platform_not_supported();
        output_single_result(&result, opts);
        exit_with_result(&result);
    }

    if opts.platform_only {
        let result = TestResult::new(runtime_name).platform_supported();
        output_single_result(&result, opts);
        exit_with_result(&result);
    }

    // Handle --install mode: install the runtime and verify executable exists
    if opts.install {
        return handle_install_test(ctx, runtime_name, opts).await;
    }

    // Get test configuration from manifest
    let test_config = ctx
        .get_runtime_manifest(runtime_name)
        .and_then(|def| def.test.clone());

    // Get executable path
    let executable_path = get_installed_executable(ctx, runtime_name);

    // Create and configure tester
    let mut tester = RuntimeTester::new(runtime_name);

    if let Some(path) = executable_path {
        tester = tester.with_executable(path);
    }

    if let Some(config) = test_config {
        tester = tester.with_config(config);
    }

    // Run tests
    let manifest_result = tester.run_all();

    // Convert to our result format
    let result = TestResult::from_manifest_result(runtime_name, manifest_result, opts);

    output_single_result(&result, opts);
    exit_with_result(&result);
}

/// Handle --install test mode: install the runtime and verify executable exists
async fn handle_install_test(
    ctx: &CommandContext,
    runtime_name: &str,
    opts: &Args,
) -> Result<()> {
    use std::time::Instant;

    let start = Instant::now();

    // Try to install the runtime
    let install_result = install_runtime_for_test(ctx, runtime_name).await;

    let duration = start.elapsed();

    let mut result = TestResult::new(runtime_name);
    result.install_test = Some(install_result.is_ok());

    match install_result {
        Ok(exe_path) => {
            result.passed = true;
            result.vx_installed = true;
            result.available = true;

            if opts.verbose && !opts.quiet {
                println!(
                    "  ✓ Installed {} in {:.2}s",
                    runtime_name,
                    duration.as_secs_f64()
                );
                println!("    Executable: {}", exe_path.display());
            }
        }
        Err(e) => {
            result.passed = false;
            result.vx_installed = false;

            if !opts.quiet {
                eprintln!("  ✗ Failed to install {}: {}", runtime_name, e);
            }
        }
    }

    output_single_result(&result, opts);
    exit_with_result(&result);
}

/// Install a runtime for testing and return the executable path
async fn install_runtime_for_test(
    ctx: &CommandContext,
    runtime_name: &str,
) -> Result<std::path::PathBuf> {
    // Use the install handler to install the runtime
    crate::commands::install::handle_install(
        ctx.registry(),
        ctx.runtime_context(),
        &[runtime_name.to_string()],
        false, // don't force reinstall
    )
    .await
    .context("Failed to install runtime")?;

    // Get the installed executable path
    let path_manager = vx_paths::PathManager::new()?;
    let runtime = ctx
        .registry()
        .get_runtime(runtime_name)
        .ok_or_else(|| anyhow::anyhow!("Runtime not found: {}", runtime_name))?;

    // Find the installed version
    let versions = path_manager.list_store_versions(runtime_name)?;
    let version = versions
        .first()
        .ok_or_else(|| anyhow::anyhow!("No version installed for {}", runtime_name))?;

    let platform = vx_runtime::Platform::current();
    let store_dir = path_manager.version_store_dir(runtime_name, version);
    let exe_relative = runtime.executable_relative_path(version, &platform);
    let exe_path = store_dir.join(&exe_relative);

    if exe_path.exists() {
        Ok(exe_path)
    } else {
        anyhow::bail!(
            "Executable not found after installation: {}",
            exe_path.display()
        )
    }
}

/// Test all registered providers
async fn handle_test_all_providers(ctx: &CommandContext, opts: &Args) -> Result<()> {
    let mut summary = TestSummary::default();
    let registry = ctx.registry();
    let all_providers = registry.providers();

    for provider in all_providers {
        if opts.verbose && !opts.quiet {
            println!("\n=== Testing Provider: {} ===", provider.name());
        }

        for runtime in provider.runtimes() {
            let runtime_name = runtime.name();

            // Check platform support first
            let current_platform = vx_runtime::Platform::current();
            if !runtime.is_platform_supported(&current_platform) {
                let result = TestResult::new(runtime_name).platform_not_supported();
                if !opts.quiet && !opts.json {
                    print_result_line(&result);
                }
                summary.add_result(result);
                continue;
            }

            if opts.platform_only {
                let result = TestResult::new(runtime_name).platform_supported();
                if !opts.quiet && !opts.json {
                    print_result_line(&result);
                }
                summary.add_result(result);
                continue;
            }

            // Get test configuration from manifest
            let test_config = ctx
                .get_runtime_manifest(runtime_name)
                .and_then(|def| def.test.clone());

            // Get executable path
            let executable_path = get_installed_executable(ctx, runtime_name);

            // Create and configure tester
            let mut tester = RuntimeTester::new(runtime_name);

            if let Some(path) = executable_path {
                tester = tester.with_executable(path);
            }

            if let Some(config) = test_config {
                tester = tester.with_config(config);
            }

            // Run tests
            let manifest_result = tester.run_all();
            let result = TestResult::from_manifest_result(runtime_name, manifest_result, opts);

            if !opts.quiet && !opts.json {
                print_result_line(&result);
            }
            summary.add_result(result);
        }
    }

    output_summary(&summary, opts);
    exit_with_summary(&summary);
}

/// Test a local provider (development mode)
async fn handle_test_local_provider(
    _ctx: &CommandContext,
    path: &PathBuf,
    opts: &Args,
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

    let content =
        std::fs::read_to_string(&provider_toml).context("Failed to read provider.toml")?;
    let manifest: vx_manifest::ProviderManifest =
        toml::from_str(&content).context("Failed to parse provider.toml")?;

    if !opts.quiet {
        println!(
            "✓ Provider: {} ({})",
            manifest.provider.name,
            manifest.provider.description.as_deref().unwrap_or("")
        );
        println!("✓ Runtimes: {}", manifest.runtimes.len());
    }

    let mut summary = TestSummary::default();

    for runtime_def in &manifest.runtimes {
        let runtime_name = &runtime_def.name;

        if opts.verbose && !opts.quiet {
            println!("\n--- Testing Runtime: {} ---", runtime_name);
        }

        // Create tester with manifest config
        let mut tester = RuntimeTester::new(runtime_name);

        if let Some(ref test_config) = runtime_def.test {
            tester = tester.with_config(test_config.clone());
        }

        // Check if executable exists on system PATH for local testing
        if let Ok(exe_path) = which::which(runtime_name) {
            tester = tester.with_executable(exe_path);
        }

        let manifest_result = tester.run_all();
        let result = TestResult::from_manifest_result(runtime_name, manifest_result, opts);

        if !opts.quiet && !opts.json {
            print_result_line(&result);
        }
        summary.add_result(result);
    }

    output_summary(&summary, opts);
    exit_with_summary(&summary);
}

/// Test a remote provider extension
async fn handle_test_extension(_ctx: &CommandContext, url: &str, opts: &Args) -> Result<()> {
    if !opts.quiet {
        println!("🌐 Testing extension from: {}", url);
    }

    // TODO: Implement extension download and testing
    anyhow::bail!("Extension testing not yet implemented. URL: {}", url)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get the path to an installed executable
fn get_installed_executable(ctx: &CommandContext, runtime_name: &str) -> Option<PathBuf> {
    let path_manager = vx_paths::PathManager::new().ok()?;
    let versions = path_manager.list_store_versions(runtime_name).ok()?;

    if let Some(version) = versions.first() {
        let runtime = ctx.registry().get_runtime(runtime_name)?;
        let platform = vx_runtime::Platform::current();
        let store_dir = path_manager.version_store_dir(runtime_name, version);
        let exe_relative = runtime.executable_relative_path(version, &platform);
        let exe_path = store_dir.join(&exe_relative);

        if exe_path.exists() {
            return Some(exe_path);
        }
    }

    // Check for bundled runtimes (e.g., ffprobe bundled with ffmpeg)
    if let Some(runtime) = ctx.registry().get_runtime(runtime_name) {
        if let Some(parent_tool) = runtime.metadata().get("bundled_with") {
            let parent_versions = path_manager.list_store_versions(parent_tool).ok()?;
            let platform = vx_runtime::Platform::current();

            for version in &parent_versions {
                let parent_path = path_manager.version_store_dir(parent_tool, version);
                let exe_relative = runtime.executable_relative_path(version, &platform);
                let exe_path = parent_path.join(&exe_relative);
                if exe_path.exists() {
                    return Some(exe_path);
                }
            }
        }
    }

    None
}

fn print_result_line(result: &TestResult) {
    if !result.platform_supported {
        println!("  ⚠ {} - platform not supported", result.runtime);
    } else if result.passed {
        println!("  ✓ {} - passed", result.runtime);
    } else {
        println!("  ✗ {} - failed", result.runtime);
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

    /// Individual test case results (RFC 0020)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    test_cases: Vec<TestCaseResult>,
}

impl TestResult {
    fn new(runtime: &str) -> Self {
        Self {
            runtime: runtime.to_string(),
            platform_supported: true,
            vx_installed: false,
            system_available: false,
            available: false,
            passed: false,
            installed_versions: Vec::new(),
            system_path: None,
            functional_test: None,
            install_test: None,
            test_cases: Vec::new(),
        }
    }

    fn platform_not_supported(mut self) -> Self {
        self.platform_supported = false;
        self.passed = false;
        self
    }

    fn platform_supported(mut self) -> Self {
        self.platform_supported = true;
        self.passed = true;
        self
    }

    fn from_manifest_result(runtime_name: &str, result: RuntimeTestResult, opts: &Args) -> Self {
        let path_manager = vx_paths::PathManager::new().ok();
        let installed_versions = path_manager
            .as_ref()
            .and_then(|pm| pm.list_store_versions(runtime_name).ok())
            .unwrap_or_default();

        let system_path = which::which(runtime_name)
            .ok()
            .map(|p| p.display().to_string());

        let vx_installed = result.installed || !installed_versions.is_empty();
        let system_available = result.system_available || system_path.is_some();
        let available = vx_installed || system_available;

        // Determine if functional test passed
        let functional_test = if opts.functional || !result.test_cases.is_empty() {
            Some(result.test_cases.iter().all(|t| t.passed))
        } else {
            None
        };

        // Determine pass criteria based on test mode:
        // - Default (no flags): platform supported is enough (configuration check)
        // - --functional: requires available AND functional tests pass
        // - --install: requires installation success (handled elsewhere)
        // - --installed: requires vx_installed
        // - --system: requires system_available
        let passed = if opts.functional {
            // Functional mode: must be available and tests must pass
            result.platform_supported
                && available
                && functional_test.unwrap_or(true)
                && result.error.is_none()
        } else if opts.installed {
            // Installed check: must be installed in vx
            result.platform_supported && vx_installed
        } else if opts.system {
            // System check: must be on system PATH
            result.platform_supported && system_available
        } else {
            // Default mode: just check platform support and no errors
            // This is a "configuration check" - verifies the provider is correctly configured
            result.platform_supported && result.error.is_none()
        };

        Self {
            runtime: runtime_name.to_string(),
            platform_supported: result.platform_supported,
            vx_installed,
            system_available,
            available,
            passed,
            installed_versions,
            system_path,
            functional_test,
            install_test: None,
            test_cases: result.test_cases,
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

// ============================================================================
// Output Functions
// ============================================================================

fn output_single_result(result: &TestResult, opts: &Args) {
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

                // Show individual test case results (RFC 0020)
                if !result.test_cases.is_empty() {
                    println!("\n  Test Cases:");
                    for tc in &result.test_cases {
                        let status = if tc.passed { "✓" } else { "✗" };
                        let duration = format!("{:.2}ms", tc.duration.as_secs_f64() * 1000.0);
                        println!("    {} {} ({})", status, tc.name, duration);

                        if opts.verbose {
                            if let Some(ref stdout) = tc.stdout {
                                if !stdout.trim().is_empty() {
                                    println!(
                                        "      stdout: {}",
                                        stdout.trim().lines().next().unwrap_or("")
                                    );
                                }
                            }
                            if let Some(ref error) = tc.error {
                                println!("      error: {}", error);
                            }
                        }
                    }
                }
            }
        } else {
            println!("⚠ Runtime '{}' - Platform not supported", result.runtime);
        }
    }
}

fn output_summary(summary: &TestSummary, opts: &Args) {
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
                println!(
                    "  - {}: {}",
                    result.runtime,
                    if result.passed { "✓" } else { "✗" }
                );
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
