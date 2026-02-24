# provider.star - vcpkg provider
#
# C++ library manager. Tags are date-based: "2025-12-16" (no "v" prefix).
# Asset: vcpkg-{arch}-{os}[.exe]  (single binary)
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "dep_def")
load("@vx//stdlib:github.star", "make_fetch_versions")
load("@vx//stdlib:env.star",    "env_set", "env_prepend")

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
    runtime_def("vcpkg",
        aliases      = ["vcpkg-cli"],
        system_paths = [
            "C:/vcpkg/vcpkg.exe",
            "/usr/local/bin/vcpkg",
            "/usr/bin/vcpkg",
        ],
        test_commands = [
            {"command": "{executable} version", "name": "version_check",
             "expected_output": "vcpkg"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(extra_hosts = [])

# ---------------------------------------------------------------------------
# fetch_versions — date-based tags (no "v" prefix)
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("microsoft", "vcpkg-tool")

# ---------------------------------------------------------------------------
# download_url — single binary per platform
# ---------------------------------------------------------------------------

_VCPKG_ASSETS = {
    "windows/x64":   "vcpkg-x64-windows.exe",
    "windows/arm64": "vcpkg-arm64-windows.exe",
    "macos/x64":     "vcpkg-x64-osx",
    "macos/arm64":   "vcpkg-arm64-osx",
    "linux/x64":     "vcpkg-x64-linux",
    "linux/arm64":   "vcpkg-arm64-linux",
}

def download_url(ctx, version):
    asset = _VCPKG_ASSETS.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))
    if not asset:
        return None
    return "https://github.com/microsoft/vcpkg-tool/releases/download/{}/{}".format(
        version, asset)

# ---------------------------------------------------------------------------
# install_layout — single binary
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    asset = _VCPKG_ASSETS.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))
    exe   = "vcpkg.exe" if ctx.platform.os == "windows" else "vcpkg"
    return {
        "type":             "binary",
        "source_name":      asset,
        "target_name":      exe,
        "executable_paths": [exe],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/vcpkg"

def get_execute_path(ctx, _version):
    exe = "vcpkg.exe" if ctx.platform.os == "windows" else "vcpkg"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [
        env_set("VCPKG_ROOT",                 ctx.install_dir),
        env_set("VCPKG_DOWNLOADS",            ctx.install_dir + "/.cache/downloads"),
        env_set("VCPKG_DEFAULT_BINARY_CACHE", ctx.install_dir + "/.cache/archives"),
        env_prepend("PATH",                   ctx.install_dir),
    ]

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return [
        dep_def("git",   reason = "Git is required to clone the vcpkg package registry"),
        dep_def("cmake", optional = True, reason = "CMake is commonly used with vcpkg"),
        dep_def("ninja", optional = True, reason = "Ninja provides faster builds"),
    ]
