# provider.star - sccache (Shared Compilation Cache) provider
#
# sccache is a ccache-like tool that caches compilation results
# to speed up future builds. It supports multiple compilers:
# - Rust (rustc)
# - C/C++ (gcc, clang, MSVC/cl)
# - NVIDIA CUDA (nvcc)
# - And more
#
# Benefits:
# - 20-50% faster rebuilds across projects
# - Cross-language support (Rust, C++, etc.)
# - Remote cache backends (S3, GCS, Redis, Azure Blob)
# - Works with CI/CD pipelines
#
# Usage:
#   vx sccache --start-server     # Start cache server
#   vx sccache --stop-server      # Stop cache server
#   vx sccache --show-stats       # Show cache statistics
#   vx sccache --zero-stats       # Reset statistics

load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "sccache"
description = "sccache - Shared Compilation Cache for Rust, C/C++ and more"
homepage    = "https://github.com/mozilla/sccache"
repository  = "https://github.com/mozilla/sccache"
license     = "Apache-2.0"
ecosystem   = "devtools"
aliases     = ["shared-cache", "compiler-cache"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("sccache",
        aliases         = ["shared-cache", "compiler-cache"],
        version_pattern = "sccache",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("mozilla", "sccache")

# ---------------------------------------------------------------------------
# Platform helpers
#
# sccache releases use:
#   - "x86_64-pc-windows-msvc" for Windows
#   - "x86_64-apple-darwin" / "aarch64-apple-darwin" for macOS
#   - "x86_64-unknown-linux-musl" for Linux (static linked)
#   - tar.gz for all platforms
# ---------------------------------------------------------------------------

def _sccache_platform(ctx):
    os   = ctx.platform.os
    arch = ctx.platform.arch

    # sccache uses Rust target triples
    target_triples = {
        ("windows", "x64"):   ("x86_64-pc-windows-msvc", "tar.gz"),
        ("macos",   "x64"):   ("x86_64-apple-darwin", "tar.gz"),
        ("macos",   "arm64"): ("aarch64-apple-darwin", "tar.gz"),
        ("linux",   "x64"):   ("x86_64-unknown-linux-musl", "tar.gz"),
        ("linux",   "arm64"): ("aarch64-unknown-linux-musl", "tar.gz"),
    }

    return target_triples.get((os, arch))

# ---------------------------------------------------------------------------
# download_url
# Asset: sccache-{version}-{target}.{ext}
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _sccache_platform(ctx)
    if not platform:
        return None
    target, ext = platform[0], platform[1]
    asset = "sccache-{}-{}.{}".format(version, target, ext)
    return github_asset_url("mozilla", "sccache", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# Archive structure: sccache-{version}-{target}/sccache[.exe]
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    platform = _sccache_platform(ctx)
    if not platform:
        return {"type": "archive", "strip_prefix": "", "executable_paths": ["sccache"]}

    target, _ext = platform[0], platform[1]
    exe = "sccache.exe" if ctx.platform.os == "windows" else "sccache"

    return {
        "type":             "archive",
        "strip_prefix":     "sccache-{}-{}/".format(version, target),
        "executable_paths": [exe],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/sccache"

def get_execute_path(ctx, _version):
    exe = "sccache.exe" if ctx.platform.os == "windows" else "sccache"
    return ctx.install_dir + "/" + exe

def post_install(ctx, _version):
    """Print usage hints after installation."""
    return """
sccache installed successfully!

Quick Start:
  vx sccache --start-server    # Start the cache server
  vx sccache --show-stats      # View cache statistics

Configuration (optional):
  SCCACHE_CACHE_SIZE=20G       # Set cache size (default: 10G)
  SCCACHE_DIR=~/.cache/sccache # Set cache directory

The Rust compiler will automatically use sccache when it's in PATH.
See .cargo/config.toml for the rustc-wrapper setting.
"""

def environment(ctx, _version):
    return [
        env_prepend("PATH", ctx.install_dir),
        # Set default cache size if not already set
        # {"type": "env_set", "name": "SCCACHE_CACHE_SIZE", "value": "20G"},
    ]

def deps(_ctx, _version):
    # sccache has no runtime dependencies
    return []
