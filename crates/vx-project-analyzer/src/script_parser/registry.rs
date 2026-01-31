//! Pattern registry and provider trait for script parsing

use super::types::{ParseContext, ScriptTool, ToolInvocation};
use regex::Regex;
use std::sync::LazyLock;

/// Trait for language-specific script pattern providers
///
/// Implement this trait to add support for detecting tools from script commands
/// for a specific language or ecosystem.
///
/// # Example
///
/// ```ignore
/// use vx_project_analyzer::script_parser::{ScriptPatternProvider, ParseContext, ScriptTool};
///
/// struct MyProvider;
///
/// impl ScriptPatternProvider for MyProvider {
///     fn name(&self) -> &'static str { "my_provider" }
///     fn parse(&self, _cmd: &str, _ctx: &ParseContext) -> Option<ScriptTool> { None }
///     fn known_tools(&self) -> &[&'static str] { &[] }
/// }
/// ```
#[allow(dead_code)]
pub trait ScriptPatternProvider: Send + Sync {
    /// Name of this pattern provider (e.g., "python", "nodejs")
    ///
    /// This is used for debugging and logging purposes.
    fn name(&self) -> &'static str;

    /// Try to parse a command and extract a tool
    ///
    /// Returns `Some(ScriptTool)` if the command matches a known pattern,
    /// or `None` if it doesn't match.
    fn parse(&self, command: &str, context: &ParseContext) -> Option<ScriptTool>;

    /// Get the list of known common tools for this language
    ///
    /// These are tools that are typically installed via package managers
    /// and are NOT project-specific modules.
    ///
    /// This is useful for checking if a tool name is a known tool.
    fn known_tools(&self) -> &[&'static str];
}

/// Registry of script pattern providers
pub struct PatternRegistry {
    providers: Vec<Box<dyn ScriptPatternProvider>>,
}

impl PatternRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Register a pattern provider
    pub fn register(&mut self, provider: Box<dyn ScriptPatternProvider>) {
        self.providers.push(provider);
    }

    /// Get all registered providers
    pub fn providers(&self) -> &[Box<dyn ScriptPatternProvider>] {
        &self.providers
    }
}

impl Default for PatternRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Python Pattern Provider
// ============================================================================

/// Known common Python tools/modules that are typically installed via pip/uv
pub static KNOWN_PYTHON_TOOLS: &[&str] = &[
    // Testing
    "pytest",
    "unittest",
    "nose",
    "nose2",
    "tox",
    "nox",
    // Linting & Formatting
    "ruff",
    "black",
    "isort",
    "flake8",
    "pylint",
    "mypy",
    "pyright",
    // Build & Package
    "build",
    "setuptools",
    "wheel",
    "twine",
    "hatch",
    "pdm",
    "poetry",
    // Documentation
    "sphinx",
    "mkdocs",
    // Coverage
    "coverage",
    "pytest_cov",
    // Other common tools
    "pip",
    "pipenv",
    "virtualenv",
    "bandit",
    "safety",
];

/// Python script pattern provider
pub struct PythonPatternProvider {
    uv_run: Regex,
    uvx: Regex,
    python_m: Regex,
}

impl PythonPatternProvider {
    /// Create a new Python pattern provider
    pub fn new() -> Self {
        Self {
            uv_run: Regex::new(r"uv\s+run\s+([a-zA-Z0-9_-]+)(?:\s+(.*))?").unwrap(),
            uvx: Regex::new(r"uvx\s+([a-zA-Z0-9_-]+)(?:\s+(.*))?").unwrap(),
            python_m: Regex::new(r"python(?:3)?\s+-m\s+([a-zA-Z0-9_]+)(?:\s+(.*))?").unwrap(),
        }
    }

    fn parse_args(args_str: &str) -> Vec<String> {
        args_str.split_whitespace().map(|s| s.to_string()).collect()
    }

    fn try_uv_run(&self, command: &str) -> Option<ScriptTool> {
        self.uv_run.captures(command).map(|caps| {
            let name = caps.get(1).unwrap().as_str().to_string();
            let args = caps
                .get(2)
                .map(|m| Self::parse_args(m.as_str()))
                .unwrap_or_default();
            let mut tool = ScriptTool::new(name.clone(), ToolInvocation::UvRun).with_args(args);
            if !KNOWN_PYTHON_TOOLS.contains(&name.as_str()) {
                tool = tool.project_specific();
            }
            tool
        })
    }

    fn try_uvx(&self, command: &str) -> Option<ScriptTool> {
        self.uvx.captures(command).map(|caps| {
            let name = caps.get(1).unwrap().as_str().to_string();
            let args = caps
                .get(2)
                .map(|m| Self::parse_args(m.as_str()))
                .unwrap_or_default();
            let mut tool = ScriptTool::new(name.clone(), ToolInvocation::Uvx).with_args(args);
            if !KNOWN_PYTHON_TOOLS.contains(&name.as_str()) {
                tool = tool.project_specific();
            }
            tool
        })
    }

    fn try_python_m(&self, command: &str) -> Option<ScriptTool> {
        self.python_m.captures(command).map(|caps| {
            let name = caps.get(1).unwrap().as_str().to_string();
            let args = caps
                .get(2)
                .map(|m| Self::parse_args(m.as_str()))
                .unwrap_or_default();
            let mut tool =
                ScriptTool::new(name.clone(), ToolInvocation::PythonModule).with_args(args);
            if !KNOWN_PYTHON_TOOLS.contains(&name.as_str()) {
                tool = tool.project_specific();
            }
            tool
        })
    }
}

impl Default for PythonPatternProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptPatternProvider for PythonPatternProvider {
    fn name(&self) -> &'static str {
        "python"
    }

    fn parse(&self, command: &str, _context: &ParseContext) -> Option<ScriptTool> {
        self.try_uv_run(command)
            .or_else(|| self.try_uvx(command))
            .or_else(|| self.try_python_m(command))
    }

    fn known_tools(&self) -> &[&'static str] {
        KNOWN_PYTHON_TOOLS
    }
}

// ============================================================================
// Node.js Pattern Provider
// ============================================================================

/// Known common Node.js tools that are typically installed via npm/pnpm/yarn
pub static KNOWN_NODEJS_TOOLS: &[&str] = &[
    // Build tools
    "vite",
    "webpack",
    "rollup",
    "esbuild",
    "parcel",
    // Testing
    "jest",
    "vitest",
    "mocha",
    "ava",
    "playwright",
    "cypress",
    // Linting & Formatting
    "eslint",
    "prettier",
    "stylelint",
    "tslint",
    // TypeScript
    "tsc",
    "typescript",
    // Package managers
    "npm",
    "yarn",
    "pnpm",
    // Other common tools
    "nodemon",
    "ts-node",
    "tsx",
];

/// Built-in package manager commands that should not be treated as tools
static PM_BUILTINS: &[&str] = &[
    "install",
    "add",
    "remove",
    "update",
    "upgrade",
    "run",
    "exec",
    "init",
    "publish",
    "pack",
    "test",
    "start",
    "build",
    "dev",
    "lint",
    "format",
    "typecheck",
];

/// Node.js script pattern provider
pub struct NodeJsPatternProvider {
    npx: Regex,
    pnpm_exec: Regex,
    pnpm_run: Regex,
    yarn_exec: Regex,
    bunx: Regex,
}

impl NodeJsPatternProvider {
    /// Create a new Node.js pattern provider
    pub fn new() -> Self {
        Self {
            npx: Regex::new(r"npx\s+(?:--yes\s+)?([a-zA-Z0-9@/_-]+)(?:\s+(.*))?").unwrap(),
            pnpm_exec: Regex::new(r"pnpm\s+exec\s+([a-zA-Z0-9_-]+)(?:\s+(.*))?").unwrap(),
            pnpm_run: Regex::new(r"pnpm\s+(?:run\s+)?([a-zA-Z0-9_:-]+)(?:\s+(.*))?").unwrap(),
            yarn_exec: Regex::new(r"yarn\s+(?:exec\s+)?([a-zA-Z0-9_-]+)(?:\s+(.*))?").unwrap(),
            bunx: Regex::new(r"bunx?\s+([a-zA-Z0-9@/_-]+)(?:\s+(.*))?").unwrap(),
        }
    }

    fn parse_args(args_str: &str) -> Vec<String> {
        args_str.split_whitespace().map(|s| s.to_string()).collect()
    }

    fn try_npx(&self, command: &str) -> Option<ScriptTool> {
        self.npx.captures(command).map(|caps| {
            let name = caps.get(1).unwrap().as_str().to_string();
            let args = caps
                .get(2)
                .map(|m| Self::parse_args(m.as_str()))
                .unwrap_or_default();
            let mut tool = ScriptTool::new(name.clone(), ToolInvocation::Npx).with_args(args);
            if !KNOWN_NODEJS_TOOLS.contains(&name.as_str()) {
                tool = tool.project_specific();
            }
            tool
        })
    }

    fn try_pnpm(&self, command: &str, context: &ParseContext) -> Option<ScriptTool> {
        // First try explicit exec pattern
        if let Some(caps) = self.pnpm_exec.captures(command) {
            let name = caps.get(1).unwrap().as_str();
            if PM_BUILTINS.contains(&name) || context.known_scripts.contains(&name) {
                return None;
            }
            let args = caps
                .get(2)
                .map(|m| Self::parse_args(m.as_str()))
                .unwrap_or_default();
            return Some(
                ScriptTool::new(name.to_string(), ToolInvocation::PnpmExec).with_args(args),
            );
        }

        // Then try pnpm run pattern
        if let Some(caps) = self.pnpm_run.captures(command) {
            let name = caps.get(1).unwrap().as_str();
            if PM_BUILTINS.contains(&name) || context.known_scripts.contains(&name) {
                return None;
            }
            // Skip script-like names (containing : or -)
            if name.contains(':') || name.contains('-') {
                return None;
            }
            let args = caps
                .get(2)
                .map(|m| Self::parse_args(m.as_str()))
                .unwrap_or_default();
            return Some(
                ScriptTool::new(name.to_string(), ToolInvocation::PnpmExec).with_args(args),
            );
        }

        None
    }

    fn try_yarn(&self, command: &str, context: &ParseContext) -> Option<ScriptTool> {
        self.yarn_exec.captures(command).and_then(|caps| {
            let name = caps.get(1).unwrap().as_str();
            if PM_BUILTINS.contains(&name) || context.known_scripts.contains(&name) {
                return None;
            }
            if name.contains(':') || name.contains('-') {
                return None;
            }
            let args = caps
                .get(2)
                .map(|m| Self::parse_args(m.as_str()))
                .unwrap_or_default();
            Some(ScriptTool::new(name.to_string(), ToolInvocation::YarnExec).with_args(args))
        })
    }

    fn try_bunx(&self, command: &str) -> Option<ScriptTool> {
        self.bunx.captures(command).map(|caps| {
            let name = caps.get(1).unwrap().as_str().to_string();
            let args = caps
                .get(2)
                .map(|m| Self::parse_args(m.as_str()))
                .unwrap_or_default();
            ScriptTool::new(name, ToolInvocation::Bunx).with_args(args)
        })
    }
}

impl Default for NodeJsPatternProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptPatternProvider for NodeJsPatternProvider {
    fn name(&self) -> &'static str {
        "nodejs"
    }

    fn parse(&self, command: &str, context: &ParseContext) -> Option<ScriptTool> {
        self.try_npx(command)
            .or_else(|| self.try_pnpm(command, context))
            .or_else(|| self.try_yarn(command, context))
            .or_else(|| self.try_bunx(command))
    }

    fn known_tools(&self) -> &[&'static str] {
        KNOWN_NODEJS_TOOLS
    }
}

// ============================================================================
// Default Registry
// ============================================================================

/// Default pattern registry with all built-in providers
pub static DEFAULT_REGISTRY: LazyLock<PatternRegistry> = LazyLock::new(|| {
    let mut registry = PatternRegistry::new();
    registry.register(Box::new(PythonPatternProvider::new()));
    registry.register(Box::new(NodeJsPatternProvider::new()));
    registry
});

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_context() -> ParseContext<'static> {
        ParseContext { known_scripts: &[] }
    }

    // ========================================================================
    // User's specific test cases (from the bug report)
    // ========================================================================

    #[test]
    fn test_python_m_project_specific_module() {
        // User's case: python -m auroraview.__main__
        // auroraview is the project name, not a known tool
        let provider = PythonPatternProvider::new();
        let ctx = empty_context();

        let result = provider.parse("python -m auroraview", &ctx);
        assert!(result.is_some(), "Should parse python -m auroraview");

        let tool = result.unwrap();
        assert_eq!(tool.name, "auroraview");
        assert_eq!(tool.invocation, ToolInvocation::PythonModule);
        assert!(
            tool.is_project_specific,
            "auroraview should be marked as project-specific"
        );
    }

    #[test]
    fn test_uvx_known_tool() {
        // User's case: uvx nox
        // nox is a known Python tool
        let provider = PythonPatternProvider::new();
        let ctx = empty_context();

        let result = provider.parse("uvx nox", &ctx);
        assert!(result.is_some(), "Should parse uvx nox");

        let tool = result.unwrap();
        assert_eq!(tool.name, "nox");
        assert_eq!(tool.invocation, ToolInvocation::Uvx);
        assert!(
            !tool.is_project_specific,
            "nox should NOT be marked as project-specific"
        );
    }

    #[test]
    fn test_uvx_unknown_tool_is_project_specific() {
        // uvx with an unknown tool should be marked as project-specific
        let provider = PythonPatternProvider::new();
        let ctx = empty_context();

        let result = provider.parse("uvx my_custom_tool", &ctx);
        assert!(result.is_some());

        let tool = result.unwrap();
        assert_eq!(tool.name, "my_custom_tool");
        assert!(
            tool.is_project_specific,
            "Unknown tool should be marked as project-specific"
        );
    }

    // ========================================================================
    // Python Pattern Provider tests
    // ========================================================================

    #[test]
    fn test_python_m_known_tools() {
        let provider = PythonPatternProvider::new();
        let ctx = empty_context();

        // pytest is a known tool
        let result = provider.parse("python -m pytest tests/", &ctx);
        assert!(result.is_some());
        let tool = result.unwrap();
        assert_eq!(tool.name, "pytest");
        assert!(!tool.is_project_specific);

        // ruff is a known tool
        let result = provider.parse("python3 -m ruff check .", &ctx);
        assert!(result.is_some());
        let tool = result.unwrap();
        assert_eq!(tool.name, "ruff");
        assert!(!tool.is_project_specific);
    }

    #[test]
    fn test_uv_run_patterns() {
        let provider = PythonPatternProvider::new();
        let ctx = empty_context();

        // Known tool
        let result = provider.parse("uv run pytest", &ctx);
        assert!(result.is_some());
        let tool = result.unwrap();
        assert_eq!(tool.name, "pytest");
        assert_eq!(tool.invocation, ToolInvocation::UvRun);
        assert!(!tool.is_project_specific);

        // Unknown tool
        let result = provider.parse("uv run myapp --serve", &ctx);
        assert!(result.is_some());
        let tool = result.unwrap();
        assert_eq!(tool.name, "myapp");
        assert!(tool.is_project_specific);
    }

    // ========================================================================
    // Node.js Pattern Provider tests
    // ========================================================================

    #[test]
    fn test_npx_known_tools() {
        let provider = NodeJsPatternProvider::new();
        let ctx = empty_context();

        // vite is a known tool
        let result = provider.parse("npx vite build", &ctx);
        assert!(result.is_some());
        let tool = result.unwrap();
        assert_eq!(tool.name, "vite");
        assert!(!tool.is_project_specific);

        // Unknown tool
        let result = provider.parse("npx my-custom-cli", &ctx);
        assert!(result.is_some());
        let tool = result.unwrap();
        assert_eq!(tool.name, "my-custom-cli");
        assert!(tool.is_project_specific);
    }

    #[test]
    fn test_pnpm_exec_filters_builtins() {
        let provider = NodeJsPatternProvider::new();
        let ctx = empty_context();

        // pnpm run build should be filtered out (builtin)
        let result = provider.parse("pnpm run build", &ctx);
        assert!(result.is_none(), "pnpm run build should be filtered");

        // pnpm install should be filtered out
        let result = provider.parse("pnpm install", &ctx);
        assert!(result.is_none(), "pnpm install should be filtered");
    }

    #[test]
    fn test_pnpm_exec_filters_known_scripts() {
        let provider = NodeJsPatternProvider::new();
        let ctx = ParseContext {
            known_scripts: &["my-script"],
        };

        // Known script should be filtered out
        let result = provider.parse("pnpm exec my-script", &ctx);
        assert!(
            result.is_none(),
            "Known script 'my-script' should be filtered"
        );
    }

    #[test]
    fn test_yarn_patterns() {
        let provider = NodeJsPatternProvider::new();
        let ctx = empty_context();

        // Builtin should be filtered
        let result = provider.parse("yarn install", &ctx);
        assert!(result.is_none());

        // yarn exec with known tool
        let result = provider.parse("yarn exec eslint .", &ctx);
        assert!(result.is_some());
        let tool = result.unwrap();
        assert_eq!(tool.name, "eslint");
    }

    #[test]
    fn test_bunx_patterns() {
        let provider = NodeJsPatternProvider::new();
        let ctx = empty_context();

        let result = provider.parse("bunx vite", &ctx);
        assert!(result.is_some());
        let tool = result.unwrap();
        assert_eq!(tool.name, "vite");
        assert_eq!(tool.invocation, ToolInvocation::Bunx);
    }

    // ========================================================================
    // Registry tests
    // ========================================================================

    #[test]
    fn test_default_registry_has_providers() {
        assert!(
            !DEFAULT_REGISTRY.providers().is_empty(),
            "Default registry should have providers"
        );
    }

    #[test]
    fn test_provider_name_and_known_tools() {
        // Test Python provider
        let python = PythonPatternProvider::new();
        assert_eq!(python.name(), "python");
        assert!(python.known_tools().contains(&"pytest"));
        assert!(python.known_tools().contains(&"nox"));
        assert!(!python.known_tools().contains(&"unknown_tool"));

        // Test Node.js provider
        let nodejs = NodeJsPatternProvider::new();
        assert_eq!(nodejs.name(), "nodejs");
        assert!(nodejs.known_tools().contains(&"vite"));
        assert!(nodejs.known_tools().contains(&"eslint"));
        assert!(!nodejs.known_tools().contains(&"unknown_tool"));
    }
}
