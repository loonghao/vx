//! Python script detection rules
//!
//! Defines rules for detecting common Python scripts based on file presence.

use crate::languages::rules::ScriptRule;

/// All Python script detection rules
///
/// Rules are evaluated by priority (highest first).
/// For each script name, only the highest priority matching rule is used.
pub const PYTHON_RULES: &[ScriptRule] = &[
    // =========================================================================
    // Nox - Task automation
    // =========================================================================
    ScriptRule::new("nox", "uvx nox", "Run nox sessions")
        .triggers(&["noxfile.py"])
        .priority(100),
    // =========================================================================
    // Test runners (ordered by priority)
    // =========================================================================
    // nox-based testing (highest priority)
    ScriptRule::new("test", "uvx nox -s tests", "Run tests via nox")
        .triggers(&["noxfile.py"])
        .priority(100),
    // tox-based testing
    ScriptRule::new("test", "uvx tox", "Run tests via tox")
        .triggers(&["tox.ini", "tox.toml"])
        .excludes(&["noxfile.py"])
        .priority(90),
    // pytest (default)
    ScriptRule::new("test", "uv run pytest", "Run tests with pytest")
        .triggers(&["pytest.ini", "pyproject.toml", "tests", "test"])
        .excludes(&["noxfile.py", "tox.ini", "tox.toml"])
        .priority(50),
    // =========================================================================
    // Linting
    // =========================================================================
    ScriptRule::new("lint", "uvx nox -s lint", "Run linter via nox")
        .triggers(&["noxfile.py"])
        .priority(100),
    ScriptRule::new("lint", "uv run ruff check .", "Run ruff linter")
        .triggers(&["ruff.toml", "pyproject.toml"])
        .excludes(&["noxfile.py"])
        .priority(50),
    // =========================================================================
    // Formatting
    // =========================================================================
    ScriptRule::new("format", "uv run ruff format .", "Format code with ruff")
        .triggers(&["ruff.toml", "pyproject.toml"])
        .excludes(&["noxfile.py"])
        .priority(50),
    // =========================================================================
    // Type checking
    // =========================================================================
    ScriptRule::new("typecheck", "uv run mypy .", "Run type checker")
        .triggers(&["mypy.ini", "pyproject.toml"])
        .excludes(&["noxfile.py"])
        .priority(50),
];
