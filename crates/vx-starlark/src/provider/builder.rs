//! Builder functions that construct `Provider` and `Runtime` instances from
//! embedded `provider.star` content.
//!
//! - [`create_provider`] — build a single `Arc<dyn Provider>` from a star file.
//! - [`build_runtimes`]  — build a `Vec<Arc<dyn Runtime>>` from a star file.

use std::sync::Arc;

use vx_star_metadata::StarMetadata;

use super::bridge::{
    make_download_url_fn, make_download_url_fn_owned, make_fetch_versions_fn,
    make_fetch_versions_fn_owned, make_install_layout_fn, make_install_layout_fn_owned,
};

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
        Some("dotnet") => Ecosystem::Dotnet,
        Some("system") => Ecosystem::System,
        Some(other) => Ecosystem::Custom(other.to_string()),
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
                    .with_ecosystem(ecosystem.clone());

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

            Arc::new(runtime) as Arc<dyn vx_runtime::Runtime>
        })
        .collect()
}
