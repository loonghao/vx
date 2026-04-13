# provider.star - k3d provider
#
# k3d is a lightweight wrapper to run k3s (Rancher Lab's minimal Kubernetes
# distribution) in Docker.
#
# Release assets (GitHub releases):
#   - Linux/macOS: k3d-{os}-{arch}           (direct binary, no extension)
#   - Windows:     k3d-windows-amd64.exe      (direct executable)
#
# OS:   linux, darwin, windows
# Arch: amd64, arm64 (Linux/macOS), amd64 (Windows)
#
# Version source: k3d-io/k3d releases on GitHub (tag prefix "v")

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "fetch_versions_with_tag_prefix")
load("@vx//stdlib:env.star", "env_prepend")
load("@vx//stdlib:layout.star", "binary_layout")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "k3d"
description = "k3d - Lightweight wrapper to run k3s in Docker"
homepage    = "https://k3d.io"
repository  = "https://github.com/k3d-io/k3d"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("k3d",
        version_cmd     = "{executable} version",
        version_pattern = "k3d version v\\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions - from k3d-io/k3d releases
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix("k3d-io", "k3d", tag_prefix = "v")

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":   ("windows", "amd64"),
    "macos/x64":     ("darwin",  "amd64"),
    "macos/arm64":   ("darwin",  "arm64"),
    "linux/x64":     ("linux",   "amd64"),
    "linux/arm64":   ("linux",   "arm64"),
}

def _k3d_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# download_url - GitHub releases
# All platforms: direct binary download (no archive)
# Linux/macOS: https://github.com/k3d-io/k3d/releases/download/v{version}/k3d-{os}-{arch}
# Windows:     https://github.com/k3d-io/k3d/releases/download/v{version}/k3d-windows-amd64.exe
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _k3d_platform(ctx)
    if not platform:
        return None
    os_str, arch_str = platform
    exe = ".exe" if ctx.platform.os == "windows" else ""
    return "https://github.com/k3d-io/k3d/releases/download/v{}/k3d-{}-{}{}".format(
        version, os_str, arch_str, exe)

# ---------------------------------------------------------------------------
# install_layout - all platforms are direct binary downloads
# binary_layout places the binary at <install_dir>/bin/k3d[.exe]
# ---------------------------------------------------------------------------

install_layout = binary_layout("k3d")

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/k3d"


def get_execute_path(ctx, _version):
    exe = "k3d.exe" if ctx.platform.os == "windows" else "k3d"
    return ctx.install_dir + "/bin/" + exe


def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]


def post_install(_ctx, _version):
    return None


def deps(_ctx, _version):
    return []
