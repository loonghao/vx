# provider.star - kind provider
#
# kind (Kubernetes IN Docker) is a tool for running local Kubernetes clusters
# using Docker container "nodes".
#
# Binary download URL: https://kind.sigs.k8s.io/dl/v{version}/kind-{os}-{arch}[.exe]
#   OS:   linux, darwin, windows
#   Arch: amd64, arm64
#
# Version source: kubernetes-sigs/kind releases on GitHub
#
# Note: kind is a single binary (no archive), Windows gets .exe suffix.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "fetch_versions_with_tag_prefix")
load("@vx//stdlib:env.star", "env_prepend")
load("@vx//stdlib:layout.star", "binary_layout")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "kind"
description = "kind - Kubernetes IN Docker, run local Kubernetes clusters using Docker container nodes"
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
        version_pattern = "kind v\\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(extra_hosts = ["kind.sigs.k8s.io"])

# ---------------------------------------------------------------------------
# fetch_versions - from kubernetes-sigs/kind releases
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix(
    "kubernetes-sigs", "kind", tag_prefix = "v"
)

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":   ("windows", "amd64"),
    "windows/arm64": ("windows", "arm64"),
    "macos/x64":     ("darwin",  "amd64"),
    "macos/arm64":   ("darwin",  "arm64"),
    "linux/x64":     ("linux",   "amd64"),
    "linux/arm64":   ("linux",   "arm64"),
}

def _kind_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# download_url - kind.sigs.k8s.io/dl/ single binary
# URL: https://kind.sigs.k8s.io/dl/v{version}/kind-{os}-{arch}[.exe]
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _kind_platform(ctx)
    if not platform:
        return None
    os_str, arch_str = platform
    exe = ".exe" if ctx.platform.os == "windows" else ""
    return "https://kind.sigs.k8s.io/dl/v{}/kind-{}-{}{}".format(
        version, os_str, arch_str, exe)

# ---------------------------------------------------------------------------
# Layout + path/env functions
# kind is a single binary (no archive compression on Linux/macOS,
# .exe directly on Windows)
# binary_layout places the binary at <install_dir>/bin/kind[.exe]
# ---------------------------------------------------------------------------

install_layout = binary_layout("kind")


def store_root(ctx):
    return ctx.vx_home + "/store/kind"


def get_execute_path(ctx, _version):
    exe = "kind.exe" if ctx.platform.os == "windows" else "kind"
    return ctx.install_dir + "/bin/" + exe


def post_install(_ctx, _version):
    return None


def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]


def deps(_ctx, _version):
    return []
