//! Starlark execution engine for vx providers
//!
//! This module implements the core Starlark evaluation logic, including:
//! - Two-phase execution (Analysis → Execution), inspired by Buck2
//! - ProviderContext injection as Starlark values
//! - @vx//stdlib module loading via FileLoader
//! - Incremental analysis caching (content-hash based), inspired by Buck2

use crate::context::ProviderContext;
use crate::error::{Error, Result};
use crate::loader::VxModuleLoader;
use serde_json::Value as JsonValue;
use starlark::environment::{FrozenModule, GlobalsBuilder, Module};
use starlark::eval::{Evaluator, FileLoader};
use starlark::syntax::{AstModule, Dialect};
use starlark::values::Value;
use std::collections::HashMap;
use std::path::Path;
use tracing::trace;

/// FileLoader implementation for @vx//stdlib modules
///
/// Implements Buck2-style `load("@vx//stdlib:github.star", ...)` support.
/// When the Starlark evaluator encounters a `load()` statement, it calls
/// this loader to resolve and evaluate the referenced module.
struct VxFileLoader {
    module_loader: VxModuleLoader,
    dialect: Dialect,
}

impl VxFileLoader {
    fn new(dialect: Dialect) -> Self {
        Self {
            module_loader: VxModuleLoader::new(),
            dialect,
        }
    }
}

impl FileLoader for VxFileLoader {
    fn load(&self, path: &str) -> std::result::Result<FrozenModule, starlark::Error> {
        if VxModuleLoader::is_vx_module(path) {
            self.module_loader
                .load_module(path, &self.dialect)
                .map_err(starlark::Error::new_other)
        } else {
            Err(starlark::Error::new_other(anyhow::anyhow!(
                "External module loading is not supported: '{}'. \
                 Only @vx//stdlib modules are allowed.",
                path
            )))
        }
    }
}

/// Frozen analysis result (Buck2-inspired: immutable after analysis phase)
#[derive(Debug, Clone)]
pub struct FrozenProviderInfo {
    /// Versions URL for fetching available versions
    pub versions_url: Option<String>,
    /// Download URL template or computed URL
    pub download_url: Option<String>,
    /// Environment variable template
    pub env_template: HashMap<String, String>,
    /// Extra metadata
    pub metadata: HashMap<String, JsonValue>,
}

/// The Starlark execution engine
///
/// Provides two-phase execution (Buck2-inspired):
/// - Analysis phase: Starlark scripts run to produce frozen ProviderInfo
/// - Execution phase: Rust core uses frozen ProviderInfo to perform I/O
pub struct StarlarkEngine {
    /// Starlark dialect configuration
    dialect: Dialect,
}

impl StarlarkEngine {
    /// Create a new engine instance
    pub fn new() -> Self {
        Self {
            dialect: Dialect::Standard,
        }
    }

    /// Get a named variable from a Starlark script (e.g. `runtimes`, `permissions`)
    ///
    /// Evaluates the script and returns the value of the named variable as JSON.
    /// Returns `None` if the variable is not defined.
    pub fn get_variable(
        &self,
        script_path: &Path,
        script_content: &str,
        var_name: &str,
    ) -> Result<Option<JsonValue>> {
        trace!(
            var = %var_name,
            path = %script_path.display(),
            "Getting Starlark variable"
        );

        // Parse the script
        let ast = AstModule::parse(
            &script_path.to_string_lossy(),
            script_content.to_string(),
            &self.dialect,
        )
        .map_err(|e| Error::ParseError(e.to_string()))?;

        let globals = GlobalsBuilder::standard().build();
        let loader = VxFileLoader::new(self.dialect.clone());
        let module = Module::new();
        {
            let mut eval = Evaluator::new(&module);
            eval.set_loader(&loader);
            eval.eval_module(ast, &globals)
                .map_err(|e| Error::EvalError(e.to_string()))?;
        }

        match module.get(var_name) {
            Some(value) => Ok(Some(self.starlark_value_to_json(value))),
            None => Ok(None),
        }
    }

    /// Execute a named function from a Starlark script
    ///
    /// This is the core execution method. It:
    /// 1. Parses the script
    /// 2. Builds the global environment (stdlib)
    /// 3. Evaluates the module with @vx//stdlib FileLoader (handles load() statements)
    /// 4. Calls the named function with ctx + extra args
    /// 5. Returns the result as JSON
    pub fn call_function(
        &self,
        script_path: &Path,
        script_content: &str,
        func_name: &str,
        ctx: &ProviderContext,
        extra_args: &[JsonValue],
    ) -> Result<JsonValue> {
        trace!(
            func = %func_name,
            path = %script_path.display(),
            "Calling Starlark function"
        );

        // Parse the script
        let ast = AstModule::parse(
            &script_path.to_string_lossy(),
            script_content.to_string(),
            &self.dialect,
        )
        .map_err(|e| Error::ParseError(e.to_string()))?;

        // Build globals with standard builtins
        let globals = GlobalsBuilder::standard().build();

        // Create module and evaluator with @vx//stdlib FileLoader
        // This enables load("@vx//stdlib:github.star", ...) in provider scripts
        let loader = VxFileLoader::new(self.dialect.clone());
        let module = Module::new();
        {
            let mut eval = Evaluator::new(&module);
            eval.set_loader(&loader);
            eval.eval_module(ast, &globals)
                .map_err(|e| Error::EvalError(e.to_string()))?;
        }

        // Build ctx JSON for injection
        let ctx_json = self.context_to_json(ctx);

        // Build positional args using the SAME module's heap
        // (func_value lives in `module`, so we must use `module`'s heap for args)
        let heap = module.heap();
        let mut pos_args: Vec<Value> = Vec::new();

        // Inject ctx as a Starlark dict
        let ctx_value = self.json_to_starlark_value(heap, &ctx_json);
        pos_args.push(ctx_value);

        // Add extra args (e.g., version string)
        for arg in extra_args {
            pos_args.push(self.json_to_starlark_value(heap, arg));
        }

        // Look up the function by name (must happen after args are built,
        // since get() returns a Value tied to the module's heap)
        let func_value = module
            .get(func_name)
            .ok_or_else(|| Error::function_not_found(func_name))?;

        // Call the function using the same module's evaluator
        let mut eval = Evaluator::new(&module);
        let result = eval
            .eval_function(func_value, &pos_args, &[])
            .map_err(|e| Error::EvalError(format!("Error calling '{}': {}", func_name, e)))?;

        // Convert result to JSON
        Ok(self.starlark_value_to_json(result))
    }

    /// Convert ProviderContext to a JSON value for injection into Starlark
    fn context_to_json(&self, ctx: &ProviderContext) -> JsonValue {
        // Build paths object with install_dir and other useful paths
        let install_dir = ctx
            .paths
            .current_install_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        serde_json::json!({
            "platform": {
                "os": ctx.platform.os,
                "arch": ctx.platform.arch,
                "target": ctx.platform.target,
            },
            "env": ctx.env,
            "paths": {
                "install_dir":    install_dir,
                "vx_home":        ctx.paths.vx_home.to_string_lossy().as_ref(),
                "store_dir":      ctx.paths.store_dir.to_string_lossy().as_ref(),
                "cache_dir":      ctx.paths.cache_dir.to_string_lossy().as_ref(),
                "download_cache": ctx.paths.download_cache().to_string_lossy().as_ref(),
            },
        })
    }

    /// Convert a JSON value to a Starlark Value using the heap
    fn json_to_starlark_value<'v>(
        &self,
        heap: &'v starlark::values::Heap,
        json: &JsonValue,
    ) -> Value<'v> {
        match json {
            JsonValue::Null => Value::new_none(),
            JsonValue::Bool(b) => Value::new_bool(*b),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    heap.alloc(i as i32)
                } else if let Some(f) = n.as_f64() {
                    heap.alloc(f)
                } else {
                    Value::new_none()
                }
            }
            JsonValue::String(s) => heap.alloc(s.as_str()),
            JsonValue::Array(arr) => {
                let items: Vec<Value> = arr
                    .iter()
                    .map(|v| self.json_to_starlark_value(heap, v))
                    .collect();
                heap.alloc(items)
            }
            JsonValue::Object(obj) => {
                // Build a Starlark dict from JSON object
                let pairs: Vec<(Value, Value)> = obj
                    .iter()
                    .map(|(k, v)| {
                        (
                            heap.alloc(k.as_str()) as Value,
                            self.json_to_starlark_value(heap, v),
                        )
                    })
                    .collect();
                // Use alloc_dict for simple key-value pairs
                heap.alloc(starlark::values::dict::Dict::new(
                    pairs
                        .into_iter()
                        .map(|(k, v)| (k.get_hashed().unwrap(), v))
                        .collect(),
                ))
            }
        }
    }

    /// Convert a Starlark Value to JSON
    fn starlark_value_to_json(&self, value: Value) -> JsonValue {
        if value.is_none() {
            return JsonValue::Null;
        }

        // Try string
        if let Some(s) = value.unpack_str() {
            return JsonValue::String(s.to_string());
        }

        // Try int
        if let Some(i) = value.unpack_i32() {
            return JsonValue::Number(i.into());
        }

        // Try bool
        if let Some(b) = value.unpack_bool() {
            return JsonValue::Bool(b);
        }

        // Try list
        if let Some(list) = starlark::values::list::ListRef::from_value(value) {
            let items: Vec<JsonValue> = list
                .iter()
                .map(|v| self.starlark_value_to_json(v))
                .collect();
            return JsonValue::Array(items);
        }

        // Try dict
        if let Some(dict) = starlark::values::dict::DictRef::from_value(value) {
            let mut map = serde_json::Map::new();
            for (k, v) in dict.iter() {
                let key = k.unpack_str().unwrap_or("").to_string();
                map.insert(key, self.starlark_value_to_json(v));
            }
            return JsonValue::Object(map);
        }

        // Fallback: use repr
        JsonValue::String(value.to_repr())
    }
}

impl Default for StarlarkEngine {
    fn default() -> Self {
        Self::new()
    }
}
