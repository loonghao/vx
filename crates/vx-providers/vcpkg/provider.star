# provider.star - vcpkg provider
#
# C++ library manager for Windows, Linux, and macOS
# Version source: https://github.com/microsoft/vcpkg-tool/releases
#
# vcpkg-tool provides standalone binary downloads per platform.
# Tags are date-based: "2025-12-16" (no "v" prefix).
#
# Inheritance pattern: Level 2 (custom download_url for platform-specific binary)

load("@vx//stdlib:github.star", "make_fetch_versions")
load("@vx//stdlib:env.star", "env_set", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "vcpkg"
description = "C++ library manager for Windows, Linux, and macOS"
homepage    = "https://vcpkg.io/"
repository  = "https://github.com/microsoft/vcpkg-tool"
license     = "MIT"
ecosystem   = "cpp"
aliases     = ["vcpkg-cli"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "vcpkg",
        "executable":  "vcpkg",
        "description": "vcpkg - C++ library manager",
        "aliases":     ["vcpkg-cli"],
        "priority":    100,
        "system_paths": [
            "C:/vcpkg/vcpkg.exe",
            "/usr/local/bin/vcpkg",
            "/usr/bin/vcpkg",
        ],
        "env_hints": ["VCPKG_ROOT"],
        "test_commands": [
            {"command": "{executable} version", "name": "version_check", "expected_output": "vcpkg"},
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": ["git", "cmake", "ninja"],
}

# ---------------------------------------------------------------------------
# fetch_versions — vcpkg-tool GitHub releases (date-based tags)
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("microsoft", "vcpkg-tool")

# ---------------------------------------------------------------------------
# download_url — platform-specific standalone binary
# ---------------------------------------------------------------------------

def _vcpkg_asset(ctx):
    """Map vx platform to vcpkg-tool binary asset name.

    vcpkg-tool release assets:
      vcpkg-arm64-osx
      vcpkg-arm64-osx.sha512
      vcpkg-arm64-windows.exe
      vcpkg-arm64-windows.exe.sha512
      vcpkg-x64-linux
      vcpkg-x64-linux.sha512
      vcpkg-x64-osx
      vcpkg-x64-osx.sha512
      vcpkg-x64-windows.exe
      vcpkg-x64-windows.exe.sha512
      vcpkg-x64-windows-static.exe
      vcpkg-x64-windows-static.exe.sha512
    """
    os   = ctx.platform.os
    arch = ctx.platform.arch

    assets = {
        "windows/x64":   "vcpkg-x64-windows.exe",
        "windows/arm64": "vcpkg-arm64-windows.exe",
        "macos/x64":     "vcpkg-x64-osx",
        "macos/arm64":   "vcpkg-arm64-osx",
        "linux/x64":     "vcpkg-x64-linux",
        "linux/arm64":   "vcpkg-arm64-linux",
    }
    return assets.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    """Build the vcpkg-tool download URL from GitHub releases.

    Args:
        ctx:     Provider context
        version: Version string, e.g. "2025-12-16" (date-based tag)

    Returns:
        Download URL string, or None if platform is unsupported
    """
    asset = _vcpkg_asset(ctx)
    if not asset:
        return None

    # vcpkg-tool tags are date-based without "v" prefix: "2025-12-16"
    return "https://github.com/microsoft/vcpkg-tool/releases/download/{}/{}".format(
        version, asset
    )

# ---------------------------------------------------------------------------
# install_layout — single binary
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    os    = ctx.platform.os
    asset = _vcpkg_asset(ctx)
    if not asset:
        return {"type": "binary", "executable_paths": ["vcpkg"]}

    return {
        "type":             "binary",
        "source_name":      asset,
        "target_name":      "vcpkg.exe" if os == "windows" else "vcpkg",
        "executable_paths": ["vcpkg.exe" if os == "windows" else "vcpkg"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, _version):
    return [
        env_set("VCPKG_ROOT",                 ctx.install_dir),
        env_set("VCPKG_DOWNLOADS",            ctx.install_dir + "/.cache/downloads"),
        env_set("VCPKG_DEFAULT_BINARY_CACHE", ctx.install_dir + "/.cache/archives"),
        env_prepend("PATH",                   ctx.install_dir),
    ]

# ---------------------------------------------------------------------------
# Path queries (RFC-0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for vcpkg."""
    return ctx.vx_home + "/store/vcpkg"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx.platform.os
    exe = "vcpkg.exe" if os == "windows" else "vcpkg"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    """No post-install actions needed for vcpkg."""
    return None

# ---------------------------------------------------------------------------
# deps — explicit dependency declarations
# ---------------------------------------------------------------------------

def deps(_ctx, version):
    """vcpkg requires git; recommends cmake and ninja."""
    return [
        {"runtime": "git",   "version": "*", "optional": False,
         "reason": "Git is required to clone the vcpkg package registry"},
        {"runtime": "cmake", "version": "*", "optional": True,
         "reason": "CMake is commonly used with vcpkg for C++ projects"},
        {"runtime": "ninja", "version": "*", "optional": True,
         "reason": "Ninja build system provides faster builds"},
    ]
