//! Module loader for Starlark provider scripts
//!
//! Implements the `@vx//` virtual module system, inspired by Buck2's `load()` mechanism.
//! Provider scripts can import shared utilities via:
//!
//! ```python
//! load("@vx//stdlib:semver.star", "semver_compare", "semver_strip_v")
//! ```

use starlark::environment::FrozenModule;
use starlark::eval::FileLoader;
use starlark::syntax::{AstModule, Dialect};
use std::collections::HashMap;

/// Built-in Starlark stdlib modules bundled with vx
const SEMVER_STAR: &str = include_str!("../stdlib/semver.star");
const PLATFORM_STAR: &str = include_str!("../stdlib/platform.star");
const HTTP_STAR: &str = include_str!("../stdlib/http.star");
const GITHUB_STAR: &str = include_str!("../stdlib/github.star");
const INSTALL_STAR: &str = include_str!("../stdlib/install.star");
const ENV_STAR: &str = include_str!("../stdlib/env.star");
const LAYOUT_STAR: &str = include_str!("../stdlib/layout.star");
const PERMISSIONS_STAR: &str = include_str!("../stdlib/permissions.star");
const PROVIDER_STAR: &str = include_str!("../stdlib/provider.star");
const PROVIDER_TEMPLATES_STAR: &str = include_str!("../stdlib/provider_templates.star");
const RUNTIME_STAR: &str = include_str!("../stdlib/runtime.star");
const SCRIPT_INSTALL_STAR: &str = include_str!("../stdlib/script_install.star");
const SYSTEM_INSTALL_STAR: &str = include_str!("../stdlib/system_install.star");
const SMART_DETECT_STAR: &str = include_str!("../stdlib/smart_detect.star");
const TEST_STAR: &str = include_str!("../stdlib/test.star");

/// Module loader for `@vx//stdlib:*` virtual modules.
///
/// Inspired by Buck2's prelude module system. Allows provider scripts to
/// share common utilities without duplicating code.
pub struct VxModuleLoader {
    /// Map from stdlib module path to source code
    stdlib_modules: HashMap<String, &'static str>,
}

impl VxModuleLoader {
    /// Create a new module loader with all built-in modules.
    pub fn new() -> Self {
        let mut stdlib_modules = HashMap::new();
        stdlib_modules.insert("@vx//stdlib:semver.star".to_string(), SEMVER_STAR);
        stdlib_modules.insert("@vx//stdlib:platform.star".to_string(), PLATFORM_STAR);
        stdlib_modules.insert("@vx//stdlib:http.star".to_string(), HTTP_STAR);
        stdlib_modules.insert("@vx//stdlib:github.star".to_string(), GITHUB_STAR);
        stdlib_modules.insert("@vx//stdlib:install.star".to_string(), INSTALL_STAR);
        stdlib_modules.insert("@vx//stdlib:env.star".to_string(), ENV_STAR);
        stdlib_modules.insert("@vx//stdlib:layout.star".to_string(), LAYOUT_STAR);
        stdlib_modules.insert("@vx//stdlib:permissions.star".to_string(), PERMISSIONS_STAR);
        stdlib_modules.insert("@vx//stdlib:provider.star".to_string(), PROVIDER_STAR);
        stdlib_modules.insert(
            "@vx//stdlib:provider_templates.star".to_string(),
            PROVIDER_TEMPLATES_STAR,
        );
        stdlib_modules.insert("@vx//stdlib:runtime.star".to_string(), RUNTIME_STAR);
        stdlib_modules.insert(
            "@vx//stdlib:script_install.star".to_string(),
            SCRIPT_INSTALL_STAR,
        );
        stdlib_modules.insert(
            "@vx//stdlib:system_install.star".to_string(),
            SYSTEM_INSTALL_STAR,
        );
        stdlib_modules.insert("@vx//stdlib:test.star".to_string(), TEST_STAR);
        stdlib_modules.insert(
            "@vx//stdlib:smart_detect.star".to_string(),
            SMART_DETECT_STAR,
        );

        Self { stdlib_modules }
    }

    /// Check if a module path is a vx virtual module.
    pub fn is_vx_module(path: &str) -> bool {
        path.starts_with("@vx//")
    }

    /// Get the source code for a stdlib module.
    pub fn get_source(&self, path: &str) -> Option<&'static str> {
        self.stdlib_modules.get(path).copied()
    }

    /// Load and evaluate a vx virtual module, returning a FrozenModule.
    ///
    /// Supports recursive loading: `github.star` can `load("@vx//stdlib:http.star", ...)`
    /// because the evaluator is given a self-referential loader.
    pub fn load_module(&self, path: &str, dialect: &Dialect) -> anyhow::Result<FrozenModule> {
        let source = self.get_source(path).ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown vx module: '{}'. Available modules: {}",
                path,
                self.available_modules().join(", ")
            )
        })?;

        let ast = AstModule::parse(path, source.to_string(), dialect)
            .map_err(|e| anyhow::anyhow!("Failed to parse vx module '{}': {}", path, e))?;

        let globals = starlark::environment::GlobalsBuilder::standard().build();
        // starlark 0.14 removed `Module::new()`: a thawed module borrows a
        // temporary heap, so evaluation happens inside a scoped callback.
        starlark::environment::Module::with_temp_heap(|module| {
            {
                // Use a recursive loader so modules can load each other.
                let recursive_loader = VxModuleLoader::new();
                let dialect_clone = dialect.clone();
                let file_loader = RecursiveVxLoader {
                    loader: recursive_loader,
                    dialect: dialect_clone,
                };
                let mut eval = starlark::eval::Evaluator::new(&module);
                eval.set_loader(&file_loader);
                eval.eval_module(ast, &globals).map_err(|e| {
                    anyhow::anyhow!("Failed to evaluate vx module '{}': {}", path, e)
                })?;
            }

            module
                .freeze()
                .map_err(|e| anyhow::anyhow!("Failed to freeze vx module '{}': {:?}", path, e))
        })
    }

    /// Load and evaluate a vx virtual module, handing the thawed `Module` to `f`.
    ///
    /// Unlike [`load_module`], the module is not frozen, so it allows individual
    /// function lookups and calls. Used by the Rust runtime to invoke scoring
    /// functions in `smart_detect.star`.
    ///
    /// starlark 0.14 removed `Module::new()`: a thawed module borrows a temporary
    /// heap that only lives for the duration of a scope, so the module cannot be
    /// returned. Callers receive it inside `f` instead.
    pub fn with_evaluable_module<R, F>(
        &self,
        path: &str,
        dialect: &Dialect,
        f: F,
    ) -> anyhow::Result<R>
    where
        F: for<'v> FnOnce(starlark::environment::Module<'v>) -> anyhow::Result<R>,
    {
        let source = self.get_source(path).ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown vx module: '{}'. Available modules: {}",
                path,
                self.available_modules().join(", ")
            )
        })?;

        let ast = AstModule::parse(path, source.to_string(), dialect)
            .map_err(|e| anyhow::anyhow!("Failed to parse vx module '{}': {}", path, e))?;

        let globals = starlark::environment::GlobalsBuilder::standard().build();
        starlark::environment::Module::with_temp_heap(|module| {
            {
                let recursive_loader = VxModuleLoader::new();
                let dialect_clone = dialect.clone();
                let file_loader = RecursiveVxLoader {
                    loader: recursive_loader,
                    dialect: dialect_clone,
                };
                let mut eval = starlark::eval::Evaluator::new(&module);
                eval.set_loader(&file_loader);
                eval.eval_module(ast, &globals).map_err(|e| {
                    anyhow::anyhow!("Failed to evaluate vx module '{}': {}", path, e)
                })?;
            }

            f(module)
        })
    }

    /// List all available stdlib vx modules.
    pub fn available_modules(&self) -> Vec<&str> {
        self.stdlib_modules.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for VxModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal recursive loader used when evaluating vx modules that themselves
/// contain `load()` statements.
struct RecursiveVxLoader {
    loader: VxModuleLoader,
    dialect: Dialect,
}

impl FileLoader for RecursiveVxLoader {
    fn load(&self, path: &str) -> std::result::Result<FrozenModule, starlark::Error> {
        if VxModuleLoader::is_vx_module(path) {
            self.loader
                .load_module(path, &self.dialect)
                .map_err(starlark::Error::new_other)
        } else {
            Err(starlark::Error::new_other(anyhow::anyhow!(
                "External module loading is not supported in vx modules: '{}'",
                path
            )))
        }
    }
}
