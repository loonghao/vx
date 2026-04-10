# provider.star - minikube provider
#
# minikube quickly sets up a local Kubernetes cluster on macOS, Linux, and Windows.
#
# Binary download URL: https://storage.googleapis.com/minikube/releases/v{version}/minikube-{os}-{arch}[.exe]
#   OS:   linux, darwin, windows
#   Arch: amd64, arm64
#
# Version source: kubernetes/minikube releases on GitHub (tag prefix "v")
#
# Note: minikube is a single binary (no archive). Windows gets .exe suffix.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "path_fns",
     "fetch_versions_with_tag_prefix")
load("@vx//stdlib:env.star", "env_prepend")
load("@vx//stdlib:layout.star", "binary_layout")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "minikube"
description = "minikube - Run Kubernetes locally for development and testing"
homepage    = "https://minikube.sigs.k8s.io"
repository  = "https://github.com/kubernetes/minikube"
license     = "Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("minikube",
        version_cmd     = "{executable} version",
        version_pattern = "minikube version: v\\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(extra_hosts = ["storage.googleapis.com"])

# ---------------------------------------------------------------------------
# fetch_versions - from kubernetes/minikube releases
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix(
    "kubernetes", "minikube", tag_prefix = "v"
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

def _minikube_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# download_url - Google Storage single binary
# URL: https://storage.googleapis.com/minikube/releases/v{version}/minikube-{os}-{arch}[.exe]
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _minikube_platform(ctx)
    if not platform:
        return None
    os_str, arch_str = platform
    exe = ".exe" if ctx.platform.os == "windows" else ""
    return "https://storage.googleapis.com/minikube/releases/v{}/minikube-{}-{}{}".format(
        version, os_str, arch_str, exe)

# ---------------------------------------------------------------------------
# Layout + path/env functions
# minikube is a single binary
# ---------------------------------------------------------------------------

install_layout = binary_layout("minikube")

paths            = path_fns("minikube")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]


def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]


def post_install(_ctx, _version):
    return None


def deps(_ctx, _version):
    return []
