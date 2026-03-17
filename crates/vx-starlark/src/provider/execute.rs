//! Execution methods for Starlark provider functions.

use crate::context::{InstallResult, ProviderContext};
use crate::engine::StarlarkEngine;
use crate::error::{Error, Result};
use tracing::debug;

use super::StarlarkProvider;
use super::types::{EnvOp, InstallLayout, PostExtractAction, PreRunAction};

impl StarlarkProvider {
    pub(super) async fn execute_install(
        &self,
        ctx: &ProviderContext,
        version: &str,
    ) -> Result<InstallResult> {
        let engine = StarlarkEngine::new();
        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "install",
            ctx,
            &[serde_json::Value::String(version.to_string())],
        );
        match result {
            Ok(json) => {
                let success = json
                    .get("success")
                    .and_then(|s| s.as_bool())
                    .unwrap_or(false);
                let install_path = json
                    .get("path")
                    .and_then(|p| p.as_str())
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| ctx.paths.install_dir(version));
                if success {
                    Ok(InstallResult::success(install_path))
                } else {
                    let msg = json
                        .get("error")
                        .and_then(|e| e.as_str())
                        .unwrap_or("Installation failed")
                        .to_string();
                    Ok(InstallResult::failure(msg))
                }
            }
            Err(Error::FunctionNotFound { .. }) => {
                debug!(provider = %self.meta.name, "install() not found, using default installer");
                Ok(InstallResult::failure("No install() function defined"))
            }
            Err(e) => Err(e),
        }
    }

    /// Call `environment(ctx, version)` from provider.star.
    ///
    /// Returns a list of [`EnvOp`]s that describe how to set up the environment
    /// for this provider. The caller applies them in order via [`EnvOp::apply`].
    ///
    /// Supports two return formats from Starlark:
    ///
    /// **New format** (recommended) — list of `EnvOp` structs from `@vx//stdlib:env.star`:
    /// ```python
    /// load("@vx//stdlib:env.star", "env_set", "env_prepend")
    /// def environment(ctx, version):
    ///     return [
    ///         env_set("GOROOT", ctx.install_dir),
    ///         env_prepend("PATH", ctx.install_dir + "/bin"),
    ///     ]
    /// ```
    ///
    /// **Legacy format** (still supported) — plain dict:
    /// ```python
    /// def environment(ctx, version):
    ///     return {"PATH": ctx.install_dir + "/bin"}
    /// ```
    pub(super) async fn execute_prepare_environment(
        &self,
        ctx: &ProviderContext,
        version: &str,
    ) -> Result<Vec<EnvOp>> {
        let engine = StarlarkEngine::new();

        // Try 'environment' first — this is the canonical name used in all provider.star files.
        // Fall back to 'prepare_environment' for forward compatibility with any future scripts.
        let result = engine
            .call_function(
                &self.script_path,
                &self.script_content,
                "environment",
                ctx,
                &[serde_json::Value::String(version.to_string())],
            )
            .or_else(|e| match e {
                Error::FunctionNotFound { .. } => engine.call_function(
                    &self.script_path,
                    &self.script_content,
                    "prepare_environment",
                    ctx,
                    &[serde_json::Value::String(version.to_string())],
                ),
                other => Err(other),
            });

        match result {
            Ok(json) => Ok(Self::parse_env_ops(&json)),
            Err(Error::FunctionNotFound { .. }) => Ok(vec![]),
            Err(e) => Err(e),
        }
    }

    /// Parse the return value of `environment()` into a list of [`EnvOp`]s.
    ///
    /// Handles both:
    /// - New format: `[{"op": "set", "key": "K", "value": "V"}, ...]`
    /// - Legacy format: `{"KEY": "value", ...}` → converted to `EnvOp::Set`
    fn parse_env_ops(json: &serde_json::Value) -> Vec<EnvOp> {
        // New format: list of EnvOp dicts
        if let Some(arr) = json.as_array() {
            return arr
                .iter()
                .filter_map(|item| {
                    // Each item must have an "op" field
                    let op = item.get("op").and_then(|o| o.as_str())?;
                    let key = item.get("key").and_then(|k| k.as_str())?.to_string();
                    match op {
                        "set" => {
                            let value = item.get("value").and_then(|v| v.as_str())?.to_string();
                            Some(EnvOp::Set { key, value })
                        }
                        "prepend" => {
                            let value = item.get("value").and_then(|v| v.as_str())?.to_string();
                            let sep = item
                                .get("sep")
                                .and_then(|s| s.as_str())
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| {
                                    if cfg!(windows) {
                                        ";".to_string()
                                    } else {
                                        ":".to_string()
                                    }
                                });
                            Some(EnvOp::Prepend { key, value, sep })
                        }
                        "append" => {
                            let value = item.get("value").and_then(|v| v.as_str())?.to_string();
                            let sep = item
                                .get("sep")
                                .and_then(|s| s.as_str())
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| {
                                    if cfg!(windows) {
                                        ";".to_string()
                                    } else {
                                        ":".to_string()
                                    }
                                });
                            Some(EnvOp::Append { key, value, sep })
                        }
                        "unset" => Some(EnvOp::Unset { key }),
                        other => {
                            tracing::warn!("Unknown EnvOp type '{}', skipping", other);
                            None
                        }
                    }
                })
                .collect();
        }

        // Legacy format: plain dict {"KEY": "value"} → EnvOp::Set
        if let Some(obj) = json.as_object() {
            return obj
                .iter()
                .filter_map(|(k, v)| {
                    v.as_str().map(|s| EnvOp::Set {
                        key: k.clone(),
                        value: s.to_string(),
                    })
                })
                .collect();
        }

        vec![]
    }

    pub(super) async fn execute_download_url(
        &self,
        ctx: &ProviderContext,
        version: &str,
    ) -> Result<Option<String>> {
        let engine = StarlarkEngine::new();
        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "download_url",
            ctx,
            &[serde_json::Value::String(version.to_string())],
        );
        match result {
            Ok(json) => {
                if json.is_null() {
                    Ok(None)
                } else if let Some(url) = json.as_str() {
                    Ok(Some(url.to_string()))
                } else {
                    Ok(None)
                }
            }
            Err(Error::FunctionNotFound { .. }) => {
                debug!(provider = %self.meta.name, "download_url() not found in provider script");
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    pub(super) async fn execute_install_layout(
        &self,
        ctx: &ProviderContext,
        version: &str,
    ) -> Result<Option<InstallLayout>> {
        let engine = StarlarkEngine::new();
        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "install_layout",
            ctx,
            &[serde_json::Value::String(version.to_string())],
        );
        match result {
            Ok(json) => {
                if json.is_null() {
                    return Ok(None);
                }
                let type_str = json
                    .get("__type")
                    .or_else(|| json.get("type"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("");
                match type_str {
                    "msi_install" => {
                        let url = json
                            .get("url")
                            .and_then(|u| u.as_str())
                            .ok_or_else(|| {
                                Error::EvalError("msi_install descriptor missing 'url'".into())
                            })?
                            .to_string();
                        let executable_paths = json
                            .get("executable_paths")
                            .and_then(|p| p.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect()
                            })
                            .unwrap_or_default();
                        let strip_prefix = json
                            .get("strip_prefix")
                            .and_then(|s| s.as_str())
                            .map(|s| s.to_string());
                        let extra_args = json
                            .get("extra_args")
                            .and_then(|a| a.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect()
                            })
                            .unwrap_or_default();
                        debug!(provider = %self.meta.name, url = %url, "Resolved msi_install descriptor");
                        Ok(Some(InstallLayout::Msi {
                            url,
                            executable_paths,
                            strip_prefix,
                            extra_args,
                        }))
                    }
                    "archive_install" | "archive" => {
                        // "archive" is the legacy format (no URL, just layout hints)
                        // "archive_install" includes the URL for complete install info
                        let url = json
                            .get("url")
                            .and_then(|u| u.as_str())
                            .map(|s| s.to_string());
                        let strip_prefix = json
                            .get("strip_prefix")
                            .and_then(|s| s.as_str())
                            .map(|s| s.to_string());
                        let executable_paths = json
                            .get("executable_paths")
                            .and_then(|p| p.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect()
                            })
                            .unwrap_or_default();
                        debug!(provider = %self.meta.name, url = ?url, strip_prefix = ?strip_prefix, "Resolved archive_install/archive descriptor");
                        Ok(Some(InstallLayout::Archive {
                            url,
                            strip_prefix,
                            executable_paths,
                        }))
                    }
                    "binary_install" => {
                        let url = json
                            .get("url")
                            .and_then(|u| u.as_str())
                            .ok_or_else(|| {
                                Error::EvalError("binary_install descriptor missing 'url'".into())
                            })?
                            .to_string();
                        let executable_name = json
                            .get("executable_name")
                            .and_then(|n| n.as_str())
                            .map(|s| s.to_string());
                        let permissions = json
                            .get("permissions")
                            .and_then(|p| p.as_str())
                            .unwrap_or("755")
                            .to_string();
                        debug!(provider = %self.meta.name, url = %url, "Resolved binary_install descriptor");
                        Ok(Some(InstallLayout::Binary {
                            url,
                            executable_name,
                            permissions,
                        }))
                    }
                    "system_find" => {
                        let executable = json
                            .get("executable")
                            .and_then(|e| e.as_str())
                            .ok_or_else(|| {
                                Error::EvalError(
                                    "system_find descriptor missing 'executable'".into(),
                                )
                            })?
                            .to_string();
                        let system_paths = json
                            .get("system_paths")
                            .and_then(|p| p.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect()
                            })
                            .unwrap_or_default();
                        let hint = json
                            .get("hint")
                            .and_then(|h| h.as_str())
                            .map(|s| s.to_string());
                        debug!(provider = %self.meta.name, executable = %executable, "Resolved system_find descriptor");
                        Ok(Some(InstallLayout::SystemFind {
                            executable,
                            system_paths,
                            hint,
                        }))
                    }
                    other => {
                        tracing::warn!(provider = %self.meta.name, type_ = %other, "Unknown install_layout descriptor type, ignoring");
                        Ok(None)
                    }
                }
            }
            Err(Error::FunctionNotFound { .. }) => {
                debug!(provider = %self.meta.name, "install_layout() not found in provider script");
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    /// Call `install_layout` and return the raw JSON value without type-parsing.
    ///
    /// Used as a fallback when the Starlark function returns a plain dict
    /// without a `__type` field (e.g. `{ "source_name": ..., "target_name": ... }`).
    pub(super) async fn execute_install_layout_raw(
        &self,
        ctx: &ProviderContext,
        version: &str,
    ) -> Result<Option<serde_json::Value>> {
        let engine = StarlarkEngine::new();
        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "install_layout",
            ctx,
            &[serde_json::Value::String(version.to_string())],
        );
        match result {
            Ok(json) => {
                if json.is_null() {
                    return Ok(None);
                }
                // Only return raw JSON if it has useful fields (source_name, executable_paths, etc.)
                // and no __type (which would have been handled by execute_install_layout already).
                let has_type = json.get("__type").is_some();
                if has_type {
                    return Ok(None); // Already handled by execute_install_layout
                }
                let has_useful_fields = json.get("source_name").is_some()
                    || json.get("executable_paths").is_some()
                    || json.get("strip_prefix").is_some();
                if has_useful_fields {
                    debug!(provider = %self.meta.name, "install_layout returned raw dict (no __type)");
                    Ok(Some(json))
                } else {
                    Ok(None)
                }
            }
            Err(Error::FunctionNotFound { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub(super) async fn execute_post_extract(
        &self,
        ctx: &ProviderContext,
        version: &str,
        install_dir: &std::path::Path,
    ) -> Result<Vec<PostExtractAction>> {
        let engine = StarlarkEngine::new();
        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "post_extract",
            ctx,
            &[
                serde_json::Value::String(version.to_string()),
                serde_json::Value::String(install_dir.to_string_lossy().to_string()),
            ],
        );
        match result {
            Ok(json) => {
                let actions = self.parse_hook_actions(&json, "post_extract")?;
                Ok(actions
                    .into_iter()
                    .filter_map(|a| self.json_to_post_extract_action(&a))
                    .collect())
            }
            Err(Error::FunctionNotFound { .. }) => {
                debug!(provider = %self.meta.name, "post_extract() not found in provider script");
                Ok(vec![])
            }
            Err(e) => Err(e),
        }
    }

    /// Call the `uninstall` function in provider.star (if defined).
    ///
    /// Returns:
    /// - `Ok(true)`  — provider.star handled the uninstall
    /// - `Ok(false)` — function not found; caller should use default logic
    /// - `Err(_)`    — provider.star returned an error
    pub(super) async fn execute_uninstall(
        &self,
        ctx: &ProviderContext,
        version: &str,
    ) -> Result<bool> {
        let engine = StarlarkEngine::new();
        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "uninstall",
            ctx,
            &[serde_json::Value::String(version.to_string())],
        );
        match result {
            Ok(json) => {
                // If the function returns false explicitly, treat as "not handled"
                if json.as_bool() == Some(false) {
                    debug!(provider = %self.meta.name, version = %version, "uninstall() returned false, falling back to default");
                    return Ok(false);
                }

                // If the function returns a system_uninstall descriptor, execute it
                if let Some(type_str) = json.get("type").and_then(|t| t.as_str())
                    && type_str == "system_uninstall"
                {
                    return self.execute_system_uninstall(&json, version);
                }

                debug!(provider = %self.meta.name, version = %version, "uninstall() handled by provider.star");
                Ok(true)
            }
            Err(Error::FunctionNotFound { .. }) => {
                debug!(provider = %self.meta.name, "uninstall() not defined in provider.star, using default");
                Ok(false)
            }
            Err(e) => Err(e),
        }
    }

    /// Execute a system_uninstall descriptor returned by provider.star::uninstall().
    ///
    /// Tries each strategy in priority order until one succeeds.
    /// Returns `Ok(true)` if a package manager successfully uninstalled the tool,
    /// `Ok(false)` if no strategy succeeded (caller should warn the user).
    fn execute_system_uninstall(&self, json: &serde_json::Value, version: &str) -> Result<bool> {
        let strategies = match json.get("strategies").and_then(|s| s.as_array()) {
            Some(s) => s.clone(),
            None => {
                debug!(provider = %self.meta.name, "system_uninstall has no strategies");
                return Ok(false);
            }
        };

        // Sort strategies by priority (descending)
        let mut sorted: Vec<&serde_json::Value> = strategies.iter().collect();
        sorted.sort_by(|a, b| {
            let pa = a.get("priority").and_then(|p| p.as_i64()).unwrap_or(0);
            let pb = b.get("priority").and_then(|p| p.as_i64()).unwrap_or(0);
            pb.cmp(&pa)
        });

        for strategy in sorted {
            let manager = match strategy.get("manager").and_then(|m| m.as_str()) {
                Some(m) => m,
                None => continue,
            };
            let package = match strategy.get("package").and_then(|p| p.as_str()) {
                Some(p) => p,
                None => continue,
            };

            // Check if the package manager is available
            if which::which(manager).is_err() {
                debug!(provider = %self.meta.name, manager = %manager, "Package manager not found, skipping");
                continue;
            }

            // Build the uninstall command for each package manager
            let (cmd, args) = match manager {
                "winget" => (manager, vec!["uninstall", "--id", package, "--silent"]),
                "choco" => (manager, vec!["uninstall", package, "-y"]),
                "scoop" => (manager, vec!["uninstall", package]),
                "brew" => (manager, vec!["uninstall", package]),
                "apt" => ("sudo", vec!["apt", "remove", "-y", package]),
                "dnf" => ("sudo", vec!["dnf", "remove", "-y", package]),
                other => {
                    debug!(provider = %self.meta.name, manager = %other, "Unknown package manager, skipping");
                    continue;
                }
            };

            tracing::info!(
                provider = %self.meta.name,
                version  = %version,
                manager  = %manager,
                package  = %package,
                "Uninstalling via system package manager"
            );

            let status = std::process::Command::new(cmd).args(&args).status();

            match status {
                Ok(s) if s.success() => {
                    debug!(provider = %self.meta.name, manager = %manager, "System uninstall succeeded");
                    return Ok(true);
                }
                Ok(s) => {
                    tracing::warn!(
                        provider = %self.meta.name,
                        manager  = %manager,
                        code     = ?s.code(),
                        "Package manager uninstall failed, trying next strategy"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        provider = %self.meta.name,
                        manager  = %manager,
                        error    = %e,
                        "Failed to run package manager, trying next strategy"
                    );
                }
            }
        }

        // No strategy succeeded
        tracing::warn!(
            provider = %self.meta.name,
            version  = %version,
            "All system_uninstall strategies failed"
        );
        Ok(false)
    }

    /// Call `deps(ctx, version)` from provider.star.
    ///
    /// Returns a list of dependency descriptors, each with:
    /// - `runtime`: the runtime name (e.g. "git")
    /// - `version`: version constraint (e.g. "*", ">=2.0")
    /// - `optional`: whether the dependency is optional
    /// - `reason`: human-readable reason for the dependency
    pub(super) async fn execute_deps(
        &self,
        ctx: &ProviderContext,
        version: &str,
    ) -> Result<Vec<serde_json::Value>> {
        let engine = StarlarkEngine::new();
        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "deps",
            ctx,
            &[serde_json::Value::String(version.to_string())],
        );
        match result {
            Ok(json) => {
                if let Some(arr) = json.as_array() {
                    Ok(arr.clone())
                } else {
                    Ok(vec![])
                }
            }
            Err(Error::FunctionNotFound { .. }) => {
                debug!(provider = %self.meta.name, "deps() not found in provider script");
                Ok(vec![])
            }
            Err(e) => Err(e),
        }
    }

    pub(super) async fn execute_pre_run(
        &self,
        ctx: &ProviderContext,
        args: &[String],
        executable: &std::path::Path,
    ) -> Result<Vec<PreRunAction>> {
        let engine = StarlarkEngine::new();
        let args_json: Vec<serde_json::Value> = args
            .iter()
            .map(|a| serde_json::Value::String(a.clone()))
            .collect();
        let result = engine.call_function(
            &self.script_path,
            &self.script_content,
            "pre_run",
            ctx,
            &[
                serde_json::Value::Array(args_json),
                serde_json::Value::String(executable.to_string_lossy().to_string()),
            ],
        );
        match result {
            Ok(json) => {
                let actions = self.parse_hook_actions(&json, "pre_run")?;
                Ok(actions
                    .into_iter()
                    .filter_map(|a| self.json_to_pre_run_action(&a))
                    .collect())
            }
            Err(Error::FunctionNotFound { .. }) => {
                debug!(provider = %self.meta.name, "pre_run() not found in provider script");
                Ok(vec![])
            }
            Err(e) => Err(e),
        }
    }
}
