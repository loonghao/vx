load("@vx//stdlib:system_install.star", "cross_platform_install")
# provider.star - Python provider
#
# Version source: python-build-standalone GitHub releases
#   https://github.com/astral-sh/python-build-standalone/releases
# Python 2.7 compatibility source: PyPy standalone downloads
#   https://downloads.python.org/pypy/
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

permissions = github_permissions(extra_hosts = ["downloads.python.org"])

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

_LEGACY_37_ASSETS = {
    # Fixed lifecycle catalog: release metadata must not select a different
    # artifact after upstream stops publishing this Python line.
    # Digests are VX intake pins of official HTTPS release assets; these old
    # releases publish no checksums or signatures. They are not publisher hashes.
    "windows/x64": {
        "release": "20200822",
        "asset": "cpython-3.7.9-x86_64-pc-windows-msvc-shared-pgo-20200823T0118.tar.zst",
        "sha256": "8769a244cfe54c32c5253b78897a0fe82fe419dfde653b1afc3f5f20594cca89",
        "asset_id": 24195911,
        "size": 31429615,
        "required_paths": ["Lib/site.py", "Lib/venv/__init__.py", "include/Python.h", "libs/python37.lib"],
    },
    "macos/x64": {
        # The 20200822 build has an unwanted libintl.dylib dependency.
        "release": "20200823",
        "asset": "cpython-3.7.9-x86_64-apple-darwin-pgo-20200823T2228.tar.zst",
        "sha256": "53657e7712cc7b24491fb1fc66dcc8f47a577fc77df137178746987ba4c5afb8",
        "asset_id": 24211674,
        "size": 26136896,
        "required_paths": ["lib/python3.7/site.py", "lib/python3.7/venv/__init__.py", "include/python3.7m/Python.h", "lib/libpython3.7m.a", "lib/libpython3.7m.dylib"],
    },
    "linux/x64": {
        "release": "20200822",
        "asset": "cpython-3.7.9-x86_64-unknown-linux-gnu-pgo-20200823T0036.tar.zst",
        "sha256": "c6d6256d13e929e77e7ee6e53470fe63ad19d173fee6d56bb1b2dbda67081543",
        "asset_id": 24195897,
        "size": 28357634,
        "required_paths": ["lib/python3.7/site.py", "lib/python3.7/venv/__init__.py", "include/python3.7m/Python.h", "lib/libpython3.7m.a", "lib/libpython3.7m.so.1.0"],
    },
}

_PYPY27_VERSION = "2.7.18"
_PYPY27_ASSETS = {
    "windows/x64": "pypy2.7-v7.3.20-win64.zip",
    "linux/x64":   "pypy2.7-v7.3.20-linux64.tar.bz2",
    "linux/arm64": "pypy2.7-v7.3.20-aarch64.tar.bz2",
    "macos/x64":   "pypy2.7-v7.3.20-macos_x86_64.tar.bz2",
    "macos/arm64": "pypy2.7-v7.3.20-macos_arm64.tar.bz2",
}

def _pbs_triple(ctx):
    return _PBS_TRIPLES.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))

def _platform_key(ctx):
    return "{}/{}".format(ctx.platform.os, ctx.platform.arch)

def _is_pypy27(version):
    return version == _PYPY27_VERSION

def _legacy37_artifact(ctx, version):
    if version != "3.7.9":
        fail("Python {} has no pinned portable build; request Python 3.7.9 explicitly".format(version))
    platform = _platform_key(ctx)
    artifact = _LEGACY_37_ASSETS.get(platform)
    if artifact == None:
        fail("Python 3.7.9 portable build unavailable for {}; supported platforms: Windows x64, Linux x64, macOS x64".format(platform))
    return artifact

# ---------------------------------------------------------------------------
# download_url — python-build-standalone asset
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    if _is_pypy27(version):
        asset = _PYPY27_ASSETS.get(_platform_key(ctx))
        if not asset:
            return None
        return "https://downloads.python.org/pypy/{}".format(asset)

    if version.startswith("3.7."):
        artifact = _legacy37_artifact(ctx, version)
        return github_asset_url("astral-sh", "python-build-standalone", artifact["release"], artifact["asset"])

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

def install_layout(ctx, version):
    if _is_pypy27(version):
        asset = _PYPY27_ASSETS.get(_platform_key(ctx))
        if not asset:
            return None
        strip = asset
        if strip.endswith(".zip"):
            strip = strip[:-4]
        elif strip.endswith(".tar.bz2"):
            strip = strip[:-8]
        if ctx.platform.os == "windows":
            exe_paths = ["pypy.exe", "python.exe"]
        else:
            exe_paths = ["bin/pypy", "bin/python"]
        return {
            "type":             "archive",
            "strip_prefix":     strip,
            "executable_paths": exe_paths,
        }

    if version.startswith("3.7."):
        artifact = _legacy37_artifact(ctx, version)
        if ctx.platform.os == "windows":
            exe_paths = ["python.exe"]
        else:
            exe_paths = ["bin/python3", "bin/python3.7", "bin/python"]
        return {
            "type":             "archive",
            "strip_prefix":     "python/install",
            "executable_paths": exe_paths,
            "required_paths":   artifact["required_paths"],
            "sha256":           artifact["sha256"],
            "mirror_urls":      [],
        }

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

def get_execute_path(ctx, version):
    if _is_pypy27(version):
        if ctx.platform.os == "windows":
            return ctx.install_dir + "/pypy.exe"
        return ctx.install_dir + "/bin/pypy"

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
