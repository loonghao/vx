//! Test running commands
//!
//! Provides test execution functionality:
//! - Run tests based on project type
//! - Coverage reporting
//! - Watch mode

use anyhow::Result;
use std::env;
use std::path::Path;
use std::process::Command;

use crate::ui::UI;

/// Handle test run command
pub async fn handle_run(
    filter: Option<String>,
    coverage: bool,
    watch: bool,
    verbose: bool,
    parallel: bool,
) -> Result<()> {
    let current_dir = env::current_dir()?;
    let config_path = current_dir.join(".vx.toml");

    UI::header("Running Tests");

    // Load config if exists
    let config = if config_path.exists() {
        Some(vx_config::parse_config(&config_path)?)
    } else {
        None
    };

    let testing_config = config
        .as_ref()
        .and_then(|c| c.test.clone())
        .unwrap_or_default();

    // Detect project type
    let project_types = detect_project_types(&current_dir);

    if project_types.is_empty() {
        return Err(anyhow::anyhow!(
            "No recognized project type found. Supported: Rust, Node.js, Python, Go"
        ));
    }

    // Get test framework from config or auto-detect
    let framework = testing_config.framework.clone();

    for project_type in &project_types {
        match project_type.as_str() {
            "rust" => {
                run_rust_tests(
                    &current_dir,
                    &filter,
                    coverage,
                    watch,
                    verbose,
                    parallel,
                    &testing_config,
                )
                .await?;
            }
            "node" => {
                run_node_tests(
                    &current_dir,
                    &filter,
                    coverage,
                    watch,
                    verbose,
                    &framework,
                    &testing_config,
                )
                .await?;
            }
            "python" => {
                run_python_tests(
                    &current_dir,
                    &filter,
                    coverage,
                    watch,
                    verbose,
                    &framework,
                    &testing_config,
                )
                .await?;
            }
            "go" => {
                run_go_tests(
                    &current_dir,
                    &filter,
                    coverage,
                    watch,
                    verbose,
                    &testing_config,
                )
                .await?;
            }
            _ => {}
        }
    }

    Ok(())
}

/// Handle coverage report command
pub async fn handle_coverage(
    format: Option<String>,
    output: Option<String>,
    verbose: bool,
) -> Result<()> {
    let current_dir = env::current_dir()?;

    UI::header("Generating Coverage Report");

    let project_types = detect_project_types(&current_dir);

    for project_type in &project_types {
        match project_type.as_str() {
            "rust" => {
                generate_rust_coverage(&current_dir, &format, &output, verbose).await?;
            }
            "node" => {
                generate_node_coverage(&current_dir, &format, &output, verbose).await?;
            }
            "python" => {
                generate_python_coverage(&current_dir, &format, &output, verbose).await?;
            }
            "go" => {
                generate_go_coverage(&current_dir, &format, &output, verbose).await?;
            }
            _ => {}
        }
    }

    Ok(())
}

/// Handle test status command
pub async fn handle_status(verbose: bool) -> Result<()> {
    let current_dir = env::current_dir()?;
    let config_path = current_dir.join(".vx.toml");

    UI::header("Test Configuration Status");

    // Check for test configuration
    if config_path.exists() {
        let config = vx_config::parse_config(&config_path)?;
        if let Some(testing) = &config.test {
            println!("\n{}", UI::format_header("Configuration"));
            if let Some(framework) = &testing.framework {
                println!("  Framework: {}", framework);
            }
            if let Some(parallel) = testing.parallel {
                println!("  Parallel: {}", parallel);
            }
            if let Some(workers) = testing.workers {
                println!("  Workers: {}", workers);
            }
            if let Some(timeout) = testing.timeout {
                println!("  Timeout: {}s", timeout);
            }

            if let Some(cov) = &testing.coverage {
                println!("\n{}", UI::format_header("Coverage"));
                if cov.enabled.unwrap_or(false) {
                    println!("  Enabled: Yes");
                    if let Some(threshold) = cov.threshold {
                        println!("  Threshold: {}%", threshold);
                    }
                    if !cov.exclude.is_empty() {
                        println!("  Exclude: {}", cov.exclude.join(", "));
                    }
                } else {
                    println!("  Enabled: No");
                }
            }
        } else {
            UI::info("No [test] section in .vx.toml");
        }
    }

    // Detect project types and show available test frameworks
    println!("\n{}", UI::format_header("Detected Projects"));
    let project_types = detect_project_types(&current_dir);

    for project_type in &project_types {
        println!("  {}", project_type);

        if verbose {
            match project_type.as_str() {
                "rust" => {
                    println!("    Test command: cargo test");
                    if current_dir.join("Cargo.toml").exists() {
                        // Check for test dependencies
                        let content = std::fs::read_to_string(current_dir.join("Cargo.toml"))?;
                        if content.contains("[dev-dependencies]") {
                            println!("    Has dev-dependencies: Yes");
                        }
                    }
                }
                "node" => {
                    let pkg_json = current_dir.join("package.json");
                    if pkg_json.exists() {
                        let content = std::fs::read_to_string(&pkg_json)?;
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(scripts) = json["scripts"].as_object() {
                                if scripts.contains_key("test") {
                                    println!("    Test script: {}", scripts["test"]);
                                }
                            }
                            // Detect test framework
                            let deps = json["devDependencies"].as_object();
                            if let Some(deps) = deps {
                                if deps.contains_key("jest") {
                                    println!("    Framework: Jest");
                                } else if deps.contains_key("vitest") {
                                    println!("    Framework: Vitest");
                                } else if deps.contains_key("mocha") {
                                    println!("    Framework: Mocha");
                                }
                            }
                        }
                    }
                }
                "python" => {
                    if current_dir.join("pytest.ini").exists()
                        || current_dir.join("pyproject.toml").exists()
                    {
                        println!("    Framework: pytest");
                    }
                }
                "go" => {
                    println!("    Test command: go test");
                }
                _ => {}
            }
        }
    }

    Ok(())
}

// Helper functions

fn detect_project_types(dir: &Path) -> Vec<String> {
    let mut types = Vec::new();

    if dir.join("Cargo.toml").exists() {
        types.push("rust".to_string());
    }
    if dir.join("package.json").exists() {
        types.push("node".to_string());
    }
    if dir.join("requirements.txt").exists()
        || dir.join("pyproject.toml").exists()
        || dir.join("setup.py").exists()
    {
        types.push("python".to_string());
    }
    if dir.join("go.mod").exists() {
        types.push("go".to_string());
    }

    types
}

async fn run_rust_tests(
    dir: &Path,
    filter: &Option<String>,
    coverage: bool,
    watch: bool,
    verbose: bool,
    parallel: bool,
    _config: &vx_config::TestConfig,
) -> Result<()> {
    UI::step("Running Rust tests...");

    if watch {
        // Use cargo-watch
        let check = Command::new("cargo").args(["watch", "--version"]).output();

        if check.is_err() || !check.unwrap().status.success() {
            UI::warn("cargo-watch not installed. Install with: cargo install cargo-watch");
            return Ok(());
        }

        let mut cmd = Command::new("cargo");
        cmd.current_dir(dir).args(["watch", "-x", "test"]);

        if let Some(f) = filter {
            cmd.args(["--", f]);
        }

        let status = cmd.status()?;
        if !status.success() {
            return Err(anyhow::anyhow!("Tests failed"));
        }

        return Ok(());
    }

    if coverage {
        // Use cargo-tarpaulin or cargo-llvm-cov
        let tarpaulin_check = Command::new("cargo")
            .args(["tarpaulin", "--version"])
            .output();

        if tarpaulin_check.is_ok() && tarpaulin_check.unwrap().status.success() {
            let mut cmd = Command::new("cargo");
            cmd.current_dir(dir).arg("tarpaulin");

            if let Some(f) = filter {
                cmd.args(["--", f]);
            }

            let status = cmd.status()?;
            if !status.success() {
                return Err(anyhow::anyhow!("Coverage failed"));
            }

            return Ok(());
        }

        UI::warn("cargo-tarpaulin not installed. Running tests without coverage.");
    }

    // Default cargo test
    let mut cmd = Command::new("cargo");
    cmd.current_dir(dir).arg("test");

    if verbose {
        cmd.arg("--verbose");
    }

    if !parallel {
        cmd.args(["--", "--test-threads=1"]);
    }

    if let Some(f) = filter {
        cmd.arg(f);
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Tests failed"));
    }

    UI::success("Rust tests passed");
    Ok(())
}

async fn run_node_tests(
    dir: &Path,
    filter: &Option<String>,
    coverage: bool,
    watch: bool,
    verbose: bool,
    framework: &Option<String>,
    _config: &vx_config::TestConfig,
) -> Result<()> {
    UI::step("Running Node.js tests...");

    // Detect framework
    let detected_framework = framework.clone().unwrap_or_else(|| {
        let pkg_json = dir.join("package.json");
        if pkg_json.exists() {
            if let Ok(content) = std::fs::read_to_string(&pkg_json) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    let deps = json["devDependencies"].as_object();
                    if let Some(deps) = deps {
                        if deps.contains_key("vitest") {
                            return "vitest".to_string();
                        } else if deps.contains_key("jest") {
                            return "jest".to_string();
                        } else if deps.contains_key("mocha") {
                            return "mocha".to_string();
                        }
                    }
                }
            }
        }
        "npm".to_string() // Fall back to npm test
    });

    let mut cmd = Command::new("npx");
    cmd.current_dir(dir);

    match detected_framework.as_str() {
        "vitest" => {
            cmd.arg("vitest");
            if !watch {
                cmd.arg("run");
            }
            if coverage {
                cmd.arg("--coverage");
            }
            if let Some(f) = filter {
                cmd.args(["--filter", f]);
            }
        }
        "jest" => {
            cmd.arg("jest");
            if watch {
                cmd.arg("--watch");
            }
            if coverage {
                cmd.arg("--coverage");
            }
            if let Some(f) = filter {
                cmd.arg(f);
            }
        }
        "mocha" => {
            cmd.arg("mocha");
            if watch {
                cmd.arg("--watch");
            }
            if let Some(f) = filter {
                cmd.args(["--grep", f]);
            }
        }
        _ => {
            // Use npm test
            cmd = Command::new("npm");
            cmd.current_dir(dir).arg("test");
        }
    }

    if verbose {
        cmd.arg("--verbose");
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Tests failed"));
    }

    UI::success("Node.js tests passed");
    Ok(())
}

async fn run_python_tests(
    dir: &Path,
    filter: &Option<String>,
    coverage: bool,
    watch: bool,
    verbose: bool,
    framework: &Option<String>,
    _config: &vx_config::TestConfig,
) -> Result<()> {
    UI::step("Running Python tests...");

    let detected_framework = framework.clone().unwrap_or_else(|| "pytest".to_string());

    match detected_framework.as_str() {
        "pytest" => {
            let mut cmd = Command::new("pytest");
            cmd.current_dir(dir);

            if coverage {
                cmd.args(["--cov", "."]);
            }

            if verbose {
                cmd.arg("-v");
            }

            if let Some(f) = filter {
                cmd.args(["-k", f]);
            }

            if watch {
                // Use pytest-watch
                cmd = Command::new("ptw");
                cmd.current_dir(dir);
            }

            let status = cmd.status()?;
            if !status.success() {
                return Err(anyhow::anyhow!("Tests failed"));
            }
        }
        "unittest" => {
            let mut cmd = Command::new("python");
            cmd.current_dir(dir).args(["-m", "unittest"]);

            if verbose {
                cmd.arg("-v");
            }

            if let Some(f) = filter {
                cmd.arg(f);
            }

            let status = cmd.status()?;
            if !status.success() {
                return Err(anyhow::anyhow!("Tests failed"));
            }
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown Python test framework: {}",
                detected_framework
            ));
        }
    }

    UI::success("Python tests passed");
    Ok(())
}

async fn run_go_tests(
    dir: &Path,
    filter: &Option<String>,
    coverage: bool,
    _watch: bool,
    verbose: bool,
    _config: &vx_config::TestConfig,
) -> Result<()> {
    UI::step("Running Go tests...");

    let mut cmd = Command::new("go");
    cmd.current_dir(dir).arg("test");

    if coverage {
        cmd.arg("-cover");
    }

    if verbose {
        cmd.arg("-v");
    }

    if let Some(f) = filter {
        cmd.args(["-run", f]);
    }

    cmd.arg("./...");

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Tests failed"));
    }

    UI::success("Go tests passed");
    Ok(())
}

async fn generate_rust_coverage(
    dir: &Path,
    format: &Option<String>,
    output: &Option<String>,
    _verbose: bool,
) -> Result<()> {
    UI::step("Generating Rust coverage report...");

    let mut cmd = Command::new("cargo");
    cmd.current_dir(dir).arg("tarpaulin");

    match format.as_deref() {
        Some("html") => cmd.args(["--out", "Html"]),
        Some("xml") | Some("cobertura") => cmd.args(["--out", "Xml"]),
        Some("lcov") => cmd.args(["--out", "Lcov"]),
        _ => cmd.args(["--out", "Html"]),
    };

    if let Some(out) = output {
        cmd.args(["--output-dir", out]);
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Coverage generation failed"));
    }

    UI::success("Coverage report generated");
    Ok(())
}

async fn generate_node_coverage(
    dir: &Path,
    format: &Option<String>,
    _output: &Option<String>,
    _verbose: bool,
) -> Result<()> {
    UI::step("Generating Node.js coverage report...");

    // Detect test framework
    let pkg_json = dir.join("package.json");
    let framework = if pkg_json.exists() {
        let content = std::fs::read_to_string(&pkg_json)?;
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            let deps = json["devDependencies"].as_object();
            if let Some(deps) = deps {
                if deps.contains_key("vitest") {
                    "vitest"
                } else if deps.contains_key("jest") {
                    "jest"
                } else {
                    "nyc"
                }
            } else {
                "nyc"
            }
        } else {
            "nyc"
        }
    } else {
        "nyc"
    };

    let mut cmd = Command::new("npx");
    cmd.current_dir(dir);

    match framework {
        "vitest" => {
            cmd.args(["vitest", "run", "--coverage"]);
        }
        "jest" => {
            cmd.args(["jest", "--coverage"]);
            match format.as_deref() {
                Some("html") => cmd.args(["--coverageReporters", "html"]),
                Some("lcov") => cmd.args(["--coverageReporters", "lcov"]),
                _ => &mut cmd,
            };
        }
        _ => {
            cmd.args(["nyc", "npm", "test"]);
        }
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Coverage generation failed"));
    }

    UI::success("Coverage report generated");
    Ok(())
}

async fn generate_python_coverage(
    dir: &Path,
    format: &Option<String>,
    output: &Option<String>,
    _verbose: bool,
) -> Result<()> {
    UI::step("Generating Python coverage report...");

    // Run tests with coverage
    let mut cmd = Command::new("pytest");
    cmd.current_dir(dir).args(["--cov", ".", "--cov-report"]);

    match format.as_deref() {
        Some("html") => cmd.arg("html"),
        Some("xml") => cmd.arg("xml"),
        _ => cmd.arg("html"),
    };

    if let Some(out) = output {
        cmd.args(["--cov-report", &format!("html:{}", out)]);
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Coverage generation failed"));
    }

    UI::success("Coverage report generated");
    Ok(())
}

async fn generate_go_coverage(
    dir: &Path,
    format: &Option<String>,
    output: &Option<String>,
    _verbose: bool,
) -> Result<()> {
    UI::step("Generating Go coverage report...");

    // Run tests with coverage
    let coverage_file = output.clone().unwrap_or_else(|| "coverage.out".to_string());

    let mut cmd = Command::new("go");
    cmd.current_dir(dir)
        .args(["test", "-coverprofile", &coverage_file, "./..."]);

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Coverage generation failed"));
    }

    // Generate HTML report if requested
    if format.as_deref() == Some("html") {
        let html_file = coverage_file.replace(".out", ".html");
        let mut cmd = Command::new("go");
        cmd.current_dir(dir)
            .args(["tool", "cover", "-html", &coverage_file, "-o", &html_file]);

        cmd.status()?;
        UI::info(&format!("HTML report: {}", html_file));
    }

    UI::success("Coverage report generated");
    Ok(())
}
