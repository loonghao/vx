# provider.star - xmake build system provider
#
# xmake is a lightweight, cross-platform build utility based on Lua.
# It supports C/C++, Rust, Go, Swift, and many other languages.
#
# GitHub releases provide pre-built binaries for all platforms.
# License: Apache-2.0
# Homepage: https://xmake.io

load("@vx//stdlib:provider.star",
     "runtime_def", "dep_def",
     "github_permissions", "platform_map")
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
# xmake asset naming: xmake-v{version}.{os}.{arch}.{ext}
# Windows: xmake-v2.9.7.win64.exe  (installer, but also zip available)
# macOS:   xmake-v2.9.7.macosx.arm64.pkg  (but we use the archive)
# Linux:   xmake-v2.9.7.linux.x86_64.tar.gz
# ---------------------------------------------------------------------------

_XMAKE_PLATFORMS = {
    "windows/x64":   ("win64",   "exe"),
    "windows/x86":   ("win32",   "exe"),
    "macos/x64":     ("macosx",  "tar.gz"),
    "macos/arm64":   ("macosx",  "tar.gz"),
    "linux/x64":     ("linux",   "tar.gz"),
    "linux/arm64":   ("linux",   "tar.gz"),
}

_XMAKE_ARCH = {
    "windows/x64":   "",
    "windows/x86":   "",
    "macos/x64":     ".x86_64",
    "macos/arm64":   ".arm64",
    "linux/x64":     ".x86_64",
    "linux/arm64":   ".aarch64",
}

def _xmake_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _XMAKE_PLATFORMS.get(key)

def _xmake_arch_suffix(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _XMAKE_ARCH.get(key, "")

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _xmake_platform(ctx)
    if not platform:
        return None
    os_str, ext = platform
    arch_suffix = _xmake_arch_suffix(ctx)
    # e.g. xmake-v2.9.7.win64.exe  or  xmake-v2.9.7.linux.x86_64.tar.gz
    asset = "xmake-v{}.{}{}.{}".format(version, os_str, arch_suffix, ext)
    return "https://github.com/xmake-io/xmake/releases/download/v{}/{}".format(
        version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    os = ctx.platform.os
    if os == "windows":
        # Windows: self-extracting exe — treat as binary
        return {
            "type":             "binary",
            "executable_paths": ["xmake.exe", "xmake"],
        }
    else:
        return {
            "type":             "archive",
            "strip_prefix":     "",
            "executable_paths": ["xmake"],
        }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/xmake"

def get_execute_path(ctx, _version):
    exe = "xmake.exe" if ctx.platform.os == "windows" else "xmake"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

# ---------------------------------------------------------------------------
# system_install — fallback via package managers
# ---------------------------------------------------------------------------

def system_install(ctx):
    os = ctx.platform.os
    if os == "windows":
        return {
            "strategies": [
                {"manager": "winget", "package": "tboox.xmake",  "priority": 95},
                {"manager": "scoop",  "package": "xmake",        "priority": 60},
                {"manager": "choco",  "package": "xmake",        "priority": 80},
            ],
        }
    elif os == "macos":
        return {
            "strategies": [
                {"manager": "brew", "package": "xmake", "priority": 90},
            ],
        }
    elif os == "linux":
        return {
            "strategies": [
                {"manager": "apt", "package": "xmake", "priority": 80},
            ],
        }
    return {}

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
