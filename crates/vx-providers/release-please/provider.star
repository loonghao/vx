# provider.star - release-please provider
#
# release-please: Automated release PRs based on Conventional Commits
# Inheritance pattern: Level 3 (package alias - runs via npx)
#   - fetch_versions: inherited from github.star
#   - download_url:   None (installed via npx, not direct download)
#   - deps:           requires node
#
# release-please is a Node.js tool, run via `npx release-please`

load("@vx//stdlib:github.star", "make_fetch_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "release-please"
description = "Automated release PRs based on Conventional Commits"
homepage    = "https://github.com/googleapis/release-please"
repository  = "https://github.com/googleapis/release-please"
license     = "Apache-2.0"
ecosystem   = "nodejs"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "release-please",
        "executable":  "release-please",
        "description": "Automated release PR tool",
        "aliases":     [],
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "\\d+\\.\\d+"},
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com", "registry.npmjs.org"],
    "fs":   [],
    "exec": ["npx", "node"],
}

# ---------------------------------------------------------------------------
# fetch_versions — inherited
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("googleapis", "release-please")

# ---------------------------------------------------------------------------
# download_url — None (installed via npx)
#
# release-please is a Node.js package, not a direct binary download.
# It should be run via: npx release-please
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    """release-please is run via npx, not direct download."""
    return None

# ---------------------------------------------------------------------------
# deps — requires node
# ---------------------------------------------------------------------------

def deps(_ctx, version):
    """release-please requires Node.js for execution."""
    return [{"runtime": "node", "version": ">=18"}]

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(_ctx, _version):
    return []
