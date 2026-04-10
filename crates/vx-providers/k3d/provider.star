# provider.star - k3d provider
#
# k3d is a lightweight wrapper to run k3s (Rancher Lab's minimal Kubernetes
# distribution) in Docker.
#
# Release assets (GitHub releases) — all are standalone binaries, no archives:
#   k3d-linux-amd64
#   k3d-linux-arm64
#   k3d-darwin-amd64
#   k3d-darwin-arm64
#   k3d-windows-amd64.exe
#
# Version source: k3d-io/k3d releases on GitHub (tag prefix "v")

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "path_fns",
     "fetch_versions_with_tag_prefix")
load("@vx//stdlib:env.star", "env_prepend")

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
# download_url
#
# All assets are standalone binaries (no archive):
#   k3d-{os}-{arch}       (Linux/macOS)
#   k3d-{os}-{arch}.exe   (Windows)
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _k3d_platform(ctx)
    if not platform:
        return None
    os_str, arch_str = platform
    ext = ".exe" if ctx.platform.os == "windows" else ""
    return "https://github.com/k3d-io/k3d/releases/download/v{}/k3d-{}-{}{}".format(
        version, os_str, arch_str, ext)

# ---------------------------------------------------------------------------
# install_layout — single binary for all platforms
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    platform = _k3d_platform(ctx)
    if not platform:
        return None
    os_str, arch_str = platform
    ext = ".exe" if ctx.platform.os == "windows" else ""
    source_name = "k3d-{}-{}{}".format(os_str, arch_str, ext)
    target_name = "k3d" + ext
    return {
        "type":        "binary",
        "source_name": source_name,
        "target_name": target_name,
        "target_dir":  "bin",
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

paths            = path_fns("k3d", executable = "bin/k3d")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
