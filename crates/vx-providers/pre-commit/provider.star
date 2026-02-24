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
name        = "pre-commit"
description = "A framework for managing and maintaining multi-language pre-commit hooks"
homepage    = "https://pre-commit.com"
repository  = "https://github.com/pre-commit/pre-commit"
license     = "MIT"
ecosystem   = "python"

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
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "pre-commit \\d+"},
        ],
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

def download_url(_ctx, _version):
    """pre-commit is installed via uvx, not direct download."""
    return None

# ---------------------------------------------------------------------------
# deps — requires uv
# ---------------------------------------------------------------------------

def deps(_ctx, version):
    """pre-commit requires uv for installation."""
    return [{"runtime": "uv", "version": "*"}]

# ---------------------------------------------------------------------------
# store_root — not managed by vx (installed via uvx)
# ---------------------------------------------------------------------------

def store_root(_ctx, _version):
    """pre-commit is installed via uvx — no vx store root."""
    return None

# ---------------------------------------------------------------------------
# get_execute_path — resolve pre-commit executable
# ---------------------------------------------------------------------------

def get_execute_path(_ctx, _version, install_dir):
    """pre-commit is run via uvx; no vx-managed install_dir."""
    return None

# ---------------------------------------------------------------------------
# post_install — nothing to do
# ---------------------------------------------------------------------------

def post_install(_ctx, _version):
    """No post-install steps required for pre-commit."""
    return []

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(_ctx, _version):
    return []
