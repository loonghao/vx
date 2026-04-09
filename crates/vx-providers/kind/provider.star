# provider.star - kind (Kubernetes IN Docker) provider
#
# kind is a tool for running local Kubernetes clusters using Docker containers.
# Release assets are single binaries (no archive) with the naming format:
#   kind-{os}-{arch}[.exe]
# e.g. kind-linux-amd64, kind-darwin-arm64, kind-windows-amd64.exe
#
# Download URL format:
#   https://github.com/kubernetes-sigs/kind/releases/download/v{version}/kind-{os}-{arch}[.exe]
#
# Uses binary_layout (single binary, no archive).

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "binary_layout")
load("@vx//stdlib:github.star", "make_fetch_versions")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "kind"
description = "kind - Kubernetes IN Docker: local Kubernetes clusters for testing"
homepage    = "https://kind.sigs.k8s.io"
repository  = "https://github.com/kubernetes-sigs/kind"
license     = "Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("kind",
        version_cmd     = "{executable} version",
        version_pattern = "kind v",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions — from kubernetes-sigs/kind releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("kubernetes-sigs", "kind")

# ---------------------------------------------------------------------------
# Platform helpers
# kind uses os/arch format:  linux/amd64, darwin/arm64, windows/amd64
# ---------------------------------------------------------------------------

def _kind_platform(ctx):
    os_map   = {"windows": "windows", "macos": "darwin", "linux": "linux"}
    arch_map = {"x64": "amd64", "arm64": "arm64", "arm": "arm"}
    os_str   = os_map.get(ctx.platform.os)
    arch_str = arch_map.get(ctx.platform.arch, "amd64")
    return os_str, arch_str

# ---------------------------------------------------------------------------
# download_url — single binary, no archive
# URL: https://github.com/kubernetes-sigs/kind/releases/download/v{version}/kind-{os}-{arch}[.exe]
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    os_str, arch_str = _kind_platform(ctx)
    if not os_str:
        return None
    exe = ".exe" if ctx.platform.os == "windows" else ""
    return "https://github.com/kubernetes-sigs/kind/releases/download/v{}/kind-{}-{}{}".format(
        version, os_str, arch_str, exe)

# ---------------------------------------------------------------------------
# Layout + path/env functions
# kind is a single binary, no archive
# ---------------------------------------------------------------------------

install_layout = binary_layout("kind")


def store_root(ctx):
    return ctx.vx_home + "/store/kind"


def get_execute_path(ctx, _version):
    exe = "kind.exe" if ctx.platform.os == "windows" else "kind"
    return ctx.install_dir + "/" + exe


def post_install(_ctx, _version):
    return None


def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]


def deps(_ctx, _version):
    return []
