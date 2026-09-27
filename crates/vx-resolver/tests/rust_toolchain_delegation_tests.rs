//! Regression tests for Rust toolchain ownership (PIP-3732).
//!
//! A repository with a `rust-toolchain.toml` has already decided which toolchain it
//! wants. vx used to ignore that and resolve `cargo` / `rustc` from its own store, so
//! a project pinning `1.90.0` silently ran on `stable` — and the install fallback
//! (`rustup default stable`) rewrote the user's global rustup default for every other
//! project on the machine.
//!
//! These tests pin the contract: when rustup owns the toolchain, vx installs nothing,
//! injects no store paths, and leaves the global default alone.
//!
//! Two notes on how this file is written:
//!
//! * `cargo test` runs under rustup's `cargo` shim, which exports `RUSTUP_TOOLCHAIN`.
//!   Every test therefore sets that variable explicitly, so results do not depend on
//!   how the suite was launched.
//! * The tests are synchronous and share a `std::sync::Mutex`. The process environment
//!   is process-global and not thread-safe, so mutation has to be serialized — and a
//!   std mutex must not be held across an await point.

use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use vx_resolver::{
    EnsureStage, ExecuteStage, ExecutionPlan, PrepareStage, ProjectToolsConfig,
    RUST_TOOLCHAIN_TOML, RUSTUP_MANAGED, RUSTUP_TOOLCHAIN_ENV, ResolveStage, Resolver,
    ResolverConfig, RuntimeMap, Stage, VersionResolution,
};
use vx_runtime::mock_context;

/// Serializes all access to the process environment from this test binary.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn env_lock() -> MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

/// Sets (or clears) `RUSTUP_TOOLCHAIN` for the duration of a test, restoring the
/// previous value on drop. The caller must hold [`env_lock`].
struct EnvOverride {
    saved: Option<String>,
}

impl EnvOverride {
    fn set_rustup_toolchain(value: Option<&str>) -> Self {
        let saved = std::env::var(RUSTUP_TOOLCHAIN_ENV).ok();
        // SAFETY: the caller holds ENV_LOCK, so no other test observes or mutates the
        // process environment while this override is in place.
        unsafe {
            match value {
                Some(v) => std::env::set_var(RUSTUP_TOOLCHAIN_ENV, v),
                None => std::env::remove_var(RUSTUP_TOOLCHAIN_ENV),
            }
        }
        Self { saved }
    }
}

impl Drop for EnvOverride {
    fn drop(&mut self) {
        // SAFETY: same as above — restored while ENV_LOCK is still held.
        unsafe {
            match &self.saved {
                Some(v) => std::env::set_var(RUSTUP_TOOLCHAIN_ENV, v),
                None => std::env::remove_var(RUSTUP_TOOLCHAIN_ENV),
            }
        }
    }
}

/// Drive an async pipeline stage to completion without an await point, so the env
/// lock above never has to be held across one.
fn block_on<F: Future>(future: F) -> F::Output {
    tokio::runtime::Runtime::new()
        .expect("tokio runtime")
        .block_on(future)
}

/// Skip the test when rustup is not installed — there is nothing to delegate to.
fn require_rustup() -> bool {
    which::which("rustup").is_ok()
}

fn resolver() -> Resolver {
    Resolver::new(ResolverConfig::default(), RuntimeMap::empty()).unwrap()
}

fn fixture_with_toolchain_file(channel: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join(RUST_TOOLCHAIN_TOML),
        format!("[toolchain]\nchannel = \"{channel}\"\n"),
    )
    .unwrap();
    dir
}

fn project_config(pins: &[(&str, &str)]) -> ProjectToolsConfig {
    ProjectToolsConfig::from_tools(
        pins.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<HashMap<_, _>>(),
    )
}

/// Resolve `runtime` in `working_dir` with the given `vx.toml` pins.
fn resolve_in(runtime: &str, working_dir: &Path, pins: &[(&str, &str)]) -> ExecutionPlan {
    let resolver = resolver();
    let config = ResolverConfig::default();
    let project_config = project_config(pins);

    let stage = ResolveStage::new(&resolver, &config).with_project_config(&project_config);

    let mut request = vx_resolver::ResolveRequest::new(runtime, vec!["--version".to_string()]);
    request.working_dir = Some(working_dir.to_path_buf());

    block_on(stage.execute(request)).unwrap()
}

/// The reported channel when the plan delegates, or `None` when vx keeps ownership.
fn delegated_channel(plan: &ExecutionPlan) -> Option<String> {
    match &plan.primary.version {
        VersionResolution::SystemAvailable { version, .. } => version.clone(),
        _ => None,
    }
}

#[test]
fn test_toolchain_file_delegates_cargo_to_rustup() {
    if !require_rustup() {
        return;
    }
    let _lock = env_lock();
    let _env = EnvOverride::set_rustup_toolchain(None);

    // The vx.toml pin deliberately disagrees with the toolchain file: honouring the
    // repository is the whole point, and the disagreement must never pull vx's own
    // toolchain in.
    let fixture = fixture_with_toolchain_file("1.83.0");
    let plan = resolve_in("cargo", fixture.path(), &[("rust", "1.93.1")]);

    assert_eq!(
        delegated_channel(&plan).as_deref(),
        Some("1.83.0"),
        "the rust-toolchain.toml channel must win over the vx.toml pin"
    );

    let executable = plan
        .primary
        .executable
        .clone()
        .expect("executable resolved");
    assert_eq!(
        executable.file_stem().and_then(|s| s.to_str()),
        Some("cargo"),
        "should delegate to the system cargo, got {executable:?}"
    );
    assert!(
        !executable.components().any(|c| c.as_os_str() == "store"),
        "delegated cargo must not come from the vx store: {executable:?}"
    );

    // Nothing to install, and the rust store must stay off PATH for children.
    assert!(plan.dependencies.is_empty(), "{:?}", plan.dependencies);
    assert!(!plan.needs_install());
    assert!(
        plan.config
            .delegated_to_system
            .contains(&"rust".to_string())
    );
}

#[test]
fn test_rustup_toolchain_env_delegates_and_outranks_the_toolchain_file() {
    if !require_rustup() {
        return;
    }
    let _lock = env_lock();
    let _env = EnvOverride::set_rustup_toolchain(Some("1.90.0"));

    let fixture = fixture_with_toolchain_file("1.83.0");
    let plan = resolve_in("cargo", fixture.path(), &[("rust", "1.93.1")]);

    // rustup honours RUSTUP_TOOLCHAIN above the toolchain file, and vx reports the
    // toolchain that will actually run.
    assert_eq!(delegated_channel(&plan).as_deref(), Some("1.90.0"));
}

#[test]
fn test_rustup_managed_pin_delegates_without_a_toolchain_file() {
    if !require_rustup() {
        return;
    }
    let _lock = env_lock();
    let _env = EnvOverride::set_rustup_toolchain(None);

    let fixture = tempfile::tempdir().unwrap();
    let plan = resolve_in("cargo", fixture.path(), &[("rust", RUSTUP_MANAGED)]);

    assert!(
        matches!(
            plan.primary.version,
            VersionResolution::SystemAvailable { .. }
        ),
        "rust = \"{RUSTUP_MANAGED}\" must delegate even with no toolchain file: {:?}",
        plan.primary.version
    );
    assert!(
        plan.config
            .delegated_to_system
            .contains(&"rust".to_string())
    );
}

/// A `vx.lock` written before the repository opted out still carries a numeric rust
/// version. That must not put vx back in charge — `rustup-managed` declares ownership,
/// and vx.lock only ever records versions.
#[test]
fn test_stale_lock_entry_does_not_mask_the_rustup_managed_opt_out() {
    if !require_rustup() {
        return;
    }
    let _lock = env_lock();
    let _env = EnvOverride::set_rustup_toolchain(None);

    let resolver = resolver();
    let config = ResolverConfig::default();
    let project_config = ProjectToolsConfig::from_tools_with_locked(
        HashMap::from([("rust".to_string(), RUSTUP_MANAGED.to_string())]),
        HashMap::from([("rust".to_string(), "1.29.0".to_string())]),
    );

    let stage = ResolveStage::new(&resolver, &config).with_project_config(&project_config);

    // No toolchain file here: the only signal is the vx.toml opt-out.
    let fixture = tempfile::tempdir().unwrap();
    let mut request = vx_resolver::ResolveRequest::new("cargo", vec!["--version".to_string()]);
    request.working_dir = Some(fixture.path().to_path_buf());

    let plan = block_on(stage.execute(request)).unwrap();

    assert!(
        matches!(
            plan.primary.version,
            VersionResolution::SystemAvailable { .. }
        ),
        "a stale vx.lock version must not override rust = \"{RUSTUP_MANAGED}\": {:?}",
        plan.primary.version
    );
}

#[test]
fn test_no_override_keeps_vx_in_charge() {
    let _lock = env_lock();
    let _env = EnvOverride::set_rustup_toolchain(None);

    let fixture = tempfile::tempdir().unwrap();
    let plan = resolve_in("cargo", fixture.path(), &[("rust", "1.93.1")]);

    assert!(
        !matches!(
            plan.primary.version,
            VersionResolution::SystemAvailable { .. }
        ),
        "without a toolchain file or RUSTUP_TOOLCHAIN vx must still manage rust: {:?}",
        plan.primary.version
    );
}

#[test]
fn test_explicit_version_request_is_honoured_through_vx() {
    if !require_rustup() {
        return;
    }
    let _lock = env_lock();
    let _env = EnvOverride::set_rustup_toolchain(None);

    // `vx cargo@1.90.0` names the toolchain explicitly; silently ignoring it in favour
    // of the toolchain file would be surprising.
    let fixture = fixture_with_toolchain_file("1.83.0");
    let resolver = resolver();
    let config = ResolverConfig::default();
    let stage = ResolveStage::new(&resolver, &config);

    let mut request = vx_resolver::ResolveRequest::new("cargo", vec!["--version".to_string()])
        .with_version("1.90.0");
    request.working_dir = Some(fixture.path().to_path_buf());

    let plan = block_on(stage.execute(request)).unwrap();

    assert!(
        !matches!(
            plan.primary.version,
            VersionResolution::SystemAvailable { .. }
        ),
        "an explicit version must not be overridden by the toolchain file: {:?}",
        plan.primary.version
    );
}

#[test]
fn test_ensure_stage_does_not_repair_a_delegated_runtime() {
    if !require_rustup() {
        return;
    }
    let _lock = env_lock();
    let _env = EnvOverride::set_rustup_toolchain(None);

    // The repair path is what turned "not in the vx store" into "install vx's rust and
    // repoint rustup at it". A delegated runtime is absent from the store on purpose.
    let resolver = resolver();
    let config = ResolverConfig::default();
    let context = mock_context();
    let registry = vx_runtime::ProviderRegistry::new();

    let fixture = fixture_with_toolchain_file("1.83.0");
    let plan = resolve_in("cargo", fixture.path(), &[("rust", RUSTUP_MANAGED)]);
    let executable = plan.primary.executable.clone().unwrap();

    let stage = EnsureStage::new(&resolver, &config, Some(&registry), Some(&context));
    let plan = block_on(stage.execute(plan)).unwrap();

    assert!(
        !plan.needs_install(),
        "EnsureStage must not flag a delegated runtime for installation"
    );
    assert_eq!(
        plan.primary.executable.as_deref(),
        Some(executable.as_path())
    );
}

/// The acceptance criterion from the issue: after running a command through vx,
/// `rustup show` still reports the same default toolchain.
#[test]
fn test_running_cargo_through_vx_leaves_the_rustup_default_alone() {
    let Some(rustup) = which::which("rustup").ok() else {
        return;
    };
    let _lock = env_lock();
    let _env = EnvOverride::set_rustup_toolchain(None);

    let Some(before) = rustup_active_toolchain(&rustup) else {
        return;
    };

    let resolver = resolver();
    let config = ResolverConfig::default();
    let context = mock_context();
    let registry = vx_runtime::ProviderRegistry::new();

    // Ask for a channel that is already installed so `cargo --version` succeeds
    // without downloading anything. The pin matches the toolchain file so the run
    // stays quiet — this test is about the rustup default, not about pin warnings.
    let fixture = fixture_with_toolchain_file(&before);
    let plan = resolve_in("cargo", fixture.path(), &[("rust", &before)]);

    let ensure = EnsureStage::new(&resolver, &config, Some(&registry), Some(&context));
    let prepare = PrepareStage::new(&resolver, &config, Some(&registry), Some(&context));

    let plan = block_on(ensure.execute(plan)).unwrap();
    let prepared = block_on(prepare.execute(plan)).unwrap();

    // The delegated environment must not carry vx's RUSTUP_HOME / CARGO_HOME, or the
    // child would be redirected to the store toolchain.
    assert!(
        !prepared.env.contains_key("RUSTUP_HOME"),
        "delegated environment must not set RUSTUP_HOME"
    );
    assert!(
        !prepared.env.contains_key("CARGO_HOME"),
        "delegated environment must not set CARGO_HOME"
    );

    let code = block_on(ExecuteStage::new().execute(prepared)).unwrap();
    assert_eq!(code, 0, "cargo --version should succeed");

    let after = rustup_active_toolchain(&rustup).unwrap();
    assert_eq!(
        before, after,
        "vx must not change the global rustup default"
    );
}

/// The active toolchain as rustup reports it, if it can be determined.
fn rustup_active_toolchain(rustup: &PathBuf) -> Option<String> {
    let output = std::process::Command::new(rustup)
        .args(["show", "active-toolchain"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    // e.g. "1.98.1-x86_64-pc-windows-msvc (default)"
    let first = String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string();

    (!first.is_empty()).then_some(first)
}
