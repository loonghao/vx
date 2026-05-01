# provider.star - Python provider
#
# Version source: python-build-standalone GitHub releases
#   https://github.com/astral-sh/python-build-standalone/releases
# Bundled runtimes: pip
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def", "github_permissions", "dep_def")
load("@vx//stdlib:http.star",   "fetch_json_versions")
load("@vx//stdlib:github.star", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

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

# Supported package prefixes for ecosystem:package syntax (RFC 0027)
# Enables `vx pip:<package>` for Python package installation via pip
package_prefixes = ["pip"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("python",
        aliases = ["python3", "py"],
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "Python \\d+\\.\\d+"},
            {"command": "{executable} -c \"import sys; print(sys.version)\"",
             "name": "eval_check"},
        ],
    ),
    bundled_runtime_def("pip", bundled_with = "python",
        aliases         = ["pip3"],
        version_pattern = "pip \\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions — python-build-standalone GitHub releases
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    return fetch_json_versions(
        ctx,
        "https://api.github.com/repos/astral-sh/python-build-standalone/releases?per_page=50",
        "python_build_standalone",
    )

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

_PBS_TRIPLES = {
    "windows/x64":  "x86_64-pc-windows-msvc",
    "macos/x64":    "x86_64-apple-darwin",
    "macos/arm64":  "aarch64-apple-darwin",
    "linux/x64":    "x86_64-unknown-linux-gnu",
    "linux/arm64":  "aarch64-unknown-linux-gnu",
}

def _pbs_triple(ctx):
    return _PBS_TRIPLES.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))

# ---------------------------------------------------------------------------
# download_url — python-build-standalone asset
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    triple = _pbs_triple(ctx)
    if not triple:
        return None
    build_tag = ctx.version_date
    if not build_tag:
        return None
    asset = "cpython-{}+{}-{}-install_only_stripped.tar.gz".format(version, build_tag, triple)
    return github_asset_url("astral-sh", "python-build-standalone", build_tag, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    # The python-build-standalone tarball has a top-level "python/" directory.
    # We explicitly strip it so the install dir contains bin/, lib/, etc. directly.
    # This avoids the auto-flatten heuristic which caused PYTHONHOME mismatches (#696).
    if ctx.platform.os == "windows":
        exe_paths = ["python.exe"]
    else:
        exe_paths = ["bin/python3", "bin/python"]
    return {
        "type":             "archive",
        "strip_prefix":     "python",
        "executable_paths": exe_paths,
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/python"

def get_execute_path(ctx, _version):
    if ctx.platform.os == "windows":
        return ctx.install_dir + "/python.exe"
    return ctx.install_dir + "/bin/python3"

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    # Do NOT set PYTHONHOME — python-build-standalone is self-contained and
    # auto-detects its prefix.  Setting PYTHONHOME incorrectly causes
    # "ModuleNotFoundError: No module named 'encodings'" (see #696).
    if ctx.platform.os == "windows":
        return [
            env_prepend("PATH", ctx.install_dir),
        ]
    return [
        env_prepend("PATH", ctx.install_dir + "/bin"),
    ]

# ---------------------------------------------------------------------------
# deps — uv recommended for package management
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return [
        dep_def("uv", optional = True,
                reason = "uv provides faster package management for Python"),
    ]

system_install = cross_platform_install(
    windows = "python",
    macos   = "python",
    linux   = "python",
)
