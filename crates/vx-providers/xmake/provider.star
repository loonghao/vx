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
     "github_permissions")
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
# macOS:   xmake-bundle-v{version}.macos.arm64 or xmake-bundle-v{version}.macos.x86_64
# Linux:   xmake-bundle-v{version}.linux.x86_64
# ---------------------------------------------------------------------------

_XMAKE_PLATFORMS = {
    "windows/x64":   ("win64",   ".exe",     ""),
    "windows/x86":   ("win32",   ".exe",     ""),
    "macos/x64":     ("macos",   "",         ".x86_64"),
    "macos/arm64":   ("macos",   "",         ".arm64"),
    "linux/x64":     ("linux",   "",         ".x86_64"),
    "linux/arm64":   None,  # No arm64 linux binary available
}

def download_url(ctx, version):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    platform = _XMAKE_PLATFORMS.get(key)
    if not platform:
        return None
    os_str, ext, arch_suffix = platform
    # xmake-bundle-v3.0.7.win64.exe or xmake-bundle-v3.0.7.linux.x86_64
    asset = "xmake-bundle-v{}.{}{}{}".format(version, os_str, arch_suffix, ext)
    return "https://github.com/xmake-io/xmake/releases/download/v{}/{}".format(version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    os_name = ctx.platform.os
    if os_name == "windows":
        # Windows: self-extracting exe — treat as binary
        return {
            "__type":           "binary",
            "target_name":      "xmake.exe",
            "target_dir":       "",
            "executable_paths": ["xmake.exe", "xmake"],
        }
    else:
        return {
            "__type":           "archive",
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
