# provider.star - rez provider
#
# Rez: Cross-platform package manager for deterministic environments
# Installed via uvx/pip — no direct binary download.
# Bundled runtimes: rez-env, rez-build
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def",
     "github_permissions", "dep_def")
load("@vx//stdlib:github.star", "make_fetch_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "rez"
description = "Cross-platform package manager for deterministic environments"
homepage    = "https://rez.readthedocs.io"
repository  = "https://github.com/AcademySoftwareFoundation/rez"
license     = "Apache-2.0"
ecosystem   = "python"

# Supported package prefixes for ecosystem:package syntax (RFC 0027)
# Enables `vx rez:<package>` for VFX package installation
package_prefixes = ["rez"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("rez",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "\\d+\\.\\d+"},
        ],
    ),
    bundled_runtime_def("rez-env",   bundled_with = "rez"),
    bundled_runtime_def("rez-build", bundled_with = "rez"),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("AcademySoftwareFoundation", "rez")

# ---------------------------------------------------------------------------
# download_url — installed via uvx/pip, not direct download
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# system_install — use pip/uvx as fallback
# ---------------------------------------------------------------------------

def system_install(_ctx, _version):
    return {
        "strategies": [
            {"manager": "uvx", "package": "rez", "priority": 95},
            {"manager": "pip", "package": "rez", "priority": 80},
        ],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/rez"

def get_execute_path(ctx, _version):
    exe = "rez.exe" if ctx.platform.os == "windows" else "rez"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(_ctx, _version):
    return []

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return [
        dep_def("python", version = ">=3.8",
                reason = "Rez requires Python 3.8+"),
    ]
