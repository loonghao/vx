//! Tool path resolution shared by the dev / run / export commands
//!
//! Builds [`vx_env::RuntimeSpec`] values for the tools declared in `vx.toml`.
//! A tool's bin directory can come from two different places:
//!
//! 1. **vx-managed install** — `Runtime::get_executable_path_for_version()`
//!    resolves the executable inside the vx store.
//! 2. **System-detected install** — providers that vx never installs into its
//!    own store (MSVC's `cl.exe`, LLVM's `clang-cl.exe`, …) declare well-known
//!    locations through `runtimes[].system_paths` glob patterns in
//!    `provider.star`.
//!
//! Case 2 is what makes `cl` / `clang-cl` reachable: those directories are
//! deliberately absent from the system PATH (MSVC needs `vcvarsall.bat`, LLVM
//! installs side-by-side), so a plain `which` lookup fails and the tool never
//! makes it into the dev shell's PATH even though `vx cl` runs it fine.
//!
//! See <https://github.com/loonghao/vx/issues/1083>.

use crate::commands::common::find_system_tool;
use std::path::PathBuf;
use vx_env::RuntimeSpec;
use vx_runtime::{ProviderRegistry, Runtime, RuntimeContext};
use vx_starlark::{RuntimeMeta, global_registry};

/// Build the [`RuntimeSpec`] used to place one declared tool on the PATH.
///
/// Falls back to `provider.star`'s `system_paths` globs when the tool has no
/// vx-managed install and cannot be found on the system PATH either.
pub async fn build_runtime_spec(
    registry: &ProviderRegistry,
    ctx: &RuntimeContext,
    tool: &str,
    version: &str,
) -> RuntimeSpec {
    let mut bin_dirs: Vec<String> = vec!["bin".to_string()];

    let providers = registry.providers();
    if let Some(provider) = providers.iter().find(|p| p.supports(tool))
        && let Some(runtime) = provider.get_runtime(tool)
    {
        bin_dirs = runtime
            .possible_bin_dirs()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        if let Some(bin_dir) = resolve_bin_dir(runtime.as_ref(), ctx, tool, version).await {
            return RuntimeSpec::with_bin_dirs(tool, version, bin_dirs)
                .set_resolved_bin_dir(bin_dir);
        }
    }

    RuntimeSpec::with_bin_dirs(tool, version, bin_dirs)
}

/// Resolve the directory that should be prepended to PATH for one tool.
async fn resolve_bin_dir(
    runtime: &dyn Runtime,
    ctx: &RuntimeContext,
    tool: &str,
    version: &str,
) -> Option<PathBuf> {
    // 1. vx-managed install inside the store.
    if let Ok(Some(exe_path)) = runtime.get_executable_path_for_version(version, ctx).await
        && let Some(bin_dir) = exe_path.parent()
    {
        return Some(bin_dir.to_path_buf());
    }

    // 2. System-detected install. Only reached when the tool would otherwise be
    //    missing: `find_system_tool()` performs the same lookup `ToolEnvironment`
    //    does, so a tool that is already reachable via the store or the system
    //    PATH keeps its existing resolution. This prevents a system copy from
    //    shadowing the version pinned in vx.toml.
    if find_system_tool(tool).is_some() {
        return None;
    }

    let executable = find_system_executable(tool).await?;
    executable.parent().map(|dir| dir.to_path_buf())
}

/// Locate a system-installed executable via `provider.star::runtimes[].system_paths`.
///
/// Returns the executable itself — callers usually want its parent directory.
pub async fn find_system_executable(tool: &str) -> Option<PathBuf> {
    // Release the registry lock before doing any further async work.
    let metas = {
        let reg = global_registry().await;
        let handle = reg.get(tool)?;
        handle.runtime_metas().to_vec()
    };

    // A multi-runtime provider exposes the metadata of *every* runtime it owns,
    // so prefer the one matching the requested tool (e.g. `clang-cl` inside the
    // `llvm` provider) before trying any runtime that happens to match a glob.
    let preferred = metas.iter().find(|meta| runtime_matches(meta, tool));

    for meta in preferred.into_iter().chain(metas.iter()) {
        if let Some(path) = first_glob_match(&meta.system_paths) {
            return Some(path);
        }
    }

    None
}

/// Whether a runtime definition declares `tool` as its name or alias.
fn runtime_matches(meta: &RuntimeMeta, tool: &str) -> bool {
    meta.name.eq_ignore_ascii_case(tool)
        || meta.aliases.iter().any(|a| a.eq_ignore_ascii_case(tool))
}

/// First existing path matching any of the glob patterns (newest first).
///
/// Patterns are evaluated in declaration order so providers control priority;
/// within a pattern, paths are sorted descending so that newer toolsets win
/// (e.g. Visual Studio 2022 over 2019).
fn first_glob_match(patterns: &[String]) -> Option<PathBuf> {
    for pattern in patterns {
        let Ok(paths) = glob::glob(pattern) else {
            continue;
        };
        let mut found: Vec<PathBuf> = paths
            .filter_map(|p| p.ok())
            .filter(|p| p.exists())
            .collect();
        #[allow(clippy::unnecessary_sort_by)]
        found.sort_by(|a, b| b.cmp(a));
        if let Some(path) = found.into_iter().next() {
            return Some(path);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_glob_match_returns_newest_match() {
        // Two entries share a pattern; the lexicographically larger one wins so
        // that newer toolset versions take precedence.
        let dir = tempfile::tempdir().unwrap();
        let newer = dir.path().join("14.44.35207");
        let older = dir.path().join("14.36.32532");
        std::fs::create_dir_all(&newer).unwrap();
        std::fs::create_dir_all(&older).unwrap();
        std::fs::write(newer.join("cl.exe"), b"").unwrap();
        std::fs::write(older.join("cl.exe"), b"").unwrap();

        let pattern = dir.path().join("*/cl.exe");
        let found = first_glob_match(&[pattern.to_string_lossy().to_string()]).unwrap();

        assert_eq!(found, newer.join("cl.exe"));
    }

    #[test]
    fn test_first_glob_match_ignores_missing_patterns() {
        assert_eq!(
            first_glob_match(&[
                "/nonexistent-vx-tool/1.0/bin/tool.exe".to_string(),
                "not a glob [".to_string(),
            ]),
            None
        );
    }

    #[test]
    fn test_runtime_matches_name_and_aliases() {
        let meta = RuntimeMeta {
            name: "msvc".to_string(),
            description: String::new(),
            executable: "cl".to_string(),
            aliases: vec!["cl".to_string(), "vs-build-tools".to_string()],
            priority: 100,
            command_prefix: vec![],
            system_paths: vec![],
            test_commands: vec![],
            install_deps: vec![],
            bundled_with: None,
        };

        assert!(runtime_matches(&meta, "msvc"));
        assert!(runtime_matches(&meta, "cl"));
        assert!(runtime_matches(&meta, "CL"));
        assert!(!runtime_matches(&meta, "nmake"));
    }
}
