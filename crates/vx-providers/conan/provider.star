# provider.star - Conan C/C++ Package Manager provider
#
# Conan is a decentralized, open-source C/C++ package manager.
# It is distributed as a Python package on PyPI, so we route
# `vx conan` → `vx uvx:conan` for isolated per-version environments.
#
# License: MIT
# Homepage: https://conan.io

load("@vx//stdlib:provider.star",
     "runtime_def", "dep_def",
     "github_permissions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "conan"
description = "Conan - The C/C++ Package Manager"
homepage    = "https://conan.io"
repository  = "https://github.com/conan-io/conan"
license     = "MIT"
ecosystem   = "cpp"
aliases     = ["conan2"]

# ---------------------------------------------------------------------------
# RFC 0033: route `vx conan` → `vx uvx:conan`
# Each version runs in its own isolated uv-managed Python environment.
# ---------------------------------------------------------------------------
package_alias = {"ecosystem": "uvx", "package": "conan"}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("conan",
        aliases         = ["conan2"],
        description     = "C/C++ package manager",
        version_pattern = "\\d+\\.\\d+\\.\\d+",
        test_commands   = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "Conan version \\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(extra_hosts = ["pypi.org", "conan.io"])

# ---------------------------------------------------------------------------
# fetch_versions — not applicable (runs via uvx)
# ---------------------------------------------------------------------------

def fetch_versions(_ctx):
    return []

# ---------------------------------------------------------------------------
# download_url — not applicable; runs via uvx
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path queries
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/conan"

def get_execute_path(_ctx, _version):
    return None

def post_install(_ctx, _version):
    return None

def environment(_ctx, _version):
    return []

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return [
        dep_def("uv", reason = "Conan is installed and run via uv/uvx"),
        dep_def("cmake", optional = True,
                reason = "CMake is commonly used with Conan for building packages"),
        dep_def("ninja", optional = True,
                reason = "Ninja provides faster builds with Conan"),
    ]
