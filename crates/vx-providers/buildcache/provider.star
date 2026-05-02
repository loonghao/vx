load("@vx//stdlib:system_install.star", "cross_platform_install")
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

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "path_fns", "post_extract_flatten")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

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
        version_pattern = "BuildCache version",
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
# buildcache release assets are published per-OS:
# - buildcache-windows.zip
# - buildcache-macos.zip
# - buildcache-linux.tar.gz
# ---------------------------------------------------------------------------

_BUILDCACHE_ASSETS = {
    "windows": "buildcache-windows.zip",
    "macos":   "buildcache-macos.zip",
    "linux":   "buildcache-linux.tar.gz",
}

def _buildcache_platform(ctx):
    return _BUILDCACHE_ASSETS.get(ctx.platform.os)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    asset = _buildcache_platform(ctx)
    if not asset:
        return None
    return github_asset_url("mbitsnbites", "buildcache", "v" + version, asset)

# ---------------------------------------------------------------------------
# Layout + path functions
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "buildcache.exe" if ctx.platform.os == "windows" else "buildcache"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": ["bin/" + exe, exe, "buildcache"],
    }

paths      = path_fns("buildcache")
store_root = paths["store_root"]
post_extract = post_extract_flatten(pattern = "buildcache")

def get_execute_path(ctx, _version):
    exe = "buildcache.exe" if ctx.platform.os == "windows" else "buildcache"
    return ctx.install_dir + "/bin/" + exe

# ---------------------------------------------------------------------------
# post_install — usage instructions
# ---------------------------------------------------------------------------

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
        env_prepend("PATH", ctx.install_dir + "/bin"),
        env_prepend("PATH", ctx.install_dir),
    ]

system_install = cross_platform_install(
    windows = "buildcache",
    macos   = "buildcache",
    linux   = "buildcache",
)
