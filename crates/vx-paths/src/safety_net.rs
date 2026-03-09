//! Safety net for child process environment
//!
//! This module provides functions to ensure that child processes always have
//! access to fundamental system executables (cmd.exe, sh, bash, etc.) and
//! essential environment variables, regardless of how the parent environment
//! was constructed.
//!
//! Both `vx-resolver`'s `command.rs` (final command execution) and
//! `environment.rs` (runtime environment preparation) use these helpers to
//! avoid duplicating the same defensive logic.

use std::collections::HashMap;
use std::path::Path;

/// Essential Windows environment variables that must always be present
/// for child processes to function correctly.
///
/// Without these, fundamental operations like `cmd /c "..."` or PowerShell
/// script execution will fail because the system cannot locate executables.
#[cfg(windows)]
pub const WINDOWS_ESSENTIAL_ENV_VARS: &[&str] = &[
    "SYSTEMROOT",             // C:\Windows — needed for cmd.exe, system DLLs
    "SYSTEMDRIVE",            // C: — base drive letter
    "WINDIR",                 // C:\Windows — legacy alias for SYSTEMROOT
    "COMSPEC",                // C:\Windows\System32\cmd.exe — default command processor
    "PATHEXT",                // .COM;.EXE;.BAT;.CMD;... — executable extensions
    "OS",                     // Windows_NT — OS identification
    "PROCESSOR_ARCHITECTURE", // AMD64/ARM64 — needed by build tools
    "NUMBER_OF_PROCESSORS",   // CPU count — used by parallel builds
];

/// Get essential system paths that must always be present in PATH.
///
/// These paths contain fundamental system executables (cmd.exe, powershell,
/// sh, bash, etc.) that child processes expect to find.
///
/// On Windows, derives paths from `SYSTEMROOT` env var (defaults to
/// `C:\Windows`). On Unix, returns the standard `/bin`, `/usr/bin`,
/// `/usr/local/bin` directories.
pub fn essential_system_paths() -> Vec<String> {
    let mut paths = Vec::new();

    #[cfg(windows)]
    {
        let system_root =
            std::env::var("SYSTEMROOT").unwrap_or_else(|_| r"C:\Windows".to_string());

        // System32 — contains cmd.exe, powershell.exe, and most system utilities
        let system32 = format!(r"{}\System32", system_root);
        paths.push(system32.clone());

        // Wbem — Windows Management Instrumentation tools
        paths.push(format!(r"{}\Wbem", system32));

        // Windows PowerShell 5.x
        paths.push(format!(r"{}\WindowsPowerShell\v1.0", system32));

        // SYSTEMROOT itself (contains some executables)
        paths.push(system_root);

        // PowerShell 7+ (if installed)
        if let Ok(pf) = std::env::var("ProgramFiles") {
            let ps7 = format!(r"{}\PowerShell\7", pf);
            if Path::new(&ps7).exists() {
                paths.push(ps7);
            }
        }
    }

    #[cfg(unix)]
    {
        paths.extend([
            "/bin".to_string(),
            "/usr/bin".to_string(),
            "/usr/local/bin".to_string(),
        ]);
    }

    paths
}

/// Ensure essential system paths are present in the given path list.
///
/// Appends any missing essential paths (that actually exist on disk) to
/// `path_parts`. Uses case-insensitive comparison on Windows.
///
/// Returns `true` if any paths were added.
pub fn ensure_essential_paths(path_parts: &mut Vec<String>) -> bool {
    let essential = essential_system_paths();
    let mut added_any = false;

    for ep in &essential {
        let already_present = {
            #[cfg(windows)]
            {
                let ep_lower = ep.to_lowercase();
                path_parts.iter().any(|p| p.to_lowercase() == ep_lower)
            }
            #[cfg(not(windows))]
            {
                path_parts.iter().any(|p| p == ep)
            }
        };

        if !already_present && Path::new(ep).exists() {
            path_parts.push(ep.clone());
            added_any = true;
        }
    }

    added_any
}

/// Ensure essential Windows system environment variables are present in `env`.
///
/// When a runtime environment is built from scratch (or filtered), variables
/// like `SYSTEMROOT`, `COMSPEC`, and `PATHEXT` may be missing. Without them,
/// child processes that invoke `cmd.exe` or PowerShell will fail with errors
/// like "'cmd' is not recognized as an internal or external command".
///
/// This is a no-op on non-Windows platforms.
pub fn ensure_essential_env_vars(env: &mut HashMap<String, String>) {
    #[cfg(windows)]
    {
        for var_name in WINDOWS_ESSENTIAL_ENV_VARS {
            if !env.keys().any(|k| k.eq_ignore_ascii_case(var_name)) {
                if let Ok(value) = std::env::var(var_name) {
                    env.insert(var_name.to_string(), value);
                }
            }
        }
    }
    #[cfg(not(windows))]
    let _ = env; // suppress unused warning
}

/// Ensure the directory containing the current `vx` executable is in PATH.
///
/// This allows sub-processes (e.g., `just` recipes calling `vx npm ci`) to
/// find the `vx` binary without requiring it to be on the system PATH.
///
/// The directory is appended (not prepended) so it doesn't shadow other tools.
pub fn ensure_vx_in_path(env: &mut HashMap<String, String>) {
    if let Ok(current_exe) = std::env::current_exe()
        && let Some(exe_dir) = current_exe.parent()
    {
        let exe_dir_str = exe_dir.to_string_lossy().to_string();
        let current_path = env.get("PATH").cloned().unwrap_or_default();

        let already_present = {
            #[cfg(windows)]
            {
                current_path
                    .to_lowercase()
                    .contains(&exe_dir_str.to_lowercase())
            }
            #[cfg(not(windows))]
            {
                current_path.contains(&exe_dir_str)
            }
        };

        if !already_present {
            let sep = if cfg!(windows) { ";" } else { ":" };
            let new_path = if current_path.is_empty() {
                exe_dir_str
            } else {
                format!("{}{}{}", current_path, sep, exe_dir_str)
            };
            env.insert("PATH".to_string(), new_path);
        }
    }
}

/// Apply the full safety net to an environment map.
///
/// Convenience function that calls:
/// 1. [`ensure_essential_paths`] — adds missing essential PATH entries
/// 2. [`ensure_essential_env_vars`] — adds missing Windows system env vars
/// 3. [`ensure_vx_in_path`] — ensures `vx` itself is findable
///
/// This is the recommended entry point for both `command.rs` and
/// `environment.rs`.
pub fn apply_safety_net(env: &mut HashMap<String, String>) {
    // Step 1: ensure essential system paths
    {
        let current_path = env
            .get("PATH")
            .cloned()
            .or_else(|| std::env::var("PATH").ok())
            .unwrap_or_default();

        let mut path_parts: Vec<String> = super::split_path(&current_path)
            .map(String::from)
            .collect();

        if ensure_essential_paths(&mut path_parts) {
            if let Ok(new_path) = std::env::join_paths(&path_parts) {
                env.insert("PATH".to_string(), new_path.to_string_lossy().to_string());
            }
        }
    }

    // Step 2: ensure Windows essential env vars
    ensure_essential_env_vars(env);

    // Step 3: ensure vx is findable
    ensure_vx_in_path(env);
}
