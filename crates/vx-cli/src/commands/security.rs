//! Security scanning commands
//!
//! Provides security-related functionality:
//! - Dependency vulnerability scanning
//! - Secret detection
//! - License compliance checking

use anyhow::Result;
use std::env;
use std::path::Path;
use std::process::Command;

use crate::ui::UI;

/// Handle security scan command
pub async fn handle_scan(verbose: bool, fix: bool, format: Option<String>) -> Result<()> {
    let current_dir = env::current_dir()?;
    let config_path = current_dir.join(".vx.toml");

    // Load config if exists
    let config = if config_path.exists() {
        Some(vx_config::parse_config(&config_path)?)
    } else {
        None
    };

    let security_config = config
        .as_ref()
        .and_then(|c| c.security.clone())
        .unwrap_or_default();

    if !security_config.enabled.unwrap_or(true) {
        UI::warn("Security scanning is disabled in configuration");
        return Ok(());
    }

    UI::header("Security Scan");

    let mut total_issues = 0;
    let mut critical_issues = 0;

    // Detect project type and run appropriate scanners
    let project_types = detect_project_types(&current_dir);

    if project_types.is_empty() {
        UI::warn("No recognized project type found");
        UI::hint("Supported: Rust (Cargo.toml), Node.js (package.json), Python (requirements.txt, pyproject.toml), Go (go.mod), C++ (CMakeLists.txt, vcpkg.json, conanfile.txt)");
        return Ok(());
    }

    for project_type in &project_types {
        match project_type.as_str() {
            "rust" => {
                let (issues, critical) = scan_rust(&current_dir, verbose, fix, &format).await?;
                total_issues += issues;
                critical_issues += critical;
            }
            "node" => {
                let (issues, critical) = scan_node(&current_dir, verbose, fix, &format).await?;
                total_issues += issues;
                critical_issues += critical;
            }
            "python" => {
                let (issues, critical) = scan_python(&current_dir, verbose, fix, &format).await?;
                total_issues += issues;
                critical_issues += critical;
            }
            "go" => {
                let (issues, critical) = scan_go(&current_dir, verbose, &format).await?;
                total_issues += issues;
                critical_issues += critical;
            }
            "cpp" => {
                let (issues, critical) = scan_cpp(&current_dir, verbose, &format).await?;
                total_issues += issues;
                critical_issues += critical;
            }
            _ => {}
        }
    }

    // Summary
    println!();
    if total_issues == 0 {
        UI::success("No vulnerabilities found");
    } else {
        UI::warn(&format!(
            "Found {} vulnerabilities ({} critical)",
            total_issues, critical_issues
        ));

        // Check fail threshold
        let fail_on = security_config.fail_on.as_deref().unwrap_or("high");

        if (fail_on == "critical" && critical_issues > 0) || (fail_on == "high" && total_issues > 0)
        {
            return Err(anyhow::anyhow!(
                "Security scan failed: {} vulnerabilities exceed threshold",
                total_issues
            ));
        }
    }

    Ok(())
}

/// Handle security audit command
pub async fn handle_audit(verbose: bool, json: bool) -> Result<()> {
    let current_dir = env::current_dir()?;

    UI::header("Dependency Audit");

    let project_types = detect_project_types(&current_dir);

    for project_type in &project_types {
        match project_type.as_str() {
            "rust" => audit_rust(&current_dir, verbose, json).await?,
            "node" => audit_node(&current_dir, verbose, json).await?,
            "python" => audit_python(&current_dir, verbose, json).await?,
            "go" => audit_go(&current_dir, verbose, json).await?,
            "cpp" => audit_cpp(&current_dir, verbose, json).await?,
            _ => {}
        }
    }

    Ok(())
}

/// Handle secret detection command
pub async fn handle_secrets(
    path: Option<String>,
    baseline: bool,
    update_baseline: bool,
    verbose: bool,
) -> Result<()> {
    let scan_path = path
        .map(|p| Path::new(&p).to_path_buf())
        .unwrap_or_else(|| env::current_dir().unwrap());

    UI::header("Secret Detection");

    if !scan_path.exists() {
        return Err(anyhow::anyhow!(
            "Path does not exist: {}",
            scan_path.display()
        ));
    }

    let baseline_path = scan_path.join(".secrets.baseline");
    let mut baseline_secrets: Vec<String> = Vec::new();

    // Load baseline if exists
    if baseline && baseline_path.exists() {
        let content = std::fs::read_to_string(&baseline_path)?;
        baseline_secrets = content.lines().map(|s| s.to_string()).collect();
        if verbose {
            UI::info(&format!(
                "Loaded {} baseline entries",
                baseline_secrets.len()
            ));
        }
    }

    // Get secret patterns
    let patterns = vx_config::patterns::get_patterns();

    let mut findings: Vec<vx_config::SecretFinding> = Vec::new();

    // Scan files
    scan_directory_for_secrets(
        &scan_path,
        &patterns,
        &baseline_secrets,
        &mut findings,
        verbose,
    )?;

    // Report findings
    let active_findings: Vec<_> = findings.iter().filter(|f| !f.in_baseline).collect();

    if active_findings.is_empty() {
        UI::success("No secrets detected");
    } else {
        UI::error(&format!(
            "Found {} potential secrets",
            active_findings.len()
        ));
        println!();

        for finding in &active_findings {
            println!(
                "  {} {}:{}",
                UI::format_error(&finding.secret_type),
                finding.file,
                finding.line
            );
            if verbose {
                println!("    Pattern: {}", finding.pattern);
            }
        }
    }

    // Update baseline if requested
    if update_baseline {
        let baseline_content: Vec<String> = findings
            .iter()
            .map(|f| format!("{}:{}:{}", f.file, f.line, f.secret_type))
            .collect();
        std::fs::write(&baseline_path, baseline_content.join("\n"))?;
        UI::success(&format!(
            "Updated baseline with {} entries",
            baseline_content.len()
        ));
    }

    if !active_findings.is_empty() {
        return Err(anyhow::anyhow!(
            "Secret detection failed: {} secrets found",
            active_findings.len()
        ));
    }

    Ok(())
}

/// Handle license check command
pub async fn handle_licenses(verbose: bool, format: Option<String>) -> Result<()> {
    let current_dir = env::current_dir()?;
    let config_path = current_dir.join(".vx.toml");

    UI::header("License Compliance");

    // Load config
    let config = if config_path.exists() {
        Some(vx_config::parse_config(&config_path)?)
    } else {
        None
    };

    let security_config = config
        .as_ref()
        .and_then(|c| c.security.clone())
        .unwrap_or_default();

    let allowed = &security_config.allowed_licenses;
    let denied = &security_config.denied_licenses;

    if verbose {
        if !allowed.is_empty() {
            UI::info(&format!("Allowed licenses: {}", allowed.join(", ")));
        }
        if !denied.is_empty() {
            UI::info(&format!("Denied licenses: {}", denied.join(", ")));
        }
    }

    let project_types = detect_project_types(&current_dir);
    let mut violations: Vec<vx_config::LicenseViolation> = Vec::new();

    for project_type in &project_types {
        match project_type.as_str() {
            "rust" => {
                check_rust_licenses(&current_dir, allowed, denied, &mut violations, verbose)
                    .await?;
            }
            "node" => {
                check_node_licenses(&current_dir, allowed, denied, &mut violations, verbose)
                    .await?;
            }
            _ => {}
        }
    }

    // Report
    if violations.is_empty() {
        UI::success("All licenses are compliant");
    } else {
        UI::error(&format!("Found {} license violations", violations.len()));
        println!();

        for violation in &violations {
            println!(
                "  {} {} @ {} - {} ({})",
                UI::format_error("✗"),
                violation.package,
                violation.version,
                violation.license,
                violation.reason
            );
        }

        match format.as_deref() {
            Some("json") => {
                println!("\n{}", serde_json::to_string_pretty(&violations)?);
            }
            _ => {}
        }

        return Err(anyhow::anyhow!(
            "License check failed: {} violations",
            violations.len()
        ));
    }

    Ok(())
}

/// Handle security report command
pub async fn handle_report(output: Option<String>, format: Option<String>) -> Result<()> {
    let current_dir = env::current_dir()?;

    UI::header("Generating Security Report");

    // Run all scans
    let mut result = vx_config::SecurityScanResult {
        vulnerabilities: Vec::new(),
        secrets: Vec::new(),
        license_violations: Vec::new(),
        status: vx_config::ScanStatus::Pass,
    };

    // Collect vulnerabilities
    let project_types = detect_project_types(&current_dir);
    for project_type in &project_types {
        collect_vulnerabilities(&current_dir, project_type, &mut result.vulnerabilities).await?;
    }

    // Collect secrets
    let patterns = vx_config::patterns::get_patterns();
    scan_directory_for_secrets(&current_dir, &patterns, &[], &mut result.secrets, false)?;

    // Determine status
    if !result.vulnerabilities.is_empty() || !result.secrets.iter().any(|s| !s.in_baseline) {
        result.status = vx_config::ScanStatus::Warn;
    }

    let has_critical = result
        .vulnerabilities
        .iter()
        .any(|v| v.severity == vx_config::Severity::Critical);
    let has_secrets = result.secrets.iter().any(|s| !s.in_baseline);

    if has_critical || has_secrets || !result.license_violations.is_empty() {
        result.status = vx_config::ScanStatus::Fail;
    }

    // Generate report
    let report = match format.as_deref() {
        Some("json") => serde_json::to_string_pretty(&result)?,
        Some("sarif") => generate_sarif_report(&result)?,
        _ => vx_config::generate_security_report(&result),
    };

    // Output
    if let Some(output_path) = output {
        std::fs::write(&output_path, &report)?;
        UI::success(&format!("Report written to {}", output_path));
    } else {
        println!("{}", report);
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
    // C++ project detection
    if dir.join("CMakeLists.txt").exists()
        || dir.join("conanfile.txt").exists()
        || dir.join("conanfile.py").exists()
        || dir.join("vcpkg.json").exists()
    {
        types.push("cpp".to_string());
    }

    types
}

async fn scan_rust(
    dir: &Path,
    verbose: bool,
    _fix: bool,
    _format: &Option<String>,
) -> Result<(usize, usize)> {
    UI::step("Scanning Rust dependencies...");

    // Check if cargo-audit is available
    let audit_check = Command::new("cargo").args(["audit", "--version"]).output();

    if audit_check.is_err() || !audit_check.unwrap().status.success() {
        UI::warn("cargo-audit not installed. Install with: cargo install cargo-audit");
        return Ok((0, 0));
    }

    let mut cmd = Command::new("cargo");
    cmd.current_dir(dir).args(["audit", "--json"]);

    let output = cmd.output()?;

    if output.status.success() {
        UI::success("No vulnerabilities found in Rust dependencies");
        return Ok((0, 0));
    }

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    if verbose {
        println!("{}", stdout);
    }

    // Count vulnerabilities (simplified)
    let total = stdout.matches("\"id\":").count();
    let critical = stdout.matches("\"severity\":\"critical\"").count();

    UI::warn(&format!(
        "Found {} vulnerabilities in Rust dependencies",
        total
    ));

    Ok((total, critical))
}

async fn scan_node(
    dir: &Path,
    verbose: bool,
    fix: bool,
    _format: &Option<String>,
) -> Result<(usize, usize)> {
    UI::step("Scanning Node.js dependencies...");

    let mut cmd = Command::new("npm");
    cmd.current_dir(dir).args(["audit", "--json"]);

    if fix {
        cmd.arg("--fix");
    }

    let output = cmd.output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    if verbose {
        println!("{}", stdout);
    }

    // Parse npm audit output
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
        let total = json["metadata"]["vulnerabilities"]["total"]
            .as_u64()
            .unwrap_or(0) as usize;
        let critical = json["metadata"]["vulnerabilities"]["critical"]
            .as_u64()
            .unwrap_or(0) as usize;

        if total == 0 {
            UI::success("No vulnerabilities found in Node.js dependencies");
        } else {
            UI::warn(&format!(
                "Found {} vulnerabilities in Node.js dependencies",
                total
            ));
        }

        return Ok((total, critical));
    }

    Ok((0, 0))
}

async fn scan_python(
    dir: &Path,
    verbose: bool,
    _fix: bool,
    _format: &Option<String>,
) -> Result<(usize, usize)> {
    UI::step("Scanning Python dependencies...");

    // Try pip-audit first
    let audit_check = Command::new("pip-audit").arg("--version").output();

    if audit_check.is_err() || !audit_check.unwrap().status.success() {
        // Try safety
        let safety_check = Command::new("safety").arg("--version").output();

        if safety_check.is_err() || !safety_check.unwrap().status.success() {
            UI::warn("Neither pip-audit nor safety installed");
            UI::hint("Install with: pip install pip-audit");
            return Ok((0, 0));
        }

        // Use safety
        let mut cmd = Command::new("safety");
        cmd.current_dir(dir).args(["check", "--json"]);

        let output = cmd.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        if verbose {
            println!("{}", stdout);
        }

        let total = stdout.matches("\"vulnerability_id\"").count();
        return Ok((total, 0));
    }

    // Use pip-audit
    let mut cmd = Command::new("pip-audit");
    cmd.current_dir(dir).args(["--format", "json"]);

    let output = cmd.output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    if verbose {
        println!("{}", stdout);
    }

    if output.status.success() {
        UI::success("No vulnerabilities found in Python dependencies");
        return Ok((0, 0));
    }

    let total = stdout.matches("\"id\"").count();
    UI::warn(&format!(
        "Found {} vulnerabilities in Python dependencies",
        total
    ));

    Ok((total, 0))
}

async fn scan_go(dir: &Path, verbose: bool, _format: &Option<String>) -> Result<(usize, usize)> {
    UI::step("Scanning Go dependencies...");

    // Check if govulncheck is available
    let check = Command::new("govulncheck").arg("-version").output();

    if check.is_err() || !check.unwrap().status.success() {
        UI::warn("govulncheck not installed. Install with: go install golang.org/x/vuln/cmd/govulncheck@latest");
        return Ok((0, 0));
    }

    let mut cmd = Command::new("govulncheck");
    cmd.current_dir(dir).args(["-json", "./..."]);

    let output = cmd.output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    if verbose {
        println!("{}", stdout);
    }

    if output.status.success() {
        UI::success("No vulnerabilities found in Go dependencies");
        return Ok((0, 0));
    }

    // Count vulnerabilities from JSON output
    let total = stdout.matches("\"osv\"").count();
    let critical = stdout.matches("\"HIGH\"").count() + stdout.matches("\"CRITICAL\"").count();

    UI::warn(&format!(
        "Found {} vulnerabilities in Go dependencies",
        total
    ));

    Ok((total, critical))
}

async fn scan_cpp(dir: &Path, verbose: bool, _format: &Option<String>) -> Result<(usize, usize)> {
    UI::step("Scanning C++ dependencies...");

    // Check if osv-scanner is available
    let check = Command::new("osv-scanner").arg("--version").output();

    if check.is_err() || !check.unwrap().status.success() {
        UI::warn("osv-scanner not installed. Install from: https://github.com/google/osv-scanner");
        UI::hint("osv-scanner can scan vcpkg.json, conanfile.txt, and CMakeLists.txt");
        return Ok((0, 0));
    }

    let mut cmd = Command::new("osv-scanner");
    cmd.current_dir(dir).args(["--format", "json", "-r", "."]);

    let output = cmd.output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    if verbose {
        println!("{}", stdout);
    }

    if output.status.success() {
        UI::success("No vulnerabilities found in C++ dependencies");
        return Ok((0, 0));
    }

    // Parse JSON output
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
        let total = json["results"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|r| r["packages"].as_array())
                    .flatten()
                    .filter_map(|p| p["vulnerabilities"].as_array())
                    .flatten()
                    .count()
            })
            .unwrap_or(0);

        let critical = json["results"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|r| r["packages"].as_array())
                    .flatten()
                    .filter_map(|p| p["vulnerabilities"].as_array())
                    .flatten()
                    .filter(|v| {
                        v["database_specific"]["severity"]
                            .as_str()
                            .map(|s| s == "CRITICAL" || s == "HIGH")
                            .unwrap_or(false)
                    })
                    .count()
            })
            .unwrap_or(0);

        if total > 0 {
            UI::warn(&format!(
                "Found {} vulnerabilities in C++ dependencies",
                total
            ));
        }

        return Ok((total, critical));
    }

    Ok((0, 0))
}

async fn audit_rust(dir: &Path, verbose: bool, json: bool) -> Result<()> {
    UI::step("Auditing Rust dependencies...");

    let mut cmd = Command::new("cargo");
    cmd.current_dir(dir).arg("audit");

    if json {
        cmd.arg("--json");
    }

    let status = cmd.status()?;

    if !status.success() && verbose {
        UI::warn("cargo audit found issues");
    }

    Ok(())
}

async fn audit_node(dir: &Path, verbose: bool, json: bool) -> Result<()> {
    UI::step("Auditing Node.js dependencies...");

    let mut cmd = Command::new("npm");
    cmd.current_dir(dir).arg("audit");

    if json {
        cmd.arg("--json");
    }

    let status = cmd.status()?;

    if !status.success() && verbose {
        UI::warn("npm audit found issues");
    }

    Ok(())
}

async fn audit_python(dir: &Path, verbose: bool, json: bool) -> Result<()> {
    UI::step("Auditing Python dependencies...");

    // Try pip-audit
    let mut cmd = Command::new("pip-audit");
    cmd.current_dir(dir);

    if json {
        cmd.args(["--format", "json"]);
    }

    let result = cmd.status();

    if result.is_err() {
        if verbose {
            UI::warn("pip-audit not available");
        }
        return Ok(());
    }

    Ok(())
}

async fn audit_go(dir: &Path, verbose: bool, json: bool) -> Result<()> {
    UI::step("Auditing Go dependencies...");

    let check = Command::new("govulncheck").arg("-version").output();

    if check.is_err() || !check.unwrap().status.success() {
        if verbose {
            UI::warn("govulncheck not available");
        }
        return Ok(());
    }

    let mut cmd = Command::new("govulncheck");
    cmd.current_dir(dir);

    if json {
        cmd.arg("-json");
    }

    cmd.arg("./...");

    let status = cmd.status()?;

    if !status.success() && verbose {
        UI::warn("govulncheck found issues");
    }

    Ok(())
}

async fn audit_cpp(dir: &Path, verbose: bool, json: bool) -> Result<()> {
    UI::step("Auditing C++ dependencies...");

    let check = Command::new("osv-scanner").arg("--version").output();

    if check.is_err() || !check.unwrap().status.success() {
        if verbose {
            UI::warn("osv-scanner not available");
        }
        return Ok(());
    }

    let mut cmd = Command::new("osv-scanner");
    cmd.current_dir(dir);

    if json {
        cmd.args(["--format", "json"]);
    }

    cmd.args(["-r", "."]);

    let status = cmd.status()?;

    if !status.success() && verbose {
        UI::warn("osv-scanner found issues");
    }

    Ok(())
}

fn scan_directory_for_secrets(
    dir: &Path,
    patterns: &[(&str, regex::Regex)],
    baseline: &[String],
    findings: &mut Vec<vx_config::SecretFinding>,
    verbose: bool,
) -> Result<()> {
    use std::fs;

    let ignore_patterns = [
        ".git",
        "node_modules",
        "target",
        ".venv",
        "venv",
        "__pycache__",
        ".secrets.baseline",
    ];

    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !ignore_patterns.iter().any(|p| name == *p)
        })
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();

        // Skip binary files
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            if [
                "png", "jpg", "jpeg", "gif", "ico", "woff", "woff2", "ttf", "eot", "pdf", "zip",
                "tar", "gz", "exe", "dll", "so", "dylib",
            ]
            .contains(&ext.as_str())
            {
                continue;
            }
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue, // Skip files that can't be read as text
        };

        let relative_path = path
            .strip_prefix(dir)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        for (line_num, line) in content.lines().enumerate() {
            for (name, pattern) in patterns {
                if pattern.is_match(line) {
                    let baseline_key = format!("{}:{}:{}", relative_path, line_num + 1, name);
                    let in_baseline = baseline.contains(&baseline_key);

                    if verbose && !in_baseline {
                        UI::warn(&format!(
                            "Found {} in {}:{}",
                            name,
                            relative_path,
                            line_num + 1
                        ));
                    }

                    findings.push(vx_config::SecretFinding {
                        file: relative_path.clone(),
                        line: (line_num + 1) as u32,
                        secret_type: name.to_string(),
                        pattern: pattern.to_string(),
                        in_baseline,
                    });
                }
            }
        }
    }

    Ok(())
}

async fn check_rust_licenses(
    dir: &Path,
    allowed: &[String],
    denied: &[String],
    violations: &mut Vec<vx_config::LicenseViolation>,
    verbose: bool,
) -> Result<()> {
    // Check if cargo-license is available
    let check = Command::new("cargo")
        .args(["license", "--version"])
        .output();

    if check.is_err() || !check.unwrap().status.success() {
        if verbose {
            UI::warn("cargo-license not installed. Install with: cargo install cargo-license");
        }
        return Ok(());
    }

    let output = Command::new("cargo")
        .current_dir(dir)
        .args(["license", "--json"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    if let Ok(licenses) = serde_json::from_str::<Vec<serde_json::Value>>(&stdout) {
        for pkg in licenses {
            let name = pkg["name"].as_str().unwrap_or("");
            let version = pkg["version"].as_str().unwrap_or("");
            let license = pkg["license"].as_str().unwrap_or("Unknown");

            // Check denied
            if denied.iter().any(|d| license.contains(d)) {
                violations.push(vx_config::LicenseViolation {
                    package: name.to_string(),
                    version: version.to_string(),
                    license: license.to_string(),
                    reason: "License is in denied list".to_string(),
                });
                continue;
            }

            // Check allowed (if list is not empty)
            if !allowed.is_empty() && !allowed.iter().any(|a| license.contains(a)) {
                violations.push(vx_config::LicenseViolation {
                    package: name.to_string(),
                    version: version.to_string(),
                    license: license.to_string(),
                    reason: "License is not in allowed list".to_string(),
                });
            }
        }
    }

    Ok(())
}

async fn check_node_licenses(
    dir: &Path,
    allowed: &[String],
    denied: &[String],
    violations: &mut Vec<vx_config::LicenseViolation>,
    verbose: bool,
) -> Result<()> {
    // Check if license-checker is available
    let check = Command::new("npx")
        .args(["license-checker", "--version"])
        .output();

    if check.is_err() {
        if verbose {
            UI::warn("license-checker not available");
        }
        return Ok(());
    }

    let output = Command::new("npx")
        .current_dir(dir)
        .args(["license-checker", "--json"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    if let Ok(licenses) = serde_json::from_str::<serde_json::Value>(&stdout) {
        if let Some(obj) = licenses.as_object() {
            for (pkg_name, info) in obj {
                let license = info["licenses"]
                    .as_str()
                    .or_else(|| {
                        info["licenses"]
                            .as_array()
                            .and_then(|a| a.first()?.as_str())
                    })
                    .unwrap_or("Unknown");

                // Parse name@version
                let parts: Vec<&str> = pkg_name.rsplitn(2, '@').collect();
                let (name, version) = if parts.len() == 2 {
                    (parts[1], parts[0])
                } else {
                    (pkg_name.as_str(), "")
                };

                // Check denied
                if denied.iter().any(|d| license.contains(d)) {
                    violations.push(vx_config::LicenseViolation {
                        package: name.to_string(),
                        version: version.to_string(),
                        license: license.to_string(),
                        reason: "License is in denied list".to_string(),
                    });
                    continue;
                }

                // Check allowed
                if !allowed.is_empty() && !allowed.iter().any(|a| license.contains(a)) {
                    violations.push(vx_config::LicenseViolation {
                        package: name.to_string(),
                        version: version.to_string(),
                        license: license.to_string(),
                        reason: "License is not in allowed list".to_string(),
                    });
                }
            }
        }
    }

    Ok(())
}

async fn collect_vulnerabilities(
    dir: &Path,
    project_type: &str,
    vulnerabilities: &mut Vec<vx_config::Vulnerability>,
) -> Result<()> {
    match project_type {
        "rust" => {
            let output = Command::new("cargo")
                .current_dir(dir)
                .args(["audit", "--json"])
                .output()?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                if let Some(vulns) = json["vulnerabilities"]["list"].as_array() {
                    for v in vulns {
                        vulnerabilities.push(vx_config::Vulnerability {
                            id: v["advisory"]["id"].as_str().unwrap_or("").to_string(),
                            package: v["package"]["name"].as_str().unwrap_or("").to_string(),
                            version: v["package"]["version"].as_str().unwrap_or("").to_string(),
                            severity: vx_config::Severity::from_str(
                                v["advisory"]["severity"].as_str().unwrap_or("medium"),
                            )
                            .unwrap_or(vx_config::Severity::Medium),
                            description: v["advisory"]["description"]
                                .as_str()
                                .unwrap_or("")
                                .to_string(),
                            fixed_version: v["versions"]["patched"]
                                .as_array()
                                .and_then(|a| a.first()?.as_str())
                                .map(|s| s.to_string()),
                            references: v["advisory"]["references"]
                                .as_array()
                                .map(|a| {
                                    a.iter()
                                        .filter_map(|r| r.as_str().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_default(),
                        });
                    }
                }
            }
        }
        "node" => {
            let output = Command::new("npm")
                .current_dir(dir)
                .args(["audit", "--json"])
                .output()?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                if let Some(vulns) = json["vulnerabilities"].as_object() {
                    for (name, info) in vulns {
                        vulnerabilities.push(vx_config::Vulnerability {
                            id: info["via"]
                                .as_array()
                                .and_then(|a| a.first()?.as_object()?.get("source")?.as_u64())
                                .map(|n| format!("GHSA-{}", n))
                                .unwrap_or_default(),
                            package: name.clone(),
                            version: info["range"].as_str().unwrap_or("").to_string(),
                            severity: vx_config::Severity::from_str(
                                info["severity"].as_str().unwrap_or("medium"),
                            )
                            .unwrap_or(vx_config::Severity::Medium),
                            description: info["via"]
                                .as_array()
                                .and_then(|a| a.first()?.as_object()?.get("title")?.as_str())
                                .unwrap_or("")
                                .to_string(),
                            fixed_version: info["fixAvailable"]
                                .as_object()
                                .and_then(|o| o.get("version")?.as_str())
                                .map(|s| s.to_string()),
                            references: Vec::new(),
                        });
                    }
                }
            }
        }
        "go" => {
            let output = Command::new("govulncheck")
                .current_dir(dir)
                .args(["-json", "./..."])
                .output()?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            // Parse govulncheck JSON output (line-delimited JSON)
            for line in stdout.lines() {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(osv) = json.get("osv") {
                        vulnerabilities.push(vx_config::Vulnerability {
                            id: osv["id"].as_str().unwrap_or("").to_string(),
                            package: osv["affected"]
                                .as_array()
                                .and_then(|a| a.first()?.get("package")?.get("name")?.as_str())
                                .unwrap_or("")
                                .to_string(),
                            version: "".to_string(),
                            severity: vx_config::Severity::from_str(
                                osv["database_specific"]["severity"]
                                    .as_str()
                                    .unwrap_or("medium"),
                            )
                            .unwrap_or(vx_config::Severity::Medium),
                            description: osv["summary"].as_str().unwrap_or("").to_string(),
                            fixed_version: osv["affected"]
                                .as_array()
                                .and_then(|a| {
                                    a.first()?
                                        .get("ranges")?
                                        .as_array()?
                                        .first()?
                                        .get("events")?
                                        .as_array()?
                                        .iter()
                                        .find_map(|e| e.get("fixed")?.as_str())
                                })
                                .map(|s| s.to_string()),
                            references: osv["references"]
                                .as_array()
                                .map(|a| {
                                    a.iter()
                                        .filter_map(|r| r["url"].as_str().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_default(),
                        });
                    }
                }
            }
        }
        "cpp" => {
            let output = Command::new("osv-scanner")
                .current_dir(dir)
                .args(["--format", "json", "-r", "."])
                .output()?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                if let Some(results) = json["results"].as_array() {
                    for result in results {
                        if let Some(packages) = result["packages"].as_array() {
                            for pkg in packages {
                                let pkg_name = pkg["package"]["name"].as_str().unwrap_or("");
                                let pkg_version = pkg["package"]["version"].as_str().unwrap_or("");

                                if let Some(vulns) = pkg["vulnerabilities"].as_array() {
                                    for v in vulns {
                                        vulnerabilities.push(vx_config::Vulnerability {
                                            id: v["id"].as_str().unwrap_or("").to_string(),
                                            package: pkg_name.to_string(),
                                            version: pkg_version.to_string(),
                                            severity: vx_config::Severity::from_str(
                                                v["database_specific"]["severity"]
                                                    .as_str()
                                                    .unwrap_or("medium"),
                                            )
                                            .unwrap_or(vx_config::Severity::Medium),
                                            description: v["summary"]
                                                .as_str()
                                                .unwrap_or("")
                                                .to_string(),
                                            fixed_version: v["affected"]
                                                .as_array()
                                                .and_then(|a| {
                                                    a.first()?
                                                        .get("ranges")?
                                                        .as_array()?
                                                        .first()?
                                                        .get("events")?
                                                        .as_array()?
                                                        .iter()
                                                        .find_map(|e| e.get("fixed")?.as_str())
                                                })
                                                .map(|s| s.to_string()),
                                            references: v["references"]
                                                .as_array()
                                                .map(|a| {
                                                    a.iter()
                                                        .filter_map(|r| {
                                                            r["url"].as_str().map(|s| s.to_string())
                                                        })
                                                        .collect()
                                                })
                                                .unwrap_or_default(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}

fn generate_sarif_report(result: &vx_config::SecurityScanResult) -> Result<String> {
    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "vx security",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/aspect-build/vx"
                }
            },
            "results": result.vulnerabilities.iter().map(|v| {
                serde_json::json!({
                    "ruleId": &v.id,
                    "level": match v.severity {
                        vx_config::Severity::Critical => "error",
                        vx_config::Severity::High => "error",
                        vx_config::Severity::Medium => "warning",
                        vx_config::Severity::Low => "note",
                    },
                    "message": {
                        "text": &v.description
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": format!("package://{}", v.package)
                            }
                        }
                    }]
                })
            }).collect::<Vec<_>>()
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}
