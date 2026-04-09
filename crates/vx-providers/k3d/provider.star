# provider.star - k3d provider
#
# k3d is a lightweight wrapper to run k3s (Rancher Lab's minimal Kubernetes
# distribution) in Docker containers.
#
# Release assets are single binaries with the naming format:
#   k3d-{os}-{arch}[.exe]
# e.g. k3d-linux-amd64, k3d-darwin-arm64, k3d-windows-amd64.exe
#
# Download URL format:
#   https://github.com/k3d-io/k3d/releases/download/v{version}/k3d-{os}-{arch}[.exe]
#
# Uses binary_layout (single binary, no archive).

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "binary_layout")
load("@vx//stdlib:github.star", "make_fetch_versions")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "k3d"
description = "k3d - Lightweight Kubernetes wrapper for k3s in Docker"
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
        version_pattern = "k3d version v",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions — from k3d-io/k3d releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("k3d-io", "k3d")

# ---------------------------------------------------------------------------
# Platform helpers
# k3d uses os/arch format: linux/amd64, darwin/arm64, windows/amd64
# ---------------------------------------------------------------------------

def _k3d_platform(ctx):
    os_map   = {"windows": "windows", "macos": "darwin", "linux": "linux"}
    arch_map = {"x64": "amd64", "arm64": "arm64", "arm": "arm"}
    os_str   = os_map.get(ctx.platform.os)
    arch_str = arch_map.get(ctx.platform.arch, "amd64")
    return os_str, arch_str

# ---------------------------------------------------------------------------
# download_url — single binary, no archive
# URL: https://github.com/k3d-io/k3d/releases/download/v{version}/k3d-{os}-{arch}[.exe]
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    os_str, arch_str = _k3d_platform(ctx)
    if not os_str:
        return None
    exe = ".exe" if ctx.platform.os == "windows" else ""
    return "https://github.com/k3d-io/k3d/releases/download/v{}/k3d-{}-{}{}".format(
        version, os_str, arch_str, exe)

# ---------------------------------------------------------------------------
# Layout + path/env functions
# k3d is a single binary, no archive
# ---------------------------------------------------------------------------

install_layout = binary_layout("k3d")


def store_root(ctx):
    return ctx.vx_home + "/store/k3d"


def get_execute_path(ctx, _version):
    exe = "k3d.exe" if ctx.platform.os == "windows" else "k3d"
    return ctx.install_dir + "/bin/" + exe


def post_install(_ctx, _version):
    return None


def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]


def deps(_ctx, _version):
    return []
