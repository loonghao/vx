# provider.star - Python provider
#
# Version source: GitHub releases of python-build-standalone
#   https://github.com/astral-sh/python-build-standalone/releases
#
# python-build-standalone provides pre-built, portable Python distributions
# that work without system dependencies. Used by uv, rye, etc.
#
# Bundled runtimes: pip (included in every Python release)
#
# Inheritance pattern: Level 2 (custom fetch_versions + download_url)

load("@vx//stdlib:http.star",     "fetch_json_versions")
load("@vx//stdlib:github.star",   "github_asset_url")
load("@vx//stdlib:env.star",      "env_set", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "python"
description = "Python - A programming language that lets you work quickly and integrate systems more effectively"
homepage    = "https://www.python.org"
repository  = "https://github.com/python/cpython"
license     = "PSF-2.0"
ecosystem   = "python"
aliases     = ["python3", "py"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "python",
        "executable":  "python",
        "description": "Python programming language runtime",
        "aliases":     ["python3", "py"],
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "Python \\d+\\.\\d+"},
            {"command": "{executable} -c \"import sys; print(sys.version)\"", "name": "eval_check"},
        ],
    },
    {
        "name":        "pip",
        "executable":  "pip",
        "description": "Python package installer (bundled with Python)",
        "aliases":     ["pip3"],
        "bundled_with": "python",
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "pip \\d+"},
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
# fetch_versions — python-build-standalone GitHub releases
#
# Uses the fetch_json_versions descriptor with the "python_build_standalone"
# transform strategy. The Rust runtime fetches the GitHub releases API and
# extracts Python versions from asset names:
#   cpython-3.12.1+20240107-x86_64-pc-windows-msvc-install_only_stripped.tar.gz
# The build tag (date) is stored in the version's date field.
# ---------------------------------------------------------------------------

def fetch_versions(_vx_ctx):
    """Fetch Python versions from python-build-standalone GitHub releases."""
    return fetch_json_versions(
        _vx_ctx,
        "https://api.github.com/repos/astral-sh/python-build-standalone/releases?per_page=50",
        "python_build_standalone",
    )

# ---------------------------------------------------------------------------
# download_url — python-build-standalone asset
# ---------------------------------------------------------------------------

def _pbs_triple(vx_ctx):
    """Map vx platform to python-build-standalone triple."""
    os   = vx_ctx.platform.os
    arch = vx_ctx.platform.arch

    triples = {
        "windows/x64":   "x86_64-pc-windows-msvc",
        "macos/x64":     "x86_64-apple-darwin",
        "macos/arm64":   "aarch64-apple-darwin",
        "linux/x64":     "x86_64-unknown-linux-gnu",
        "linux/arm64":   "aarch64-unknown-linux-gnu",
    }
    return triples.get("{}/{}".format(os, arch))

def download_url(vx_ctx, version):
    """Build the python-build-standalone download URL.

    Args:
        vx_ctx:  Provider context (vx_ctx.version_date contains the build tag)
        version: Python version string, e.g. "3.12.1"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    triple = _pbs_triple(vx_ctx)
    if not triple:
        return None

    # The build tag (date like "20240107") is stored in vx_ctx.version_date
    # by the Rust runtime when it resolves the python_build_standalone descriptor.
    build_tag = vx_ctx.version_date
    if not build_tag:
        return None

    asset = "cpython-{}+{}-{}-install_only_stripped.tar.gz".format(version, build_tag, triple)
    return github_asset_url("astral-sh", "python-build-standalone", build_tag, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(vx_ctx, _version):
    os = vx_ctx.platform.os

    if os == "windows":
        exe_paths = ["python/python.exe", "python.exe"]
    else:
        exe_paths = ["python/bin/python3", "python/bin/python", "bin/python3"]

    return {
        "type":             "archive",
        "strip_prefix":     "",   # python-build-standalone has variable prefix
        "executable_paths": exe_paths,
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(vx_ctx, _version):
    os = vx_ctx.platform.os
    if os == "windows":
        return [
            env_set("PYTHONHOME", vx_ctx.install_dir + "/python"),
            env_prepend("PATH", vx_ctx.install_dir + "/python"),
        ]
    else:
        return [
            env_set("PYTHONHOME", vx_ctx.install_dir + "/python"),
            env_prepend("PATH", vx_ctx.install_dir + "/python/bin"),
        ]

# ---------------------------------------------------------------------------
# Path queries (RFC-0037)
# ---------------------------------------------------------------------------

def store_root(vx_ctx):
    """Return the vx store root directory for python."""
    return vx_ctx.vx_home + "/store/python"

def get_execute_path(vx_ctx, version):
    """Return the executable path for the given version."""
    os = vx_ctx.platform.os
    if os == "windows":
        return vx_ctx.install_dir + "/python/python.exe"
    else:
        return vx_ctx.install_dir + "/python/bin/python3"

def post_install(_vx_ctx, _version):
    """No post-install steps needed for python."""
    return None

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(_vx_ctx, version):
    """Python recommends uv for package management."""
    return [
        {"runtime": "uv", "version": "*", "optional": True,
         "reason": "uv provides faster package management for Python"},
    ]
