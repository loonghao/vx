# provider.star - Spack provider
#
# Linux/macOS only. Source archive from GitHub releases.
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "dep_def")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_set", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "spack"
description = "Spack - Flexible package manager for supercomputers, Linux, and macOS"
homepage    = "https://spack.io"
repository  = "https://github.com/spack/spack"
license     = "Apache-2.0 OR MIT"
ecosystem   = "python"

platforms = {"os": ["linux", "macos"]}

# Supported package prefixes for ecosystem:package syntax (RFC 0027)
# Enables `vx spack:<package>` for HPC package installation
package_prefixes = ["spack"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("spack",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "^\\d+\\.\\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(extra_hosts = [])

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("spack", "spack")

# ---------------------------------------------------------------------------
# download_url — source tar.gz
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    if ctx.platform.os == "windows":
        return None
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
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/spack"

def get_execute_path(ctx, _version):
    return ctx.install_dir + "/bin/spack"

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [
        env_set("SPACK_ROOT", ctx.install_dir),
        env_prepend("PATH", ctx.install_dir + "/bin"),
    ]

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return [
        dep_def("python", version = ">=3.6",
                reason = "Spack requires Python 3.6+"),
        dep_def("git", optional = True,
                reason = "Git is required for fetching Spack packages"),
    ]
