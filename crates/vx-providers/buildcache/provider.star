# provider.star - buildcache (MSVC-friendly Compiler Cache) provider
#
# buildcache is a compiler cache with excellent MSVC support,
# making it ideal for Windows development with Visual Studio.
#
# Supported compilers:
# - MSVC (cl.exe) - Best support
# - GCC
# - Clang
# - NVIDIA nvcc

load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "buildcache"
description = "buildcache - Compiler Cache with excellent MSVC support"
homepage    = "https://github.com/mbitsnbites/buildcache"
repository  = "https://github.com/mbitsnbites/buildcache"
license     = "Zlib"
ecosystem   = "devtools"
aliases     = ["msvc-cache", "visual-studio-cache"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("buildcache",
        aliases         = ["msvc-cache", "visual-studio-cache"],
        version_pattern = "buildcache",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("mbitsnbites", "buildcache")

# ---------------------------------------------------------------------------
# Platform helpers
#
# buildcache releases use:
#   - Windows: buildcache-win-x86_64.zip
#   - Linux: buildcache-linux-x86_64.tar.gz
#   - macOS: buildcache-darwin-x86_64.tar.gz
# ---------------------------------------------------------------------------

def _buildcache_platform(ctx):
    os   = ctx.platform.os
    arch = ctx.platform.arch

    platform_map = {
        ("windows", "x64"):   ("win-x86_64", "zip"),
        ("macos",   "x64"):   ("darwin-x86_64", "tar.gz"),
        ("macos",   "arm64"): ("darwin-arm64", "tar.gz"),
        ("linux",   "x64"):   ("linux-x86_64", "tar.gz"),
        ("linux",   "arm64"): ("linux-arm64", "tar.gz"),
    }

    return platform_map.get((os, arch))

# ---------------------------------------------------------------------------
# download_url
# Asset: buildcache-{platform}.{ext}
# Note: buildcache uses different naming convention
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _buildcache_platform(ctx)
    if not platform:
        return None
    platform_name, ext = platform[0], platform[1]
    asset = "buildcache-{}.{}".format(platform_name, ext)
    return github_asset_url("mbitsnbites", "buildcache", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "buildcache.exe" if ctx.platform.os == "windows" else "buildcache"

    return {
        "type":             "archive",
        "strip_prefix":     "",  # No top-level directory
        "executable_paths": [exe],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/buildcache"

def get_execute_path(ctx, _version):
    exe = "buildcache.exe" if ctx.platform.os == "windows" else "buildcache"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return """
buildcache installed successfully!

Quick Start:
  vx buildcache -s          # Show cache statistics
  vx buildcache -m 20000000000  # Set max cache size to 20GB (in bytes)
  vx buildcache -C          # Clear cache

MSVC / Visual Studio Usage:
  # Method 1: Use as compiler launcher in CMake
  cmake -DCMAKE_C_COMPILER_LAUNCHER=buildcache \\
        -DCMAKE_CXX_COMPILER_LAUNCHER=buildcache \\
        -B build

  # Method 2: Use as compiler wrapper
  set(CMAKE_C_COMPILER "buildcache cl")
  set(CMAKE_CXX_COMPILER "buildcache cl")

GCC/Clang Usage:
  export CC="buildcache gcc"
  export CXX="buildcache g++"

Environment Variables:
  BUILDCACHE_MAX_CACHE_SIZE - Max cache size in bytes
  BUILDCACHE_DIR - Cache directory
  BUILDCACHE_DEBUG - Enable debug logging
"""

def environment(ctx, _version):
    return [
        env_prepend("PATH", ctx.install_dir),
    ]

def deps(_ctx, _version):
    return []
