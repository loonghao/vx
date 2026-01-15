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
    if args.ci {
        handle_ci_test(ctx, args).await
    } else if args.all {
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
             - --all: test all providers (config check)\n  \
             - --ci: full CI test (install + functional)\n  \
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
async fn handle_install_test(ctx: &CommandContext, runtime_name: &str, opts: &Args) -> Result<()> {
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

// ============================================================================
// CI Test Mode - Full End-to-End Testing
// ============================================================================

/// CI test result for a single runtime
#[derive(Debug, Clone, serde::Serialize)]
struct CITestResult {
    runtime: String,
    platform_supported: bool,
    install_success: bool,
    install_duration_secs: f64,
    functional_success: bool,
    functional_tests: Vec<TestCaseResult>,
    overall_passed: bool,
    error: Option<String>,
    version_installed: Option<String>,
}

/// CI test summary
#[derive(Debug, Default, serde::Serialize)]
struct CITestSummary {
    total: usize,
    passed: usize,
    failed: usize,
    skipped: usize,
    total_duration_secs: f64,
    results: Vec<CITestResult>,
}

/// Handle --ci mode: Full end-to-end testing
///
/// This mode:
/// 1. Installs each runtime from network
/// 2. Runs functional tests (--version, etc.)
/// 3. Reports detailed results
async fn handle_ci_test(ctx: &CommandContext, opts: &Args) -> Result<()> {
    use std::time::Instant;

    let total_start = Instant::now();
    let mut summary = CITestSummary::default();

    // Setup VX root for testing
    let (test_root, _temp_dir) = setup_test_root(opts)?;

    // Create a custom runtime context if using custom root
    let custom_context = test_root
        .as_ref()
        .map(vx_runtime::create_runtime_context_with_base);

    let runtime_context = custom_context
        .as_ref()
        .unwrap_or_else(|| ctx.runtime_context());

    // Create custom path manager for the test root
    let path_manager = if let Some(ref root) = test_root {
        vx_paths::PathManager::with_base_dir(root)?
    } else {
        vx_paths::PathManager::new()?
    };

    // Determine which runtimes to test
    let runtimes_to_test = get_ci_test_runtimes(ctx, opts);

    if !opts.quiet && !opts.json {
        println!("🚀 VX CI Test - Full End-to-End Testing");
        println!("=========================================");
        if let Some(ref root) = test_root {
            println!("VX Root: {}", root.display());
        }
        println!("Runtimes to test: {}", runtimes_to_test.len());
        if opts.verbose {
            println!("  {}", runtimes_to_test.join(", "));
        }
        println!();
    }

    for runtime_name in &runtimes_to_test {
        let result =
            run_ci_test_for_runtime(ctx, runtime_context, &path_manager, runtime_name, opts).await;

        // Update summary
        summary.total += 1;
        if !result.platform_supported {
            summary.skipped += 1;
        } else if result.overall_passed {
            summary.passed += 1;
        } else {
            summary.failed += 1;
        }

        // Print result line
        if !opts.quiet && !opts.json {
            print_ci_result_line(&result, opts);
        }

        summary.results.push(result);

        // Early exit if not keep-going and we have a failure
        if !opts.keep_going && summary.failed > 0 {
            break;
        }
    }

    summary.total_duration_secs = total_start.elapsed().as_secs_f64();

    // Output summary
    output_ci_summary(&summary, opts);

    // Exit with appropriate code
    if summary.failed == 0 {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

/// Setup VX root for testing
/// Returns (custom_root, temp_dir_guard)
fn setup_test_root(opts: &Args) -> Result<(Option<PathBuf>, Option<tempfile::TempDir>)> {
    if let Some(ref root) = opts.vx_root {
        // Use specified root
        std::fs::create_dir_all(root)?;
        Ok((Some(root.clone()), None))
    } else if opts.temp_root {
        // Create temporary directory
        let temp_dir = tempfile::Builder::new().prefix("vx-ci-test-").tempdir()?;
        let root = temp_dir.path().to_path_buf();
        Ok((Some(root), Some(temp_dir)))
    } else {
        // Use default VX root
        Ok((None, None))
    }
}

/// Get list of runtimes to test in CI mode
fn get_ci_test_runtimes(ctx: &CommandContext, opts: &Args) -> Vec<String> {
    // If specific runtimes are specified, use those
    if let Some(ref runtimes) = opts.ci_runtimes {
        return runtimes.clone();
    }

    // Otherwise, get all runtimes from registry
    let registry = ctx.registry();
    let skip_list: Vec<String> = opts.ci_skip.clone().unwrap_or_default();

    let mut runtimes = Vec::new();
    for provider in registry.providers() {
        for runtime in provider.runtimes() {
            let name = runtime.name().to_string();

            // Skip if in skip list
            if skip_list.contains(&name) {
                continue;
            }

            // Skip bundled runtimes by default in CI mode
            // They will be tested as part of their parent runtime
            // e.g., npm/npx are tested when node is tested
            if runtime.metadata().contains_key("bundled_with") {
                continue;
            }

            runtimes.push(name);
        }
    }

    runtimes
}

/// Run full CI test for a single runtime
async fn run_ci_test_for_runtime(
    ctx: &CommandContext,
    runtime_context: &vx_runtime::RuntimeContext,
    path_manager: &vx_paths::PathManager,
    runtime_name: &str,
    opts: &Args,
) -> CITestResult {
    use std::time::Instant;

    let mut result = CITestResult {
        runtime: runtime_name.to_string(),
        platform_supported: true,
        install_success: false,
        install_duration_secs: 0.0,
        functional_success: false,
        functional_tests: Vec::new(),
        overall_passed: false,
        error: None,
        version_installed: None,
    };

    // Check platform support
    let runtime = match ctx.registry().get_runtime(runtime_name) {
        Some(r) => r,
        None => {
            result.error = Some(format!("Unknown runtime: {}", runtime_name));
            return result;
        }
    };

    let current_platform = vx_runtime::Platform::current();
    if !runtime.is_platform_supported(&current_platform) {
        result.platform_supported = false;
        return result;
    }

    // Step 1: Install runtime (quietly for JSON mode)
    let install_start = Instant::now();

    if opts.verbose && !opts.quiet && !opts.json {
        println!("  📦 Installing {}...", runtime_name);
    }

    let install_result = tokio::time::timeout(
        std::time::Duration::from_secs(opts.timeout),
        crate::commands::install::install_quiet(ctx.registry(), runtime_context, runtime_name),
    )
    .await;

    result.install_duration_secs = install_start.elapsed().as_secs_f64();

    let version = match install_result {
        Ok(Ok(v)) => {
            result.install_success = true;
            result.version_installed = Some(v.clone());
            v
        }
        Ok(Err(e)) => {
            result.error = Some(format!("Install failed: {}", e));
            return result;
        }
        Err(_) => {
            result.error = Some(format!("Install timed out after {}s", opts.timeout));
            return result;
        }
    };

    // Get executable path using the provided path manager
    // Handle different installation methods: binary, npm, pip
    let exe_path = get_executable_path_for_runtime(
        &runtime,
        runtime_name,
        &version,
        path_manager,
        &current_platform,
    );

    if !exe_path.exists() {
        result.error = Some(format!("Executable not found: {}", exe_path.display()));
        return result;
    }

    // Step 2: Run functional tests
    if opts.verbose && !opts.quiet && !opts.json {
        println!("  🧪 Running functional tests...");
    }

    let test_config = ctx
        .get_runtime_manifest(runtime_name)
        .and_then(|def| def.test.clone());

    let mut tester = RuntimeTester::new(runtime_name).with_executable(exe_path);

    if let Some(config) = test_config {
        tester = tester.with_config(config);
    }

    let test_result = tester.run_all();
    result.functional_tests = test_result.test_cases;
    result.functional_success = result.functional_tests.iter().all(|t| t.passed);

    // Overall pass: install success AND functional tests pass
    result.overall_passed = result.install_success && result.functional_success;

    result
}

fn print_ci_result_line(result: &CITestResult, opts: &Args) {
    if !result.platform_supported {
        println!("  ⚠ {} - platform not supported", result.runtime);
        return;
    }

    let status = if result.overall_passed { "✓" } else { "✗" };
    let install_status = if result.install_success { "✓" } else { "✗" };
    let func_status = if result.functional_success {
        "✓"
    } else {
        "✗"
    };

    if result.overall_passed {
        println!(
            "  {} {} - passed (install: {:.1}s, tests: {})",
            status,
            result.runtime,
            result.install_duration_secs,
            result.functional_tests.len()
        );
    } else {
        println!("  {} {} - failed", status, result.runtime);
        println!(
            "      Install: {} | Functional: {}",
            install_status, func_status
        );
        if let Some(ref error) = result.error {
            println!("      Error: {}", error);
        }
    }

    if opts.verbose && !result.functional_tests.is_empty() {
        for tc in &result.functional_tests {
            let tc_status = if tc.passed { "✓" } else { "✗" };
            println!(
                "      {} {} ({:.2}ms)",
                tc_status,
                tc.name,
                tc.duration.as_secs_f64() * 1000.0
            );
            if !tc.passed {
                if let Some(ref error) = tc.error {
                    println!("        Error: {}", error);
                }
            }
        }
    }
}

fn output_ci_summary(summary: &CITestSummary, opts: &Args) {
    if opts.json {
        println!("{}", serde_json::to_string_pretty(summary).unwrap());
        return;
    }

    if opts.quiet {
        return;
    }

    println!();
    println!("=========================================");
    println!("🏁 CI Test Summary");
    println!("=========================================");
    println!("Total:    {}", summary.total);
    println!("Passed:   {} ✓", summary.passed);
    println!("Failed:   {} ✗", summary.failed);
    println!("Skipped:  {} ⚠", summary.skipped);
    println!("Duration: {:.1}s", summary.total_duration_secs);

    if summary.failed > 0 {
        println!();
        println!("Failed runtimes:");
        for result in &summary.results {
            if result.platform_supported && !result.overall_passed {
                println!("  - {}", result.runtime);
                if let Some(ref error) = result.error {
                    println!("    Error: {}", error);
                }
            }
        }
    }

    if opts.detailed {
        println!();
        println!("Detailed Results:");
        for result in &summary.results {
            if !result.platform_supported {
                println!("  ⚠ {} - skipped (platform not supported)", result.runtime);
            } else if result.overall_passed {
                println!(
                    "  ✓ {} v{} ({:.1}s)",
                    result.runtime,
                    result.version_installed.as_deref().unwrap_or("?"),
                    result.install_duration_secs
                );
            } else {
                println!("  ✗ {} - failed", result.runtime);
            }
        }
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
    path: &std::path::Path,
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
    let runtime = ctx.registry().get_runtime(runtime_name)?;
    let platform = vx_runtime::Platform::current();
    let metadata = runtime.metadata();
    let install_method = metadata.get("install_method").map(|s| s.as_str());

    match install_method {
        Some("npm") => {
            // npm packages are installed in npm-tools directory
            let package_name = metadata
                .get("npm_package")
                .map(|s| s.as_str())
                .unwrap_or(runtime_name);
            let versions = path_manager.list_npm_tool_versions(package_name).ok()?;
            if let Some(version) = versions.first() {
                let bin_dir = path_manager.npm_tool_bin_dir(package_name, version);
                let exe_name = if cfg!(windows) {
                    format!("{}.cmd", runtime_name)
                } else {
                    runtime_name.to_string()
                };
                let exe_path = bin_dir.join(exe_name);
                if exe_path.exists() {
                    return Some(exe_path);
                }
            }
        }
        Some("pip") => {
            // pip packages are installed in pip-tools directory with venv
            let package_name = metadata
                .get("pip_package")
                .map(|s| s.as_str())
                .unwrap_or(runtime_name);
            let versions = path_manager.list_pip_tool_versions(package_name).ok()?;
            if let Some(version) = versions.first() {
                let bin_dir = path_manager.pip_tool_bin_dir(package_name, version);
                let exe_name = if cfg!(windows) {
                    format!("{}.exe", runtime_name)
                } else {
                    runtime_name.to_string()
                };
                let exe_path = bin_dir.join(exe_name);
                if exe_path.exists() {
                    return Some(exe_path);
                }
            }
        }
        _ => {
            // Binary installation - check store directory
            // IMPORTANT: Use runtime.store_name() which handles aliases and bundled runtimes
            let store_name = runtime.store_name();
            let versions = path_manager.list_store_versions(store_name).ok()?;
            if let Some(version) = versions.first() {
                let store_dir = path_manager.version_store_dir(store_name, version);
                let exe_relative = runtime.executable_relative_path(version, &platform);
                let exe_path = store_dir.join(&exe_relative);
                if exe_path.exists() {
                    return Some(exe_path);
                }
            }
        }
    }

    None
}

/// Get executable path for a runtime, handling different installation methods
fn get_executable_path_for_runtime(
    runtime: &std::sync::Arc<dyn vx_runtime::Runtime>,
    runtime_name: &str,
    version: &str,
    path_manager: &vx_paths::PathManager,
    platform: &vx_runtime::Platform,
) -> std::path::PathBuf {
    let metadata = runtime.metadata();
    let install_method = metadata.get("install_method").map(|s| s.as_str());

    match install_method {
        Some("npm") => {
            // npm packages are installed in npm-tools directory
            let package_name = metadata
                .get("npm_package")
                .map(|s| s.as_str())
                .unwrap_or(runtime_name);
            let bin_dir = path_manager.npm_tool_bin_dir(package_name, version);
            let exe_name = if cfg!(windows) {
                format!("{}.cmd", runtime_name)
            } else {
                runtime_name.to_string()
            };
            bin_dir.join(exe_name)
        }
        Some("pip") => {
            // pip packages are installed in pip-tools directory with venv
            let package_name = metadata
                .get("pip_package")
                .map(|s| s.as_str())
                .unwrap_or(runtime_name);
            let bin_dir = path_manager.pip_tool_bin_dir(package_name, version);
            let exe_name = if cfg!(windows) {
                format!("{}.exe", runtime_name)
            } else {
                runtime_name.to_string()
            };
            bin_dir.join(exe_name)
        }
        _ => {
            // Binary installation - use store directory
            // IMPORTANT: Use runtime.store_name() which handles:
            // 1. Aliases (e.g., "vscode" -> "code")
            // 2. Bundled runtimes (e.g., "npm" -> "node", "uvx" -> "uv")
            let store_name = runtime.store_name();
            let store_dir = path_manager.version_store_dir(store_name, version);

            // Use verify_installation to find the actual executable path
            // This handles complex layouts like VSCode's platform-specific directories
            let verification = runtime.verify_installation(version, &store_dir, platform);
            if verification.valid {
                if let Some(exe_path) = verification.executable_path {
                    return exe_path;
                }
            }

            // Fallback to the expected path from executable_relative_path
            let exe_relative = runtime.executable_relative_path(version, platform);
            store_dir.join(&exe_relative)
        }
    }
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
