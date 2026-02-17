//! MSBuild.exe bridge for vx-managed MSVC installations.
//!
//! This binary is placed at `{msvc_store}/MSBuild/Current/Bin/MSBuild.exe`
//! so that tools like node-gyp can discover it via VCINSTALLDIR.
//!
//! ## Search Order
//!
//! 1. **System VS Build Tools MSBuild.exe** — has VCTargets (Microsoft.Cpp.*.props)
//!    needed for C/C++ compilation (node-gyp, cmake, etc.)
//! 2. **System VS Community/Professional/Enterprise MSBuild.exe** — same capabilities
//! 3. **dotnet msbuild** — fallback, but lacks VCTargetsPath for C++ projects
//!
//! ## Why not just `dotnet msbuild`?
//!
//! `dotnet msbuild` does NOT include C++ build targets (VCTargetsPath), so .vcxproj
//! files fail with MSB4278: "Microsoft.Cpp.Default.props" not found.
//! The full VS MSBuild.exe includes these targets.
//!
//! ## Spectre Mitigation Auto-Detection
//!
//! Some projects (e.g., node-pty's winpty) require Spectre-mitigated libraries
//! (`<SpectreMitigation>Spectre</SpectreMitigation>` in .vcxproj). If the system
//! VS installation doesn't have Spectre libs installed, MSBuild fails with MSB8040.
//!
//! This bridge detects whether Spectre libs exist for the active MSVC toolset.
//! If they're missing, it automatically injects `/p:SpectreMitigation=false` to
//! disable the check, allowing compilation to proceed without Spectre mitigation.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// Well-known MSBuild.exe locations (VS 2022 editions + VS 2019)
const MSBUILD_SEARCH_PATHS: &[&str] = &[
    // VS 2022 Build Tools (most common for CI/headless builds)
    r"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\MSBuild\Current\Bin\MSBuild.exe",
    r"C:\Program Files\Microsoft Visual Studio\2022\BuildTools\MSBuild\Current\Bin\MSBuild.exe",
    // VS 2022 Community
    r"C:\Program Files (x86)\Microsoft Visual Studio\2022\Community\MSBuild\Current\Bin\MSBuild.exe",
    r"C:\Program Files\Microsoft Visual Studio\2022\Community\MSBuild\Current\Bin\MSBuild.exe",
    // VS 2022 Professional
    r"C:\Program Files (x86)\Microsoft Visual Studio\2022\Professional\MSBuild\Current\Bin\MSBuild.exe",
    r"C:\Program Files\Microsoft Visual Studio\2022\Professional\MSBuild\Current\Bin\MSBuild.exe",
    // VS 2022 Enterprise
    r"C:\Program Files (x86)\Microsoft Visual Studio\2022\Enterprise\MSBuild\Current\Bin\MSBuild.exe",
    r"C:\Program Files\Microsoft Visual Studio\2022\Enterprise\MSBuild\Current\Bin\MSBuild.exe",
    // VS 2019 (legacy)
    r"C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\MSBuild\Current\Bin\MSBuild.exe",
    r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\MSBuild\Current\Bin\MSBuild.exe",
    r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Professional\MSBuild\Current\Bin\MSBuild.exe",
    r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Enterprise\MSBuild\Current\Bin\MSBuild.exe",
];

fn find_system_msbuild() -> Option<PathBuf> {
    for path_str in MSBUILD_SEARCH_PATHS {
        let path = Path::new(path_str);
        if path.exists() {
            return Some(path.to_path_buf());
        }
    }
    None
}

/// Find the VS installation root that contains the given MSBuild.exe path.
///
/// MSBuild.exe is at `{vs_root}/MSBuild/Current/Bin/MSBuild.exe`,
/// so we go up 4 levels from MSBuild.exe to get the VS root.
fn find_vs_root_for_msbuild(msbuild_path: &Path) -> Option<PathBuf> {
    // MSBuild.exe -> Bin -> Current -> MSBuild -> {vs_root}
    msbuild_path
        .parent() // Bin
        .and_then(|p| p.parent()) // Current
        .and_then(|p| p.parent()) // MSBuild
        .and_then(|p| p.parent()) // {vs_root}
        .map(|p| p.to_path_buf())
}

/// Check whether Spectre-mitigated libraries are installed in the given VS installation.
///
/// Spectre libs live at: `{vs_root}/VC/Tools/MSVC/{version}/lib/{arch}/spectre/`
/// We check the latest MSVC version directory for the current architecture.
fn has_spectre_libs(vs_root: &Path) -> bool {
    let msvc_dir = vs_root.join("VC").join("Tools").join("MSVC");
    if !msvc_dir.exists() {
        return false;
    }

    // Find the latest MSVC version directory
    let latest_version = match std::fs::read_dir(&msvc_dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok())
            .max(),
        Err(_) => return false,
    };

    let Some(version) = latest_version else {
        return false;
    };

    let arch = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x64"
    };

    let spectre_dir = msvc_dir
        .join(&version)
        .join("lib")
        .join(arch)
        .join("spectre");
    spectre_dir.exists()
}

/// Check if the caller's args already contain a SpectreMitigation property override.
fn has_spectre_override(args: &[String]) -> bool {
    args.iter().any(|arg| {
        let lower = arg.to_lowercase();
        lower.contains("/p:spectremitigation=") || lower.contains("-p:spectremitigation=")
    })
}

fn find_dotnet() -> Option<PathBuf> {
    // Check well-known paths first
    let dotnet_paths = [
        r"C:\Program Files\dotnet\dotnet.exe",
        r"C:\Program Files (x86)\dotnet\dotnet.exe",
    ];
    for path_str in &dotnet_paths {
        let path = Path::new(path_str);
        if path.exists() {
            return Some(path.to_path_buf());
        }
    }

    // Search PATH
    let path_var = std::env::var("PATH").ok()?;
    for dir in path_var.split(';') {
        let candidate = Path::new(dir).join("dotnet.exe");
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn main() -> ExitCode {
    let caller_args: Vec<String> = std::env::args().skip(1).collect();

    // Strategy 1: System VS MSBuild.exe (has VCTargets for C++ builds)
    if let Some(msbuild) = find_system_msbuild() {
        let mut args = caller_args.clone();

        // Auto-detect missing Spectre libraries and disable SpectreMitigation if needed.
        // This prevents MSB8040 errors when .vcxproj files require Spectre libs
        // but the VS installation doesn't have them installed.
        if !has_spectre_override(&args)
            && let Some(vs_root) = find_vs_root_for_msbuild(&msbuild)
            && !has_spectre_libs(&vs_root)
        {
            args.push("/p:SpectreMitigation=false".to_string());
        }

        return run_command(&msbuild, &args);
    }

    // Strategy 2: dotnet msbuild (fallback, lacks VCTargetsPath for C++)
    if let Some(dotnet) = find_dotnet() {
        let mut args = vec!["msbuild".to_string()];
        args.extend(caller_args);
        return run_command(&dotnet, &args);
    }

    eprintln!("vx MSBuild bridge: no MSBuild.exe or dotnet found.");
    eprintln!("Install Visual Studio Build Tools or .NET SDK to enable C++ compilation.");
    ExitCode::from(1)
}

fn run_command(executable: &Path, args: &[String]) -> ExitCode {
    match Command::new(executable).args(args).status() {
        Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
        Err(e) => {
            eprintln!(
                "vx MSBuild bridge: failed to execute {}: {}",
                executable.display(),
                e
            );
            ExitCode::from(1)
        }
    }
}
