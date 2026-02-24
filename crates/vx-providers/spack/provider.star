# provider.star - Spack provider
#
# Version source: https://github.com/spack/spack/releases
#
# Spack is a flexible package manager for supercomputers, Linux, and macOS.
# It is Python-based and typically installed via git clone.
# Only supports Linux and macOS (no Windows support).
#
# Inheritance pattern: Level 2 (custom fetch_versions + download_url)

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star", "env_set", "env_prepend")
load("@vx//stdlib:env.star", "env_set", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "spack"
description = "Spack - Flexible package manager for supercomputers, Linux, and macOS"
homepage    = "https://spack.io"
repository  = "https://github.com/spack/spack"
license     = "Apache-2.0 OR MIT"
ecosystem   = "python"

# ---------------------------------------------------------------------------
# Platform constraint — Linux and macOS only
# ---------------------------------------------------------------------------

platforms = {
    "os": ["linux", "macos"],
}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "spack",
        "executable":  "spack",
        "description": "Spack package manager",
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "^\\d+\\.\\d+"},
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": ["python", "git"],
}

# ---------------------------------------------------------------------------
# fetch_versions — spack/spack GitHub releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("spack", "spack")

# ---------------------------------------------------------------------------
# download_url — tar.gz source archive
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """Build Spack download URL from GitHub releases.

    Spack releases are source archives (tar.gz).
    """
    os = ctx.platform.os
    if os == "windows":
        return None

    # e.g. https://github.com/spack/spack/releases/download/v0.22.0/spack-0.22.0.tar.gz
    asset = "spack-{}.tar.gz".format(version)
    return github_asset_url("spack", "spack", version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(_ctx, version):
    return {
        "type":             "archive",
        "strip_prefix":     "spack-{}".format(version),
        "executable_paths": ["bin/spack"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, _version):
    return [
        env_set("SPACK_ROOT", ctx.install_dir),
        env_prepend("PATH", ctx.install_dir + "/bin"),
    ]

# ---------------------------------------------------------------------------
# Path queries (RFC-0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for spack."""
    return ctx.vx_home + "/store/spack"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    return ctx.install_dir + "/bin/spack"

def post_install(_ctx, _version):
    """No post-install steps needed for spack."""
    return None

# ---------------------------------------------------------------------------
# deps — requires python 3.6+ and git
# ---------------------------------------------------------------------------

def deps(_ctx, version):
    return [
        {"runtime": "python", "version": ">=3.6",
         "reason": "Spack requires Python 3.6+"},
        {"runtime": "git", "version": "*", "optional": True,
         "reason": "Git is required for fetching Spack packages"},
    ]
