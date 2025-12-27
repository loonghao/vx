//! Dependency management commands
//!
//! Provides dependency-related functionality:
//! - Install dependencies
//! - Audit dependencies
//! - Update dependencies
//! - Lock file management

use anyhow::Result;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::ui::UI;

/// Handle dependency install command
pub async fn handle_install(frozen: bool, dev: bool, prod: bool, verbose: bool) -> Result<()> {
    let current_dir = env::current_dir()?;
    let config_path = current_dir.join(".vx.toml");

    UI::header("Installing Dependencies");

    // Load config if exists
    let config = if config_path.exists() {
        Some(vx_config::parse_config(&config_path)?)
    } else {
        None
    };

    let deps_config = config
        .as_ref()
        .and_then(|c| c.dependencies.clone())
        .unwrap_or_default();

    // Detect project types
    let project_types = detect_project_types(&current_dir);

    if project_types.is_empty() {
        return Err(anyhow::anyhow!(
            "No recognized project type found. Supported: Rust, Node.js, Python, Go, C++"
        ));
    }

    for project_type in &project_types {
        match project_type.as_str() {
            "rust" => {
                install_rust_deps(&current_dir, frozen, verbose).await?;
            }
            "node" => {
                install_node_deps(&current_dir, frozen, dev, prod, verbose, &deps_config).await?;
            }
            "python" => {
                install_python_deps(&current_dir, frozen, dev, verbose, &deps_config).await?;
            }
            "go" => {
                install_go_deps(&current_dir, verbose, &deps_config).await?;
            }
            "cpp" => {
                install_cpp_deps(&current_dir, verbose, &deps_config).await?;
            }
            _ => {}
        }
    }

    UI::success("Dependencies installed");
    Ok(())
}

/// Handle dependency audit command
pub async fn handle_audit(fix: bool, verbose: bool) -> Result<()> {
    let current_dir = env::current_dir()?;

    UI::header("Auditing Dependencies");

    let project_types = detect_project_types(&current_dir);

    let mut has_issues = false;

    for project_type in &project_types {
        match project_type.as_str() {
            "rust" => {
                if let Err(e) = audit_rust_deps(&current_dir, verbose).await {
                    UI::error(&format!("Rust audit: {}", e));
                    has_issues = true;
                }
            }
            "node" => {
                if let Err(e) = audit_node_deps(&current_dir, fix, verbose).await {
                    UI::error(&format!("Node.js audit: {}", e));
                    has_issues = true;
                }
            }
            "python" => {
                if let Err(e) = audit_python_deps(&current_dir, verbose).await {
                    UI::error(&format!("Python audit: {}", e));
                    has_issues = true;
                }
            }
            "go" => {
                if let Err(e) = audit_go_deps(&current_dir, verbose).await {
                    UI::error(&format!("Go audit: {}", e));
                    has_issues = true;
                }
            }
            "cpp" => {
                if let Err(e) = audit_cpp_deps(&current_dir, verbose).await {
                    UI::error(&format!("C++ audit: {}", e));
                    has_issues = true;
                }
            }
            _ => {}
        }
    }

    if has_issues {
        return Err(anyhow::anyhow!("Dependency audit found issues"));
    }

    UI::success("No vulnerabilities found");
    Ok(())
}

/// Handle dependency update command
pub async fn handle_update(
    packages: Vec<String>,
    major: bool,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    let current_dir = env::current_dir()?;

    UI::header("Updating Dependencies");

    let project_types = detect_project_types(&current_dir);

    for project_type in &project_types {
        match project_type.as_str() {
            "rust" => {
                update_rust_deps(&current_dir, &packages, dry_run, verbose).await?;
            }
            "node" => {
                update_node_deps(&current_dir, &packages, major, dry_run, verbose).await?;
            }
            "python" => {
                update_python_deps(&current_dir, &packages, dry_run, verbose).await?;
            }
            "go" => {
                update_go_deps(&current_dir, &packages, verbose).await?;
            }
            "cpp" => {
                update_cpp_deps(&current_dir, verbose).await?;
            }
            _ => {}
        }
    }

    UI::success("Dependencies updated");
    Ok(())
}

/// Handle dependency lock command
pub async fn handle_lock(verbose: bool) -> Result<()> {
    let current_dir = env::current_dir()?;

    UI::header("Locking Dependencies");

    let project_types = detect_project_types(&current_dir);

    for project_type in &project_types {
        match project_type.as_str() {
            "rust" => {
                lock_rust_deps(&current_dir, verbose).await?;
            }
            "node" => {
                lock_node_deps(&current_dir, verbose).await?;
            }
            "python" => {
                lock_python_deps(&current_dir, verbose).await?;
            }
            _ => {}
        }
    }

    UI::success("Dependencies locked");
    Ok(())
}

/// Handle dependency status command
pub async fn handle_status(verbose: bool) -> Result<()> {
    let current_dir = env::current_dir()?;
    let config_path = current_dir.join(".vx.toml");

    UI::header("Dependency Status");

    // Check .vx.toml configuration
    if config_path.exists() {
        let config = vx_config::parse_config(&config_path)?;
        if let Some(deps) = &config.dependencies {
            println!("\n{}", UI::format_header("Configuration"));

            if let Some(auto_update) = &deps.auto_update {
                println!("  Auto-update: {}", auto_update);
            }

            if deps.lockfile.unwrap_or(false) {
                println!("  Lockfile: Yes");
            }

            if deps.audit.unwrap_or(false) {
                println!("  Audit: Yes");
            }
        }
    }

    // Detect project types
    println!("\n{}", UI::format_header("Projects"));
    let project_types = detect_project_types(&current_dir);

    for project_type in &project_types {
        match project_type.as_str() {
            "rust" => {
                println!("  Rust:");
                let cargo_lock = current_dir.join("Cargo.lock");
                println!(
                    "    Cargo.lock: {}",
                    if cargo_lock.exists() { "✓" } else { "✗" }
                );

                if verbose && cargo_lock.exists() {
                    // Count dependencies
                    let content = fs::read_to_string(&cargo_lock)?;
                    let count = content.matches("[[package]]").count();
                    println!("    Packages: {}", count);
                }
            }
            "node" => {
                println!("  Node.js:");
                let pkg_lock = current_dir.join("package-lock.json");
                let yarn_lock = current_dir.join("yarn.lock");
                let pnpm_lock = current_dir.join("pnpm-lock.yaml");

                if pkg_lock.exists() {
                    println!("    package-lock.json: ✓");
                } else if yarn_lock.exists() {
                    println!("    yarn.lock: ✓");
                } else if pnpm_lock.exists() {
                    println!("    pnpm-lock.yaml: ✓");
                } else {
                    println!("    Lock file: ✗");
                }

                if verbose {
                    let pkg_json = current_dir.join("package.json");
                    if pkg_json.exists() {
                        let content = fs::read_to_string(&pkg_json)?;
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                            let deps_count = json["dependencies"]
                                .as_object()
                                .map(|o| o.len())
                                .unwrap_or(0);
                            let dev_deps_count = json["devDependencies"]
                                .as_object()
                                .map(|o| o.len())
                                .unwrap_or(0);
                            println!("    Dependencies: {}", deps_count);
                            println!("    Dev dependencies: {}", dev_deps_count);
                        }
                    }
                }
            }
            "python" => {
                println!("  Python:");
                let requirements = current_dir.join("requirements.txt");
                let pyproject = current_dir.join("pyproject.toml");
                let poetry_lock = current_dir.join("poetry.lock");
                let uv_lock = current_dir.join("uv.lock");

                if poetry_lock.exists() {
                    println!("    poetry.lock: ✓");
                } else if uv_lock.exists() {
                    println!("    uv.lock: ✓");
                } else if requirements.exists() {
                    println!("    requirements.txt: ✓");
                } else if pyproject.exists() {
                    println!("    pyproject.toml: ✓");
                } else {
                    println!("    Lock file: ✗");
                }
            }
            "go" => {
                println!("  Go:");
                let go_sum = current_dir.join("go.sum");
                println!("    go.sum: {}", if go_sum.exists() { "✓" } else { "✗" });

                if verbose && go_sum.exists() {
                    // Count dependencies
                    let content = fs::read_to_string(&go_sum)?;
                    let count = content.lines().filter(|l| !l.is_empty()).count() / 2; // Each dep has 2 lines
                    println!("    Modules: ~{}", count);
                }
            }
            "cpp" => {
                println!("  C++:");

                // Check for different C++ package managers
                let vcpkg_json = current_dir.join("vcpkg.json");
                let conanfile_txt = current_dir.join("conanfile.txt");
                let conanfile_py = current_dir.join("conanfile.py");
                let cmake_lists = current_dir.join("CMakeLists.txt");

                if vcpkg_json.exists() {
                    println!("    vcpkg.json: ✓");
                    if verbose {
                        if let Ok(content) = fs::read_to_string(&vcpkg_json) {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                                let deps_count = json["dependencies"]
                                    .as_array()
                                    .map(|a| a.len())
                                    .unwrap_or(0);
                                println!("    Dependencies: {}", deps_count);
                            }
                        }
                    }
                } else if conanfile_txt.exists() {
                    println!("    conanfile.txt: ✓");
                    let conan_lock = current_dir.join("conan.lock");
                    println!(
                        "    conan.lock: {}",
                        if conan_lock.exists() { "✓" } else { "✗" }
                    );
                } else if conanfile_py.exists() {
                    println!("    conanfile.py: ✓");
                    let conan_lock = current_dir.join("conan.lock");
                    println!(
                        "    conan.lock: {}",
                        if conan_lock.exists() { "✓" } else { "✗" }
                    );
                } else if cmake_lists.exists() {
                    println!("    CMakeLists.txt: ✓");
                    let build_dir = current_dir.join("build");
                    println!("    build/: {}", if build_dir.exists() { "✓" } else { "✗" });
                }
            }
            _ => {}
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

async fn install_rust_deps(dir: &Path, frozen: bool, verbose: bool) -> Result<()> {
    UI::step("Installing Rust dependencies...");

    let mut cmd = Command::new("cargo");
    cmd.current_dir(dir).arg("fetch");

    if frozen {
        cmd.arg("--locked");
    }

    if verbose {
        cmd.arg("--verbose");
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Failed to fetch Rust dependencies"));
    }

    Ok(())
}

async fn install_node_deps(
    dir: &Path,
    frozen: bool,
    dev: bool,
    prod: bool,
    verbose: bool,
    config: &vx_config::DependenciesConfig,
) -> Result<()> {
    UI::step("Installing Node.js dependencies...");

    // Detect package manager
    let pm = detect_node_package_manager(dir, config);

    let mut cmd = Command::new(&pm);
    cmd.current_dir(dir);

    match pm.as_str() {
        "npm" => {
            if frozen {
                cmd.arg("ci");
            } else {
                cmd.arg("install");
            }

            if prod && !dev {
                cmd.arg("--production");
            } else if dev && !prod {
                cmd.arg("--only=dev");
            }
        }
        "yarn" => {
            cmd.arg("install");

            if frozen {
                cmd.arg("--frozen-lockfile");
            }

            if prod && !dev {
                cmd.arg("--production");
            }
        }
        "pnpm" => {
            cmd.arg("install");

            if frozen {
                cmd.arg("--frozen-lockfile");
            }

            if prod && !dev {
                cmd.arg("--prod");
            } else if dev && !prod {
                cmd.arg("--dev");
            }
        }
        _ => {
            cmd.arg("install");
        }
    }

    if verbose {
        cmd.arg("--verbose");
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Failed to install Node.js dependencies"));
    }

    Ok(())
}

async fn install_python_deps(
    dir: &Path,
    frozen: bool,
    dev: bool,
    verbose: bool,
    config: &vx_config::DependenciesConfig,
) -> Result<()> {
    UI::step("Installing Python dependencies...");

    // Detect Python package manager
    let pm = detect_python_package_manager(dir, config);

    let mut cmd = Command::new(&pm);
    cmd.current_dir(dir);

    match pm.as_str() {
        "uv" => {
            cmd.arg("sync");

            if frozen {
                cmd.arg("--frozen");
            }

            if !dev {
                cmd.arg("--no-dev");
            }
        }
        "poetry" => {
            cmd.arg("install");

            if frozen {
                cmd.arg("--no-update");
            }

            if !dev {
                cmd.arg("--no-dev");
            }
        }
        "pip" => {
            cmd.args(["install", "-r", "requirements.txt"]);

            if verbose {
                cmd.arg("-v");
            }
        }
        _ => {
            cmd.arg("install");
        }
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Failed to install Python dependencies"));
    }

    Ok(())
}

async fn install_go_deps(
    dir: &Path,
    verbose: bool,
    config: &vx_config::DependenciesConfig,
) -> Result<()> {
    UI::step("Installing Go dependencies...");

    let mut cmd = Command::new("go");
    cmd.current_dir(dir).args(["mod", "download"]);

    // Apply Go proxy configuration
    if let Some(go_config) = &config.go {
        if let Some(proxy) = &go_config.proxy {
            cmd.env("GOPROXY", proxy);
        }
        if let Some(private) = &go_config.private {
            cmd.env("GOPRIVATE", private);
        }
        if let Some(sumdb) = &go_config.sumdb {
            cmd.env("GOSUMDB", sumdb);
        }
        if let Some(nosumdb) = &go_config.nosumdb {
            cmd.env("GONOSUMDB", nosumdb);
        }
        if go_config.vendor.unwrap_or(false) {
            cmd.env("GOFLAGS", "-mod=vendor");
        } else if let Some(mod_mode) = &go_config.mod_mode {
            cmd.env("GOFLAGS", format!("-mod={}", mod_mode));
        }
    }

    if verbose {
        cmd.arg("-x");
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Failed to download Go dependencies"));
    }

    // Run go mod tidy
    let mut tidy_cmd = Command::new("go");
    tidy_cmd.current_dir(dir).args(["mod", "tidy"]);

    if let Some(go_config) = &config.go {
        if let Some(proxy) = &go_config.proxy {
            tidy_cmd.env("GOPROXY", proxy);
        }
        if let Some(private) = &go_config.private {
            tidy_cmd.env("GOPRIVATE", private);
        }
    }

    let _ = tidy_cmd.status(); // Ignore tidy errors

    Ok(())
}

async fn install_cpp_deps(
    dir: &Path,
    verbose: bool,
    config: &vx_config::DependenciesConfig,
) -> Result<()> {
    UI::step("Installing C++ dependencies...");

    let pm = config
        .cpp
        .as_ref()
        .and_then(|c| c.package_manager.as_deref())
        .unwrap_or("cmake");

    match pm {
        "vcpkg" => install_vcpkg_deps(dir, verbose, config).await,
        "conan" => install_conan_deps(dir, verbose, config).await,
        _ => configure_cmake(dir, verbose, config).await,
    }
}

async fn install_vcpkg_deps(
    dir: &Path,
    verbose: bool,
    config: &vx_config::DependenciesConfig,
) -> Result<()> {
    UI::step("Installing vcpkg dependencies...");

    let mut cmd = Command::new("vcpkg");
    cmd.current_dir(dir).arg("install");

    // Add triplet if configured
    if let Some(cpp_config) = &config.cpp {
        if let Some(triplet) = &cpp_config.vcpkg_triplet {
            cmd.args(["--triplet", triplet]);
        }
        if let Some(vcpkg_root) = &cpp_config.vcpkg_root {
            cmd.env("VCPKG_ROOT", vcpkg_root);
        }
    }

    if verbose {
        cmd.arg("--debug");
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Failed to install vcpkg dependencies"));
    }

    Ok(())
}

async fn install_conan_deps(
    dir: &Path,
    verbose: bool,
    config: &vx_config::DependenciesConfig,
) -> Result<()> {
    UI::step("Installing Conan dependencies...");

    // Add remote if configured
    if let Some(cpp_config) = &config.cpp {
        if let Some(remote) = &cpp_config.conan_remote {
            let _ = Command::new("conan")
                .args(["remote", "add", "custom", remote, "--force"])
                .status();
        }
    }

    let mut cmd = Command::new("conan");
    cmd.current_dir(dir)
        .args(["install", ".", "--build=missing"]);

    if verbose {
        cmd.arg("-v");
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Failed to install Conan dependencies"));
    }

    Ok(())
}

async fn configure_cmake(
    dir: &Path,
    verbose: bool,
    config: &vx_config::DependenciesConfig,
) -> Result<()> {
    UI::step("Configuring CMake project...");

    let mut cmd = Command::new("cmake");
    cmd.current_dir(dir).args(["-B", "build"]);

    if let Some(cpp_config) = &config.cpp {
        // Add generator if configured
        if let Some(generator) = &cpp_config.cmake_generator {
            cmd.args(["-G", generator]);
        }

        // Add build type
        let build_type = cpp_config.cmake_build_type.as_deref().unwrap_or("Release");
        cmd.arg(format!("-DCMAKE_BUILD_TYPE={}", build_type));

        // Add C++ standard
        if let Some(std) = &cpp_config.std {
            cmd.arg(format!("-DCMAKE_CXX_STANDARD={}", std));
        }

        // Add custom CMake options
        for (key, value) in &cpp_config.cmake_options {
            cmd.arg(format!("-D{}={}", key, value));
        }
    } else {
        cmd.arg("-DCMAKE_BUILD_TYPE=Release");
    }

    if verbose {
        cmd.arg("--debug-output");
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Failed to configure CMake project"));
    }

    Ok(())
}

async fn audit_rust_deps(dir: &Path, _verbose: bool) -> Result<()> {
    UI::step("Auditing Rust dependencies...");

    let check = Command::new("cargo").args(["audit", "--version"]).output();

    if check.is_err() || !check.unwrap().status.success() {
        UI::warn("cargo-audit not installed. Install with: cargo install cargo-audit");
        return Ok(());
    }

    let status = Command::new("cargo")
        .current_dir(dir)
        .arg("audit")
        .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "Vulnerabilities found in Rust dependencies"
        ));
    }

    Ok(())
}

async fn audit_node_deps(dir: &Path, fix: bool, _verbose: bool) -> Result<()> {
    UI::step("Auditing Node.js dependencies...");

    let mut cmd = Command::new("npm");
    cmd.current_dir(dir).arg("audit");

    if fix {
        cmd.arg("fix");
    }

    let status = cmd.status()?;
    if !status.success() && !fix {
        return Err(anyhow::anyhow!(
            "Vulnerabilities found in Node.js dependencies"
        ));
    }

    Ok(())
}

async fn audit_python_deps(dir: &Path, _verbose: bool) -> Result<()> {
    UI::step("Auditing Python dependencies...");

    // Try pip-audit
    let check = Command::new("pip-audit").arg("--version").output();

    if check.is_err() || !check.unwrap().status.success() {
        UI::warn("pip-audit not installed. Install with: pip install pip-audit");
        return Ok(());
    }

    let status = Command::new("pip-audit").current_dir(dir).status()?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "Vulnerabilities found in Python dependencies"
        ));
    }

    Ok(())
}

async fn audit_go_deps(dir: &Path, _verbose: bool) -> Result<()> {
    UI::step("Auditing Go dependencies...");

    // Try govulncheck
    let check = Command::new("govulncheck").arg("-version").output();

    if check.is_err() || !check.unwrap().status.success() {
        UI::warn("govulncheck not installed. Install with: go install golang.org/x/vuln/cmd/govulncheck@latest");
        return Ok(());
    }

    let status = Command::new("govulncheck")
        .current_dir(dir)
        .arg("./...")
        .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!("Vulnerabilities found in Go dependencies"));
    }

    Ok(())
}

async fn audit_cpp_deps(dir: &Path, _verbose: bool) -> Result<()> {
    UI::step("Auditing C++ dependencies...");

    // Check for vcpkg.json
    if dir.join("vcpkg.json").exists() {
        // vcpkg doesn't have built-in audit, suggest using OSV-Scanner
        let check = Command::new("osv-scanner").arg("--version").output();

        if check.is_err() || !check.unwrap().status.success() {
            UI::warn(
                "osv-scanner not installed. Install from: https://github.com/google/osv-scanner",
            );
            UI::hint("osv-scanner can scan vcpkg.json and conanfile.txt for vulnerabilities");
            return Ok(());
        }

        let status = Command::new("osv-scanner")
            .current_dir(dir)
            .args(["--lockfile", "vcpkg.json"])
            .status()?;

        if !status.success() {
            return Err(anyhow::anyhow!("Vulnerabilities found in C++ dependencies"));
        }
    } else if dir.join("conanfile.txt").exists() || dir.join("conanfile.py").exists() {
        // Try osv-scanner for Conan
        let check = Command::new("osv-scanner").arg("--version").output();

        if check.is_err() || !check.unwrap().status.success() {
            UI::warn("osv-scanner not installed for Conan dependency scanning");
            return Ok(());
        }

        let lockfile = if dir.join("conan.lock").exists() {
            "conan.lock"
        } else {
            UI::warn("No conan.lock found. Run 'conan lock create .' first");
            return Ok(());
        };

        let status = Command::new("osv-scanner")
            .current_dir(dir)
            .args(["--lockfile", lockfile])
            .status()?;

        if !status.success() {
            return Err(anyhow::anyhow!("Vulnerabilities found in C++ dependencies"));
        }
    } else {
        UI::warn("No vcpkg.json or conanfile found for C++ audit");
        UI::hint("Consider using vcpkg or Conan for dependency management");
    }

    Ok(())
}

async fn update_rust_deps(
    dir: &Path,
    packages: &[String],
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    UI::step("Updating Rust dependencies...");

    let mut cmd = Command::new("cargo");
    cmd.current_dir(dir).arg("update");

    if dry_run {
        cmd.arg("--dry-run");
    }

    if verbose {
        cmd.arg("--verbose");
    }

    for pkg in packages {
        cmd.args(["--package", pkg]);
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Failed to update Rust dependencies"));
    }

    Ok(())
}

async fn update_node_deps(
    dir: &Path,
    packages: &[String],
    major: bool,
    dry_run: bool,
    _verbose: bool,
) -> Result<()> {
    UI::step("Updating Node.js dependencies...");

    // Use npm-check-updates for major updates
    if major {
        let check = Command::new("npx")
            .args(["npm-check-updates", "--version"])
            .output();

        if check.is_ok() && check.unwrap().status.success() {
            let mut cmd = Command::new("npx");
            cmd.current_dir(dir).arg("npm-check-updates");

            if !dry_run {
                cmd.arg("-u");
            }

            if !packages.is_empty() {
                cmd.args(packages);
            }

            cmd.status()?;

            if !dry_run {
                // Run npm install after updating package.json
                Command::new("npm")
                    .current_dir(dir)
                    .arg("install")
                    .status()?;
            }

            return Ok(());
        }
    }

    let mut cmd = Command::new("npm");
    cmd.current_dir(dir).arg("update");

    if !packages.is_empty() {
        cmd.args(packages);
    }

    if dry_run {
        cmd.arg("--dry-run");
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Failed to update Node.js dependencies"));
    }

    Ok(())
}

async fn update_python_deps(
    dir: &Path,
    packages: &[String],
    dry_run: bool,
    _verbose: bool,
) -> Result<()> {
    UI::step("Updating Python dependencies...");

    // Detect package manager
    if dir.join("poetry.lock").exists() {
        let mut cmd = Command::new("poetry");
        cmd.current_dir(dir).arg("update");

        if dry_run {
            cmd.arg("--dry-run");
        }

        if !packages.is_empty() {
            cmd.args(packages);
        }

        cmd.status()?;
    } else if dir.join("uv.lock").exists() {
        let mut cmd = Command::new("uv");
        cmd.current_dir(dir).arg("lock");

        if !packages.is_empty() {
            for pkg in packages {
                cmd.args(["--upgrade-package", pkg]);
            }
        } else {
            cmd.arg("--upgrade");
        }

        if !dry_run {
            cmd.status()?;
            // Sync after lock
            Command::new("uv").current_dir(dir).arg("sync").status()?;
        } else {
            cmd.arg("--dry-run");
            cmd.status()?;
        }
    } else {
        // Use pip
        let mut cmd = Command::new("pip");
        cmd.current_dir(dir).args(["install", "--upgrade"]);

        if !packages.is_empty() {
            cmd.args(packages);
        } else {
            cmd.args(["-r", "requirements.txt"]);
        }

        if dry_run {
            cmd.arg("--dry-run");
        }

        cmd.status()?;
    }

    Ok(())
}

async fn update_go_deps(dir: &Path, packages: &[String], _verbose: bool) -> Result<()> {
    UI::step("Updating Go dependencies...");

    let mut cmd = Command::new("go");
    cmd.current_dir(dir).args(["get", "-u"]);

    if !packages.is_empty() {
        cmd.args(packages);
    } else {
        cmd.arg("./...");
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Failed to update Go dependencies"));
    }

    // Tidy up
    Command::new("go")
        .current_dir(dir)
        .args(["mod", "tidy"])
        .status()?;

    Ok(())
}

async fn update_cpp_deps(dir: &Path, _verbose: bool) -> Result<()> {
    UI::step("Updating C++ dependencies...");

    // Check for vcpkg.json
    if dir.join("vcpkg.json").exists() {
        let status = Command::new("vcpkg")
            .current_dir(dir)
            .args(["upgrade", "--no-dry-run"])
            .status()?;

        if !status.success() {
            return Err(anyhow::anyhow!("Failed to update vcpkg dependencies"));
        }
    } else if dir.join("conanfile.txt").exists() || dir.join("conanfile.py").exists() {
        // Conan update
        let status = Command::new("conan")
            .current_dir(dir)
            .args(["install", ".", "--update", "--build=missing"])
            .status()?;

        if !status.success() {
            return Err(anyhow::anyhow!("Failed to update Conan dependencies"));
        }
    } else {
        UI::warn("No vcpkg.json or conanfile found for C++ update");
    }

    Ok(())
}

async fn lock_rust_deps(dir: &Path, verbose: bool) -> Result<()> {
    UI::step("Locking Rust dependencies...");

    let mut cmd = Command::new("cargo");
    cmd.current_dir(dir).args(["generate-lockfile"]);

    if verbose {
        cmd.arg("--verbose");
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Failed to lock Rust dependencies"));
    }

    Ok(())
}

async fn lock_node_deps(dir: &Path, _verbose: bool) -> Result<()> {
    UI::step("Locking Node.js dependencies...");

    // Detect package manager
    let pm = if dir.join("yarn.lock").exists() {
        "yarn"
    } else if dir.join("pnpm-lock.yaml").exists() {
        "pnpm"
    } else {
        "npm"
    };

    let mut cmd = Command::new(pm);
    cmd.current_dir(dir);

    match pm {
        "npm" => cmd.args(["install", "--package-lock-only"]),
        "yarn" => cmd.args(["install", "--mode", "update-lockfile"]),
        "pnpm" => cmd.args(["install", "--lockfile-only"]),
        _ => &mut cmd,
    };

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Failed to lock Node.js dependencies"));
    }

    Ok(())
}

async fn lock_python_deps(dir: &Path, _verbose: bool) -> Result<()> {
    UI::step("Locking Python dependencies...");

    if dir.join("poetry.lock").exists() || dir.join("pyproject.toml").exists() {
        // Check for poetry
        let check = Command::new("poetry").arg("--version").output();
        if check.is_ok() && check.unwrap().status.success() {
            Command::new("poetry")
                .current_dir(dir)
                .arg("lock")
                .status()?;
            return Ok(());
        }
    }

    // Try uv
    let check = Command::new("uv").arg("--version").output();
    if check.is_ok() && check.unwrap().status.success() {
        Command::new("uv").current_dir(dir).arg("lock").status()?;
        return Ok(());
    }

    // Fall back to pip-compile
    let check = Command::new("pip-compile").arg("--version").output();
    if check.is_ok() && check.unwrap().status.success() {
        Command::new("pip-compile")
            .current_dir(dir)
            .arg("requirements.in")
            .status()?;
        return Ok(());
    }

    UI::warn("No lock tool available. Install poetry, uv, or pip-tools.");
    Ok(())
}

fn detect_node_package_manager(dir: &Path, config: &vx_config::DependenciesConfig) -> String {
    // Check config first
    if let Some(node) = &config.node {
        if let Some(pm) = &node.package_manager {
            return pm.clone();
        }
    }

    // Auto-detect from lock files
    if dir.join("pnpm-lock.yaml").exists() {
        "pnpm".to_string()
    } else if dir.join("yarn.lock").exists() {
        "yarn".to_string()
    } else {
        "npm".to_string()
    }
}

fn detect_python_package_manager(dir: &Path, _config: &vx_config::DependenciesConfig) -> String {
    // Auto-detect
    if dir.join("uv.lock").exists() {
        "uv".to_string()
    } else if dir.join("poetry.lock").exists() {
        "poetry".to_string()
    } else {
        "pip".to_string()
    }
}
