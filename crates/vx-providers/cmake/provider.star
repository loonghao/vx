# provider.star - cmake provider
#
# CMake: Cross-platform build system generator
# Inheritance pattern: Level 2 (custom download_url for cmake's naming)
#   - fetch_versions: inherited from github.star
#   - download_url:   custom (cmake-{version}-{os}-{arch}.{ext})
#
# cmake releases: https://github.com/Kitware/CMake/releases
# Asset format: cmake-{version}-{os}-{arch}.{ext}

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "cmake"
description = "Cross-platform build system generator"
homepage    = "https://cmake.org"
repository  = "https://github.com/Kitware/CMake"
license     = "BSD-3-Clause"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "cmake",
        "executable":  "cmake",
        "description": "CMake build system",
        "aliases":     [],
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "cmake version"},
        ],
    },
    {
        "name":         "ctest",
        "executable":   "ctest",
        "description":  "CMake test driver",
        "bundled_with": "cmake",
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
    {
        "name":         "cpack",
        "executable":   "cpack",
        "description":  "CMake packaging tool",
        "bundled_with": "cmake",
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
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
# fetch_versions — inherited
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("Kitware", "CMake")

# ---------------------------------------------------------------------------
# download_url — custom
#
# cmake asset naming: cmake-{version}-{os}-{arch}.{ext}
#   cmake-3.31.5-windows-x86_64.zip
#   cmake-3.31.5-macos-universal.tar.gz
#   cmake-3.31.5-linux-x86_64.tar.gz / cmake-3.31.5-linux-aarch64.tar.gz
# ---------------------------------------------------------------------------

def _cmake_platform(ctx):
    """Map platform to cmake's naming convention."""
    os   = ctx.platform.os
    arch = ctx.platform.arch

    platform_map = {
        "windows/x64":   ("windows", "x86_64",  "zip"),
        "windows/x86":   ("windows", "i386",     "zip"),
        "macos/x64":     ("macos",   "universal", "tar.gz"),
        "macos/arm64":   ("macos",   "universal", "tar.gz"),
        "linux/x64":     ("linux",   "x86_64",   "tar.gz"),
        "linux/arm64":   ("linux",   "aarch64",  "tar.gz"),
    }
    return platform_map.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    """Build the cmake download URL.

    Args:
        ctx:     Provider context
        version: Version string, e.g. "3.31.5"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    platform = _cmake_platform(ctx)
    if not platform:
        return None

    cmake_os, cmake_arch, ext = platform
    asset = "cmake-{}-{}-{}.{}".format(version, cmake_os, cmake_arch, ext)
    tag = "v{}".format(version)
    return github_asset_url("Kitware", "CMake", tag, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    platform = _cmake_platform(ctx)
    os = ctx.platform.os
    exe = "cmake.exe" if os == "windows" else "cmake"

    if platform:
        cmake_os, cmake_arch, _ = platform
        strip_prefix = "cmake-{}-{}-{}".format(version, cmake_os, cmake_arch)
    else:
        strip_prefix = ""

    return {
        "type":             "archive",
        "strip_prefix":     strip_prefix,
        "executable_paths": ["bin/" + exe, "bin/cmake"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]


# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for cmake."""
    return ctx.vx_home + "/store/cmake"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx.platform.os
    exe = "cmake.exe" if os == "windows" else "cmake"
    return ctx.install_dir + "/bin/" + exe

def post_install(_ctx, _version):
    """Post-install hook (no-op for cmake)."""
    return None
