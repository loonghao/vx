use crate::VersionRequest;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Default system environment variables to inherit when isolated.
///
/// These are essential for child processes (e.g., shell scripts, postinstall hooks)
/// to function correctly. Providers can extend this list with `extra_inherit_system_vars`.
///
/// **Note**: PATH is handled specially - see [`SYSTEM_PATH_PREFIXES`] and [`filter_system_path`].
///
/// Categories:
/// - User/Session: HOME, USER, USERNAME, USERPROFILE, LOGNAME
/// - Shell: SHELL, TERM, COLORTERM
/// - Locale: LANG, LANGUAGE, LC_*
/// - Timezone: TZ
/// - Temp dirs: TMPDIR, TEMP, TMP
/// - Display: DISPLAY, WAYLAND_DISPLAY (for GUI apps)
/// - XDG: XDG_* (Linux desktop integration)
pub const DEFAULT_INHERIT_SYSTEM_VARS: &[&str] = &[
    // User and session
    "HOME",
    "USER",
    "USERNAME",    // Windows
    "USERPROFILE", // Windows
    "LOGNAME",
    // Shell and terminal
    "SHELL",
    "TERM",
    "COLORTERM",
    // Locale
    "LANG",
    "LANGUAGE",
    "LC_*", // Glob pattern for all LC_ variables
    // Timezone
    "TZ",
    // Temporary directories
    "TMPDIR", // Unix
    "TEMP",   // Windows
    "TMP",    // Windows alternative
    // Display (for GUI apps)
    "DISPLAY",
    "WAYLAND_DISPLAY",
    // XDG directories (Linux)
    "XDG_*", // Glob pattern for XDG_RUNTIME_DIR, XDG_CONFIG_HOME, etc.
];

/// System PATH prefixes that should be inherited in isolated mode.
///
/// These directories contain essential system tools (sh, bash, cat, etc.)
/// that child processes may need. User-specific directories (like ~/.local/bin)
/// are intentionally excluded to maintain isolation.
///
/// ## Unix (Linux/macOS)
/// - `/bin` - Essential user commands (sh, bash, ls, cat, etc.)
/// - `/usr/bin` - Standard user commands
/// - `/usr/local/bin` - Locally installed programs
/// - `/sbin` - System binaries (root commands)
/// - `/usr/sbin` - Non-essential system binaries
/// - `/usr/local/sbin` - Local system binaries
/// - `/opt/homebrew/bin` - Homebrew on Apple Silicon
/// - `/opt/homebrew/sbin` - Homebrew system binaries
///
/// ## Windows
/// - `C:\Windows\System32` - Core Windows executables
/// - `C:\Windows\SysWOW64` - 32-bit compatibility layer
/// - `C:\Windows` - Windows directory
/// - `C:\Windows\System32\Wbem` - WMI tools
/// - `C:\Windows\System32\WindowsPowerShell` - PowerShell
/// - `C:\Windows\System32\OpenSSH` - OpenSSH client
pub const SYSTEM_PATH_PREFIXES: &[&str] = &[
    // Unix essential directories
    "/bin",
    "/usr/bin",
    "/usr/local/bin",
    "/sbin",
    "/usr/sbin",
    "/usr/local/sbin",
    // macOS Homebrew (Apple Silicon)
    "/opt/homebrew/bin",
    "/opt/homebrew/sbin",
    // macOS Homebrew (Intel)
    "/usr/local/Cellar",
    // Nix
    "/nix/var/nix/profiles/default/bin",
    "/run/current-system/sw/bin",
    // Windows (case-insensitive matching will be used)
    "C:\\Windows\\System32",
    "C:\\Windows\\SysWOW64",
    "C:\\Windows",
    "C:\\Windows\\System32\\Wbem",
    "C:\\Windows\\System32\\WindowsPowerShell",
    "C:\\Windows\\System32\\OpenSSH",
];

/// Filter the system PATH to only include essential system directories.
///
/// This function takes the full PATH and returns only the directories that
/// match [`SYSTEM_PATH_PREFIXES`], maintaining environment isolation while
/// ensuring child processes can find basic tools like `sh`, `bash`, etc.
///
/// # Arguments
/// * `path` - The full PATH string (colon-separated on Unix, semicolon on Windows)
///
/// # Returns
/// A filtered PATH string containing only system directories
///
/// # Example (Unix)
/// ```ignore
/// use vx_manifest::filter_system_path;
///
/// let full_path = "/home/user/.local/bin:/usr/local/bin:/usr/bin:/bin";
/// let filtered = filter_system_path(full_path);
/// assert_eq!(filtered, "/usr/local/bin:/usr/bin:/bin");
/// ```
///
/// # Example (Windows)
/// ```ignore
/// use vx_manifest::filter_system_path;
///
/// let full_path = "C:\\Users\\user\\.local\\bin;C:\\Windows\\System32;C:\\Windows";
/// let filtered = filter_system_path(full_path);
/// assert_eq!(filtered, "C:\\Windows\\System32;C:\\Windows");
/// ```
pub fn filter_system_path(path: &str) -> String {
    let separator = if cfg!(windows) { ';' } else { ':' };

    let filtered: Vec<&str> = path
        .split(separator)
        .filter(|entry| {
            let entry_path = Path::new(entry);
            is_system_path(entry_path)
        })
        .collect();

    filtered.join(&separator.to_string())
}

/// Check if a path is a system path that should be inherited.
fn is_system_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();

    for prefix in SYSTEM_PATH_PREFIXES {
        if cfg!(windows) {
            // Case-insensitive comparison on Windows
            if path_str.to_lowercase().starts_with(&prefix.to_lowercase()) {
                return true;
            }
        } else {
            // Case-sensitive on Unix
            if path_str.starts_with(prefix) {
                return true;
            }
            // Also check if it equals the prefix exactly
            if path_str.as_ref() == *prefix {
                return true;
            }
        }
    }

    false
}

/// Environment variable configuration
///
/// Supports static, dynamic (template), and conditional environment variables.
/// Template variables:
/// - `{install_dir}` - Installation directory
/// - `{version}` - Current version
/// - `{executable}` - Executable path
/// - `{PATH}` - Original PATH value
/// - `{env:VAR}` - Reference other environment variable
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct EnvConfig {
    /// Static environment variables
    #[serde(default)]
    pub vars: std::collections::HashMap<String, String>,

    /// Conditional environment variables (version-based)
    /// Key is version constraint (e.g., ">=18"), value is env vars
    #[serde(default)]
    pub conditional: std::collections::HashMap<String, std::collections::HashMap<String, String>>,

    /// Advanced environment configuration (similar to rez)
    #[serde(default)]
    pub advanced: Option<AdvancedEnvConfig>,
}

/// Advanced environment configuration similar to rez
///
/// Supports PATH manipulation, environment inheritance control, and isolation.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AdvancedEnvConfig {
    /// PATH prepend entries (added before existing PATH)
    #[serde(default)]
    pub path_prepend: Vec<String>,

    /// PATH append entries (added after existing PATH)
    #[serde(default)]
    pub path_append: Vec<String>,

    /// Environment variables with special handling
    #[serde(default)]
    pub env_vars: std::collections::HashMap<String, EnvVarConfig>,

    /// Whether to isolate from system environment by default
    #[serde(default = "default_isolate_env")]
    pub isolate: bool,

    /// Which system environment variables to inherit when isolated.
    ///
    /// **Note**: This field is additive to `DEFAULT_INHERIT_SYSTEM_VARS`.
    /// The default system vars are always included. Use this field to add
    /// provider-specific variables (e.g., `SSH_AUTH_SOCK`, `GPG_TTY` for git).
    ///
    /// If you need to override the defaults entirely, use `inherit_system_vars_override`.
    #[serde(default)]
    pub inherit_system_vars: Vec<String>,

    /// Override the default system variables completely (advanced use only).
    ///
    /// When set to `true`, `inherit_system_vars` replaces the defaults
    /// instead of extending them.
    #[serde(default)]
    pub inherit_system_vars_override: bool,
}

/// Configuration for individual environment variables
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum EnvVarConfig {
    /// Simple string value
    Simple(String),
    /// Advanced configuration with prepend/append
    Advanced {
        /// Value to set
        value: Option<String>,
        /// Prepend to existing value (separated by OS-specific separator)
        prepend: Option<Vec<String>>,
        /// Append to existing value (separated by OS-specific separator)
        append: Option<Vec<String>>,
        /// Replace existing value entirely
        #[serde(default)]
        replace: bool,
    },
}

fn default_isolate_env() -> bool {
    true
}

impl EnvConfig {
    /// Get environment variables for a specific version
    pub fn get_vars_for_version(&self, version: &str) -> std::collections::HashMap<String, String> {
        let mut result = self.vars.clone();

        for (constraint, vars) in &self.conditional {
            let req = VersionRequest::parse(constraint);
            if req.satisfies(version) {
                result.extend(vars.clone());
            }
        }

        result
    }

    /// Check if there are any environment variables configured
    pub fn is_empty(&self) -> bool {
        self.vars.is_empty() && self.conditional.is_empty() && self.advanced.is_none()
    }

    /// Check if environment isolation is enabled
    pub fn is_isolated(&self) -> bool {
        self.advanced.as_ref().map(|a| a.isolate).unwrap_or(true)
    }

    /// Get system variables to inherit when isolated (legacy method)
    #[deprecated(
        since = "0.5.0",
        note = "Use `effective_inherit_system_vars()` instead which includes defaults"
    )]
    pub fn inherit_system_vars(&self) -> &[String] {
        self.advanced
            .as_ref()
            .map(|a| &a.inherit_system_vars[..])
            .unwrap_or(&[])
    }

    /// Get the effective list of system variables to inherit.
    ///
    /// This combines `DEFAULT_INHERIT_SYSTEM_VARS` with provider-specific
    /// `inherit_system_vars`, unless `inherit_system_vars_override` is set.
    pub fn effective_inherit_system_vars(&self) -> Vec<String> {
        match &self.advanced {
            Some(advanced) if advanced.inherit_system_vars_override => {
                // Override mode: use only the explicitly specified vars
                advanced.inherit_system_vars.clone()
            }
            Some(advanced) => {
                // Additive mode: defaults + extra
                let mut result: Vec<String> = DEFAULT_INHERIT_SYSTEM_VARS
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                for var in &advanced.inherit_system_vars {
                    if !result.contains(var) {
                        result.push(var.clone());
                    }
                }
                result
            }
            None => {
                // No advanced config: use defaults
                DEFAULT_INHERIT_SYSTEM_VARS
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            }
        }
    }
}
