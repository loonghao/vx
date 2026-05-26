# provider.star - mcpcall provider
#
# mcpcall is a Rust CLI for exercising MCP servers from shell scripts and CI.
# Release assets are direct binaries:
#   linux:   mcpcall-linux-x86_64, mcpcall-linux-aarch64
#   macOS:   mcpcall-macos-x86_64, mcpcall-macos-aarch64
#   Windows: mcpcall-windows-x86_64.exe
#
# Tags use the component prefix "mcpcall-v", for example mcpcall-v0.4.0.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "fetch_versions_with_tag_prefix")
load("@vx//stdlib:github.star", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "mcpcall"
description = "mcpcall - Scriptable MCP client for smoke tests and CI"
homepage    = "https://github.com/loonghao/mcpcall"
repository  = "https://github.com/loonghao/mcpcall"
license     = "MIT"
ecosystem   = "ai"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("mcpcall",
        version_cmd     = "{executable} --version",
        version_pattern = "mcpcall \\d+\\.\\d+\\.\\d+",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "mcpcall \\d+\\.\\d+\\.\\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions - from loonghao/mcpcall component tags
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix("loonghao", "mcpcall",
    tag_prefix = "mcpcall-v")

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64": ("windows", "x86_64", ".exe"),
    "linux/x64":   ("linux",   "x86_64", ""),
    "linux/arm64": ("linux",   "aarch64", ""),
    "macos/x64":   ("macos",   "x86_64", ""),
    "macos/arm64": ("macos",   "aarch64", ""),
}

def _mcpcall_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

def _asset_name(ctx):
    platform = _mcpcall_platform(ctx)
    if not platform:
        return None
    os_name, arch_name, suffix = platform[0], platform[1], platform[2]
    return "mcpcall-{}-{}{}".format(os_name, arch_name, suffix)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    asset = _asset_name(ctx)
    if not asset:
        return None
    return github_asset_url("loonghao", "mcpcall", "mcpcall-v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout - direct binary assets, normalize into bin/mcpcall(.exe)
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    source = _asset_name(ctx)
    if not source:
        return None
    target = "mcpcall.exe" if ctx.platform.os == "windows" else "mcpcall"
    return {
        "__type":           "binary_install",
        "source_name":      source,
        "target_name":      target,
        "target_dir":       "bin",
        "executable_paths": ["bin/" + target],
    }

# ---------------------------------------------------------------------------
# Path + env functions
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/mcpcall"

def get_execute_path(ctx, _version):
    executable = "mcpcall.exe" if ctx.platform.os == "windows" else "mcpcall"
    return ctx.install_dir + "/bin/" + executable

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
