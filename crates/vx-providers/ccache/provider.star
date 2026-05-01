# provider.star - ccache (C/C++ Compiler Cache) provider
#
# ccache is a compiler cache that speeds up recompilation by caching
# previous compilations and detecting when the same compilation is
# being done again.
#
# Supported compilers:
# - GCC
# - Clang
# - NVIDIA nvcc
#
# Note: ccache does NOT support MSVC (use buildcache or sccache instead)

load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "ccache"
description = "ccache - C/C++ Compiler Cache for faster recompilation"
homepage    = "https://ccache.dev/"
repository  = "https://github.com/ccache/ccache"
license     = "GPL-3.0"
ecosystem   = "devtools"
aliases     = ["compiler-cache", "cpp-cache"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("ccache",
        aliases         = ["compiler-cache", "cpp-cache"],
        version_pattern = "ccache version",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("ccache", "ccache")

# ---------------------------------------------------------------------------
# Platform helpers
#
# ccache releases use:
#   - Windows: ccache-{version}-windows-x86_64.zip
#   - macOS: ccache-{version}-darwin.tar.gz (universal binary)
#   - Linux: ccache-{version}-linux-x86_64-glibc.tar.xz
# ---------------------------------------------------------------------------

def _ccache_platform(ctx):
    os   = ctx.platform.os
    arch = ctx.platform.arch

    platform_map = {
        ("windows", "x64"):   ("windows-x86_64", "zip"),
        ("macos",   "x64"):   ("darwin", "tar.gz"),  # Universal binary
        ("macos",   "arm64"): ("darwin", "tar.gz"),  # Universal binary
        ("linux",   "x64"):   ("linux-x86_64-glibc", "tar.xz"),
        ("linux",   "arm64"): ("linux-aarch64", "tar.gz"),
    }

    return platform_map.get((os, arch))

# ---------------------------------------------------------------------------
# download_url
# Asset: ccache-{version}-{platform}.{ext}
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _ccache_platform(ctx)
    if not platform:
        return None
    platform_name, ext = platform[0], platform[1]
    asset = "ccache-{}-{}.{}".format(version, platform_name, ext)
    return github_asset_url("ccache", "ccache", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    platform = _ccache_platform(ctx)
    if not platform:
        return {"type": "archive", "strip_prefix": "", "executable_paths": ["ccache"]}

    platform_name, _ext = platform[0], platform[1]
    exe = "ccache.exe" if ctx.platform.os == "windows" else "ccache"

    return {
        "type":             "archive",
        "strip_prefix":     "ccache-{}-{}/".format(version, platform_name),
        "executable_paths": [exe],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/ccache"

def get_execute_path(ctx, _version):
    exe = "ccache.exe" if ctx.platform.os == "windows" else "ccache"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return """
ccache installed successfully!

Quick Start:
  vx ccache -s           # Show cache statistics
  vx ccache -M 20G       # Set max cache size to 20GB
  vx ccache -C           # Clear cache

Usage with GCC/Clang:
  export CC="ccache gcc"
  export CXX="ccache g++"

CMake Integration:
  cmake -DCMAKE_C_COMPILER_LAUNCHER=ccache \\
        -DCMAKE_CXX_COMPILER_LAUNCHER=ccache \\
        -B build

Note: ccache does NOT support MSVC.
      For MSVC, use 'vx install buildcache' or 'vx install sccache'.
"""

def environment(ctx, _version):
    return [
        env_prepend("PATH", ctx.install_dir),
    ]

def deps(_ctx, _version):
    return []

system_install = cross_platform_install(
    windows = "ccache",
    macos   = "ccache",
    linux   = "ccache",
)
