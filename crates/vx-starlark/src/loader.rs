//! Module loader for Starlark provider scripts
//!
//! Implements the `@vx//stdlib` virtual module system, inspired by Buck2's `load()` mechanism.
//! Provider scripts can import shared utilities via:
//!
//! ```python
//! load("@vx//stdlib:semver.star", "semver_compare", "semver_strip_v")
//! load("@vx//stdlib:platform.star", "platform_triple", "is_windows")
//! ```

use starlark::environment::FrozenModule;
use starlark::syntax::{AstModule, Dialect};
use std::collections::HashMap;

/// Built-in Starlark stdlib modules bundled with vx
const SEMVER_STAR: &str = include_str!("../stdlib/semver.star");
const PLATFORM_STAR: &str = include_str!("../stdlib/platform.star");
const HTTP_STAR: &str = include_str!("../stdlib/http.star");

/// Module loader for `@vx//stdlib:*.star` virtual modules
///
/// Inspired by Buck2's prelude module system. Allows provider scripts to
/// share common utilities without duplicating code.
pub struct VxModuleLoader {
    /// Map from module path to source code
    modules: HashMap<String, &'static str>,
}

impl VxModuleLoader {
    /// Create a new module loader with all built-in stdlib modules
    pub fn new() -> Self {
        let mut modules = HashMap::new();
        modules.insert("@vx//stdlib:semver.star".to_string(), SEMVER_STAR);
        modules.insert("@vx//stdlib:platform.star".to_string(), PLATFORM_STAR);
        modules.insert("@vx//stdlib:http.star".to_string(), HTTP_STAR);
        Self { modules }
    }

    /// Check if a module path is a vx stdlib module
    pub fn is_vx_module(path: &str) -> bool {
        path.starts_with("@vx//")
    }

    /// Get the source code for a module
    pub fn get_source(&self, path: &str) -> Option<&'static str> {
        self.modules.get(path).copied()
    }

    /// Load and evaluate a stdlib module, returning a FrozenModule
    pub fn load_module(
        &self,
        path: &str,
        dialect: &Dialect,
    ) -> anyhow::Result<FrozenModule> {
        let source = self.get_source(path).ok_or_else(|| {
            anyhow::anyhow!("Unknown vx stdlib module: '{}'. Available modules: {}", path, self.available_modules().join(", "))
        })?;

        let ast = AstModule::parse(path, source.to_string(), dialect)
            .map_err(|e| anyhow::anyhow!("Failed to parse stdlib module '{}': {}", path, e))?;

        let globals = starlark::environment::GlobalsBuilder::standard().build();
        let module = starlark::environment::Module::new();
        {
            let mut eval = starlark::eval::Evaluator::new(&module);
            eval.eval_module(ast, &globals)
                .map_err(|e| anyhow::anyhow!("Failed to evaluate stdlib module '{}': {}", path, e))?;
        }

        module.freeze().map_err(|e| anyhow::anyhow!("Failed to freeze stdlib module '{}': {:?}", path, e))
    }

    /// List all available stdlib modules
    pub fn available_modules(&self) -> Vec<&str> {
        self.modules.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for VxModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}
