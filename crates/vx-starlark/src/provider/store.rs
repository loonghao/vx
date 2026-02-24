//! Store path query functions for Starlark providers.
//!
//! Implements the RFC-0037 path query interface: store_root(),
//! get_execute_path(), and post_install().

use crate::context::ProviderContext;
use crate::engine::StarlarkEngine;
use crate::error::{Error, Result};
use tracing::debug;

use super::StarlarkProvider;
use super::types::PostExtractAction;

impl StarlarkProvider {
    /// Call `store_root(ctx)` from provider.star
    ///
    /// Returns the raw template string (e.g. `"{vx_home}/store/7zip"`),
    /// or `None` if the function is not defined.
    pub async fn call_store_root(&self) -> Result<Option<String>> {
        let ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_description(&self.meta.description)
            .with_sandbox(self.sandbox.clone());

        let engine = StarlarkEngine::new();
        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "store_root",
            &ctx,
            &[],
        );

        match result {
            Ok(json) => {
                if json.is_null() {
                    Ok(None)
                } else if let Some(s) = json.as_str() {
                    Ok(Some(s.to_string()))
                } else {
                    Ok(None)
                }
            }
            Err(Error::FunctionNotFound { .. }) => {
                debug!(provider = %self.meta.name, "store_root() not found in provider script, using convention");
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    /// Call `get_execute_path(ctx, version)` from provider.star
    ///
    /// Returns the raw template string (e.g. `"{install_dir}/7z.exe"`),
    /// or `None` if the function returns `None` or is not defined.
    pub async fn call_get_execute_path(&self, version: &str) -> Result<Option<String>> {
        let ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_description(&self.meta.description)
            .with_sandbox(self.sandbox.clone())
            .with_version(version);

        let engine = StarlarkEngine::new();
        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "get_execute_path",
            &ctx,
            &[serde_json::Value::String(version.to_string())],
        );

        match result {
            Ok(json) => {
                if json.is_null() {
                    Ok(None)
                } else if let Some(s) = json.as_str() {
                    Ok(Some(s.to_string()))
                } else {
                    Ok(None)
                }
            }
            Err(Error::FunctionNotFound { .. }) => {
                debug!(provider = %self.meta.name, "get_execute_path() not found in provider script, using convention");
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    /// Call `deps(ctx, version)` from provider.star
    ///
    /// Returns a list of runtime dependency names (e.g. `["git"]`),
    /// or an empty list if the function is not defined or returns nothing.
    ///
    /// Each entry in the returned list from Starlark is a dict:
    /// ```python
    /// {"runtime": "git", "version": "*", "optional": False, "reason": "..."}
    /// ```
    /// This method returns only the `runtime` field of each entry.
    pub async fn call_deps(&self, version: &str) -> Result<Vec<String>> {
        let ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_description(&self.meta.description)
            .with_sandbox(self.sandbox.clone())
            .with_version(version);

        let engine = StarlarkEngine::new();
        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "deps",
            &ctx,
            &[serde_json::Value::String(version.to_string())],
        );

        match result {
            Ok(json) => {
                if json.is_null() {
                    return Ok(vec![]);
                }
                // Expected: list of dicts with "runtime" key
                if let Some(arr) = json.as_array() {
                    let deps = arr
                        .iter()
                        .filter_map(|item| {
                            item.get("runtime")
                                .and_then(|r| r.as_str())
                                .map(|s| s.to_string())
                        })
                        .collect();
                    return Ok(deps);
                }
                Ok(vec![])
            }
            Err(Error::FunctionNotFound { .. }) => {
                debug!(provider = %self.meta.name, "deps() not found in provider script");
                Ok(vec![])
            }
            Err(e) => Err(e),
        }
    }

    /// Call `post_install(ctx, version, install_dir)` from provider.star
    ///
    /// Returns a list of [`PostExtractAction`]s to execute after installation,
    /// or an empty list if the function returns `None` / is not defined.
    pub async fn call_post_install(
        &self,
        version: &str,
        install_dir: &std::path::Path,
    ) -> Result<Vec<PostExtractAction>> {
        let ctx = ProviderContext::new(&self.meta.name, self.vx_home.clone())
            .with_description(&self.meta.description)
            .with_sandbox(self.sandbox.clone())
            .with_version(version);

        let engine = StarlarkEngine::new();
        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "post_install",
            &ctx,
            &[
                serde_json::Value::String(version.to_string()),
                serde_json::Value::String(install_dir.to_string_lossy().to_string()),
            ],
        );

        match result {
            Ok(json) => {
                let actions = self.parse_hook_actions(&json, "post_install")?;
                Ok(actions
                    .into_iter()
                    .filter_map(|a| self.json_to_post_extract_action(&a))
                    .collect())
            }
            Err(Error::FunctionNotFound { .. }) => {
                debug!(provider = %self.meta.name, "post_install() not found in provider script");
                Ok(vec![])
            }
            Err(e) => Err(e),
        }
    }
}
