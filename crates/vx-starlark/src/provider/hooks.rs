//! Hook action parsing for Starlark provider scripts.
//!
//! Converts JSON descriptors returned by post_extract() and pre_run()
//! into typed PostExtractAction and PreRunAction values.

use tracing::{debug, warn};

use super::StarlarkProvider;
use super::types::{PostExtractAction, PreRunAction};
use crate::error::Result;

impl StarlarkProvider {
    /// Parse hook function return value into a list of action JSON objects.
    ///
    /// Hook functions must return either:
    /// - A list of descriptor dicts: `[create_shim(...), set_permissions(...)]`
    /// - An empty list: `[]`
    /// - `None`: treated as empty list
    pub(super) fn parse_hook_actions(
        &self,
        json: &serde_json::Value,
        func_name: &str,
    ) -> Result<Vec<serde_json::Value>> {
        if json.is_null() {
            return Ok(vec![]);
        }
        if let Some(arr) = json.as_array() {
            return Ok(arr.clone());
        }
        warn!(
            provider = %self.meta.name,
            func = %func_name,
            "Hook function must return a list, got: {:?}",
            json
        );
        Ok(vec![])
    }

    /// Convert a JSON descriptor to a PostExtractAction
    pub(super) fn json_to_post_extract_action(
        &self,
        json: &serde_json::Value,
    ) -> Option<PostExtractAction> {
        let type_str = json.get("__type").and_then(|t| t.as_str()).unwrap_or("");

        match type_str {
            "create_shim" => {
                let name = json.get("name").and_then(|n| n.as_str())?.to_string();
                let target = json.get("target").and_then(|t| t.as_str())?.to_string();
                let args = json
                    .get("args")
                    .and_then(|a| a.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let shim_dir = json
                    .get("shim_dir")
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string());
                debug!(provider = %self.meta.name, shim = %name, target = %target, "Resolved create_shim descriptor");
                Some(PostExtractAction::CreateShim {
                    name,
                    target,
                    args,
                    shim_dir,
                })
            }

            "set_permissions" => {
                let path = json.get("path").and_then(|p| p.as_str())?.to_string();
                let mode = json
                    .get("mode")
                    .and_then(|m| m.as_str())
                    .unwrap_or("755")
                    .to_string();
                debug!(provider = %self.meta.name, path = %path, mode = %mode, "Resolved set_permissions descriptor");
                Some(PostExtractAction::SetPermissions { path, mode })
            }

            "run_command" => {
                let executable = json.get("executable").and_then(|e| e.as_str())?.to_string();
                let args = json
                    .get("args")
                    .and_then(|a| a.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let working_dir = json
                    .get("working_dir")
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string());
                let env = json
                    .get("env")
                    .and_then(|e| e.as_object())
                    .map(|obj| {
                        obj.iter()
                            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                            .collect()
                    })
                    .unwrap_or_default();
                let on_failure = json
                    .get("on_failure")
                    .and_then(|f| f.as_str())
                    .unwrap_or("warn")
                    .to_string();
                debug!(provider = %self.meta.name, executable = %executable, "Resolved run_command descriptor (post_extract)");
                Some(PostExtractAction::RunCommand {
                    executable,
                    args,
                    working_dir,
                    env,
                    on_failure,
                })
            }

            "flatten_dir" => {
                let pattern = json
                    .get("pattern")
                    .and_then(|p| p.as_str())
                    .map(|s| s.to_string());
                let keep_subdirs = json
                    .get("keep_subdirs")
                    .and_then(|k| k.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                debug!(provider = %self.meta.name, pattern = ?pattern, "Resolved flatten_dir descriptor");
                Some(PostExtractAction::FlattenDir {
                    pattern,
                    keep_subdirs,
                })
            }

            other => {
                warn!(provider = %self.meta.name, type_ = %other, "Unknown post_extract action type, ignoring");
                None
            }
        }
    }

    /// Convert a JSON descriptor to a PreRunAction
    pub(super) fn json_to_pre_run_action(&self, json: &serde_json::Value) -> Option<PreRunAction> {
        let type_str = json.get("__type").and_then(|t| t.as_str()).unwrap_or("");

        match type_str {
            "ensure_dependencies" => {
                let package_manager = json
                    .get("package_manager")
                    .and_then(|p| p.as_str())?
                    .to_string();
                let check_file = json
                    .get("check_file")
                    .and_then(|f| f.as_str())
                    .unwrap_or("package.json")
                    .to_string();
                let lock_file = json
                    .get("lock_file")
                    .and_then(|f| f.as_str())
                    .map(|s| s.to_string());
                let install_dir = json
                    .get("install_dir")
                    .and_then(|d| d.as_str())
                    .unwrap_or("node_modules")
                    .to_string();
                debug!(provider = %self.meta.name, pm = %package_manager, "Resolved ensure_dependencies descriptor");
                Some(PreRunAction::EnsureDependencies {
                    package_manager,
                    check_file,
                    lock_file,
                    install_dir,
                })
            }

            "run_command" => {
                let executable = json.get("executable").and_then(|e| e.as_str())?.to_string();
                let args = json
                    .get("args")
                    .and_then(|a| a.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let working_dir = json
                    .get("working_dir")
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string());
                let env = json
                    .get("env")
                    .and_then(|e| e.as_object())
                    .map(|obj| {
                        obj.iter()
                            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                            .collect()
                    })
                    .unwrap_or_default();
                let on_failure = json
                    .get("on_failure")
                    .and_then(|f| f.as_str())
                    .unwrap_or("warn")
                    .to_string();
                debug!(provider = %self.meta.name, executable = %executable, "Resolved run_command descriptor (pre_run)");
                Some(PreRunAction::RunCommand {
                    executable,
                    args,
                    working_dir,
                    env,
                    on_failure,
                })
            }

            other => {
                warn!(provider = %self.meta.name, type_ = %other, "Unknown pre_run action type, ignoring");
                None
            }
        }
    }
}
