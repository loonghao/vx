//! Builder functions that construct `Provider` and `Runtime` instances from
//! embedded `provider.star` content.
//!
//! - [`create_provider`] — build a single `Arc<dyn Provider>` from a star file.
//! - [`build_runtimes`]  — build a `Vec<Arc<dyn Runtime>>` from a star file.

use std::path::Path;
use std::sync::Arc;

use vx_star_metadata::StarMetadata;

use super::bridge::{
    make_deps_fn_owned, make_download_url_fn, make_download_url_fn_owned, make_fetch_versions_fn,
    make_fetch_versions_fn_owned, make_install_layout_fn, make_install_layout_fn_owned,
    make_post_install_fn_owned, make_version_info_fn_owned,
};

use crate::context::ProviderContext;
use crate::engine::StarlarkEngine;

/// Create an `Arc<dyn Provider>` from embedded `provider.star` content.
///
/// This is the canonical way to register a star-only provider into the
/// `ProviderRegistry` without any hand-written Rust `Provider` impl.
///
/// # Example
///
/// ```rust,ignore
/// use vx_starlark::create_provider;
///
/// registry.register(create_provider("cmake", vx_provider_cmake::PROVIDER_STAR));
/// ```
pub fn create_provider(
    provider_name: impl Into<String>,
    content: impl Into<String>,
) -> Arc<dyn vx_runtime::Provider> {
    struct StarOnlyProvider {
        name: String,
        description: String,
        runtimes: Vec<Arc<dyn vx_runtime::Runtime>>,
    }

    impl vx_runtime::Provider for StarOnlyProvider {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn runtimes(&self) -> Vec<Arc<dyn vx_runtime::Runtime>> {
            self.runtimes.clone()
        }
    }

    let provider_name = provider_name.into();
    let content = content.into();
    let meta = StarMetadata::parse(&content);
    let description = meta
        .description
        .clone()
        .unwrap_or_else(|| format!("{} provider", provider_name));

    let runtimes = build_runtimes(provider_name.clone(), content, None::<String>);

    Arc::new(StarOnlyProvider {
        name: provider_name,
        description,
        runtimes,
    })
}

/// Build a list of `ManifestDrivenRuntime` instances from a `provider.star` file.
///
/// For the **primary** runtime (the first one in the list, or the one whose
/// name matches `primary_name`), `fetch_versions`, `download_url` and
/// `install_layout` functions are all wired in from the Starlark script.
///
/// # Example
///
/// ```rust,ignore
/// use vx_starlark::build_runtimes;
///
/// fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
///     build_runtimes("cmake", crate::PROVIDER_STAR, None)
/// }
/// ```
pub fn build_runtimes(
    provider_name: impl Into<String>,
    content: impl Into<String>,
    primary_name: Option<impl Into<String>>,
) -> Vec<Arc<dyn vx_runtime::Runtime>> {
    use vx_runtime::{Ecosystem, ManifestDrivenRuntime, ProviderSource};

    let provider_name: Arc<str> = Arc::from(provider_name.into());
    let content: Arc<str> = Arc::from(content.into());
    let _primary_name: Option<String> = primary_name.map(|s| s.into());
    let meta = StarMetadata::parse(&content);

    // Parse ecosystem from provider metadata
    let ecosystem = match meta.ecosystem.as_deref() {
        Some("nodejs") | Some("node") => Ecosystem::NodeJs,
        Some("python") => Ecosystem::Python,
        Some("rust") => Ecosystem::Rust,
        Some("go") => Ecosystem::Go,
        Some("git") => Ecosystem::Git,
        Some("dotnet") | Some(".net") => Ecosystem::DotNet,
        Some("java") => Ecosystem::Java,
        Some("ruby") => Ecosystem::Ruby,
        Some("devtools") => Ecosystem::DevTools,
        Some("container") => Ecosystem::Container,
        Some("cloud") => Ecosystem::Cloud,
        Some("ai") => Ecosystem::Ai,
        Some("cpp") | Some("c++") => Ecosystem::Cpp,
        Some("zig") => Ecosystem::Zig,
        Some("system") => Ecosystem::System,
        Some("generic") | Some("custom") | Some(_) => Ecosystem::Generic,
        None => Ecosystem::Unknown,
    };

    let pip_package = meta.pip_package.clone();

    if meta.runtimes.is_empty() {
        // Fallback: create a single runtime with the provider name
        let mut rt =
            ManifestDrivenRuntime::new(&*provider_name, &*provider_name, ProviderSource::BuiltIn)
                .with_ecosystem(ecosystem);
        if let Some(ref pkg) = pip_package {
            rt = rt.with_pip_package(pkg.clone());
        } else {
            rt = rt
                .with_fetch_versions(make_fetch_versions_fn(
                    Arc::clone(&provider_name).to_string(),
                    Arc::clone(&content).to_string(),
                ))
                .with_download_url(make_download_url_fn(
                    Arc::clone(&provider_name).to_string(),
                    Arc::clone(&content).to_string(),
                ))
                .with_install_layout(make_install_layout_fn(
                    Arc::clone(&provider_name).to_string(),
                    Arc::clone(&content).to_string(),
                ));
        }
        rt = rt.with_deps_fn(make_deps_fn_owned(
            Arc::clone(&provider_name),
            Arc::clone(&content),
            provider_name.to_string(),
        ));
        rt = rt.with_post_install(make_post_install_fn_owned(
            Arc::clone(&provider_name),
            Arc::clone(&content),
            provider_name.to_string(),
        ));

        return vec![Arc::new(rt)];
    }

    // Provider-level platform OS constraint
    let provider_platform_os: Vec<String> = meta.platforms.unwrap_or_default();

    let _primary = _primary_name.unwrap_or_else(|| {
        meta.runtimes
            .first()
            .and_then(|rt| rt.name.as_deref())
            .unwrap_or(&provider_name)
            .to_string()
    });

    meta.runtimes
        .iter()
        .map(|rt| {
            let name = rt.name.clone().unwrap_or_else(|| provider_name.to_string());
            let executable = rt.executable.clone().unwrap_or_else(|| name.clone());
            let description = rt.description.clone().unwrap_or_default();

            let mut runtime =
                ManifestDrivenRuntime::new(name.clone(), &*provider_name, ProviderSource::BuiltIn)
                    .with_executable(executable)
                    .with_description(description)
                    .with_aliases(rt.aliases.clone())
                    .with_ecosystem(ecosystem);

            if let Some(ref bundled) = rt.bundled_with {
                runtime = runtime.with_bundled_with(bundled.clone());
            }

            // Runtime-level platform_os takes priority over provider-level
            let effective_platform_os = if !rt.platform_os.is_empty() {
                rt.platform_os.clone()
            } else {
                provider_platform_os.clone()
            };
            if !effective_platform_os.is_empty() {
                runtime = runtime.with_platform_os(effective_platform_os);
            }

            if !rt.install_deps.is_empty() {
                runtime = runtime.with_install_deps(rt.install_deps.clone());
            }

            if !rt.shells.is_empty() {
                use vx_runtime::manifest_runtime::ShellDefinition;
                let shells: Vec<ShellDefinition> = rt
                    .shells
                    .iter()
                    .map(|(name, path)| ShellDefinition {
                        name: name.clone(),
                        path: path.clone(),
                    })
                    .collect();
                runtime = runtime.with_shells(shells);
            }

            if let Some(ref pkg) = pip_package {
                runtime = runtime.with_pip_package(pkg.clone());
            } else {
                let rt_name_owned = name.clone();
                runtime = runtime
                    .with_fetch_versions(make_fetch_versions_fn_owned(
                        Arc::clone(&provider_name),
                        Arc::clone(&content),
                        rt_name_owned.clone(),
                    ))
                    .with_download_url(make_download_url_fn_owned(
                        Arc::clone(&provider_name),
                        Arc::clone(&content),
                        rt_name_owned.clone(),
                    ))
                    .with_install_layout(make_install_layout_fn_owned(
                        Arc::clone(&provider_name),
                        Arc::clone(&content),
                        rt_name_owned,
                    ));
            }

            runtime = runtime.with_deps_fn(make_deps_fn_owned(
                Arc::clone(&provider_name),
                Arc::clone(&content),
                name.clone(),
            ));

            // RFC 0040: Wire up version_info for toolchain-managed tools (e.g., Rust)
            runtime = runtime.with_version_info(make_version_info_fn_owned(
                Arc::clone(&provider_name),
                Arc::clone(&content),
                name.clone(),
            ));

            // Wire up post_install for Starlark post_extract hooks (e.g., rustup-init)
            runtime = runtime.with_post_install(make_post_install_fn_owned(
                Arc::clone(&provider_name),
                Arc::clone(&content),
                name.clone(),
            ));

            // Wire up system_paths glob patterns (for tools like MSVC cl.exe that are
            // not on PATH — used to locate the executable after system installation)
            if !rt.system_paths.is_empty() {
                runtime = runtime.with_system_paths(rt.system_paths.clone());
            }

            // Wire up system_install strategies (for tools like msvc, cmake, etc.
            // that are installed via system package managers rather than direct download)
            let strategies = parse_system_install_strategies(&provider_name, &content);
            if !strategies.is_empty() {
                runtime = runtime.with_install_strategies(strategies);
            }

            Arc::new(runtime) as Arc<dyn vx_runtime::Runtime>
        })
        .collect()
}

/// Parse `system_install` from a provider.star script and return a list of
/// [`InstallStrategy`] values.
///
/// Handles two forms:
/// 1. **Static dict** — `system_install = system_install_strategies([...])`
///    The variable is a plain `{"strategies": [...]}` dict.
/// 2. **Function** — `system_install = cross_platform_install(...)` or
///    `def system_install(ctx): ...`
///    The variable is a callable; we call it with the current platform context.
fn parse_system_install_strategies(
    provider_name: &str,
    content: &str,
) -> Vec<vx_runtime::manifest_runtime::InstallStrategy> {
    let engine = StarlarkEngine::new();
    let script_path = Path::new(provider_name);

    // First try: read as a static variable (covers `system_install = system_install_strategies([...])`)
    let json = match engine.get_variable(script_path, content, "system_install") {
        Ok(Some(v)) => v,
        Ok(None) => return vec![],
        Err(e) => {
            tracing::debug!(provider = %provider_name, "system_install variable read failed: {}", e);
            return vec![];
        }
    };

    // If the value is a dict with "strategies" key, parse it directly
    if let Some(strategies_arr) = json.get("strategies").and_then(|s| s.as_array()) {
        return parse_strategies_array(strategies_arr);
    }

    // If the value is a callable (function), call it with the current platform context
    // The engine returns a string like "<function ...>" for callables — we need to call it
    if json
        .as_str()
        .map(|s| s.starts_with("<function"))
        .unwrap_or(false)
        || json.get("__type").and_then(|t| t.as_str()) == Some("function")
    {
        let vx_home = vx_paths::VxPaths::new()
            .map(|p| p.base_dir)
            .unwrap_or_else(|_| dirs::home_dir().unwrap_or_default().join(".vx"));
        let ctx = ProviderContext::new(provider_name, vx_home);
        let result = engine.call_function(script_path, content, "system_install", &ctx, &[]);
        match result {
            Ok(v) => {
                if let Some(strategies_arr) = v.get("strategies").and_then(|s| s.as_array()) {
                    return parse_strategies_array(strategies_arr);
                }
            }
            Err(e) => {
                tracing::debug!(provider = %provider_name, "system_install() call failed: {}", e);
            }
        }
    }

    vec![]
}

/// Parse a JSON array of strategy dicts into [`InstallStrategy`] values.
fn parse_strategies_array(
    arr: &[serde_json::Value],
) -> Vec<vx_runtime::manifest_runtime::InstallStrategy> {
    use vx_runtime::manifest_runtime::InstallStrategy;
    arr.iter()
        .filter_map(|s| {
            let manager = s.get("manager").and_then(|m| m.as_str())?.to_string();
            let package = s.get("package").and_then(|p| p.as_str())?.to_string();
            let priority = s.get("priority").and_then(|p| p.as_i64()).unwrap_or(80) as i32;
            let install_args = s
                .get("install_args")
                .and_then(|a| a.as_str())
                .map(|s| s.to_string());
            let params = s
                .get("params")
                .and_then(|p| p.as_str())
                .map(|s| s.to_string());
            let platforms: Vec<String> = s
                .get("platforms")
                .and_then(|p| p.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            Some(InstallStrategy::PackageManager {
                manager,
                package,
                params,
                install_args,
                priority,
                platforms,
            })
        })
        .collect()
}
