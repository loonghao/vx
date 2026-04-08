# provider.star - xmake build system provider
#
# xmake is a lightweight, cross-platform build utility based on Lua.
# It supports C/C++, Rust, Go, Swift, and many other languages.
#
# GitHub releases provide self-contained bundle binaries for all platforms.
# The bundle binary IS the xmake executable (single-file distribution).
# License: Apache-2.0
# Homepage: https://xmake.io

load("@vx//stdlib:provider.star",
     "runtime_def", "dep_def",
     "github_permissions",
     "system_install_strategies",
     "winget_install", "choco_install", "scoop_install",
     "brew_install", "apt_install")
load("@vx//stdlib:github.star", "make_fetch_versions")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "xmake"
description = "xmake - A cross-platform build utility based on Lua"
homepage    = "https://xmake.io"
repository  = "https://github.com/xmake-io/xmake"
license     = "Apache-2.0"
ecosystem   = "cpp"
aliases     = ["xmake-build"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("xmake",
        aliases         = ["xmake-build"],
        description     = "Cross-platform build system",
        version_pattern = "\\d+\\.\\d+\\.\\d+",
        test_commands   = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "xmake v\\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions — GitHub releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("xmake-io", "xmake")

# ---------------------------------------------------------------------------
# Platform helpers
# xmake asset naming: xmake-bundle-v{version}.{os}.{arch} (Linux/macOS)
# Windows: xmake-bundle-v{version}.win64.exe
# macOS:   xmake-bundle-v{version}.macos.arm64 or .x86_64
# Linux:   xmake-bundle-v{version}.linux.x86_64
# ---------------------------------------------------------------------------

_XMAKE_PLATFORMS = {
    "windows/x64":   ("win64",  ".exe",     ""),
    "windows/arm64": ("arm64",  ".exe",     ""),  # asset: xmake-bundle-v{ver}.arm64.exe
    "windows/x86":   ("win32",  ".exe",     ""),
    "macos/x64":     ("macos",  "",         ".x86_64"),
    "macos/arm64":   ("macos",  "",         ".arm64"),
    "linux/x64":     ("linux",  "",         ".x86_64"),
    "linux/arm64":   None,  # No arm64 linux binary available
}

def _xmake_asset_name(ctx, version):
    """Build the xmake bundle asset filename for the current platform."""
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    platform = _XMAKE_PLATFORMS.get(key)
    if not platform:
        return None
    os_str, ext, arch_suffix = platform
    return "xmake-bundle-v{}.{}{}{}".format(version, os_str, arch_suffix, ext)

def download_url(ctx, version):
    asset = _xmake_asset_name(ctx, version)
    if not asset:
        return None
    return "https://github.com/xmake-io/xmake/releases/download/v{}/{}".format(version, asset)

# ---------------------------------------------------------------------------
# install_layout — binary_install: rename bundle to xmake[.exe]
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    source = _xmake_asset_name(ctx, version)
    if not source:
        return None

    if ctx.platform.os == "windows":
        target = "xmake.exe"
    else:
        target = "xmake"

    return {
        "__type":           "binary_install",
        "source_name":      source,
        "target_name":      target,
        "target_dir":       "bin",
        "executable_paths": ["bin/" + target],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/xmake"

def get_execute_path(ctx, _version):
    exe = "xmake.exe" if ctx.platform.os == "windows" else "xmake"
    return ctx.install_dir + "/bin/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

# ---------------------------------------------------------------------------
# system_install — static dict with all platforms' strategies
# ---------------------------------------------------------------------------
# NOTE: Use static dict (not function) so parse_system_install_strategies
# can read it directly without calling. Platform filtering is handled
# automatically by the per-manager helpers which set the "platforms" field.

system_install = system_install_strategies([
    winget_install("tboox.xmake", priority = 95),
    choco_install("xmake",        priority = 80),
    scoop_install("xmake",        priority = 60),
    brew_install("xmake"),
    apt_install("xmake"),
])

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return [
        dep_def("cmake", optional = True,
                reason = "CMake generator is supported by xmake"),
        dep_def("ninja", optional = True,
                reason = "Ninja backend is supported by xmake"),
    ]
