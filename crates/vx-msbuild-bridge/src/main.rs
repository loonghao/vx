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
        return run_command(&msbuild, &caller_args);
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
