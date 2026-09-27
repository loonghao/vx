//! Rust toolchain ownership — who decides which Rust runs in this repository?
//!
//! # Why this module exists
//!
//! vx ships a `rust` provider that installs `rustup` into its own store. That is the
//! right behaviour on a bare machine (a slim Docker image with no Rust at all), but it
//! is wrong the moment the *repository* already declares which toolchain it wants.
//!
//! Previously vx ignored that declaration entirely. On a clean CI runner it installed
//! the `stable` toolchain into its store and — through the install fallback
//! (`rustup default stable`) — rewrote the user's **global** rustup default. Projects
//! that pinned `1.90.0` silently ran on `stable`, with no warning. For repositories
//! that publish binary wheels that means the version number and the artifact disagree.
//!
//! # Policy (highest priority first)
//!
//! 1. `rust-toolchain.toml` / `rust-toolchain` found walking up from the working
//!    directory → **rustup owns it**. vx does not install, does not put its store on
//!    `PATH`, and never changes the rustup default.
//! 2. `RUSTUP_TOOLCHAIN` set in the environment → **rustup owns it**.
//! 3. `rust = "rustup-managed"` in `vx.toml` → **rustup owns it** (explicit opt-out for
//!    repositories that intentionally keep Rust out of vx).
//! 4. Otherwise → **vx owns it** and may install into its own store. Even then it must
//!    not mutate the user's global rustup default; the toolchain is injected into the
//!    subprocess environment via `RUSTUP_TOOLCHAIN` instead.
//!
//! When both (1) and (2) are present the *reported* toolchain is the one rustup itself
//! would honour (`RUSTUP_TOOLCHAIN` wins over the toolchain file). The reported string
//! feeds the "pinned vs effective" warning, so it has to describe what actually runs.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Modern toolchain declaration file (`rust-toolchain.toml`).
pub const RUST_TOOLCHAIN_TOML: &str = "rust-toolchain.toml";
/// Legacy single-line toolchain declaration file (`rust-toolchain`).
pub const RUST_TOOLCHAIN_LEGACY: &str = "rust-toolchain";
/// Environment variable rustup honours above every toolchain file.
pub const RUSTUP_TOOLCHAIN_ENV: &str = "RUSTUP_TOOLCHAIN";
/// `vx.toml` sentinel: `rust = "rustup-managed"` means "vx, stay out of the way".
pub const RUSTUP_MANAGED: &str = "rustup-managed";

/// Runtimes that belong to the Rust ecosystem and are toolchain-governed.
///
/// `cargo:<crate>` package invocations and `clippy` resolve through `cargo`, so they
/// are intentionally not listed here.
pub fn is_rust_toolchain_runtime(name: &str) -> bool {
    matches!(
        name,
        "rust" | "rustup" | "rustc" | "cargo" | "rustfmt" | "clippy-driver"
    )
}

/// The executable rustup installs for a Rust runtime name.
///
/// `rust` is an alias for the rustup manager itself; every bundled runtime keeps its
/// own name.
pub fn executable_for(runtime: &str) -> &str {
    match runtime {
        "rust" | "rustup" => "rustup",
        other => other,
    }
}

/// Locate the system executable for a Rust runtime (`rustup`, `cargo`, `rustc`, ...).
///
/// Returns `None` when rustup is not installed — the caller decides whether that is an
/// error (a repository with a toolchain file) or a reason to fall back to vx's store.
pub fn find_rustup_executable(runtime: &str) -> Option<PathBuf> {
    which::which(executable_for(runtime)).ok()
}

/// Where a Rust toolchain override came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustToolchainSource {
    /// `rust-toolchain.toml` or legacy `rust-toolchain`.
    ToolchainFile { path: PathBuf, toolchain: String },
    /// The `RUSTUP_TOOLCHAIN` environment variable.
    Env { toolchain: String },
    /// `rust = "rustup-managed"` in `vx.toml`.
    VxTomlOptOut,
}

impl RustToolchainSource {
    /// The toolchain this source selects, if it names one.
    pub fn toolchain(&self) -> Option<&str> {
        match self {
            Self::ToolchainFile { toolchain, .. } | Self::Env { toolchain } => Some(toolchain),
            Self::VxTomlOptOut => None,
        }
    }

    /// Human-readable description, for warnings and `vx check` output.
    pub fn describe(&self) -> String {
        match self {
            Self::ToolchainFile { path, toolchain } => {
                format!("{} (channel = \"{}\")", path.display(), toolchain)
            }
            Self::Env { toolchain } => format!("{RUSTUP_TOOLCHAIN_ENV}={toolchain}"),
            Self::VxTomlOptOut => format!("vx.toml ([tools] rust = \"{RUSTUP_MANAGED}\")"),
        }
    }
}

/// Who owns the Rust toolchain for a given request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustToolchainOwner {
    /// rustup owns the toolchain. vx must not install, must not put its store on
    /// `PATH`, and must never change the rustup default.
    Rustup {
        /// The channel / toolchain the repository asked for, when it names one.
        channel: Option<String>,
        source: RustToolchainSource,
    },
    /// vx owns it and may install Rust into its own store.
    Vx,
}

impl RustToolchainOwner {
    /// True when an external rustup installation owns the toolchain.
    pub fn is_rustup_managed(&self) -> bool {
        matches!(self, Self::Rustup { .. })
    }

    /// The channel the owner selected, if it named one.
    pub fn channel(&self) -> Option<&str> {
        match self {
            Self::Rustup { channel, .. } => channel.as_deref(),
            Self::Vx => None,
        }
    }

    /// Why this owner was chosen — used in log lines and `vx check` output.
    pub fn reason(&self) -> String {
        match self {
            Self::Rustup { source, .. } => source.describe(),
            Self::Vx => "no rust-toolchain file, RUSTUP_TOOLCHAIN unset".to_string(),
        }
    }

    /// vx.toml explicitly opted out of managing Rust (`rust = "rustup-managed"`).
    pub fn rustup_opt_out() -> Self {
        Self::Rustup {
            channel: None,
            source: RustToolchainSource::VxTomlOptOut,
        }
    }
}

/// Decide who owns the Rust toolchain for commands run in `working_dir`.
///
/// Uses the process environment for `RUSTUP_TOOLCHAIN`. The toolchain file lookup
/// walks up from `working_dir`, so a `rust-toolchain.toml` at the repository root
/// still applies from a subdirectory.
pub fn detect_toolchain_owner(working_dir: &Path) -> RustToolchainOwner {
    detect_toolchain_owner_with_env(working_dir, &std::env::vars().collect())
}

/// `detect_toolchain_owner` with an explicit environment (used by tests).
pub fn detect_toolchain_owner_with_env(
    working_dir: &Path,
    env: &HashMap<String, String>,
) -> RustToolchainOwner {
    match detect_toolchain_override(working_dir, env) {
        Some(source) => RustToolchainOwner::Rustup {
            channel: source.toolchain().map(str::to_string),
            source,
        },
        None => RustToolchainOwner::Vx,
    }
}

/// Detect a Rust toolchain override for `start_dir`.
///
/// `start_dir` is normally the working directory of the command. The lookup walks up
/// the directory tree, mirroring rustup's own directory-override search, so a
/// `rust-toolchain.toml` at the repository root still applies from a subdirectory.
///
/// `env` is consulted for `RUSTUP_TOOLCHAIN` and takes precedence over a toolchain
/// file, because that is the toolchain rustup will actually run.
pub fn detect_toolchain_override(
    start_dir: &Path,
    env: &HashMap<String, String>,
) -> Option<RustToolchainSource> {
    if let Some(toolchain) = env
        .get(RUSTUP_TOOLCHAIN_ENV)
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
    {
        return Some(RustToolchainSource::Env {
            toolchain: toolchain.to_string(),
        });
    }

    find_toolchain_file(start_dir)
        .map(|(path, toolchain)| RustToolchainSource::ToolchainFile { path, toolchain })
}

/// Look for `rust-toolchain.toml` / `rust-toolchain` in `dir` only (no upward walk).
pub fn detect_toolchain_file_in_dir(dir: &Path) -> Option<(PathBuf, String)> {
    let toml_path = dir.join(RUST_TOOLCHAIN_TOML);
    if let Some(toolchain) = parse_toolchain_toml(&toml_path) {
        return Some((toml_path, toolchain));
    }

    let legacy_path = dir.join(RUST_TOOLCHAIN_LEGACY);
    if let Some(toolchain) = parse_legacy_toolchain_file(&legacy_path) {
        return Some((legacy_path, toolchain));
    }

    None
}

/// Walk up from `start_dir` looking for a toolchain declaration file.
pub fn find_toolchain_file(start_dir: &Path) -> Option<(PathBuf, String)> {
    let mut current = Some(start_dir);

    while let Some(dir) = current {
        if let Some(found) = detect_toolchain_file_in_dir(dir) {
            return Some(found);
        }
        current = dir.parent();
    }

    None
}

/// Whether a pinned version and an effective version disagree.
///
/// Only concrete numeric versions are compared. Channels (`stable`, `beta`,
/// `nightly`, `latest`, `*`) and the `rustup-managed` sentinel never conflict — there
/// is nothing meaningful to warn about, and warning would be noise.
///
/// Missing components are treated as zero, so `1.83` and `1.83.0` agree.
pub fn versions_conflict(pinned: &str, effective: &str) -> bool {
    match (normalize_version(pinned), normalize_version(effective)) {
        (Some(a), Some(b)) => a != b,
        _ => false,
    }
}

/// Reduce a version string to `major.minor.patch`, or `None` if it is not numeric.
fn normalize_version(value: &str) -> Option<String> {
    let value = value.trim();
    let value = value
        .strip_prefix('v')
        .or_else(|| value.strip_prefix('V'))
        .unwrap_or(value);

    if !value.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }

    let numeric: String = value
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();

    let mut parts = numeric.split('.');
    let major = parts.next().unwrap_or("0");
    let minor = parts.next().unwrap_or("0");
    let patch = parts.next().unwrap_or("0");

    Some(format!("{}.{}.{}", pad(major), pad(minor), pad(patch)))
}

fn pad(component: &str) -> String {
    if component.is_empty() {
        "0".to_string()
    } else {
        component.to_string()
    }
}

/// Read the `channel` (or `path`) key out of a `rust-toolchain.toml`.
fn parse_toolchain_toml(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let parsed: toml::Value = toml::from_str(&content).ok()?;
    let toolchain = parsed.get("toolchain")?;

    for key in ["channel", "path"] {
        if let Some(value) = toolchain.get(key).and_then(|v| v.as_str()) {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }

    None
}

/// Read the legacy `rust-toolchain` file — first non-empty, non-comment line.
fn parse_legacy_toolchain_file(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;

    content.lines().map(str::trim).find_map(|line| {
        if line.is_empty() || line.starts_with('#') {
            None
        } else {
            Some(line.to_string())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_with(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn write_toolchain_toml(dir: &Path, body: &str) {
        std::fs::write(dir.join(RUST_TOOLCHAIN_TOML), body).unwrap();
    }

    #[test]
    fn test_is_rust_toolchain_runtime() {
        for name in ["rust", "rustup", "rustc", "cargo", "rustfmt"] {
            assert!(
                is_rust_toolchain_runtime(name),
                "{name} should be a rust runtime"
            );
        }
        for name in ["node", "python", "go", "cargo:ripgrep", ""] {
            assert!(
                !is_rust_toolchain_runtime(name),
                "{name} should not be a rust runtime"
            );
        }
    }

    #[test]
    fn test_executable_for() {
        assert_eq!(executable_for("rust"), "rustup");
        assert_eq!(executable_for("rustup"), "rustup");
        assert_eq!(executable_for("cargo"), "cargo");
        assert_eq!(executable_for("rustc"), "rustc");
    }

    #[test]
    fn test_toolchain_toml_channel() {
        let dir = tempfile::tempdir().unwrap();
        write_toolchain_toml(
            dir.path(),
            "[toolchain]\nchannel = \"1.83.0\"\ncomponents = [\"rustfmt\"]\n",
        );

        let source = detect_toolchain_override(dir.path(), &HashMap::new()).unwrap();

        assert_eq!(source.toolchain(), Some("1.83.0"));
        assert!(matches!(source, RustToolchainSource::ToolchainFile { .. }));
    }

    #[test]
    fn test_toolchain_toml_path_key() {
        let dir = tempfile::tempdir().unwrap();
        write_toolchain_toml(dir.path(), "[toolchain]\npath = \"/opt/custom-rust\"\n");

        let source = detect_toolchain_override(dir.path(), &HashMap::new()).unwrap();
        assert_eq!(source.toolchain(), Some("/opt/custom-rust"));
    }

    #[test]
    fn test_legacy_toolchain_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(RUST_TOOLCHAIN_LEGACY),
            "nightly-2024-01-01\n",
        )
        .unwrap();

        let source = detect_toolchain_override(dir.path(), &HashMap::new()).unwrap();
        assert_eq!(source.toolchain(), Some("nightly-2024-01-01"));
    }

    #[test]
    fn test_legacy_toolchain_file_skips_comments() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(RUST_TOOLCHAIN_LEGACY),
            "# pinned for MSRV\n\n1.70.0\n",
        )
        .unwrap();

        let source = detect_toolchain_override(dir.path(), &HashMap::new()).unwrap();
        assert_eq!(source.toolchain(), Some("1.70.0"));
    }

    #[test]
    fn test_env_overrides_toolchain_file() {
        let dir = tempfile::tempdir().unwrap();
        write_toolchain_toml(dir.path(), "[toolchain]\nchannel = \"1.83.0\"\n");

        // rustup honours RUSTUP_TOOLCHAIN above the toolchain file, so the reported
        // toolchain must match what will actually run.
        let source =
            detect_toolchain_override(dir.path(), &env_with(&[(RUSTUP_TOOLCHAIN_ENV, "1.90.0")]))
                .unwrap();

        assert_eq!(source.toolchain(), Some("1.90.0"));
        assert!(matches!(source, RustToolchainSource::Env { .. }));
    }

    #[test]
    fn test_blank_env_is_ignored() {
        let dir = tempfile::tempdir().unwrap();
        write_toolchain_toml(dir.path(), "[toolchain]\nchannel = \"1.83.0\"\n");

        let source =
            detect_toolchain_override(dir.path(), &env_with(&[(RUSTUP_TOOLCHAIN_ENV, "   ")]))
                .unwrap();
        assert_eq!(source.toolchain(), Some("1.83.0"));
        assert!(matches!(source, RustToolchainSource::ToolchainFile { .. }));
    }

    #[test]
    fn test_find_toolchain_file_walks_up() {
        let dir = tempfile::tempdir().unwrap();
        write_toolchain_toml(dir.path(), "[toolchain]\nchannel = \"1.83.0\"\n");

        let nested = dir.path().join("crates").join("vx-cli").join("src");
        std::fs::create_dir_all(&nested).unwrap();

        let (path, toolchain) = find_toolchain_file(&nested).unwrap();
        assert_eq!(toolchain, "1.83.0");
        assert_eq!(path, dir.path().join(RUST_TOOLCHAIN_TOML));
    }

    #[test]
    fn test_detect_toolchain_owner_vx_without_override() {
        let dir = tempfile::tempdir().unwrap();

        assert_eq!(
            detect_toolchain_owner_with_env(dir.path(), &HashMap::new()),
            RustToolchainOwner::Vx
        );
    }

    #[test]
    fn test_detect_toolchain_owner_rustup_on_toolchain_file() {
        let dir = tempfile::tempdir().unwrap();
        write_toolchain_toml(dir.path(), "[toolchain]\nchannel = \"1.83.0\"\n");

        let owner = detect_toolchain_owner_with_env(dir.path(), &HashMap::new());

        match owner {
            RustToolchainOwner::Rustup { channel, source } => {
                assert_eq!(channel.as_deref(), Some("1.83.0"));
                assert!(matches!(source, RustToolchainSource::ToolchainFile { .. }));
            }
            other => panic!("expected rustup ownership, got {other:?}"),
        }
    }

    #[test]
    fn test_detect_toolchain_owner_rustup_on_env() {
        let dir = tempfile::tempdir().unwrap();

        let owner = detect_toolchain_owner_with_env(
            dir.path(),
            &env_with(&[(RUSTUP_TOOLCHAIN_ENV, "1.90.0")]),
        );

        assert!(owner.is_rustup_managed());
        assert_eq!(owner.channel(), Some("1.90.0"));
    }

    #[test]
    fn test_rustup_opt_out_owner() {
        let owner = RustToolchainOwner::rustup_opt_out();
        assert!(owner.is_rustup_managed());
        assert_eq!(owner.channel(), None);
        assert!(
            owner.reason().contains(RUSTUP_MANAGED),
            "{}",
            owner.reason()
        );
    }

    #[test]
    fn test_vx_owner_reason_is_explanatory() {
        assert!(!RustToolchainOwner::Vx.is_rustup_managed());
        assert_eq!(RustToolchainOwner::Vx.channel(), None);
    }

    #[test]
    fn test_versions_conflict_numeric() {
        assert!(versions_conflict("1.93.1", "1.83.0"));
        assert!(versions_conflict("1.83", "1.90.0"));
        assert!(versions_conflict("1.90.0", "1.90.1"));
    }

    #[test]
    fn test_versions_conflict_equivalent() {
        assert!(!versions_conflict("1.83.0", "1.83.0"));
        assert!(!versions_conflict("1.83", "1.83.0"));
        assert!(!versions_conflict("v1.83.0", "1.83.0"));
    }

    #[test]
    fn test_versions_conflict_ignores_channels() {
        assert!(!versions_conflict("stable", "1.83.0"));
        assert!(!versions_conflict("1.83.0", "stable"));
        assert!(!versions_conflict("nightly", "beta"));
        assert!(!versions_conflict("latest", "1.99.0"));
        assert!(!versions_conflict(RUSTUP_MANAGED, "1.99.0"));
        assert!(!versions_conflict("1.99.0", "*"));
    }

    #[test]
    fn test_source_describe_is_human_readable() {
        let dir = tempfile::tempdir().unwrap();
        write_toolchain_toml(dir.path(), "[toolchain]\nchannel = \"1.83.0\"\n");

        let source = detect_toolchain_override(dir.path(), &HashMap::new()).unwrap();
        let text = source.describe();
        assert!(text.contains("1.83.0"), "{text}");
        assert!(text.contains(RUST_TOOLCHAIN_TOML), "{text}");

        let env_source = RustToolchainSource::Env {
            toolchain: "1.90.0".to_string(),
        };
        assert_eq!(env_source.describe(), "RUSTUP_TOOLCHAIN=1.90.0");
    }
}
