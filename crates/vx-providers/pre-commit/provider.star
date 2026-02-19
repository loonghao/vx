# provider.star - pre-commit provider
#
# pre-commit: A framework for managing and maintaining multi-language pre-commit hooks
# Inheritance pattern: Level 3 (package alias - runs via uvx)
#   - fetch_versions: inherited from github.star
#   - download_url:   None (installed via uvx, not direct download)
#   - deps:           requires uv
#
# pre-commit is a Python tool, installed via `uvx pre-commit` or `pip install pre-commit`

load("@vx//stdlib:github.star", "make_fetch_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "pre-commit"

def description():
    return "A framework for managing and maintaining multi-language pre-commit hooks"

def homepage():
    return "https://pre-commit.com"

def repository():
    return "https://github.com/pre-commit/pre-commit"

def license():
    return "MIT"

def ecosystem():
    return "python"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "pre-commit",
        "executable":  "pre-commit",
        "description": "Pre-commit hook framework",
        "aliases":     [],
        "priority":    100,
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — inherited
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("pre-commit", "pre-commit")

# ---------------------------------------------------------------------------
# download_url — None (installed via uvx)
#
# pre-commit is a Python package, not a direct binary download.
# It should be installed via: uvx pre-commit
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """pre-commit is installed via uvx, not direct download."""
    return None

# ---------------------------------------------------------------------------
# deps — requires uv
# ---------------------------------------------------------------------------

def deps(ctx, version):
    """pre-commit requires uv for installation."""
    return [{"runtime": "uv", "version": "*"}]

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {}
